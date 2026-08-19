use nova_crypto::{
    compute_safety_number, generate_one_time_prekey, generate_signed_prekey, verify_prekey_bundle,
    x3dh_initiate, x3dh_respond, DeviceIdentity, DoubleRatchetSession, MnemonicPhrase,
};
use nova_protocol::{
    prekey_bundle_from_bytes, prekey_bundle_to_bytes, EncryptedFrame, FrameType,
    HandshakeInitPayload, MessagePayload, NovaPacket,
};
use nova_storage::{
    ContactRecord, ConversationRecord, DbMessageStatus, MessageRecord, StorageEngine, StorageError,
};
use nova_transport::{P2PNode, PeerConnectionInfo, TransportSupervisor};
use rand::Rng;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::Mutex;
use tracing::{debug, warn};
use uuid::Uuid;

/// How many one-time prekeys to provision per device on first setup. Each is consumed
/// (deleted) the first time it is either published in a bundle or used to answer a handshake.
const ONE_TIME_PREKEY_POOL_SIZE: usize = 10;

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("Crypto error: {0}")]
    Crypto(#[from] nova_crypto::CryptoError),
    #[error("Protocol error: {0}")]
    Protocol(#[from] nova_protocol::ProtocolError),
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),
    #[error("No local identity: create or restore an account first")]
    NoIdentity,
    #[error("No session established with peer: {0}")]
    NoSession(String),
    #[error("Unknown contact: {0} — add them first with a verified prekey bundle")]
    NoContact(String),
    #[error("No local prekeys have been provisioned for this device")]
    NoPrekeys,
    #[error("X3DH handshake could not be completed: {0}")]
    HandshakeFailed(String),
    #[error("Packet's claimed sender does not match its envelope — possible spoofing")]
    SenderMismatch,
    #[error("Unsupported frame type for receive_packet: {0:?}")]
    UnsupportedFrame(FrameType),
}

/// How often the network pump retries delivering whatever is still sitting in the outbox (e.g.
/// because the recipient was offline or unreachable last attempt).
const OUTBOX_PUMP_INTERVAL: Duration = Duration::from_millis(500);

pub struct NovaEngine {
    pub identity: Arc<Mutex<Option<DeviceIdentity>>>,
    pub storage: Arc<StorageEngine>,
    pub transport: Arc<TransportSupervisor>,
    pub sessions: Arc<Mutex<HashMap<String, DoubleRatchetSession>>>,
    /// Set via `attach_network` once this engine is wired to a real `P2PNode`. Until then, sent
    /// messages are encrypted and queued in the local outbox but never actually leave the
    /// device — exactly the honest, testable state the crypto/storage layers are verified in
    /// isolation, without silently pretending a network delivery happened.
    network: Mutex<Option<Arc<P2PNode>>>,
}

impl NovaEngine {
    /// Opens (or creates) the local encrypted database at `db_path`, unlocked with
    /// `storage_passphrase`. This passphrase never touches disk — see `StorageEngine::open`.
    pub fn new(db_path: &str, storage_passphrase: &str) -> Result<Self, EngineError> {
        let storage = Arc::new(StorageEngine::open(db_path, storage_passphrase)?);
        let transport = Arc::new(TransportSupervisor::new());
        let sessions = Arc::new(Mutex::new(HashMap::new()));

        Ok(Self {
            identity: Arc::new(Mutex::new(None)),
            storage,
            transport,
            sessions,
            network: Mutex::new(None),
        })
    }

    /// Wires this engine to a real P2P network node: outbound messages are actually delivered
    /// (direct QUIC to the peer, falling back to the rendezvous relay when direct connection
    /// fails) instead of only sitting in the local outbox, and incoming packets are decrypted
    /// and persisted as soon as they arrive. This is what turns "two engines in the same test
    /// process" into "two independent devices talking over the real internet" — see
    /// `nova-transport::P2PNode` for the connection strategy (direct QUIC, NAT-punching via a
    /// rendezvous server, opaque relay fallback).
    ///
    /// Requires the engine to be held behind an `Arc`, since the spawned background tasks
    /// (inbound packet ingestion, outbox delivery retries) outlive this call.
    pub async fn attach_network(self: &Arc<Self>, node: Arc<P2PNode>) {
        *self.network.lock().await = Some(node.clone());

        let engine = self.clone();
        let recv_node = node.clone();
        tokio::spawn(async move {
            while let Some(bytes) = recv_node.recv_next().await {
                if let Err(e) = engine.receive_packet(&bytes).await {
                    warn!("Failed to process an incoming packet: {e}");
                }
            }
        });

        let engine = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(OUTBOX_PUMP_INTERVAL);
            loop {
                interval.tick().await;
                if let Err(e) = engine.pump_outbox_once().await {
                    warn!("Outbox pump failed: {e}");
                }
            }
        });
    }

    /// Attempts to deliver every currently-queued outbox entry over the attached network node,
    /// removing each one from the outbox on successful hand-off (direct or relayed — both count
    /// as real delivery to the transport layer, not just "queued locally"). A no-op returning
    /// `Ok(0)` if no network has been attached yet.
    pub async fn pump_outbox_once(&self) -> Result<usize, EngineError> {
        let Some(node) = self.network.lock().await.clone() else {
            return Ok(0);
        };

        let pending = self.storage.get_pending_outbox()?;
        let mut delivered = 0;
        for item in pending {
            match node.send_to_peer(&item.recipient_id, item.payload.clone()).await {
                Ok(mode) => {
                    self.storage.remove_from_outbox(&item.message_id)?;
                    debug!("Delivered outbox item {} via {:?}", item.message_id, mode);
                    delivered += 1;
                }
                Err(e) => debug!("Outbox item {} not yet delivered: {e}", item.message_id),
            }
        }
        Ok(delivered)
    }

    /// Create a new sovereign account with a 12-word mnemonic phrase, provisioning the X3DH
    /// prekeys this device needs before anyone can establish a session with it.
    pub async fn create_account(&self, username: &str) -> Result<(String, String), EngineError> {
        let mnemonic = MnemonicPhrase::generate()?;
        let identity = DeviceIdentity::from_mnemonic(&mnemonic, username)?;
        let pub_hex = identity.public_id_hex();
        let mnemonic_str = mnemonic.as_str().to_string();

        self.storage.save_identity(&identity, &mnemonic_str)?;
        self.provision_prekeys(&identity)?;
        *self.identity.lock().await = Some(identity);

        Ok((pub_hex, mnemonic_str))
    }

    /// Restore an existing account from 12 words. Prekeys are device-local key material and
    /// are therefore (re)provisioned here if this device does not already have any — exactly
    /// like restoring a Signal account on a new device.
    pub async fn restore_account(
        &self,
        mnemonic_phrase: &str,
        username: &str,
    ) -> Result<String, EngineError> {
        let mnemonic = MnemonicPhrase::from_phrase(mnemonic_phrase)?;
        let identity = DeviceIdentity::from_mnemonic(&mnemonic, username)?;
        let pub_hex = identity.public_id_hex();

        self.storage
            .save_identity(&identity, mnemonic_phrase.trim())?;
        if self.storage.load_active_signed_prekey()?.is_none() {
            self.provision_prekeys(&identity)?;
        }
        *self.identity.lock().await = Some(identity);

        Ok(pub_hex)
    }

    fn provision_prekeys(&self, identity: &DeviceIdentity) -> Result<(), EngineError> {
        let signed_key_id = random_key_id();
        let (spk_secret, spk_public) = generate_signed_prekey(identity, signed_key_id);
        self.storage.save_signed_prekey(&spk_secret, &spk_public)?;

        let one_time_keys: Vec<_> = (0..ONE_TIME_PREKEY_POOL_SIZE)
            .map(|_| generate_one_time_prekey(random_key_id()))
            .collect();
        self.storage.save_one_time_prekeys(&one_time_keys)?;

        Ok(())
    }

    /// Serializes this device's current publishable X3DH prekey bundle (identity keys, signed
    /// prekey, and — if any remain — a fresh one-time prekey). Share the returned bytes
    /// out-of-band (QR code, direct exchange) so a peer can pass them to `add_contact`.
    pub async fn get_own_prekey_bundle_bytes(&self) -> Result<Vec<u8>, EngineError> {
        let id_lock = self.identity.lock().await;
        let identity = id_lock.as_ref().ok_or(EngineError::NoIdentity)?;
        let bundle = self.storage.get_own_prekey_bundle(identity)?;
        Ok(prekey_bundle_to_bytes(&bundle)?)
    }

    /// Adds a new contact from their serialized prekey bundle. The bundle's signed-prekey
    /// signature is verified against the identity key it claims to belong to *before* it is
    /// trusted or persisted — a malformed or unsigned bundle is rejected outright rather than
    /// silently degrading to a null/placeholder key.
    pub async fn add_contact(
        &self,
        username: &str,
        display_name: &str,
        bundle_bytes: &[u8],
    ) -> Result<ContactRecord, EngineError> {
        let bundle = prekey_bundle_from_bytes(bundle_bytes)?;
        verify_prekey_bundle(&bundle)?;

        let own_pub = {
            let id_lock = self.identity.lock().await;
            id_lock
                .as_ref()
                .map(|id| id.verifying_key_bytes)
                .ok_or(EngineError::NoIdentity)?
        };

        let peer_id = hex::encode(bundle.identity_ed25519_pub);
        let safety_number = compute_safety_number(&own_pub, &bundle.identity_ed25519_pub)?;

        let contact = ContactRecord {
            peer_id: peer_id.clone(),
            username: username.to_string(),
            display_name: display_name.to_string(),
            prekey_bundle: bundle,
            safety_number,
            is_online: true,
            is_blocked: false,
            last_seen_utc: chrono::Utc::now().timestamp(),
        };
        self.storage.save_contact(&contact)?;

        let conv = ConversationRecord {
            id: format!("conv_{peer_id}"),
            peer_id: peer_id.clone(),
            title: display_name.to_string(),
            last_message_text: String::new(),
            last_message_time_utc: chrono::Utc::now().timestamp(),
            unread_count: 0,
        };
        self.storage.save_conversation(&conv)?;

        Ok(contact)
    }

    /// Send a message to a contact. The first message to a peer with no existing session
    /// performs a real X3DH handshake (using their verified, stored prekey bundle) and travels
    /// as a `HandshakeInit` frame carrying the handshake material plus the first Double Ratchet
    /// ciphertext; every later message reuses the established session as a plain
    /// `EncryptedMessage` frame. The resulting wire packet is queued in the Outbox for delivery
    /// by the (network) transport layer.
    pub async fn send_message(
        &self,
        conversation_id: &str,
        recipient_peer_id: &str,
        text: &str,
    ) -> Result<MessageRecord, EngineError> {
        let msg_id = Uuid::new_v4().to_string();
        let (sender_id, identity_x25519_pub, identity_ed25519_pub) = {
            let id_lock = self.identity.lock().await;
            let identity = id_lock.as_ref().ok_or(EngineError::NoIdentity)?;
            (
                identity.public_id_hex(),
                identity.dh_public_bytes,
                identity.verifying_key_bytes,
            )
        };

        let now = chrono::Utc::now().timestamp();

        let payload = MessagePayload::new_text(
            msg_id.clone(),
            conversation_id.to_string(),
            sender_id.clone(),
            recipient_peer_id.to_string(),
            text.to_string(),
        );
        let payload_bytes = payload.to_bytes()?;

        let handshake_material = self
            .ensure_session_for_send(recipient_peer_id)
            .await?;

        let mut sessions_lock = self.sessions.lock().await;
        let session = sessions_lock
            .get_mut(recipient_peer_id)
            .expect("session was just established or loaded above");

        let (header, ciphertext) = session.ratchet_encrypt(&payload_bytes, b"NOVA_MSG")?;
        self.storage.save_session(recipient_peer_id, session)?;
        drop(sessions_lock);

        let frame = EncryptedFrame { header, ciphertext };

        let (frame_type, packet_payload) = match handshake_material {
            Some(init) => {
                let handshake = HandshakeInitPayload {
                    sender_identity_ed25519_pub: identity_ed25519_pub,
                    sender_identity_x25519_pub: identity_x25519_pub,
                    sender_ephemeral_pub: init.ephemeral_pub,
                    used_signed_prekey_id: init.used_signed_prekey_id,
                    used_one_time_prekey_id: init.used_one_time_prekey_id,
                    first_message: frame,
                };
                (FrameType::HandshakeInit, handshake.to_bytes()?)
            }
            None => {
                let mut buf = Vec::new();
                ciborium::into_writer(&frame, &mut buf)
                    .map_err(|e| nova_protocol::ProtocolError::SerializationFailed(e.to_string()))?;
                (FrameType::EncryptedMessage, buf)
            }
        };

        let packet = NovaPacket::new(frame_type, sender_id.clone(), packet_payload);
        let packet_cbor = packet.to_cbor()?;

        let msg_record = MessageRecord {
            id: msg_id.clone(),
            conversation_id: conversation_id.to_string(),
            sender_id,
            recipient_id: recipient_peer_id.to_string(),
            text_content: text.to_string(),
            timestamp_utc: now,
            status: DbMessageStatus::Sent,
            is_outgoing: true,
        };

        self.storage.save_message(&msg_record)?;
        self.storage
            .enqueue_outbox(&msg_id, conversation_id, recipient_peer_id, &packet_cbor)?;

        Ok(msg_record)
    }

    /// Ensures a Double Ratchet session exists in memory for `recipient_peer_id`, checking the
    /// in-memory cache, then persisted storage, and only performing a fresh X3DH handshake as a
    /// last resort (which requires the peer to already be a verified contact). Returns the X3DH
    /// output when a fresh handshake was performed, so the caller knows to send a `HandshakeInit`
    /// frame instead of a plain `EncryptedMessage`.
    async fn ensure_session_for_send(
        &self,
        recipient_peer_id: &str,
    ) -> Result<Option<nova_crypto::X3dhInitResult>, EngineError> {
        {
            let sessions_lock = self.sessions.lock().await;
            if sessions_lock.contains_key(recipient_peer_id) {
                return Ok(None);
            }
        }

        if let Some(loaded) = self.storage.load_session(recipient_peer_id)? {
            self.sessions
                .lock()
                .await
                .insert(recipient_peer_id.to_string(), loaded);
            return Ok(None);
        }

        let contact = self
            .storage
            .get_contact(recipient_peer_id)?
            .ok_or_else(|| EngineError::NoContact(recipient_peer_id.to_string()))?;
        verify_prekey_bundle(&contact.prekey_bundle)?;

        let id_lock = self.identity.lock().await;
        let identity = id_lock.as_ref().ok_or(EngineError::NoIdentity)?;

        let init = x3dh_initiate(identity, &contact.prekey_bundle)?;
        let session = DoubleRatchetSession::init_alice(
            &init.shared_secret,
            &contact.prekey_bundle.signed_prekey.public,
        )?;
        drop(id_lock);

        self.sessions
            .lock()
            .await
            .insert(recipient_peer_id.to_string(), session);

        Ok(Some(init))
    }

    /// Processes one raw wire packet received from the network (or, until the transport layer
    /// exists, fed in directly from the Outbox for local testing). Handles both a fresh
    /// `HandshakeInit` (performs the responder side of X3DH, establishes the session) and a
    /// regular `EncryptedMessage` on an already-established session, persisting the decrypted
    /// message and updating the local conversation on success.
    pub async fn receive_packet(&self, raw_packet: &[u8]) -> Result<Option<MessageRecord>, EngineError> {
        let packet = NovaPacket::from_cbor(raw_packet)?;

        match packet.frame_type {
            FrameType::HandshakeInit => {
                let handshake = HandshakeInitPayload::from_bytes(&packet.payload)?;
                let sender_peer_id = hex::encode(handshake.sender_identity_ed25519_pub);
                if sender_peer_id != packet.session_id {
                    return Err(EngineError::SenderMismatch);
                }

                let id_lock = self.identity.lock().await;
                let identity = id_lock.as_ref().ok_or(EngineError::NoIdentity)?;

                let (spk_secret, spk_public) = self
                    .storage
                    .load_active_signed_prekey()?
                    .ok_or(EngineError::NoPrekeys)?;
                if spk_public.key_id != handshake.used_signed_prekey_id {
                    return Err(EngineError::HandshakeFailed(
                        "handshake references a signed prekey id we no longer have".into(),
                    ));
                }

                let opk_secret = match handshake.used_one_time_prekey_id {
                    Some(id) => self.storage.consume_one_time_prekey_secret(id)?,
                    None => None,
                };

                let shared_secret = x3dh_respond(
                    identity,
                    &spk_secret,
                    opk_secret.as_ref(),
                    &handshake.sender_identity_x25519_pub,
                    &handshake.sender_ephemeral_pub,
                )?;
                drop(id_lock);

                let mut session =
                    DoubleRatchetSession::init_bob(&shared_secret, spk_secret.into_static_secret());
                let plaintext = session.ratchet_decrypt(
                    &handshake.first_message.header,
                    &handshake.first_message.ciphertext,
                    b"NOVA_MSG",
                )?;

                self.storage.save_session(&sender_peer_id, &session)?;
                self.sessions
                    .lock()
                    .await
                    .insert(sender_peer_id.clone(), session);

                self.persist_incoming_message(&sender_peer_id, &plaintext).await
            }
            FrameType::EncryptedMessage => {
                let sender_peer_id = packet.session_id.clone();
                let frame: EncryptedFrame = ciborium::from_reader(packet.payload.as_slice())
                    .map_err(|e| nova_protocol::ProtocolError::DeserializationFailed(e.to_string()))?;

                let mut sessions_lock = self.sessions.lock().await;
                let mut session = match sessions_lock.remove(&sender_peer_id) {
                    Some(s) => s,
                    None => self
                        .storage
                        .load_session(&sender_peer_id)?
                        .ok_or_else(|| EngineError::NoSession(sender_peer_id.clone()))?,
                };

                let plaintext = session.ratchet_decrypt(&frame.header, &frame.ciphertext, b"NOVA_MSG")?;
                self.storage.save_session(&sender_peer_id, &session)?;
                sessions_lock.insert(sender_peer_id.clone(), session);
                drop(sessions_lock);

                self.persist_incoming_message(&sender_peer_id, &plaintext).await
            }
            other => Err(EngineError::UnsupportedFrame(other)),
        }
    }

    async fn persist_incoming_message(
        &self,
        sender_peer_id: &str,
        plaintext: &[u8],
    ) -> Result<Option<MessageRecord>, EngineError> {
        let payload = MessagePayload::from_bytes(plaintext)?;
        let text = payload.text_content.clone().unwrap_or_default();

        let conv_id = payload.conversation_id.clone();
        let mut conv = self
            .storage
            .get_conversations()?
            .into_iter()
            .find(|c| c.id == conv_id)
            .unwrap_or_else(|| ConversationRecord {
                id: conv_id.clone(),
                peer_id: sender_peer_id.to_string(),
                title: sender_peer_id.to_string(),
                last_message_text: String::new(),
                last_message_time_utc: payload.timestamp_utc,
                unread_count: 0,
            });
        conv.unread_count += 1;
        self.storage.save_conversation(&conv)?;

        let msg_record = MessageRecord {
            id: payload.message_id.clone(),
            conversation_id: conv_id,
            sender_id: sender_peer_id.to_string(),
            recipient_id: payload.recipient_id.clone(),
            text_content: text,
            timestamp_utc: payload.timestamp_utc,
            status: DbMessageStatus::Delivered,
            is_outgoing: false,
        };
        self.storage.save_message(&msg_record)?;

        Ok(Some(msg_record))
    }

    /// Retrieve all conversations for the main UI screen.
    pub fn get_conversations(&self) -> Result<Vec<ConversationRecord>, EngineError> {
        Ok(self.storage.get_conversations()?)
    }

    /// Retrieve message history for a conversation.
    pub fn get_messages(&self, conversation_id: &str) -> Result<Vec<MessageRecord>, EngineError> {
        Ok(self.storage.get_messages(conversation_id)?)
    }

    /// Retrieve peer connection info for connection diagnostics screen. Reflects the attached
    /// `P2PNode`'s real connection state once `attach_network` has been called; falls back to
    /// this engine's own (empty, until then) supervisor otherwise.
    pub async fn get_diagnostics(&self, peer_id: &str) -> Option<PeerConnectionInfo> {
        if let Some(node) = self.network.lock().await.clone() {
            return node.supervisor.get_peer_info(peer_id).await;
        }
        self.transport.get_peer_info(peer_id).await
    }
}

fn random_key_id() -> u32 {
    // 0 is reserved as a "no key" sentinel in a couple of Option<u32> comparisons upstream;
    // keep generated ids strictly positive.
    rand::thread_rng().gen_range(1..=u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_end_to_end_handshake_and_bidirectional_messaging() {
        let alice = NovaEngine::new(":memory:", "alice-storage-pass").unwrap();
        let bob = NovaEngine::new(":memory:", "bob-storage-pass").unwrap();

        let (alice_pub, alice_mnemonic) = alice.create_account("alex").await.unwrap();
        let (bob_pub, _bob_mnemonic) = bob.create_account("bob").await.unwrap();
        assert_eq!(alice_mnemonic.split_whitespace().count(), 12);

        // Out-of-band exchange of verified prekey bundles (e.g. QR codes).
        let bob_bundle = bob.get_own_prekey_bundle_bytes().await.unwrap();
        let alice_contact = alice.add_contact("bob.nova", "Bob", &bob_bundle).await.unwrap();
        assert_eq!(alice_contact.peer_id, bob_pub);

        let alice_bundle = alice.get_own_prekey_bundle_bytes().await.unwrap();
        let bob_contact = bob.add_contact("alex.nova", "Alex", &alice_bundle).await.unwrap();
        assert_eq!(bob_contact.peer_id, alice_pub);

        // Alice sends the first message: real X3DH handshake, no hardcoded keys anywhere.
        let conv_id = format!("conv_{bob_pub}");
        let sent = alice
            .send_message(&conv_id, &bob_pub, "Salut Bob, ceci est chiffré de bout en bout.")
            .await
            .unwrap();
        assert_eq!(sent.status, DbMessageStatus::Sent);

        let pending = alice.storage.get_pending_outbox().unwrap();
        assert_eq!(pending.len(), 1);
        let wire_bytes = pending[0].payload.clone();

        // Sanity: the wire bytes really do carry a HandshakeInit frame for message #1.
        let parsed = NovaPacket::from_cbor(&wire_bytes).unwrap();
        assert_eq!(parsed.frame_type, FrameType::HandshakeInit);

        let received = bob
            .receive_packet(&wire_bytes)
            .await
            .unwrap()
            .expect("first message should decrypt via the X3DH handshake path");
        assert_eq!(received.text_content, "Salut Bob, ceci est chiffré de bout en bout.");
        assert_eq!(received.sender_id, alice_pub);
        assert!(!received.is_outgoing);

        // Bob replies on the now-established session: plain EncryptedMessage framing.
        let bob_conv_id = format!("conv_{alice_pub}");
        let reply = bob
            .send_message(&bob_conv_id, &alice_pub, "Bien reçu, Double Ratchet actif.")
            .await
            .unwrap();

        let bob_pending = bob.storage.get_pending_outbox().unwrap();
        let reply_entry = bob_pending.iter().find(|i| i.message_id == reply.id).unwrap();
        let reply_parsed = NovaPacket::from_cbor(&reply_entry.payload).unwrap();
        assert_eq!(reply_parsed.frame_type, FrameType::EncryptedMessage);

        let alice_received = alice
            .receive_packet(&reply_entry.payload)
            .await
            .unwrap()
            .expect("reply should decrypt via the established session");
        assert_eq!(alice_received.text_content, "Bien reçu, Double Ratchet actif.");
        assert_eq!(alice_received.sender_id, bob_pub);

        // A third message from Alice must also use the established session, not a new handshake.
        let sent2 = alice
            .send_message(&conv_id, &bob_pub, "Message 3, toujours sur la meme session.")
            .await
            .unwrap();
        let pending2 = alice.storage.get_pending_outbox().unwrap();
        let entry2 = pending2.iter().find(|i| i.message_id == sent2.id).unwrap();
        let parsed2 = NovaPacket::from_cbor(&entry2.payload).unwrap();
        assert_eq!(parsed2.frame_type, FrameType::EncryptedMessage);

        let final_history = bob.receive_packet(&entry2.payload).await.unwrap().unwrap();
        assert_eq!(final_history.text_content, "Message 3, toujours sur la meme session.");
    }

    #[tokio::test]
    async fn test_add_contact_rejects_malformed_and_tampered_bundles() {
        let alice = NovaEngine::new(":memory:", "alice-pass").unwrap();
        alice.create_account("alex").await.unwrap();

        // Garbage bytes: not valid CBOR at all.
        let err = alice.add_contact("x", "X", b"not a real bundle").await;
        assert!(err.is_err());

        // Well-formed bundle, but with a tampered signed-prekey public key (signature no longer matches).
        let bob = NovaEngine::new(":memory:", "bob-pass").unwrap();
        bob.create_account("bob").await.unwrap();
        let mut bundle_bytes = bob.get_own_prekey_bundle_bytes().await.unwrap();
        // Flip a byte roughly in the middle of the CBOR payload to corrupt the signed prekey
        // public key without producing a structurally invalid CBOR document.
        let mid = bundle_bytes.len() / 2;
        bundle_bytes[mid] ^= 0xFF;

        let tampered_result = alice.add_contact("bob.nova", "Bob", &bundle_bytes).await;
        assert!(tampered_result.is_err(), "tampered/undersigned bundle must be rejected");
    }

    #[tokio::test]
    async fn test_send_message_without_contact_fails_cleanly() {
        let alice = NovaEngine::new(":memory:", "alice-pass").unwrap();
        alice.create_account("alex").await.unwrap();

        let result = alice
            .send_message("conv_unknown", "deadbeef".repeat(8).as_str(), "hello")
            .await;
        assert!(matches!(result, Err(EngineError::NoContact(_))));
    }

    /// The literal claim behind "how can a user in Ouagadougou talk to a user in
    /// Bobo-Dioulasso": two fully independent `NovaEngine` instances — separate identities,
    /// separate encrypted databases, separate `P2PNode`s bound to their own OS sockets, tied
    /// together only by a shared rendezvous server neither of them trusts with anything but an
    /// address and an opaque, already-encrypted blob — exchange a real message over the real
    /// network stack (UDP sockets, QUIC handshake, X3DH, Double Ratchet), with no bytes shared
    /// in-process. Nothing here is a mock: a passing run is the network layer actually working.
    #[tokio::test]
    async fn test_two_independent_engines_exchange_a_message_over_a_real_network() {
        let rendezvous_addr = start_test_rendezvous_server().await;

        // `DeviceIdentity` deliberately does not implement `Clone` (it holds zeroized private
        // key material) — since it is deterministically derived from its mnemonic, each side
        // that needs its own independent instance just re-derives it from the same phrase,
        // exactly as a second real device restoring the same account would.
        let alice_mnemonic = MnemonicPhrase::generate().unwrap();
        let bob_mnemonic = MnemonicPhrase::generate().unwrap();
        let alice_identity = nova_crypto::DeviceIdentity::from_mnemonic(&alice_mnemonic, "alex").unwrap();
        let bob_identity = nova_crypto::DeviceIdentity::from_mnemonic(&bob_mnemonic, "bob").unwrap();
        let bob_peer_id = bob_identity.public_id_hex();
        let alice_peer_id = alice_identity.public_id_hex();

        let alice_node = nova_transport::P2PNode::start(alice_identity, "127.0.0.1:0", rendezvous_addr)
            .await
            .unwrap();
        let bob_node = nova_transport::P2PNode::start(bob_identity, "127.0.0.1:0", rendezvous_addr)
            .await
            .unwrap();
        bob_node.announce_presence().await.unwrap();

        let alice = Arc::new(NovaEngine::new(":memory:", "alice-storage-pass").unwrap());
        let bob = Arc::new(NovaEngine::new(":memory:", "bob-storage-pass").unwrap());

        // Wire each engine's identity directly, bypassing create_account's mnemonic generation
        // so both sides use the identities already registered on the network above.
        let alice_identity = nova_crypto::DeviceIdentity::from_mnemonic(&alice_mnemonic, "alex").unwrap();
        alice.storage.save_identity(&alice_identity, "n/a").unwrap();
        alice.provision_prekeys(&alice_identity).unwrap();
        *alice.identity.lock().await = Some(alice_identity);

        let bob_identity = nova_crypto::DeviceIdentity::from_mnemonic(&bob_mnemonic, "bob").unwrap();
        bob.storage.save_identity(&bob_identity, "n/a").unwrap();
        bob.provision_prekeys(&bob_identity).unwrap();
        *bob.identity.lock().await = Some(bob_identity);

        alice.attach_network(alice_node.clone()).await;
        bob.attach_network(bob_node.clone()).await;

        let bob_bundle = bob.get_own_prekey_bundle_bytes().await.unwrap();
        alice.add_contact("bob.nova", "Bob", &bob_bundle).await.unwrap();
        let conv_id = format!("conv_{bob_peer_id}");
        alice
            .send_message(&conv_id, &bob_peer_id, "Salut depuis Ouaga, ceci transite sur le vrai reseau.")
            .await
            .unwrap();

        // The outbox pump runs every 500ms; poll for up to a few seconds for the message to
        // actually cross the network and land, decrypted, in Bob's conversation history.
        let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
        loop {
            let messages = bob.get_messages(&conv_id).unwrap();
            if let Some(msg) = messages.iter().find(|m| !m.is_outgoing) {
                assert_eq!(msg.text_content, "Salut depuis Ouaga, ceci transite sur le vrai reseau.");
                assert_eq!(msg.sender_id, alice_peer_id);
                return;
            }
            if tokio::time::Instant::now() > deadline {
                panic!("message never arrived over the real network within the deadline");
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    async fn start_test_rendezvous_server() -> std::net::SocketAddr {
        let probe = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let addr = probe.local_addr().unwrap();
        drop(probe);
        let bind_addr = addr.to_string();
        tokio::spawn(async move {
            let _ = nova_server::run_server(&bind_addr).await;
        });
        tokio::time::sleep(Duration::from_millis(150)).await;
        addr
    }
}
