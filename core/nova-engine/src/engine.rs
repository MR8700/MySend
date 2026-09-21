use nova_crypto::{
    compute_safety_number, generate_one_time_prekey, generate_signed_prekey, verify_prekey_bundle,
    x3dh_initiate, x3dh_respond, DeviceIdentity, DoubleRatchetSession, MnemonicPhrase, PreKeyBundle,
};
use nova_protocol::{
    prekey_bundle_from_bytes, prekey_bundle_to_bytes, EncryptedFrame, FrameType,
    GroupControlAction, GroupMemberInfo, GroupMessageEnvelope, GroupRole,
    HandshakeInitPayload, MessageContentType, MessagePayload, NovaPacket, ParsedInvitation,
    SignedGroupInvitation,
};
use nova_storage::{
    ContactRecord, ConversationRecord, DbMessageStatus, GroupMemberRecord, GroupRecord,
    MessageRecord, StorageEngine, StorageError,
};
use nova_transport::{P2PNode, PeerConnectionInfo, TransportSupervisor, UdpFallbackClient};
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
    #[error("Transport error: {0}")]
    Transport(#[from] nova_transport::TransportError),
    #[error("Invitation error: {0}")]
    Invitation(#[from] nova_protocol::InvitationError),
    #[error("No local identity: create or restore an account first")]
    NoIdentity,
    #[error("No session established with peer: {0}")]
    NoSession(String),
    #[error("Unknown contact: {0} — add them first with a verified prekey bundle")]
    NoContact(String),
    #[error("No local prekeys have been provisioned for this device")]
    NoPrekeys,
    #[error("An identity already exists on this device — log out first to replace it")]
    IdentityAlreadyExists,
    #[error("X3DH handshake could not be completed: {0}")]
    HandshakeFailed(String),
    #[error("Packet's claimed sender does not match its envelope — possible spoofing")]
    SenderMismatch,
    #[error("Unsupported frame type for receive_packet: {0:?}")]
    UnsupportedFrame(FrameType),
    #[error("Message not found: {0}")]
    MessageNotFound(String),
}

/// How often the network pump retries delivering whatever is still sitting in the outbox (e.g.
/// because the recipient was offline or unreachable last attempt).
const OUTBOX_PUMP_INTERVAL: Duration = Duration::from_millis(500);

/// How long to let mDNS/bootstrap discovery populate the local DHT routing table before the very
/// first `announce_presence()` call — see that method's own doc comment: with an empty routing
/// table there is no peer to route the `put_record` through yet, so an immediate first attempt on
/// a cold start would just fail. Also how far into the future to schedule the next periodic
/// re-announce, forever, both to keep the record fresh and to retry now that new routing-table
/// peers may have appeared (e.g. a LAN peer that mDNS discovers a few seconds after this one).
const PRESENCE_ANNOUNCE_INTERVAL: Duration = Duration::from_secs(60);

/// Starting delay before the first retry of a failed outbox item.
const OUTBOX_RETRY_BASE_SECS: i64 = 1;
/// Retry delay never grows past this, no matter how many attempts have failed.
const OUTBOX_RETRY_CAP_SECS: i64 = 300; // 5 minutes
/// After failing for this long (wall-clock, since the message was first queued — not counting
/// how many attempts that took), the outbox stops retrying and marks the message `Failed`
/// instead of hammering an unreachable contact forever. See the 2026-08-22 audit: previously
/// there was no such ceiling at all, and no `Failed` state was ever reached.
const OUTBOX_GIVE_UP_AFTER_SECS: i64 = 7 * 24 * 3600; // 7 days

/// Exponential backoff with a cap and +/-20% jitter (so many messages queued for the same
/// unreachable contact don't all retry in perfect lockstep). `attempt_count` is the number of
/// attempts *already* failed for this item before the one that just failed.
fn next_retry_delay_secs(attempt_count: i32) -> i64 {
    let exponent = attempt_count.clamp(0, 16) as u32; // 2^16 already far exceeds the cap
    let doubled = OUTBOX_RETRY_BASE_SECS.saturating_mul(1i64.checked_shl(exponent).unwrap_or(i64::MAX));
    let capped = doubled.min(OUTBOX_RETRY_CAP_SECS);
    let jitter_span = capped / 5; // 20%
    if jitter_span > 0 {
        (capped + rand::thread_rng().gen_range(-jitter_span..=jitter_span)).max(1)
    } else {
        capped.max(1)
    }
}

/// Result of [`NovaEngine::receive_packet`] successfully processing one incoming wire packet.
/// Three of the four possible outcomes have nothing new to show the UI, but for different
/// reasons that matter to the caller — see `attach_network`'s receive loop, which maps each one
/// to a distinct [`nova_transport::DeliveryOutcome`] reported back to the sender.
#[derive(Debug)]
pub enum ReceiveOutcome {
    /// A new message was decrypted and persisted; ready for the UI.
    New(MessageRecord),
    /// Handled successfully at the protocol level but nothing new to show: either an exact
    /// duplicate of an already-persisted message, or a `HandshakeInit` this node deliberately
    /// yielded on because of a simultaneous-initiation collision (see the tie-breaking logic in
    /// `receive_packet`) — the peer's own outbox will redeliver the real content shortly after.
    ProcessedNoOp,
    /// Decrypted fine (the Double Ratchet stayed in sync) but dropped because the sender is a
    /// contact this device has blocked.
    BlockedSender,
}

impl ReceiveOutcome {
    /// The freshly-persisted message, if this outcome represents one — `None` for every other
    /// variant. Convenience for callers (mainly tests) that only care about "was there a new
    /// message to show", mirroring the old `Option<MessageRecord>` return shape.
    pub fn new_message(self) -> Option<MessageRecord> {
        match self {
            ReceiveOutcome::New(m) => Some(m),
            ReceiveOutcome::ProcessedNoOp | ReceiveOutcome::BlockedSender => None,
        }
    }
}

/// In-progress reassembly state for one media message not yet fully received — see
/// `NovaEngine::handle_incoming_media_chunk`.
struct PartialMedia {
    media_meta: Option<nova_protocol::MediaMetadata>,
    /// The sender's actual caption (see `MessagePayload::text_content` on chunk 0) — falls back
    /// to an auto-generated "filename (size)" label if the sender didn't set one.
    caption: Option<String>,
    content_type: nova_protocol::MessageContentType,
    conversation_id: String,
    recipient_id: String,
    timestamp_utc: i64,
    chunks: HashMap<u32, Vec<u8>>,
    created_at: std::time::Instant,
}

pub struct NovaEngine {
    pub identity: Arc<Mutex<Option<DeviceIdentity>>>,
    pub storage: Arc<StorageEngine>,
    pub transport: Arc<TransportSupervisor>,
    pub sessions: Arc<Mutex<HashMap<String, DoubleRatchetSession>>>,
    /// Media messages currently being received but not yet complete, keyed by
    /// `(sender_peer_id, message_id)` — see `handle_incoming_media_chunk`.
    media_reassembly: Mutex<HashMap<(String, String), PartialMedia>>,
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
            media_reassembly: Mutex::new(HashMap::new()),
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
            while let Some(incoming) = recv_node.recv_next().await {
                // Report the real DeliveryOutcome back to the sender over the same
                // request/response round-trip they're waiting on, instead of the transport layer
                // acking receipt before this engine ever attempted to decode anything — see the
                // 2026-08-22 audit's "silent packet drop" finding.
                let outcome = match engine.receive_packet(&incoming.bytes).await {
                    Ok(ReceiveOutcome::New(_)) | Ok(ReceiveOutcome::ProcessedNoOp) => {
                        nova_transport::DeliveryOutcome::Processed
                    }
                    Ok(ReceiveOutcome::BlockedSender) => nova_transport::DeliveryOutcome::Blocked,
                    Err(e) => {
                        warn!("Failed to process an incoming packet: {e}");
                        nova_transport::DeliveryOutcome::Rejected
                    }
                };
                incoming.respond(outcome);
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

        // Without this, this device's `PreKeyBundle` + reachable addresses are never published
        // into the DHT (`P2PNode::announce_presence` was previously only ever called from tests),
        // so every OTHER peer's `connect_to_peer` lookup for this device's nova_peer_id finds
        // nothing and every message to it sits in the outbox forever, retried but never
        // delivered — regardless of whether both sides had already added each other as a contact.
        // Periodic, not one-shot: a `put_record` can fail the very first time (empty routing
        // table on a cold start — see `PRESENCE_ANNOUNCE_INTERVAL`'s doc comment) or simply expire
        // on the DHT, so this keeps retrying/refreshing for the lifetime of the attached network.
        let announce_node = node.clone();
        let announce_engine = self.clone();
        tokio::spawn(async move {
            // `tokio::time::interval` fires its first tick immediately, not after one full
            // period — an explicit sleep first is what actually gives mDNS/bootstrap discovery
            // the head start described above, rather than guaranteeing the first attempt fails.
            tokio::time::sleep(Duration::from_secs(5)).await;
            let mut interval = tokio::time::interval(PRESENCE_ANNOUNCE_INTERVAL);
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
            loop {
                if let Err(e) = announce_node.announce_presence().await {
                    debug!("announce_presence failed, will retry: {e}");
                }
                // Publish directory profile with fast retry on failure (handles Render cold start).
                // On free-tier PaaS hosts the server can take up to 30s to wake up;
                // we retry up to 3 times with 12s gaps before waiting the full interval.
                let mut dir_published = false;
                for attempt in 0..4u32 {
                    match announce_engine.publish_directory_profile().await {
                        Ok(()) => { dir_published = true; break; }
                        Err(e) => {
                            if attempt < 3 {
                                debug!("publish_directory_profile attempt {attempt} failed ({e}), retrying in 12s");
                                tokio::time::sleep(Duration::from_secs(12)).await;
                            } else {
                                debug!("publish_directory_profile failed after 4 attempts: {e}");
                            }
                        }
                    }
                }
                if dir_published {
                    tracing::info!("Directory profile published successfully");
                }
                interval.tick().await;
            }
        });
    }

    /// Explicitly drains pending incoming packets from the fallback relay servers,
    /// decodes and decrypts any received messages, updates conversations and unread counts in storage.
    /// Returns the number of successfully ingested incoming packets.
    pub async fn drain_incoming_from_relay(&self) -> Result<usize, EngineError> {
        let (drain_req, urls) = {
            let id_lock = self.identity.lock().await;
            let id = match id_lock.as_ref() {
                Some(i) => i,
                None => return Ok(0),
            };
            let net_lock = self.network.lock().await;
            let node = match net_lock.as_ref() {
                Some(n) => n.clone(),
                None => return Ok(0),
            };
            let now_sec = chrono::Utc::now().timestamp() as u64;
            let req = nova_protocol::SignedDrainRequest::sign(id, now_sec);
            let urls = node.get_fallback_server_urls().await;
            (req, urls)
        };

        if urls.is_empty() {
            return Ok(0);
        }

        let mut all_packets = Vec::new();
        for url in urls {
            let client = UdpFallbackClient::new(&url);
            if let Ok(packets) = client.drain_incoming_signed(drain_req.clone()).await {
                all_packets.extend(packets);
            }
        }

        let count = all_packets.len();
        for packet_bytes in all_packets {
            if let Err(e) = self.receive_packet(&packet_bytes).await {
                tracing::warn!("Failed to process packet drained from relay: {e}");
            }
        }

        Ok(count)
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
        if pending.is_empty() {
            return Ok(0);
        }

        let futures = pending.into_iter().map(|item| {
            let node = node.clone();
            async move {
                let send_res = match unwrap_chunks_for_outbox(&item.payload) {
                    Ok(chunks) => node.send_chunks_to_peer(&item.recipient_id, chunks).await,
                    Err(e) => Err(nova_transport::TransportError::Protocol(
                        nova_protocol::ProtocolError::DeserializationFailed(e.to_string()),
                    )),
                };
                (item, send_res)
            }
        });

        let results = futures_util::future::join_all(futures).await;
        let mut delivered = 0;
        let now = chrono::Utc::now().timestamp();
        for (item, send_res) in results {
            match send_res {
                // Peer unreachable via the DHT/relay path — defer and retry later. A second,
                // independent delivery attempt is still made via `spawn_udp_fallback_send_chunks`
                // (the WebSocket discovery/relay server, see `crate::udp_fallback`) inside
                // `send_chunks_to_peer` itself on this same failure, so this isn't the only path
                // a message can go out on.
                Ok((nova_transport::P2PTransportMode::Disconnected, _)) => {
                    debug!("Outbox item {} deferred: peer unreachable", item.message_id);
                    self.back_off_or_give_up(&item, now)?;
                }
                Ok((_, outcome)) => self.apply_send_outcome(&item, outcome, &mut delivered, now)?,
                Err(e) => {
                    debug!("Outbox item {} not yet delivered: {e}", item.message_id);
                    self.back_off_or_give_up(&item, now)?;
                }
            }
        }
        Ok(delivered)
    }

    /// Records the outcome of a successful send attempt against `item`: only `Processed` ever
    /// earns "Delivered"; `Blocked` stops retrying without ever claiming delivery (see the
    /// 2026-08-22 audit); `Rejected` backs off and retries like any other failure.
    fn apply_send_outcome(
        &self,
        item: &nova_storage::OutboxItem,
        outcome: nova_transport::DeliveryOutcome,
        delivered: &mut usize,
        now: i64,
    ) -> Result<(), EngineError> {
        match outcome {
            nova_transport::DeliveryOutcome::Processed => {
                self.storage.remove_from_outbox(&item.message_id)?;
                let _ = self.storage.update_message_status(&item.message_id, DbMessageStatus::Delivered);
                debug!("Delivered outbox item {}", item.message_id);
                *delivered += 1;
            }
            nova_transport::DeliveryOutcome::Blocked => {
                self.storage.remove_from_outbox(&item.message_id)?;
                debug!("Outbox item {} reached a contact who has this device blocked — not marked Delivered", item.message_id);
            }
            nova_transport::DeliveryOutcome::Rejected => {
                debug!("Outbox item {} was rejected by the recipient's engine, will retry", item.message_id);
                self.back_off_or_give_up(item, now)?;
            }
        }
        Ok(())
    }

    /// Applies exponential backoff to `item` for another retry, or — once it has been failing
    /// for longer than `OUTBOX_GIVE_UP_AFTER_SECS` — removes it from the outbox and marks the
    /// underlying message `Failed` so the UI can surface a manual retry action instead of
    /// retrying silently and indefinitely.
    fn back_off_or_give_up(&self, item: &nova_storage::OutboxItem, now: i64) -> Result<(), EngineError> {
        let failing_for = now.saturating_sub(item.first_attempt_utc);
        if failing_for >= OUTBOX_GIVE_UP_AFTER_SECS {
            self.storage.remove_from_outbox(&item.message_id)?;
            let _ = self.storage.update_message_status(&item.message_id, DbMessageStatus::Failed);
            debug!(
                "Outbox item {} gave up after {failing_for}s of failed attempts — marked Failed",
                item.message_id
            );
        } else {
            let delay = next_retry_delay_secs(item.attempt_count);
            self.storage.record_outbox_retry(&item.message_id, now + delay)?;
            debug!(
                "Outbox item {} will retry in {delay}s (attempt {})",
                item.message_id,
                item.attempt_count + 1
            );
        }
        Ok(())
    }

    /// Create a new sovereign account with a 12-word mnemonic phrase, provisioning the X3DH
    /// prekeys this device needs before anyone can establish a session with it.
    pub async fn create_account(&self, username: &str) -> Result<(String, String), EngineError> {
        // Storage's `save_identity` is `INSERT OR REPLACE` (there is only ever one identity row)
        // — without this guard, creating an account while one already exists would silently
        // destroy its keys and mnemonic with no way back. `clear_identity` first if that is
        // genuinely intended.
        if self.storage.load_identity()?.is_some() {
            return Err(EngineError::IdentityAlreadyExists);
        }
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
        // Same `INSERT OR REPLACE` hazard as `create_account` — see the comment there.
        if self.storage.load_identity()?.is_some() {
            return Err(EngineError::IdentityAlreadyExists);
        }
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

    /// Loads a previously created/restored identity straight from local encrypted storage,
    /// without needing the mnemonic re-entered — the entire point of persisting it in the first
    /// place. Returns `Ok(None)` if this device has never created or restored an account (a
    /// genuine first run), which is a normal outcome, not an error. Returns the mnemonic
    /// alongside the peer_id/username because the caller typically also needs to start a
    /// `P2PNode`, which requires an owned `DeviceIdentity` — cheaper to re-derive one from the
    /// mnemonic (as every other identity-establishing path in this engine already does) than to
    /// thread a second reference through `self.identity`'s mutex.
    pub async fn try_resume_session(&self) -> Result<Option<(String, String, String)>, EngineError> {
        let Some(identity) = self.storage.load_identity()? else {
            return Ok(None);
        };
        let pub_hex = identity.public_id_hex();
        let username = identity.username.clone();
        if self.storage.load_active_signed_prekey()?.is_none() {
            self.provision_prekeys(&identity)?;
        }
        *self.identity.lock().await = Some(identity);

        let mnemonic = self
            .storage
            .get_identity_mnemonic()?
            .ok_or(EngineError::NoIdentity)?;
        Ok(Some((pub_hex, mnemonic, username)))
    }

    /// Logs this device out: wipes the local identity, its prekeys, and every contact/
    /// conversation/message tied to it (see `StorageEngine::clear_all_identity_data`), then
    /// clears the in-memory identity so a following `create_account`/`restore_account` starts
    /// from a genuinely clean slate instead of `REPLACE`-ing still-needed key material. Detaches
    /// the engine from any already-running `P2PNode` (see `attach_network`), but does not shut
    /// that node down — its background tasks simply idle, orphaned, until the app restarts; the
    /// caller is expected to allow a fresh `attach_network` for the next identity.
    pub async fn clear_identity(&self) -> Result<(), EngineError> {
        self.storage.clear_all_identity_data()?;
        *self.identity.lock().await = None;
        *self.network.lock().await = None;
        Ok(())
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

    /// Generates a signed, time-limited contact invitation URI (`nova://invite?d=...`) containing
    /// this device's X3DH prekey bundle and dialable network rendezvous addresses.
    pub async fn get_own_invitation_uri(&self, ttl_seconds: Option<i64>) -> Result<String, EngineError> {
        let id_lock = self.identity.lock().await;
        let identity = id_lock.as_ref().ok_or(EngineError::NoIdentity)?;
        let bundle = self.storage.get_own_prekey_bundle(identity)?;
        let ttl = ttl_seconds.unwrap_or(86400); // 24 hours default
        let addrs = self.own_full_listen_addrs().await;
        let invitation = nova_protocol::SignedContactInvitation::create(identity, bundle, ttl, addrs)?;
        Ok(invitation.to_uri()?)
    }

    /// Adds a new contact from their invitation URI or serialized prekey bundle.
    /// The ticket's signature, expiry deadline, and cryptographic bundle are verified
    /// before adding. Any embedded rendezvous addresses are automatically dialed in the background.
    pub async fn add_contact(
        &self,
        username: &str,
        display_name: &str,
        invitation_code_or_bytes: &[u8],
    ) -> Result<ContactRecord, EngineError> {
        let now_utc = chrono::Utc::now().timestamp();
        let (bundle, rendezvous_addrs) = match std::str::from_utf8(invitation_code_or_bytes) {
            Ok(inv_str) => match ParsedInvitation::parse(inv_str.trim()) {
                Ok(ParsedInvitation::Signed(ticket)) => {
                    let payload = ticket.verify(now_utc)?;
                    (payload.bundle, payload.rendezvous_addrs)
                }
                Ok(ParsedInvitation::LegacyBundle(bundle)) => {
                    verify_prekey_bundle(&bundle)?;
                    (bundle, Vec::new())
                }
                // Not a recognizable invitation code at all (e.g. raw CBOR bytes, peer_id, or @username)
                Err(_) => {
                    let trimmed = inv_str.trim();
                    let resolved_bundle = if !trimmed.is_empty() {
                        let search_res = self.search_directory(trimmed).await.unwrap_or_default();
                        if let Some(matching) = search_res.into_iter().find(|u| {
                            u.peer_id.eq_ignore_ascii_case(trimmed)
                                || u.username.eq_ignore_ascii_case(trimmed.trim_start_matches('@'))
                        }) {
                            if let Ok(bytes) = hex::decode(&matching.prekey_bundle_hex) {
                                bundle_from_raw_bytes(&bytes).ok()
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    match resolved_bundle {
                        Some(b) => (b, Vec::new()),
                        None => {
                            let raw_bytes = if let Ok(s) = std::str::from_utf8(invitation_code_or_bytes) {
                                hex::decode(s.trim()).unwrap_or_else(|_| invitation_code_or_bytes.to_vec())
                            } else {
                                invitation_code_or_bytes.to_vec()
                            };
                            (bundle_from_raw_bytes(&raw_bytes)?, Vec::new())
                        }
                    }
                }
            },
            // Raw CBOR bytes are usually not valid UTF-8 at all (e.g. the output of
            // `get_own_prekey_bundle_bytes` passed through unmodified) — go straight to the
            // unwrapped PreKeyBundle path rather than through the text-oriented parser.
            Err(_) => (bundle_from_raw_bytes(invitation_code_or_bytes)?, Vec::new()),
        };

        let own_pub = {
            let id_lock = self.identity.lock().await;
            id_lock
                .as_ref()
                .map(|id| id.verifying_key_bytes)
                .ok_or(EngineError::NoIdentity)?
        };

        let peer_id = hex::encode(bundle.identity_ed25519_pub);
        let safety_number = compute_safety_number(&own_pub, &bundle.identity_ed25519_pub)?;

        let existing_contact = self.storage.get_contact(&peer_id).ok().flatten();
        let is_trusted = existing_contact.map(|c| c.is_trusted).unwrap_or(false);

        let contact = ContactRecord {
            peer_id: peer_id.clone(),
            username: username.to_string(),
            display_name: display_name.to_string(),
            prekey_bundle: bundle,
            safety_number,
            is_online: true,
            is_blocked: false,
            is_trusted,
            last_seen_utc: now_utc,
        };
        self.storage.save_contact(&contact)?;

        // If a conversation already exists with this peer, update its title with the new display name.
        // Do NOT create an empty ghost conversation just because a contact was added (Google Messages model).
        let conv_id = format!("conv_{peer_id}");
        if let Ok(convos) = self.storage.get_conversations() {
            if let Some(mut existing) = convos.into_iter().find(|c| c.id == conv_id) {
                existing.title = display_name.to_string();
                let _ = self.storage.save_conversation(&existing);
            }
        }

        // Automatically dial the friend's rendezvous/relay addresses in the background — a
        // best-effort convenience (adding the contact must still succeed even if this device is
        // currently unreachable to them, e.g. no network at all), but a failure here used to be
        // discarded with no trace whatsoever, not even a log line — impossible to diagnose why a
        // freshly-added contact never connects (this address is almost always the peer's own LAN
        // IP, which is simply unreachable across different Wi-Fi/cellular networks; see the
        // WebSocket discovery/relay fallback in `crate::udp_fallback`, which is what actually
        // recovers that case now).
        for addr in rendezvous_addrs {
            if let Err(e) = self.bootstrap_dial(&addr).await {
                tracing::warn!("Initial rendezvous dial to {addr} failed (will retry via the outbox/relay fallback when a message is sent): {e}");
            }
        }

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
        self.encrypt_and_enqueue(&msg_id, conversation_id, recipient_peer_id, text).await
    }

    /// Re-attempts delivery of a message that previously exhausted the outbox's retry window and
    /// was marked `Failed` (see `back_off_or_give_up`) — the UI's manual "Réessayer" action.
    /// Re-encrypts the original text/attachment under the current session and re-queues it under
    /// its ORIGINAL message id (so it keeps its place in the conversation history instead of
    /// appearing as a duplicate), with a fresh attempt counter and give-up clock.
    ///
    /// If NOTHING has ever been confirmed `Delivered` to this recipient, this device's local
    /// session state cannot be trusted to reflect what they actually have: their very first
    /// X3DH handshake — carried only on the original send — may never have reached them at all.
    /// In that case the local session is discarded first, forcing a fresh handshake on retry;
    /// otherwise the retry would send a plain `EncryptedMessage` frame the recipient has no
    /// session to decrypt, and they would never receive anything at all. Caught by
    /// `test_retry_failed_media_message_resends_the_actual_attachment` during the 2026-08-22
    /// audit's J4 work.
    pub async fn retry_failed_message(
        &self,
        conversation_id: &str,
        recipient_peer_id: &str,
        message_id: &str,
    ) -> Result<MessageRecord, EngineError> {
        let all_messages = self.storage.get_messages(conversation_id)?;
        let existing = all_messages
            .iter()
            .find(|m| m.id == message_id && m.is_outgoing)
            .ok_or_else(|| EngineError::MessageNotFound(message_id.to_string()))?
            .clone();

        let ever_delivered = all_messages
            .iter()
            .any(|m| m.is_outgoing && m.status == DbMessageStatus::Delivered);
        if !ever_delivered {
            self.sessions.lock().await.remove(recipient_peer_id);
            self.storage.delete_session(recipient_peer_id)?;
        }

        match existing.attachment {
            None => {
                self.encrypt_and_enqueue(message_id, conversation_id, recipient_peer_id, &existing.text_content)
                    .await
            }
            Some(attachment) => {
                let bytes = self
                    .storage
                    .get_attachment_blob(message_id)?
                    .ok_or_else(|| EngineError::MessageNotFound(message_id.to_string()))?;
                self.encrypt_and_enqueue_media(
                    message_id,
                    conversation_id,
                    recipient_peer_id,
                    existing.content_type,
                    attachment.file_name,
                    attachment.mime_type,
                    bytes,
                    existing.text_content,
                )
                .await
            }
        }
    }

    /// Shared by [`send_message`](Self::send_message) (fresh UUID) and
    /// [`retry_failed_message`](Self::retry_failed_message) (reuses the original id): encrypts
    /// `text` for `recipient_peer_id` (performing an X3DH handshake first if no session exists
    /// yet) and queues the resulting wire packet in the Outbox.
    async fn encrypt_and_enqueue(
        &self,
        msg_id: &str,
        conversation_id: &str,
        recipient_peer_id: &str,
        text: &str,
    ) -> Result<MessageRecord, EngineError> {
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
            msg_id.to_string(),
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
        let (frame_type, packet_payload) = frame_to_packet_payload(
            handshake_material.as_ref(),
            identity_ed25519_pub,
            identity_x25519_pub,
            frame,
        )?;
        let packet = NovaPacket::new(frame_type, sender_id.clone(), packet_payload);
        let packet_cbor = packet.to_cbor()?;

        let msg_record = MessageRecord {
            id: msg_id.to_string(),
            conversation_id: conversation_id.to_string(),
            sender_id,
            recipient_id: recipient_peer_id.to_string(),
            text_content: text.to_string(),
            timestamp_utc: now,
            status: DbMessageStatus::Sent,
            is_outgoing: true,
            content_type: nova_protocol::MessageContentType::Text,
            attachment: None,
        };

        let mut conv = match self.storage.get_conversations()?.into_iter().find(|c| c.id == conversation_id) {
            Some(c) => c,
            None => {
                let title = self.storage.get_contact(recipient_peer_id).ok().flatten()
                    .map(|c| if !c.display_name.is_empty() { c.display_name } else { c.username })
                    .filter(|s| !s.is_empty())
                    .unwrap_or_else(|| {
                        if recipient_peer_id.len() >= 12 {
                            format!("Pair {}...{}", &recipient_peer_id[..6], &recipient_peer_id[recipient_peer_id.len()-4..])
                        } else {
                            recipient_peer_id.to_string()
                        }
                    });
                ConversationRecord {
                    id: conversation_id.to_string(),
                    peer_id: recipient_peer_id.to_string(),
                    title,
                    last_message_text: text.to_string(),
                    last_message_time_utc: now,
                    unread_count: 0,
                }
            }
        };
        conv.last_message_text = text.to_string();
        conv.last_message_time_utc = now;
        self.storage.save_conversation(&conv)?;

        self.storage.save_message(&msg_record)?;
        let wrapped = wrap_chunks_for_outbox(&[packet_cbor])?;
        self.storage
            .enqueue_outbox(msg_id, conversation_id, recipient_peer_id, &wrapped)?;

        Ok(msg_record)
    }

    /// Sends a media message (image/audio/file/video) to a contact: computes an integrity
    /// checksum, splits `bytes` into `nova_protocol::MEDIA_CHUNK_SIZE`-sized pieces (one packet
    /// each, all sharing one Double Ratchet session so out-of-order chunk delivery decrypts
    /// exactly as correctly as any other out-of-order message — see `nova-crypto::ratchet`'s
    /// skipped-key handling), and queues them as a single outbox entry delivered together (see
    /// `nova-transport::P2PNode::send_chunks_to_peer`). `caption` is the short text shown in the
    /// conversation list/bubble (e.g. a filename or user-typed caption) — never the file itself.
    #[allow(clippy::too_many_arguments)]
    pub async fn send_media(
        &self,
        conversation_id: &str,
        recipient_peer_id: &str,
        content_type: nova_protocol::MessageContentType,
        file_name: String,
        mime_type: String,
        bytes: Vec<u8>,
        caption: String,
    ) -> Result<MessageRecord, EngineError> {
        let msg_id = Uuid::new_v4().to_string();
        self.encrypt_and_enqueue_media(&msg_id, conversation_id, recipient_peer_id, content_type, file_name, mime_type, bytes, caption)
            .await
    }

    /// Shared by [`send_media`](Self::send_media) (fresh UUID) and
    /// [`retry_failed_message`](Self::retry_failed_message) (reuses the original id) — see
    /// [`send_media`](Self::send_media)'s docs for the chunking behavior.
    #[allow(clippy::too_many_arguments)]
    async fn encrypt_and_enqueue_media(
        &self,
        msg_id: &str,
        conversation_id: &str,
        recipient_peer_id: &str,
        content_type: nova_protocol::MessageContentType,
        file_name: String,
        mime_type: String,
        bytes: Vec<u8>,
        caption: String,
    ) -> Result<MessageRecord, EngineError> {
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

        if bytes.len() as u64 > 100 * 1024 * 1024 {
            return Err(EngineError::Protocol(nova_protocol::ProtocolError::PacketTooLarge(
                bytes.len(),
            )));
        }

        let sha256_checksum = {
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(&bytes);
            hex::encode(hasher.finalize())
        };
        let chunk_count = bytes.len().div_ceil(nova_protocol::MEDIA_CHUNK_SIZE).max(1) as u32;

        let handshake_material = self.ensure_session_for_send(recipient_peer_id).await?;

        let mut packets: Vec<Vec<u8>> = Vec::with_capacity(chunk_count as usize);
        {
            let mut sessions_lock = self.sessions.lock().await;
            let session = sessions_lock
                .get_mut(recipient_peer_id)
                .expect("session was just established or loaded above");

            for i in 0..chunk_count {
                let start = i as usize * nova_protocol::MEDIA_CHUNK_SIZE;
                let end = ((i as usize + 1) * nova_protocol::MEDIA_CHUNK_SIZE).min(bytes.len());
                let chunk_slice = bytes[start..end].to_vec();
                let media_meta = (i == 0).then(|| nova_protocol::MediaMetadata {
                    file_name: file_name.clone(),
                    mime_type: mime_type.clone(),
                    size_bytes: bytes.len() as u64,
                    sha256_checksum: sha256_checksum.clone(),
                    chunk_count,
                });

                let mut payload = MessagePayload::new_media_chunk(
                    msg_id.to_string(),
                    conversation_id.to_string(),
                    sender_id.clone(),
                    recipient_peer_id.to_string(),
                    content_type.clone(),
                    media_meta,
                    i,
                    chunk_slice,
                );
                // The caption travels once, alongside media_meta on chunk 0 — otherwise the
                // recipient has no way to know what the sender actually wrote and falls back to
                // an auto-generated "filename (size)" label (see handle_incoming_media_chunk),
                // silently discarding a real caption. Caught by
                // test_send_and_receive_media_message_multi_chunk_reassembly.
                if i == 0 {
                    payload.text_content = Some(caption.clone());
                }
                let payload_bytes = payload.to_bytes()?;
                let (header, ciphertext) = session.ratchet_encrypt(&payload_bytes, b"NOVA_MSG")?;
                // Persisted after every chunk, exactly like a text message — a restart mid-send
                // must not desynchronize the ratchet for messages that already went out.
                self.storage.save_session(recipient_peer_id, session)?;

                let frame = EncryptedFrame { header, ciphertext };
                // The X3DH handshake, if this is a first-ever message to this peer, travels once
                // on chunk 0 only — every later chunk is a plain EncryptedMessage on the now-
                // established session.
                let this_chunk_handshake = if i == 0 { handshake_material.as_ref() } else { None };
                let (frame_type, packet_payload) =
                    frame_to_packet_payload(this_chunk_handshake, identity_ed25519_pub, identity_x25519_pub, frame)?;
                let packet = NovaPacket::new(frame_type, sender_id.clone(), packet_payload);
                packets.push(packet.to_cbor()?);
            }
        }

        let msg_record = MessageRecord {
            id: msg_id.to_string(),
            conversation_id: conversation_id.to_string(),
            sender_id,
            recipient_id: recipient_peer_id.to_string(),
            text_content: caption.clone(),
            timestamp_utc: now,
            status: DbMessageStatus::Sent,
            is_outgoing: true,
            content_type,
            attachment: Some(nova_storage::AttachmentMeta {
                mime_type,
                file_name,
                size_bytes: bytes.len() as u64,
            }),
        };

        let mut conv = match self.storage.get_conversations()?.into_iter().find(|c| c.id == conversation_id) {
            Some(c) => c,
            None => {
                let title = self.storage.get_contact(recipient_peer_id).ok().flatten()
                    .map(|c| if !c.display_name.is_empty() { c.display_name } else { c.username })
                    .filter(|s| !s.is_empty())
                    .unwrap_or_else(|| {
                        if recipient_peer_id.len() >= 12 {
                            format!("Pair {}...{}", &recipient_peer_id[..6], &recipient_peer_id[recipient_peer_id.len()-4..])
                        } else {
                            recipient_peer_id.to_string()
                        }
                    });
                ConversationRecord {
                    id: conversation_id.to_string(),
                    peer_id: recipient_peer_id.to_string(),
                    title,
                    last_message_text: caption.clone(),
                    last_message_time_utc: now,
                    unread_count: 0,
                }
            }
        };
        conv.last_message_text = caption.clone();
        conv.last_message_time_utc = now;
        self.storage.save_conversation(&conv)?;

        self.storage.save_message_with_attachment(&msg_record, &sha256_checksum, &bytes)?;

        let wrapped = wrap_chunks_for_outbox(&packets)?;
        self.storage
            .enqueue_outbox(msg_id, conversation_id, recipient_peer_id, &wrapped)?;

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
    ///
    /// The `Ok` variant distinguishes *why* there may be nothing new to show the UI (duplicate,
    /// yielded handshake collision, or a blocked sender) — callers that only care about "is there
    /// a new message" can use [`ReceiveOutcome::new_message`], but `attach_network`'s background
    /// receive loop needs the finer distinction to report an accurate [`nova_transport::DeliveryOutcome`]
    /// back to the sender (see the 2026-08-22 audit's blocked-contact and silent-drop findings).
    pub async fn receive_packet(&self, raw_packet: &[u8]) -> Result<ReceiveOutcome, EngineError> {
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
                let own_pub = identity.verifying_key_bytes;
                let peer_pub = handshake.sender_identity_ed25519_pub;
                let own_id_hex = identity.public_id_hex();

                // Simultaneous handshake collision detection (Double Alice Problem):
                // If both peers initiated a session concurrently, use deterministic tie-breaking
                // (lexicographical order of Ed25519 public keys) to pick the authoritative initiator.
                let mut sessions_lock = self.sessions.lock().await;
                let has_unacked_initiator_session = match sessions_lock.get(&sender_peer_id) {
                    Some(s) => s.can_send(),
                    None => self
                        .storage
                        .load_session(&sender_peer_id)?
                        .map(|s| s.can_send())
                        .unwrap_or(false),
                };

                if has_unacked_initiator_session && own_pub < peer_pub {
                    tracing::info!(
                        "Handshake collision with {sender_peer_id}: local node is priority initiator, preserving local session."
                    );
                    return Ok(ReceiveOutcome::ProcessedNoOp);
                }

                let (spk_secret, spk_public) = self
                    .storage
                    .load_active_signed_prekey()?
                    .ok_or(EngineError::NoPrekeys)?;
                if spk_public.key_id != handshake.used_signed_prekey_id {
                    return Err(EngineError::HandshakeFailed(
                        "handshake references a signed prekey id we no longer have".into(),
                    ));
                }

                // If the handshake names a one-time prekey, that exact key MUST still be ours to
                // consume: `x3dh_respond` below derives the shared secret differently depending on
                // whether an OTP is used, so silently treating "already consumed" the same as
                // "sender never used one" would make us derive a *different* secret than the
                // sender did — not an error, just silently-wrong decryption downstream (every
                // `ratchet_decrypt` on this session fails its AEAD tag check forever after,
                // indistinguishable from tampering, and the sender's outbox retries it forever with
                // no way to ever succeed). This is reachable in completely normal use: an invitation
                // link/QR is deliberately reusable for its whole 24h TTL (see
                // `SignedContactInvitation`), so the same one-time prekey it embeds can legitimately
                // be read and used by two different people who add this device around the same
                // time — whichever handshake we process first consumes it, and the second must fail
                // loudly right here instead of limping on with a broken session.
                let opk_secret = match handshake.used_one_time_prekey_id {
                    Some(id) => Some(self.storage.consume_one_time_prekey_secret(id)?.ok_or_else(|| {
                        EngineError::HandshakeFailed(format!(
                            "one-time prekey {id} was already used by another handshake (this invitation link may have been shared with more than one person) — ask {sender_peer_id} to send a fresh invitation"
                        ))
                    })?),
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

                // If we had an outbound message pending in the outbox that was previously
                // encrypted with an obsolete initiator session, re-encrypt it on this new session.
                if let Err(e) = self.reencrypt_pending_outbox_items(&own_id_hex, &sender_peer_id, &mut session) {
                    tracing::warn!("Failed to re-encrypt pending outbox items for {sender_peer_id}: {e}");
                }

                self.storage.save_session(&sender_peer_id, &session)?;
                sessions_lock.insert(sender_peer_id.clone(), session);
                drop(sessions_lock);

                // If contact is blocked, keep ratchet in sync but drop UI persistence
                if self.storage.is_contact_blocked(&sender_peer_id)? {
                    tracing::info!("Packet processed for ratchet sync but dropped because peer {} is blocked", sender_peer_id);
                    return Ok(ReceiveOutcome::BlockedSender);
                }

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

                // If contact is blocked, keep ratchet in sync but drop UI persistence
                if self.storage.is_contact_blocked(&sender_peer_id)? {
                    tracing::info!("Packet processed for ratchet sync but dropped because peer {} is blocked", sender_peer_id);
                    return Ok(ReceiveOutcome::BlockedSender);
                }

                self.persist_incoming_message(&sender_peer_id, &plaintext).await
            }
            other => Err(EngineError::UnsupportedFrame(other)),
        }
    }

    async fn persist_incoming_message(
        &self,
        sender_peer_id: &str,
        plaintext: &[u8],
    ) -> Result<ReceiveOutcome, EngineError> {
        let payload = MessagePayload::from_bytes(plaintext)?;
        // Deliberately NOT `payload.conversation_id`: that field is whatever the *sender's* own
        // device called its side of this conversation (`conv_<this contact's peer_id>`, from the
        // sender's perspective — see `add_contact`/app.js's identical `'conv_' + peerId`
        // convention) — which, from THIS device's perspective, is `conv_<my own peer_id>`, not a
        // conversation with the sender at all. Trusting it verbatim here filed every incoming
        // "first message from a new contact" under a conversation id neither `add_contact` (run
        // when this device added that contact) nor the real UI (`state.activeContact.conversationId
        // = 'conv_' + contact.peerId`) would ever look up — the message was persisted but
        // invisible in the actual chat screen. Always derive the LOCAL, receiver's-own-convention
        // id from the sender's peer_id instead, exactly like `add_contact` does when this device
        // is the one initiating. Caught by the 2026-08-25 three-device mesh test: it showed real
        // `Delivered` status on the sender's outbox while the recipient's own `get_messages` on
        // its natural conversation id never found anything.
        // Check if this is a group control action or group message
        if payload.content_type == MessageContentType::GroupControl {
            if let Some(text) = payload.text_content.as_ref() {
                if let Ok(action) = serde_json::from_str::<GroupControlAction>(text) {
                    self.handle_incoming_group_control(sender_peer_id, action)?;
                    return Ok(ReceiveOutcome::ProcessedNoOp);
                }
            }
        }

        if payload.content_type == MessageContentType::GroupMessage {
            if let Some(text) = payload.text_content.as_ref() {
                if let Ok(env) = serde_json::from_str::<GroupMessageEnvelope>(text) {
                    return self.handle_incoming_group_message(sender_peer_id, env).await;
                }
            }
        }

        let conv_id = format!("conv_{sender_peer_id}");

        // Check if message was already persisted to avoid duplicate notifications / double unread
        // counts — covers both a genuine text-message duplicate and an already-fully-reassembled
        // media message whose sender is retrying (e.g. it never saw the final chunk's ack).
        let existing = self.storage.get_messages(&conv_id)?;
        if existing.iter().any(|m| m.id == payload.message_id) {
            return Ok(ReceiveOutcome::ProcessedNoOp);
        }

        if payload.chunk_index.is_some() {
            return self.handle_incoming_media_chunk(sender_peer_id, payload).await;
        }

        let text = payload.text_content.clone().unwrap_or_default();
        let mut conv = self.conversation_for_incoming(&conv_id, sender_peer_id, payload.timestamp_utc)?;
        conv.unread_count += 1;
        conv.last_message_text = text.clone();
        conv.last_message_time_utc = payload.timestamp_utc;

        // Auto-resolve contact info if sender is not yet in contacts
        if self.storage.get_contact(sender_peer_id).ok().flatten().is_none() {
            let sender_id_str = sender_peer_id.to_string();
            let network = self.network.lock().await.clone();
            let storage = self.storage.clone();
            tokio::spawn(async move {
                if let Some(node) = network {
                    if let Ok(results) = node.search_directory(&sender_id_str).await {
                        if let Some(matching) = results.into_iter().find(|u| u.peer_id.eq_ignore_ascii_case(&sender_id_str)) {
                            if let Ok(bundle_bytes) = hex::decode(&matching.prekey_bundle_hex) {
                                if let Ok(bundle) = bundle_from_raw_bytes(&bundle_bytes) {
                                    let now = chrono::Utc::now().timestamp();
                                    let contact = nova_storage::ContactRecord {
                                        peer_id: sender_id_str.clone(),
                                        username: matching.username.clone(),
                                        display_name: matching.display_name.clone(),
                                        prekey_bundle: bundle,
                                        safety_number: String::new(),
                                        is_online: matching.is_online,
                                        is_blocked: false,
                                        is_trusted: false,
                                        last_seen_utc: now,
                                    };
                                    let _ = storage.save_contact(&contact);
                                    let conv_id = format!("conv_{sender_id_str}");
                                    if let Ok(Some(mut c)) = storage.get_conversations().map(|cs| cs.into_iter().find(|c| c.id == conv_id)) {
                                        c.title = matching.display_name;
                                        let _ = storage.save_conversation(&c);
                                    }
                                }
                            }
                        }
                    }
                }
            });
        }

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
            content_type: nova_protocol::MessageContentType::Text,
            attachment: None,
        };
        self.storage.save_message(&msg_record)?;

        Ok(ReceiveOutcome::New(msg_record))
    }

    fn conversation_for_incoming(
        &self,
        conv_id: &str,
        sender_peer_id: &str,
        timestamp_utc: i64,
    ) -> Result<ConversationRecord, EngineError> {
        let existing = self
            .storage
            .get_conversations()?
            .into_iter()
            .find(|c| c.id == conv_id);

        if let Some(conv) = existing {
            return Ok(conv);
        }

        let contact_name = self
            .storage
            .get_contact(sender_peer_id)
            .ok()
            .flatten()
            .map(|c| {
                if !c.display_name.is_empty() {
                    c.display_name
                } else if !c.username.is_empty() {
                    format!("@{}", c.username)
                } else {
                    c.peer_id
                }
            })
            .unwrap_or_else(|| {
                if sender_peer_id.len() >= 12 {
                    format!("Pair {}...{}", &sender_peer_id[..6], &sender_peer_id[sender_peer_id.len()-4..])
                } else {
                    sender_peer_id.to_string()
                }
            });

        Ok(ConversationRecord {
            id: conv_id.to_string(),
            peer_id: sender_peer_id.to_string(),
            title: contact_name,
            last_message_text: String::new(),
            last_message_time_utc: timestamp_utc,
            unread_count: 0,
        })
    }

    /// Accumulates one media chunk into the in-memory reassembly buffer for its `message_id`,
    /// persisting the complete file (see `nova_storage::StorageEngine::save_message_with_attachment`)
    /// and returning `ReceiveOutcome::New` only once every chunk `media_meta.chunk_count`
    /// promised has arrived and the reassembled bytes pass their SHA-256 checksum. Purely
    /// in-memory and keyed by `(sender_peer_id, message_id)`: if the app restarts mid-transfer,
    /// the partial buffer is lost, but the sender's own outbox item is never marked `Delivered`
    /// until the LAST chunk is acked (see `pump_outbox_once`), so it simply retries the whole
    /// message from scratch — no persisted partial-chunk state is needed for correctness.
    async fn handle_incoming_media_chunk(
        &self,
        sender_peer_id: &str,
        payload: MessagePayload,
    ) -> Result<ReceiveOutcome, EngineError> {
        let chunk_index = payload.chunk_index.expect("caller already checked chunk_index.is_some()");
        let key = (sender_peer_id.to_string(), payload.message_id.clone());

        let mut buffers = self.media_reassembly.lock().await;

        // Evict expired reassemblies (older than 10 minutes)
        let now_instant = std::time::Instant::now();
        buffers.retain(|_, partial| now_instant.duration_since(partial.created_at).as_secs() < 600);

        if let Some(ref meta) = payload.media_meta {
            if meta.size_bytes > 100 * 1024 * 1024 {
                return Err(EngineError::Protocol(nova_protocol::ProtocolError::PacketTooLarge(
                    meta.size_bytes as usize,
                )));
            }
        }

        let entry = buffers.entry(key.clone()).or_insert_with(|| PartialMedia {
            media_meta: None,
            caption: None,
            content_type: payload.content_type.clone(),
            // See the identical fix (and its rationale) in `persist_incoming_message` just above —
            // `payload.conversation_id` is the sender's own local id, not ours.
            conversation_id: format!("conv_{sender_peer_id}"),
            recipient_id: payload.recipient_id.clone(),
            timestamp_utc: payload.timestamp_utc,
            chunks: HashMap::new(),
            created_at: now_instant,
        });

        if let Some(meta) = payload.media_meta {
            entry.media_meta = Some(meta);
        }
        // The sender's actual caption, carried once on chunk 0 alongside media_meta (see
        // encrypt_and_enqueue_media) — falls back to an auto-generated label below only if the
        // sender genuinely didn't set one.
        if let Some(caption) = payload.text_content {
            entry.caption = Some(caption);
        }
        if let Some(bytes) = payload.chunk_bytes {
            entry.chunks.insert(chunk_index, bytes);
        }

        let Some(meta) = entry.media_meta.clone() else {
            // Chunk 0 (which carries media_meta) hasn't arrived yet — nothing to reassemble
            // against, even if a later chunk arrived first over an out-of-order path.
            return Ok(ReceiveOutcome::ProcessedNoOp);
        };
        if entry.chunks.len() < meta.chunk_count as usize {
            return Ok(ReceiveOutcome::ProcessedNoOp);
        }

        // Every chunk is in — reassemble in index order, verify integrity, and persist.
        let entry = buffers.remove(&key).expect("just confirmed this key is present");
        drop(buffers);

        let mut assembled = Vec::with_capacity(meta.size_bytes as usize);
        for i in 0..meta.chunk_count {
            let Some(chunk) = entry.chunks.get(&i) else {
                // Contradicts chunk_count having been reached — treat as corrupt rather than panic.
                return Err(EngineError::Protocol(nova_protocol::ProtocolError::DeserializationFailed(
                    format!("media message {} missing chunk {i} despite reaching chunk_count", key.1),
                )));
            };
            assembled.extend_from_slice(chunk);
        }

        let actual_checksum = {
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(&assembled);
            hex::encode(hasher.finalize())
        };
        if actual_checksum != meta.sha256_checksum {
            return Err(EngineError::Protocol(nova_protocol::ProtocolError::DeserializationFailed(format!(
                "media message {} failed checksum verification after reassembly (expected {}, got {actual_checksum})",
                key.1, meta.sha256_checksum
            ))));
        }

        let mut conv = self.conversation_for_incoming(&entry.conversation_id, sender_peer_id, entry.timestamp_utc)?;
        let caption = entry
            .caption
            .clone()
            .unwrap_or_else(|| format!("{} ({})", meta.file_name, format_bytes(meta.size_bytes)));
        conv.unread_count += 1;
        conv.last_message_text = caption.clone();
        conv.last_message_time_utc = entry.timestamp_utc;
        self.storage.save_conversation(&conv)?;

        let msg_record = MessageRecord {
            id: key.1,
            conversation_id: entry.conversation_id,
            sender_id: sender_peer_id.to_string(),
            recipient_id: entry.recipient_id,
            text_content: caption,
            timestamp_utc: entry.timestamp_utc,
            status: DbMessageStatus::Delivered,
            is_outgoing: false,
            content_type: entry.content_type,
            attachment: Some(nova_storage::AttachmentMeta {
                mime_type: meta.mime_type,
                file_name: meta.file_name,
                size_bytes: meta.size_bytes,
            }),
        };
        self.storage
            .save_message_with_attachment(&msg_record, &meta.sha256_checksum, &assembled)?;

        Ok(ReceiveOutcome::New(msg_record))
    }

    /// When a handshake collision occurs and this node yields to the peer's authoritative
    /// handshake, any outgoing messages that were pre-encrypted on the obsolete initiator session
    /// must be re-encrypted using the newly established Double Ratchet session so they can be
    /// cleanly delivered.
    fn reencrypt_pending_outbox_items(
        &self,
        sender_id: &str,
        recipient_peer_id: &str,
        session: &mut DoubleRatchetSession,
    ) -> Result<(), EngineError> {
        let pending = self.storage.get_pending_outbox()?;
        let conv_id = format!("conv_{recipient_peer_id}");
        for item in pending {
            if item.recipient_id != recipient_peer_id {
                continue;
            }
            let messages = self.storage.get_messages(&conv_id)?;
            let Some(msg) = messages.iter().find(|m| m.id == item.message_id) else { continue };

            let packets: Vec<Vec<u8>> = match &msg.attachment {
                None => {
                    let payload = MessagePayload::new_text(
                        msg.id.clone(),
                        conv_id.clone(),
                        sender_id.to_string(),
                        recipient_peer_id.to_string(),
                        msg.text_content.clone(),
                    );
                    vec![Self::reencrypt_one_chunk(session, sender_id, &payload.to_bytes()?)?]
                }
                Some(attachment) => {
                    let Some(bytes) = self.storage.get_attachment_blob(&msg.id)? else { continue };
                    let sha256_checksum = {
                        use sha2::{Digest, Sha256};
                        let mut hasher = Sha256::new();
                        hasher.update(&bytes);
                        hex::encode(hasher.finalize())
                    };
                    let chunk_count = bytes.len().div_ceil(nova_protocol::MEDIA_CHUNK_SIZE).max(1) as u32;
                    let mut chunk_packets = Vec::with_capacity(chunk_count as usize);
                    for i in 0..chunk_count {
                        let start = i as usize * nova_protocol::MEDIA_CHUNK_SIZE;
                        let end = ((i as usize + 1) * nova_protocol::MEDIA_CHUNK_SIZE).min(bytes.len());
                        let media_meta = (i == 0).then(|| nova_protocol::MediaMetadata {
                            file_name: attachment.file_name.clone(),
                            mime_type: attachment.mime_type.clone(),
                            size_bytes: bytes.len() as u64,
                            sha256_checksum: sha256_checksum.clone(),
                            chunk_count,
                        });
                        let mut payload = MessagePayload::new_media_chunk(
                            msg.id.clone(),
                            conv_id.clone(),
                            sender_id.to_string(),
                            recipient_peer_id.to_string(),
                            msg.content_type.clone(),
                            media_meta,
                            i,
                            bytes[start..end].to_vec(),
                        );
                        if i == 0 {
                            payload.text_content = Some(msg.text_content.clone());
                        }
                        chunk_packets.push(Self::reencrypt_one_chunk(session, sender_id, &payload.to_bytes()?)?);
                    }
                    chunk_packets
                }
            };

            let wrapped = wrap_chunks_for_outbox(&packets)?;
            self.storage.enqueue_outbox(&msg.id, &conv_id, recipient_peer_id, &wrapped)?;
            tracing::info!("Re-encrypted outbox message {} on established session", msg.id);
        }
        Ok(())
    }

    /// Encrypts one already-serialized `MessagePayload` under `session` and wraps it as a plain
    /// `EncryptedMessage` wire packet — the no-handshake case of
    /// [`frame_to_packet_payload`], used by [`reencrypt_pending_outbox_items`](Self::reencrypt_pending_outbox_items)
    /// since the session there is always already established (that's the whole point of the
    /// collision resolution that calls it).
    fn reencrypt_one_chunk(
        session: &mut DoubleRatchetSession,
        sender_id: &str,
        payload_bytes: &[u8],
    ) -> Result<Vec<u8>, EngineError> {
        let (header, ciphertext) = session.ratchet_encrypt(payload_bytes, b"NOVA_MSG")?;
        let frame = EncryptedFrame { header, ciphertext };
        let mut buf = Vec::new();
        ciborium::into_writer(&frame, &mut buf)
            .map_err(|e| nova_protocol::ProtocolError::SerializationFailed(e.to_string()))?;
        let packet = NovaPacket::new(FrameType::EncryptedMessage, sender_id.to_string(), buf);
        Ok(packet.to_cbor()?)
    }

    /// Retrieve all conversations for the main UI screen.
    /// Only returns conversations with actual activity (messages exchanged), matching Google Messages model.
    pub fn get_conversations(&self) -> Result<Vec<ConversationRecord>, EngineError> {
        let list = self.storage.get_conversations()?;
        let active = list
            .into_iter()
            .filter(|c| {
                if !c.last_message_text.trim().is_empty() {
                    return true;
                }
                if let Ok(msgs) = self.storage.get_messages(&c.id) {
                    return !msgs.is_empty();
                }
                false
            })
            .collect();
        Ok(active)
    }

    /// Retrieve message history for a conversation. Attachment binary data is NOT included (see
    /// [`get_attachment_data`](Self::get_attachment_data)) — only the lightweight metadata needed
    /// to render a bubble, so opening/polling a media-heavy conversation stays cheap.
    pub fn get_messages(&self, conversation_id: &str) -> Result<Vec<MessageRecord>, EngineError> {
        Ok(self.storage.get_messages(conversation_id)?)
    }

    /// Full-text search across every conversation's local message history (see
    /// `nova_storage::StorageEngine::search_messages`), for the global search screen.
    pub fn search_messages(&self, query: &str) -> Result<Vec<MessageRecord>, EngineError> {
        Ok(self.storage.search_messages(query)?)
    }

    /// Fetches one message's attachment binary data, decrypted — called on demand when the UI is
    /// actually about to render/download it (an image thumbnail, a voice note about to play),
    /// never eagerly by [`get_messages`](Self::get_messages) or by full-text search
    /// (`nova_storage::StorageEngine::search_messages`). Returns `None` if the message has no
    /// attachment.
    pub fn get_attachment_data(&self, message_id: &str) -> Result<Option<Vec<u8>>, EngineError> {
        Ok(self.storage.get_attachment_blob(message_id)?)
    }

    /// Deletes a single message and its attachment from local storage.
    pub fn delete_message(&self, message_id: &str) -> Result<(), EngineError> {
        Ok(self.storage.delete_message(message_id)?)
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

    /// Every one of this device's own reachable multiaddrs (`.../p2p/<peer-id>`), if the network
    /// is attached — typically one IPv4 and, when the host has a usable IPv6 stack, one IPv6.
    /// Only meaningful to share with other devices as a bootstrap address when this device is
    /// actually dialable from outside its own network: the IPv4 one needs its listen port fixed
    /// via `NOVA_LISTEN_ADDR` and forwarded on its router (or opened automatically via UPnP); the
    /// IPv6 one, when present, usually needs neither, since IPv6 has no NAT to traverse.
    /// Otherwise these are just this device's local addresses — harmless to display, not useful
    /// to anyone else. Empty (not `None`) until the network is attached.
    pub async fn own_full_listen_addrs(&self) -> Vec<String> {
        let Some(node) = self.network.lock().await.clone() else {
            return Vec::new();
        };
        node.full_listen_addrs().iter().map(|a| a.to_string()).collect()
    }

    /// Connects directly to a remote bootstrap/rendezvous node if network is attached.
    pub async fn bootstrap_dial(&self, addr: &str) -> Result<(), EngineError> {
        let Some(node) = self.network.lock().await.clone() else {
            return Ok(());
        };
        let parsed: nova_transport::Multiaddr = addr
            .parse()
            .map_err(|e| nova_transport::TransportError::InvalidAddress(format!("{e}")))?;
        node.bootstrap_dial(parsed).await?;
        Ok(())
    }

    /// Sets or clears the fallback server URL (e.g. Render WebSocket discovery/relay server).
    pub async fn set_fallback_server_url(&self, url: Option<String>) {
        if let Some(node) = self.network.lock().await.clone() {
            node.set_fallback_server_url(url).await;
        }
    }

    /// Sets multiple fallback server URLs for multi-path relay striping.
    pub async fn set_fallback_server_urls(&self, urls: Vec<String>) {
        if let Some(node) = self.network.lock().await.clone() {
            node.set_fallback_server_urls(urls).await;
        }
    }

    /// Retrieves the active primary fallback server URL, if any.
    pub async fn get_fallback_server_url(&self) -> Option<String> {
        if let Some(node) = self.network.lock().await.clone() {
            node.get_fallback_server_url().await
        } else {
            None
        }
    }

    /// Retrieves all active fallback server URLs in the pool.
    pub async fn get_fallback_server_urls(&self) -> Option<Vec<String>> {
        if let Some(node) = self.network.lock().await.clone() {
            Some(node.get_fallback_server_urls().await)
        } else {
            None
        }
    }

    /// Retrieves the local user profile metadata.
    pub fn get_user_profile(&self) -> Result<Option<nova_storage::UserProfileRecord>, EngineError> {
        Ok(self.storage.get_user_profile()?)
    }

    /// Marks a contact as trusted.
    pub fn trust_contact(&self, peer_id: &str) -> Result<(), EngineError> {
        self.storage.trust_contact(peer_id)?;
        Ok(())
    }

    /// Blocks a contact so that incoming messages from them are dropped.
    pub fn block_contact(&self, peer_id: &str) -> Result<(), EngineError> {
        self.storage.block_contact(peer_id)?;
        Ok(())
    }

    /// Blocks a contact and completely wipes their conversation & message history.
    pub fn block_and_delete_conversation(&self, peer_id: &str) -> Result<(), EngineError> {
        self.storage.block_and_delete_conversation(peer_id)?;
        Ok(())
    }

    /// Unblocks a previously blocked contact.
    pub fn unblock_contact(&self, peer_id: &str) -> Result<(), EngineError> {
        self.storage.unblock_contact(peer_id)?;
        Ok(())
    }

    /// Checks if a contact is currently blocked.
    pub fn is_contact_blocked(&self, peer_id: &str) -> Result<bool, EngineError> {
        Ok(self.storage.is_contact_blocked(peer_id)?)
    }

    /// Retrieves all currently blocked contacts.
    pub fn get_blocked_contacts(&self) -> Result<Vec<ContactRecord>, EngineError> {
        Ok(self.storage.get_blocked_contacts()?)
    }

    /// Permanently removes a contact and their entire message history — see
    /// `StorageEngine::delete_contact` for exactly what gets wiped and why this is a stronger
    /// operation than `block_contact`.
    pub fn delete_contact(&self, peer_id: &str) -> Result<(), EngineError> {
        self.storage.delete_contact(peer_id)?;
        Ok(())
    }

    /// Retrieves all saved contacts.
    pub fn get_contacts(&self) -> Result<Vec<ContactRecord>, EngineError> {
        Ok(self.storage.get_contacts()?)
    }

    /// Saves or updates the local user profile metadata.
    pub fn save_user_profile(&self, profile: &nova_storage::UserProfileRecord) -> Result<(), EngineError> {
        self.storage.save_user_profile(profile)?;
        Ok(())
    }

    /// Publishes this user's profile and PreKey bundle to the fallback directory.
    pub async fn publish_directory_profile(&self) -> Result<(), EngineError> {
        let Some(node) = self.network.lock().await.clone() else {
            return Ok(());
        };

        let (identity_pub, username) = {
            let id_lock = self.identity.lock().await;
            let Some(id) = id_lock.as_ref() else {
                return Ok(());
            };
            (id.public_id_hex(), id.username.clone())
        };

        let user_profile = self.storage.get_user_profile().ok().flatten();
        let display_name = user_profile
            .as_ref()
            .map(|p| p.display_name.clone())
            .unwrap_or_else(|| username.clone());
        let avatar_data_url = user_profile.and_then(|p| p.avatar_data_url);
        let bundle_bytes = self.get_own_prekey_bundle_bytes().await?;
        let bundle_hex = hex::encode(bundle_bytes);

        let profile = nova_protocol::DirectoryProfile {
            peer_id: identity_pub,
            username,
            display_name,
            avatar_data_url,
            prekey_bundle_hex: bundle_hex,
        };

        let signed_entry = {
            let id_lock = self.identity.lock().await;
            let Some(id) = id_lock.as_ref() else {
                return Ok(());
            };
            let now_utc = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            nova_protocol::SignedDirectoryEntry::sign(id, profile, now_utc)
        };

        match node.register_directory_entry(signed_entry).await {
            Ok(()) => {
                tracing::info!("Directory profile published to fallback server successfully");
                Ok(())
            }
            Err(e) => {
                tracing::warn!("Failed to publish directory profile to fallback server: {e}");
                Err(EngineError::Transport(e))
            }
        }
    }

    /// Searches the public fallback directory for users matching `query` (by peer_id, @username, or display name).
    pub async fn search_directory(&self, query: &str) -> Result<Vec<nova_protocol::DirectorySearchResult>, EngineError> {
        let Some(node) = self.network.lock().await.clone() else {
            return Ok(Vec::new());
        };
        Ok(node.search_directory(query).await.unwrap_or_default())
    }

    // --- P2P Sovereign Group Chat Methods ---

    pub async fn create_group(
        &self,
        name: &str,
        description: Option<String>,
        avatar_data_url: Option<String>,
        initial_member_peer_ids: Vec<String>,
    ) -> Result<GroupRecord, EngineError> {
        let id_lock = self.identity.lock().await;
        let identity = id_lock.as_ref().ok_or(EngineError::NoIdentity)?;
        let my_peer_id = identity.public_id_hex();
        let my_name = identity.username.clone();
        drop(id_lock);

        let group_id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();

        let group = GroupRecord {
            id: group_id.clone(),
            name: name.to_string(),
            description: description.clone(),
            avatar_data_url,
            creator_peer_id: my_peer_id.clone(),
            my_role: "owner".into(),
            ephemeral_timer_sec: 0,
            created_at_utc: now,
            updated_at_utc: now,
        };
        self.storage.save_group(&group)?;

        let owner_member = GroupMemberRecord {
            group_id: group_id.clone(),
            peer_id: my_peer_id.clone(),
            display_name: my_name.clone(),
            role: "owner".into(),
            joined_at_utc: now,
            is_online: true,
        };
        self.storage.save_group_member(&owner_member)?;

        let mut all_members = vec![GroupMemberInfo {
            peer_id: my_peer_id.clone(),
            display_name: my_name.clone(),
            role: GroupRole::Owner,
            joined_at_utc: now,
        }];

        for peer_id in initial_member_peer_ids {
            if peer_id == my_peer_id {
                continue;
            }
            let display_name = self.storage.get_contact(&peer_id)?
                .map(|c| c.display_name)
                .unwrap_or_else(|| peer_id.chars().take(8).collect());

            let member = GroupMemberRecord {
                group_id: group_id.clone(),
                peer_id: peer_id.clone(),
                display_name: display_name.clone(),
                role: "member".into(),
                joined_at_utc: now,
                is_online: false,
            };
            self.storage.save_group_member(&member)?;

            all_members.push(GroupMemberInfo {
                peer_id,
                display_name,
                role: GroupRole::Member,
                joined_at_utc: now,
            });
        }

        let genesis_action = GroupControlAction::Genesis {
            group_id: group_id.clone(),
            name: name.to_string(),
            description,
            members: all_members,
        };
        let action_json = serde_json::to_string(&genesis_action).unwrap_or_default();

        let members = self.storage.get_group_members(&group_id)?;
        for m in members {
            if m.peer_id == my_peer_id {
                continue;
            }
            let _ = self.send_raw_group_control(&m.peer_id, &action_json).await;
        }

        Ok(group)
    }

    pub fn get_groups(&self) -> Result<Vec<GroupRecord>, EngineError> {
        Ok(self.storage.get_groups()?)
    }

    pub fn get_group(&self, group_id: &str) -> Result<Option<GroupRecord>, EngineError> {
        Ok(self.storage.get_group(group_id)?)
    }

    pub fn get_group_members(&self, group_id: &str) -> Result<Vec<GroupMemberRecord>, EngineError> {
        Ok(self.storage.get_group_members(group_id)?)
    }

    pub async fn create_group_invitation(
        &self,
        group_id: &str,
        ttl_seconds: i64,
    ) -> Result<String, EngineError> {
        let id_lock = self.identity.lock().await;
        let identity = id_lock.as_ref().ok_or(EngineError::NoIdentity)?;
        let bundle = self.storage.get_own_prekey_bundle(identity)?;

        let group = self.storage.get_group(group_id)?
            .ok_or_else(|| EngineError::Storage(StorageError::NotFound(group_id.into())))?;

        let rendezvous = self.own_full_listen_addrs().await;

        let signed_invite = SignedGroupInvitation::create(
            identity,
            group.id,
            group.name,
            group.description,
            bundle,
            ttl_seconds,
            rendezvous,
        )?;

        Ok(signed_invite.to_uri()?)
    }

    pub async fn join_group_by_invitation_uri(&self, uri: &str) -> Result<GroupRecord, EngineError> {
        let ticket = SignedGroupInvitation::from_uri(uri)?;
        let now = chrono::Utc::now().timestamp();
        let payload = ticket.verify(now)?;

        let id_lock = self.identity.lock().await;
        let identity = id_lock.as_ref().ok_or(EngineError::NoIdentity)?;
        let my_peer_id = identity.public_id_hex();
        let my_name = identity.username.clone();
        drop(id_lock);

        let group = GroupRecord {
            id: payload.group_id.clone(),
            name: payload.group_name.clone(),
            description: payload.group_description.clone(),
            avatar_data_url: None,
            creator_peer_id: payload.inviter_peer_id.clone(),
            my_role: "member".into(),
            ephemeral_timer_sec: 0,
            created_at_utc: payload.created_at_utc,
            updated_at_utc: now,
        };
        self.storage.save_group(&group)?;

        let me_member = GroupMemberRecord {
            group_id: payload.group_id.clone(),
            peer_id: my_peer_id.clone(),
            display_name: my_name.clone(),
            role: "member".into(),
            joined_at_utc: now,
            is_online: true,
        };
        self.storage.save_group_member(&me_member)?;

        let inviter_member = GroupMemberRecord {
            group_id: payload.group_id.clone(),
            peer_id: payload.inviter_peer_id.clone(),
            display_name: payload.inviter_name.clone(),
            role: "admin".into(),
            joined_at_utc: payload.created_at_utc,
            is_online: false,
        };
        self.storage.save_group_member(&inviter_member)?;

        let member_added = GroupControlAction::MemberAdded {
            group_id: payload.group_id.clone(),
            new_member: GroupMemberInfo {
                peer_id: my_peer_id,
                display_name: my_name,
                role: GroupRole::Member,
                joined_at_utc: now,
            },
            added_by: payload.inviter_peer_id.clone(),
        };
        let action_json = serde_json::to_string(&member_added).unwrap_or_default();
        let _ = self.send_raw_group_control(&payload.inviter_peer_id, &action_json).await;

        Ok(group)
    }

    pub async fn send_group_message(
        &self,
        group_id: &str,
        text: &str,
    ) -> Result<MessageRecord, EngineError> {
        let id_lock = self.identity.lock().await;
        let identity = id_lock.as_ref().ok_or(EngineError::NoIdentity)?;
        let my_peer_id = identity.public_id_hex();
        let my_name = identity.username.clone();
        drop(id_lock);

        let group = self.storage.get_group(group_id)?
            .ok_or_else(|| EngineError::Storage(StorageError::NotFound(group_id.into())))?;

        let now = chrono::Utc::now().timestamp();
        let msg_id = Uuid::new_v4().to_string();
        let conv_id = format!("group_{group_id}");

        let envelope = GroupMessageEnvelope {
            group_id: group_id.to_string(),
            sender_peer_id: my_peer_id.clone(),
            sender_name: my_name,
            timestamp_utc: now,
            text: text.to_string(),
            reply_to_id: None,
            media_meta: None,
        };
        let envelope_json = serde_json::to_string(&envelope).unwrap_or_default();

        let local_record = MessageRecord {
            id: msg_id.clone(),
            conversation_id: conv_id.clone(),
            sender_id: my_peer_id.clone(),
            recipient_id: group_id.to_string(),
            text_content: text.to_string(),
            timestamp_utc: now,
            status: DbMessageStatus::Sent,
            is_outgoing: true,
            content_type: MessageContentType::GroupMessage,
            attachment: None,
        };
        self.storage.save_message(&local_record)?;

        let mut conv = self.storage.get_conversations()?
            .into_iter()
            .find(|c| c.id == conv_id)
            .unwrap_or_else(|| ConversationRecord {
                id: conv_id.clone(),
                peer_id: group.id.clone(),
                title: group.name.clone(),
                last_message_text: String::new(),
                last_message_time_utc: now,
                unread_count: 0,
            });
        conv.last_message_text = text.to_string();
        conv.last_message_time_utc = now;
        self.storage.save_conversation(&conv)?;

        let members = self.storage.get_group_members(group_id)?;
        for m in members {
            if m.peer_id == my_peer_id {
                continue;
            }
            let chunk_id = Uuid::new_v4().to_string();
            let mut payload = MessagePayload::new_text(
                chunk_id,
                conv_id.clone(),
                my_peer_id.clone(),
                m.peer_id.clone(),
                envelope_json.clone(),
            );
            payload.content_type = MessageContentType::GroupMessage;
            let _ = self.encrypt_and_enqueue_payload(payload, &m.peer_id).await;
        }

        Ok(local_record)
    }

    pub async fn leave_group(&self, group_id: &str) -> Result<(), EngineError> {
        let id_lock = self.identity.lock().await;
        let identity = id_lock.as_ref().ok_or(EngineError::NoIdentity)?;
        let my_peer_id = identity.public_id_hex();
        drop(id_lock);

        let left_action = GroupControlAction::MemberLeft {
            group_id: group_id.to_string(),
            member_peer_id: my_peer_id.clone(),
        };
        let action_json = serde_json::to_string(&left_action).unwrap_or_default();

        let members = self.storage.get_group_members(group_id)?;
        for m in members {
            if m.peer_id == my_peer_id {
                continue;
            }
            let _ = self.send_raw_group_control(&m.peer_id, &action_json).await;
        }

        self.storage.delete_group(group_id)?;
        Ok(())
    }

    async fn send_raw_group_control(&self, recipient_peer_id: &str, action_json: &str) -> Result<(), EngineError> {
        let id_lock = self.identity.lock().await;
        let identity = id_lock.as_ref().ok_or(EngineError::NoIdentity)?;
        let my_peer_id = identity.public_id_hex();
        drop(id_lock);

        let msg_id = Uuid::new_v4().to_string();
        let conv_id = format!("conv_{recipient_peer_id}");
        let mut payload = MessagePayload::new_text(
            msg_id,
            conv_id,
            my_peer_id,
            recipient_peer_id.to_string(),
            action_json.to_string(),
        );
        payload.content_type = MessageContentType::GroupControl;
        self.encrypt_and_enqueue_payload(payload, recipient_peer_id).await
    }

    async fn encrypt_and_enqueue_payload(&self, payload: MessagePayload, recipient_peer_id: &str) -> Result<(), EngineError> {
        let payload_bytes = payload.to_bytes()?;
        let (verifying_key_bytes, dh_public_bytes, sender_id) = {
            let id_lock = self.identity.lock().await;
            let identity = id_lock.as_ref().ok_or(EngineError::NoIdentity)?;
            (
                identity.verifying_key_bytes,
                identity.dh_public_bytes,
                identity.public_id_hex(),
            )
        };

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
        let (frame_type, packet_payload) = frame_to_packet_payload(
            handshake_material.as_ref(),
            verifying_key_bytes,
            dh_public_bytes,
            frame,
        )?;

        let packet = NovaPacket::new(
            frame_type,
            sender_id,
            packet_payload,
        );
        let wire_bytes = packet.to_cbor()?;
        let outbox_blob = wrap_chunks_for_outbox(&[wire_bytes])?;

        self.storage.enqueue_outbox(
            &payload.message_id,
            &payload.conversation_id,
            recipient_peer_id,
            &outbox_blob,
        )?;

        Ok(())
    }

    fn handle_incoming_group_control(&self, sender_peer_id: &str, action: GroupControlAction) -> Result<(), EngineError> {
        let now = chrono::Utc::now().timestamp();
        match action {
            GroupControlAction::Genesis { group_id, name, description, members } => {
                let group = GroupRecord {
                    id: group_id.clone(),
                    name,
                    description,
                    avatar_data_url: None,
                    creator_peer_id: sender_peer_id.to_string(),
                    my_role: "member".into(),
                    ephemeral_timer_sec: 0,
                    created_at_utc: now,
                    updated_at_utc: now,
                };
                self.storage.save_group(&group)?;
                for m in members {
                    let member = GroupMemberRecord {
                        group_id: group_id.clone(),
                        peer_id: m.peer_id,
                        display_name: m.display_name,
                        role: m.role.as_str().into(),
                        joined_at_utc: m.joined_at_utc,
                        is_online: false,
                    };
                    self.storage.save_group_member(&member)?;
                }
            }
            GroupControlAction::MemberAdded { group_id, new_member, .. } => {
                let member = GroupMemberRecord {
                    group_id,
                    peer_id: new_member.peer_id,
                    display_name: new_member.display_name,
                    role: new_member.role.as_str().into(),
                    joined_at_utc: new_member.joined_at_utc,
                    is_online: false,
                };
                self.storage.save_group_member(&member)?;
            }
            GroupControlAction::MemberLeft { group_id, member_peer_id } => {
                self.storage.remove_group_member(&group_id, &member_peer_id)?;
            }
            GroupControlAction::MemberKicked { group_id, kicked_peer_id, .. } => {
                self.storage.remove_group_member(&group_id, &kicked_peer_id)?;
            }
            GroupControlAction::MetadataUpdated { group_id, name, description, avatar_data_url } => {
                if let Some(mut group) = self.storage.get_group(&group_id)? {
                    group.name = name;
                    group.description = description;
                    group.avatar_data_url = avatar_data_url;
                    group.updated_at_utc = now;
                    self.storage.save_group(&group)?;
                }
            }
        }
        Ok(())
    }

    async fn handle_incoming_group_message(
        &self,
        _sender_peer_id: &str,
        env: GroupMessageEnvelope,
    ) -> Result<ReceiveOutcome, EngineError> {
        let conv_id = format!("group_{}", env.group_id);
        let now = env.timestamp_utc;
        let msg_id = Uuid::new_v4().to_string();

        let group = self.storage.get_group(&env.group_id)?
            .unwrap_or_else(|| GroupRecord {
                id: env.group_id.clone(),
                name: format!("Groupe {}", env.group_id.chars().take(6).collect::<String>()),
                description: None,
                avatar_data_url: None,
                creator_peer_id: env.sender_peer_id.clone(),
                my_role: "member".into(),
                ephemeral_timer_sec: 0,
                created_at_utc: now,
                updated_at_utc: now,
            });
        self.storage.save_group(&group)?;

        let mut conv = self.storage.get_conversations()?
            .into_iter()
            .find(|c| c.id == conv_id)
            .unwrap_or_else(|| ConversationRecord {
                id: conv_id.clone(),
                peer_id: env.group_id.clone(),
                title: group.name.clone(),
                last_message_text: String::new(),
                last_message_time_utc: now,
                unread_count: 0,
            });

        conv.unread_count += 1;
        conv.last_message_text = format!("{}: {}", env.sender_name, env.text);
        conv.last_message_time_utc = now;
        self.storage.save_conversation(&conv)?;

        let msg_record = MessageRecord {
            id: msg_id,
            conversation_id: conv_id,
            sender_id: env.sender_peer_id,
            recipient_id: env.group_id,
            text_content: env.text,
            timestamp_utc: now,
            status: DbMessageStatus::Delivered,
            is_outgoing: false,
            content_type: MessageContentType::GroupMessage,
            attachment: None,
        };
        self.storage.save_message(&msg_record)?;

        Ok(ReceiveOutcome::New(msg_record))
    }
}

fn random_key_id() -> u32 {
    // 0 is reserved as a "no key" sentinel in a couple of Option<u32> comparisons upstream;
    // keep generated ids strictly positive.
    rand::thread_rng().gen_range(1..=u32::MAX)
}

/// Human-readable size for a media message's auto-generated caption (e.g. "vacation.jpg (3.2 MB)").
fn format_bytes(bytes: u64) -> String {
    const MB: u64 = 1024 * 1024;
    const KB: u64 = 1024;
    if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.0} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}

/// Builds the `(FrameType, packet_payload_bytes)` pair for one already Double-Ratchet-encrypted
/// `frame`: wraps it in a `HandshakeInit` envelope when `handshake_material` is `Some` (the first
/// message/chunk of a brand new session), or a plain `EncryptedMessage` envelope otherwise.
/// Shared by every place that turns an `EncryptedFrame` into wire bytes — text messages, media
/// chunks, and outbox re-encryption after a handshake collision — so the handshake-vs-plain
/// framing decision lives in exactly one place.
fn frame_to_packet_payload(
    handshake_material: Option<&nova_crypto::X3dhInitResult>,
    identity_ed25519_pub: [u8; 32],
    identity_x25519_pub: [u8; 32],
    frame: EncryptedFrame,
) -> Result<(FrameType, Vec<u8>), EngineError> {
    match handshake_material {
        Some(init) => {
            let handshake = HandshakeInitPayload {
                sender_identity_ed25519_pub: identity_ed25519_pub,
                sender_identity_x25519_pub: identity_x25519_pub,
                sender_ephemeral_pub: init.ephemeral_pub,
                used_signed_prekey_id: init.used_signed_prekey_id,
                used_one_time_prekey_id: init.used_one_time_prekey_id,
                first_message: frame,
            };
            Ok((FrameType::HandshakeInit, handshake.to_bytes()?))
        }
        None => {
            let mut buf = Vec::new();
            ciborium::into_writer(&frame, &mut buf)
                .map_err(|e| nova_protocol::ProtocolError::SerializationFailed(e.to_string()))?;
            Ok((FrameType::EncryptedMessage, buf))
        }
    }
}

/// Encodes a list of already-serialized wire packets (each the output of `NovaPacket::to_cbor`)
/// into the single BLOB `nova_storage::StorageEngine::enqueue_outbox` persists. A plain text
/// message is the one-chunk case; a media message is N chunks — see
/// `NovaEngine::send_media`/`nova_transport::P2PNode::send_chunks_to_peer`. Wrapping every outbox
/// entry the same way (rather than "one packet OR a list") keeps `pump_outbox_once` uniform.
///
/// Wraps each packet in `serde_bytes::ByteBuf` before encoding — see
/// `nova_protocol::MessagePayload::chunk_bytes`'s doc comment for why a plain `Vec<Vec<u8>>`
/// would silently inflate every packet here by roughly 1.5-2x (each already at close to
/// `MAX_PACKET_SIZE` for a media chunk) purely from this outbox-local envelope, on top of the
/// same mistake already fixed at the wire-protocol layer.
fn wrap_chunks_for_outbox(packets: &[Vec<u8>]) -> Result<Vec<u8>, EngineError> {
    let as_byte_bufs: Vec<serde_bytes::ByteBuf> =
        packets.iter().map(|p| serde_bytes::ByteBuf::from(p.clone())).collect();
    let mut buf = Vec::new();
    ciborium::into_writer(&as_byte_bufs, &mut buf)
        .map_err(|e| nova_protocol::ProtocolError::SerializationFailed(e.to_string()))?;
    Ok(buf)
}

/// The inverse of [`wrap_chunks_for_outbox`].
fn unwrap_chunks_for_outbox(payload: &[u8]) -> Result<Vec<Vec<u8>>, EngineError> {
    let as_byte_bufs: Vec<serde_bytes::ByteBuf> = ciborium::from_reader(payload)
        .map_err(|e| EngineError::Protocol(nova_protocol::ProtocolError::DeserializationFailed(e.to_string())))?;
    Ok(as_byte_bufs.into_iter().map(|b| b.into_vec()).collect())
}

/// Decodes and verifies a raw, unwrapped `PreKeyBundle` (no invitation envelope at all) — the
/// last-resort path in `add_contact` when the input isn't recognizable as any invitation format.
/// Shared by both the UTF-8 and non-UTF-8 branches so the decode+verify pairing lives in exactly
/// one place.
fn bundle_from_raw_bytes(bytes: &[u8]) -> Result<PreKeyBundle, EngineError> {
    let bundle = prekey_bundle_from_bytes(bytes)?;
    verify_prekey_bundle(&bundle)?;
    Ok(bundle)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An outbox item's `payload` is always the wrapped chunk-list encoding (see
    /// `wrap_chunks_for_outbox`), even for a single-packet text message — this extracts that one
    /// packet, for tests that feed it straight to `receive_packet` without going through the
    /// real transport layer (which does this unwrapping itself, see `pump_outbox_once`).
    fn single_wire_packet(payload: &[u8]) -> Vec<u8> {
        let mut chunks = unwrap_chunks_for_outbox(payload).unwrap();
        assert_eq!(chunks.len(), 1, "expected a single-chunk (text) outbox payload, got {}", chunks.len());
        chunks.remove(0)
    }

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
        let wire_bytes = single_wire_packet(&pending[0].payload);

        // Sanity: the wire bytes really do carry a HandshakeInit frame for message #1.
        let parsed = NovaPacket::from_cbor(&wire_bytes).unwrap();
        assert_eq!(parsed.frame_type, FrameType::HandshakeInit);

        let received = bob
            .receive_packet(&wire_bytes)
            .await
            .unwrap()
            .new_message()
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
        let reply_wire = single_wire_packet(&reply_entry.payload);
        let reply_parsed = NovaPacket::from_cbor(&reply_wire).unwrap();
        assert_eq!(reply_parsed.frame_type, FrameType::EncryptedMessage);

        let alice_received = alice
            .receive_packet(&reply_wire)
            .await
            .unwrap()
            .new_message()
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
        let wire2 = single_wire_packet(&entry2.payload);
        let parsed2 = NovaPacket::from_cbor(&wire2).unwrap();
        assert_eq!(parsed2.frame_type, FrameType::EncryptedMessage);

        let final_history = bob.receive_packet(&wire2).await.unwrap().new_message().unwrap();
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

    /// Regression test for the 2026-08-22 audit finding: the UI falls back to a bare hex prekey
    /// bundle (via the `get_own_prekey_bundle_hex` Tauri command) whenever minting a signed
    /// `nova://invite?...` link fails. That fallback code used to be rejected unconditionally by
    /// `add_contact` regardless of correctness — see `ParsedInvitation` in nova-protocol.
    #[tokio::test]
    async fn test_add_contact_via_legacy_hex_bundle_matches_ui_fallback_path() {
        let alice = NovaEngine::new(":memory:", "alice-pass").unwrap();
        let bob = NovaEngine::new(":memory:", "bob-pass").unwrap();

        alice.create_account("alice").await.unwrap();
        let (bob_pub, _) = bob.create_account("bob").await.unwrap();

        // Exactly what ui/src-tauri's get_own_prekey_bundle_hex returns: hex::encode of the raw
        // PreKeyBundle bytes, no "nova://" wrapper at all.
        let bob_bundle_bytes = bob.get_own_prekey_bundle_bytes().await.unwrap();
        let bare_hex = hex::encode(bob_bundle_bytes);

        let contact = alice
            .add_contact("bob.nova", "Bob", bare_hex.as_bytes())
            .await
            .expect("a correctly-copied legacy hex code must be accepted, not rejected");
        assert_eq!(contact.peer_id, bob_pub);
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

    /// A device closing and reopening the app (a new `NovaEngine` instance over the same
    /// on-disk database) must not have to re-enter its mnemonic to keep using its identity —
    /// that is the entire point of `try_resume_session`. Uses a real temp file rather than
    /// `:memory:` since the whole scenario under test is "a second, independent open of the same
    /// persisted database."
    #[tokio::test]
    async fn test_try_resume_session_restores_identity_without_the_mnemonic() {
        let db_path = std::env::temp_dir().join(format!("nova_resume_test_{}.db", uuid::Uuid::new_v4()));
        let db_path_str = db_path.to_str().unwrap();

        let (original_peer_id, mnemonic) = {
            let engine = NovaEngine::new(db_path_str, "resume-test-pass").unwrap();
            let (peer_id, mnemonic) = engine.create_account("alex").await.unwrap();
            (peer_id, mnemonic)
        };

        // A fresh engine instance, as a real app relaunch would create — no mnemonic supplied.
        let reopened = NovaEngine::new(db_path_str, "resume-test-pass").unwrap();
        let resumed = reopened.try_resume_session().await.unwrap();
        let (resumed_peer_id, resumed_mnemonic, resumed_username) =
            resumed.expect("a previously created identity must be found on reopen");

        assert_eq!(resumed_peer_id, original_peer_id);
        assert_eq!(resumed_mnemonic, mnemonic);
        assert_eq!(resumed_username, "alex");
        assert!(reopened.identity.lock().await.is_some());

        let _ = std::fs::remove_file(&db_path);
    }

    #[tokio::test]
    async fn test_try_resume_session_returns_none_on_first_run() {
        let engine = NovaEngine::new(":memory:", "fresh-pass").unwrap();
        let resumed = engine.try_resume_session().await.unwrap();
        assert!(resumed.is_none());
    }

    /// The literal claim behind "how can a user in Ouagadougou talk to a user in
    /// Bobo-Dioulasso": two fully independent `NovaEngine` instances — separate identities,
    /// separate encrypted databases, separate `P2PNode`s bound to their own OS sockets — find
    /// each other purely through the distributed Kademlia DHT (not a hardcoded address or a
    /// single trusted server) and exchange a real message over the real network stack (QUIC
    /// handshake, X3DH, Double Ratchet), with no bytes shared in-process. Nothing here is a
    /// mock: a passing run is the network layer actually working.
    #[tokio::test]
    async fn test_two_independent_engines_exchange_a_message_over_a_real_network() {
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

        let alice_node = nova_transport::P2PNode::start(alice_identity, "/ip4/127.0.0.1/udp/0/quic-v1")
            .await
            .unwrap();
        let bob_node = nova_transport::P2PNode::start(bob_identity, "/ip4/127.0.0.1/udp/0/quic-v1")
            .await
            .unwrap();

        // In production, first contact between peers who don't already know each other's
        // current address goes through mDNS (same LAN) or a small list of well-known DHT
        // bootstrap peers. This test stands in for that with one explicit, known address
        // (deterministic, so it doesn't depend on this machine's mDNS/multicast setup) — after
        // this single dial, all further discovery goes through the real DHT, not this bootstrap.
        alice_node.bootstrap_dial(bob_node.listen_addr().clone()).await.unwrap();
        tokio::time::sleep(Duration::from_millis(500)).await;
        bob_node.announce_presence().await.unwrap();
        tokio::time::sleep(Duration::from_millis(500)).await;

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

        // Bob looks this up under HIS OWN conversation-id convention (`conv_<the sender's
        // peer_id>`, i.e. `conv_<alice_peer_id>`) — deliberately not Alice's `conv_id` above,
        // which names *her* side of the same conversation (`conv_<bob_peer_id>`). See the fix in
        // `persist_incoming_message`: the receiver never trusts the sender's embedded id.
        let bob_conv_id = format!("conv_{alice_peer_id}");

        // The outbox pump runs every 500ms; poll for up to a few seconds for the message to
        // actually cross the network and land, decrypted, in Bob's conversation history.
        let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
        loop {
            let messages = bob.get_messages(&bob_conv_id).unwrap();
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

    #[tokio::test]
    async fn test_add_contact_with_signed_invitation_and_expired_ticket_rejection() {
        let alice = NovaEngine::new(":memory:", "alice-pass").unwrap();
        let bob = NovaEngine::new(":memory:", "bob-pass").unwrap();

        let (_alice_id, _) = alice.create_account("alice").await.unwrap();
        let (_bob_id, bob_mnemonic) = bob.create_account("bob").await.unwrap();

        // 1. Valid invitation ticket with 24h deadline
        let valid_uri = bob.get_own_invitation_uri(Some(86400)).await.unwrap();
        assert!(valid_uri.starts_with("nova://invite?d="));

        let contact = alice.add_contact("bob", "Bob", valid_uri.as_bytes()).await.unwrap();
        assert_eq!(contact.display_name, "Bob");

        // 2. Expired invitation ticket (-10s TTL)
        let mnemonic = MnemonicPhrase::from_phrase(&bob_mnemonic).unwrap();
        let bob_identity = nova_crypto::DeviceIdentity::from_mnemonic(&mnemonic, "bob").unwrap();
        let bundle = bob.storage.get_own_prekey_bundle(&bob_identity).unwrap();
        let expired_inv = nova_protocol::SignedContactInvitation::create(&bob_identity, bundle, -10, vec![]).unwrap();
        let expired_uri = expired_inv.to_uri().unwrap();

        let err = alice.add_contact("bob_expired", "Bob Expired", expired_uri.as_bytes()).await.unwrap_err();
        assert!(format!("{err}").contains("expiré") || format!("{err}").contains("Expired"));
    }

    #[tokio::test]
    async fn test_blocked_contact_messages_are_dropped() {
        let alice = NovaEngine::new(":memory:", "alice-pass").unwrap();
        let bob = NovaEngine::new(":memory:", "bob-pass").unwrap();

        let (alice_pub, _) = alice.create_account("alice").await.unwrap();
        let (bob_pub, _) = bob.create_account("bob").await.unwrap();

        let bob_bundle = bob.get_own_prekey_bundle_bytes().await.unwrap();
        alice.add_contact("bob", "Bob", &bob_bundle).await.unwrap();

        let alice_bundle = alice.get_own_prekey_bundle_bytes().await.unwrap();
        bob.add_contact("alice", "Alice", &alice_bundle).await.unwrap();

        // Bob blocks Alice
        bob.block_contact(&alice_pub).unwrap();
        assert!(bob.is_contact_blocked(&alice_pub).unwrap());

        // Alice sends a message to Bob
        let conv_id = format!("conv_{bob_pub}");
        let _sent = alice.send_message(&conv_id, &bob_pub, "Message de test non desiré").await.unwrap();

        let pending = alice.storage.get_pending_outbox().unwrap();
        assert_eq!(pending.len(), 1);
        let wire_bytes = single_wire_packet(&pending[0].payload);

        // Bob receives packet: should report it as blocked specifically (not a generic no-op)
        // and NOT save anything.
        let received = bob.receive_packet(&wire_bytes).await.unwrap();
        assert!(matches!(received, ReceiveOutcome::BlockedSender));

        let bob_conv_id = format!("conv_{alice_pub}");
        let bob_messages = bob.get_messages(&bob_conv_id).unwrap();
        assert_eq!(bob_messages.len(), 0);

        // Bob unblocks Alice
        bob.unblock_contact(&alice_pub).unwrap();
        assert!(!bob.is_contact_blocked(&alice_pub).unwrap());
    }

    #[tokio::test]
    async fn test_readding_contact_preserves_conversation_history() {
        let alice = NovaEngine::new(":memory:", "alice-pass").unwrap();
        let bob = NovaEngine::new(":memory:", "bob-pass").unwrap();

        let (_alice_pub, _) = alice.create_account("alice").await.unwrap();
        let (bob_pub, _) = bob.create_account("bob").await.unwrap();

        let bob_bundle = bob.get_own_prekey_bundle_bytes().await.unwrap();
        alice.add_contact("bob", "Bob", &bob_bundle).await.unwrap();

        let conv_id = format!("conv_{bob_pub}");
        alice.send_message(&conv_id, &bob_pub, "Premier message").await.unwrap();

        let convs_before = alice.get_conversations().unwrap();
        assert_eq!(convs_before[0].last_message_text, "Premier message");

        // Re-add Bob with an updated bundle/ticket
        let bob_new_ticket = bob.get_own_invitation_uri(Some(86400)).await.unwrap();
        alice.add_contact("bob", "Bob Renommé", bob_new_ticket.as_bytes()).await.unwrap();

        let convs_after = alice.get_conversations().unwrap();
        assert_eq!(convs_after[0].title, "Bob Renommé");
        assert_eq!(convs_after[0].last_message_text, "Premier message");
    }

    #[tokio::test]
    async fn test_unblocking_contact_preserves_double_ratchet_sync() {
        let alice = NovaEngine::new(":memory:", "alice-pass").unwrap();
        let bob = NovaEngine::new(":memory:", "bob-pass").unwrap();

        let (alice_pub, _) = alice.create_account("alice").await.unwrap();
        let (bob_pub, _) = bob.create_account("bob").await.unwrap();

        let bob_bundle = bob.get_own_prekey_bundle_bytes().await.unwrap();
        alice.add_contact("bob", "Bob", &bob_bundle).await.unwrap();

        let alice_bundle = alice.get_own_prekey_bundle_bytes().await.unwrap();
        bob.add_contact("alice", "Alice", &alice_bundle).await.unwrap();

        // 1. Establish initial session
        let conv_alice = format!("conv_{bob_pub}");
        let _conv_bob = format!("conv_{alice_pub}");
        let msg1 = alice.send_message(&conv_alice, &bob_pub, "Message 1 avant blocage").await.unwrap();
        let p1 = single_wire_packet(&alice.storage.get_pending_outbox().unwrap()[0].payload);
        let r1 = bob.receive_packet(&p1).await.unwrap().new_message().unwrap();
        assert_eq!(r1.text_content, "Message 1 avant blocage");

        // 2. Bob blocks Alice
        bob.block_contact(&alice_pub).unwrap();

        // 3. Alice sends 3 messages while blocked
        let _msg2 = alice.send_message(&conv_alice, &bob_pub, "Message 2 pendant blocage").await.unwrap();
        let p2 = single_wire_packet(&alice.storage.get_pending_outbox().unwrap().iter().find(|i| i.message_id != msg1.id).unwrap().payload);
        let r2 = bob.receive_packet(&p2).await.unwrap();
        assert!(matches!(r2, ReceiveOutcome::BlockedSender), "message while blocked must be reported as blocked, not returned to UI");

        let _msg3 = alice.send_message(&conv_alice, &bob_pub, "Message 3 pendant blocage").await.unwrap();
        let p3 = single_wire_packet(&alice.storage.get_pending_outbox().unwrap().last().unwrap().payload);
        let r3 = bob.receive_packet(&p3).await.unwrap();
        assert!(matches!(r3, ReceiveOutcome::BlockedSender));

        // 4. Bob unblocks Alice
        bob.unblock_contact(&alice_pub).unwrap();

        // 5. Alice sends Message 4 after unblocking -> must decrypt seamlessly!
        let _msg4 = alice.send_message(&conv_alice, &bob_pub, "Message 4 apres deblocage").await.unwrap();
        let p4 = single_wire_packet(&alice.storage.get_pending_outbox().unwrap().last().unwrap().payload);
        let r4 = bob.receive_packet(&p4).await.unwrap().new_message().expect("Message 4 must decrypt cleanly after unblocking");
        assert_eq!(r4.text_content, "Message 4 apres deblocage");
        assert_eq!(r4.sender_id, alice_pub);
    }

    #[tokio::test]
    async fn test_simultaneous_handshake_init_collision_resolution() {
        let alice = NovaEngine::new(":memory:", "alice-pass").unwrap();
        let bob = NovaEngine::new(":memory:", "bob-pass").unwrap();

        let (alice_pub, _) = alice.create_account("alice").await.unwrap();
        let (bob_pub, _) = bob.create_account("bob").await.unwrap();

        let bob_bundle = bob.get_own_prekey_bundle_bytes().await.unwrap();
        alice.add_contact("bob", "Bob", &bob_bundle).await.unwrap();

        let alice_bundle = alice.get_own_prekey_bundle_bytes().await.unwrap();
        bob.add_contact("alice", "Alice", &alice_bundle).await.unwrap();

        // Alice and Bob both send a message simultaneously (both create HandshakeInit)
        let conv_alice = format!("conv_{bob_pub}");
        let conv_bob = format!("conv_{alice_pub}");

        let _msg_alice = alice.send_message(&conv_alice, &bob_pub, "Message initial Alice").await.unwrap();
        let _msg_bob = bob.send_message(&conv_bob, &alice_pub, "Message initial Bob").await.unwrap();

        let alice_packet = single_wire_packet(&alice.storage.get_pending_outbox().unwrap()[0].payload);
        let bob_packet = single_wire_packet(&bob.storage.get_pending_outbox().unwrap()[0].payload);

        // Both deliver their packets to each other concurrently
        let r_bob = bob.receive_packet(&alice_packet).await.unwrap().new_message();
        let r_alice = alice.receive_packet(&bob_packet).await.unwrap().new_message();

        // One of the handshakes is processed as authoritative according to lexicographical order
        assert!(r_bob.is_some() || r_alice.is_some());

        // The subordinate peer has re-encrypted its outbound message on the newly established session.
        // Deliver the re-encrypted message to the priority peer:
        if alice_pub < bob_pub {
            // Alice was priority initiator; Bob yielded and re-encrypted his outbox message.
            let bob_pending = bob.storage.get_pending_outbox().unwrap();
            for item in bob_pending {
                if let Some(msg) = alice.receive_packet(&single_wire_packet(&item.payload)).await.unwrap().new_message() {
                    assert_eq!(msg.text_content, "Message initial Bob");
                }
            }
        } else {
            // Bob was priority initiator; Alice yielded and re-encrypted her outbox message.
            let alice_pending = alice.storage.get_pending_outbox().unwrap();
            for item in alice_pending {
                if let Some(msg) = bob.receive_packet(&single_wire_packet(&item.payload)).await.unwrap().new_message() {
                    assert_eq!(msg.text_content, "Message initial Alice");
                }
            }
        }

        // Alice replies on the established session
        let reply = alice.send_message(&conv_alice, &bob_pub, "Reponse Alice apres collision resolue").await.unwrap();
        let reply_packet = single_wire_packet(&alice.storage.get_pending_outbox().unwrap().iter().find(|i| i.message_id == reply.id).unwrap().payload);
        let bob_received_reply = bob.receive_packet(&reply_packet).await.unwrap().new_message().unwrap();
        assert_eq!(bob_received_reply.text_content, "Reponse Alice apres collision resolue");
    }

    /// Regression test for the 2026-08-22 audit's J4 media-chunking work: a file spanning just
    /// over two `MEDIA_CHUNK_SIZE`s splits into exactly three wire packets, each intermediate
    /// chunk is accepted but reports nothing new to the UI yet, and only the final chunk
    /// completes reassembly — with the reassembled bytes exactly matching the original file and
    /// fetchable afterward via `get_attachment_data`, never inline in `get_messages`.
    #[tokio::test]
    async fn test_send_and_receive_media_message_multi_chunk_reassembly() {
        let alice = NovaEngine::new(":memory:", "alice-pass").unwrap();
        let bob = NovaEngine::new(":memory:", "bob-pass").unwrap();
        let (alice_pub, _) = alice.create_account("alice").await.unwrap();
        let (bob_pub, _) = bob.create_account("bob").await.unwrap();

        let bob_bundle = bob.get_own_prekey_bundle_bytes().await.unwrap();
        alice.add_contact("bob", "Bob", &bob_bundle).await.unwrap();

        let file_bytes: Vec<u8> = (0..(nova_protocol::MEDIA_CHUNK_SIZE * 2 + 12_345))
            .map(|i| (i % 251) as u8)
            .collect();

        let conv_id = format!("conv_{bob_pub}");
        let sent = alice
            .send_media(
                &conv_id,
                &bob_pub,
                nova_protocol::MessageContentType::File,
                "report.pdf".into(),
                "application/pdf".into(),
                file_bytes.clone(),
                "📄 report.pdf".into(),
            )
            .await
            .unwrap();
        assert!(sent.attachment.is_some());
        assert_eq!(sent.text_content, "📄 report.pdf", "text_content must stay a caption, never the file itself");

        let pending = alice.storage.get_pending_outbox().unwrap();
        assert_eq!(pending.len(), 1, "the whole media message is one outbox entry, not one per chunk");
        let chunks = unwrap_chunks_for_outbox(&pending[0].payload).unwrap();
        assert_eq!(chunks.len(), 3, "a file spanning just over 2 chunk-sizes must split into 3 packets");

        let mut final_outcome = None;
        for (i, chunk) in chunks.iter().enumerate() {
            let outcome = bob.receive_packet(chunk).await.unwrap();
            if i + 1 < chunks.len() {
                assert!(
                    matches!(outcome, ReceiveOutcome::ProcessedNoOp),
                    "intermediate chunk {i} must not surface a new message yet"
                );
            } else {
                final_outcome = Some(outcome);
            }
        }
        let final_msg = final_outcome
            .unwrap()
            .new_message()
            .expect("the last chunk must complete reassembly and surface the message");
        assert_eq!(final_msg.id, sent.id);
        assert_eq!(final_msg.content_type, nova_protocol::MessageContentType::File);
        assert_eq!(final_msg.text_content, "📄 report.pdf");

        // get_messages must never inline the attachment bytes. Looked up under Bob's OWN
        // conversation-id convention (`conv_<the sender's peer_id>` — see the fix in
        // `persist_incoming_message`/`handle_incoming_media_chunk`), not Alice's `conv_id` above:
        // those are deliberately different strings, one per side's own local naming.
        let bob_conv_id = format!("conv_{alice_pub}");
        let bob_messages = bob.get_messages(&bob_conv_id).unwrap();
        assert_eq!(bob_messages.len(), 1);
        assert_eq!(bob_messages[0].text_content, "📄 report.pdf");
        assert!(bob_messages[0].attachment.is_some());

        // Fetched separately, on demand, and byte-for-byte identical to the original.
        let fetched = bob
            .get_attachment_data(&sent.id)
            .unwrap()
            .expect("attachment must be retrievable by id");
        assert_eq!(fetched, file_bytes, "reassembled bytes must exactly match the original file");
    }

    /// The multi-chunk media pipeline over the real network stack (attach_network,
    /// pump_outbox_once, P2PNode::send_chunks_to_peer) — not just direct `receive_packet` calls.
    /// Proves the outbox correctly sends every chunk over one connection and only marks the
    /// message `Delivered` once the recipient's engine has fully reassembled it.
    #[tokio::test]
    async fn test_media_message_delivered_over_the_real_network() {
        let alice_mnemonic = MnemonicPhrase::generate().unwrap();
        let bob_mnemonic = MnemonicPhrase::generate().unwrap();
        let alice_id_for_net = nova_crypto::DeviceIdentity::from_mnemonic(&alice_mnemonic, "alex").unwrap();
        let bob_id_for_net = nova_crypto::DeviceIdentity::from_mnemonic(&bob_mnemonic, "bob").unwrap();
        let bob_pub = bob_id_for_net.public_id_hex();

        let alice_node = nova_transport::P2PNode::start(alice_id_for_net, "/ip4/127.0.0.1/udp/0/quic-v1").await.unwrap();
        let bob_node = nova_transport::P2PNode::start(bob_id_for_net, "/ip4/127.0.0.1/udp/0/quic-v1").await.unwrap();
        alice_node.bootstrap_dial(bob_node.listen_addr().clone()).await.unwrap();
        tokio::time::sleep(Duration::from_millis(500)).await;
        bob_node.announce_presence().await.unwrap();
        tokio::time::sleep(Duration::from_millis(500)).await;

        let alice = Arc::new(NovaEngine::new(":memory:", "alice-storage-pass").unwrap());
        let bob = Arc::new(NovaEngine::new(":memory:", "bob-storage-pass").unwrap());

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

        let conv_id = format!("conv_{bob_pub}");
        // Just over one chunk size, so this exercises real multi-packet delivery.
        let file_bytes: Vec<u8> = (0..(nova_protocol::MEDIA_CHUNK_SIZE + 500_000)).map(|i| (i % 200) as u8).collect();
        let sent = alice
            .send_media(
                &conv_id,
                &bob_pub,
                nova_protocol::MessageContentType::Image,
                "photo.jpg".into(),
                "image/jpeg".into(),
                file_bytes.clone(),
                "📷 photo.jpg".into(),
            )
            .await
            .unwrap();

        // Generous window: this exercises real chunked delivery (X3DH handshake on chunk 0, then
        // every chunk awaited sequentially over one connection — see
        // P2PNode::send_chunks_to_peer), not just a single small packet.
        let deadline = tokio::time::Instant::now() + Duration::from_secs(25);
        loop {
            if let Some(fetched) = bob.get_attachment_data(&sent.id).unwrap() {
                assert_eq!(fetched, file_bytes);
                break;
            }
            if tokio::time::Instant::now() > deadline {
                panic!("media message never arrived over the real network within the deadline");
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        // The sender's own copy must eventually show Delivered, not stuck at Sent.
        let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
        loop {
            let history = alice.get_messages(&conv_id).unwrap();
            let mine = history.iter().find(|m| m.id == sent.id).unwrap();
            if mine.status == DbMessageStatus::Delivered {
                break;
            }
            if tokio::time::Instant::now() > deadline {
                panic!("sender's outbox never marked the media message Delivered");
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    /// Regression test for the 2026-08-22 audit's blocked-contact finding, over the *real*
    /// network path (attach_network + pump_outbox_once + DeliveryOutcome), not the direct
    /// `receive_packet` call the unit-level blocking tests above use: a message sent to a
    /// contact who blocks the sender must never be marked `Delivered` in the sender's own
    /// history, even though the recipient's engine did receive and decrypt it to stay in ratchet
    /// sync — see `pump_outbox_once`'s handling of `DeliveryOutcome::Blocked`.
    #[tokio::test]
    async fn test_blocked_recipient_never_shows_as_delivered_over_the_real_network() {
        let alice_mnemonic = MnemonicPhrase::generate().unwrap();
        let bob_mnemonic = MnemonicPhrase::generate().unwrap();
        let alice_id_for_net = nova_crypto::DeviceIdentity::from_mnemonic(&alice_mnemonic, "alex").unwrap();
        let bob_id_for_net = nova_crypto::DeviceIdentity::from_mnemonic(&bob_mnemonic, "bob").unwrap();
        let alice_pub = alice_id_for_net.public_id_hex();
        let bob_pub = bob_id_for_net.public_id_hex();

        let alice_node = nova_transport::P2PNode::start(alice_id_for_net, "/ip4/127.0.0.1/udp/0/quic-v1").await.unwrap();
        let bob_node = nova_transport::P2PNode::start(bob_id_for_net, "/ip4/127.0.0.1/udp/0/quic-v1").await.unwrap();
        alice_node.bootstrap_dial(bob_node.listen_addr().clone()).await.unwrap();
        tokio::time::sleep(Duration::from_millis(500)).await;
        bob_node.announce_presence().await.unwrap();
        tokio::time::sleep(Duration::from_millis(500)).await;

        let alice = Arc::new(NovaEngine::new(":memory:", "alice-storage-pass").unwrap());
        let bob = Arc::new(NovaEngine::new(":memory:", "bob-storage-pass").unwrap());

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
        let alice_bundle = alice.get_own_prekey_bundle_bytes().await.unwrap();
        bob.add_contact("alice.nova", "Alice", &alice_bundle).await.unwrap();

        bob.block_contact(&alice_pub).unwrap();

        let conv_id = format!("conv_{bob_pub}");
        let sent = alice
            .send_message(&conv_id, &bob_pub, "Message vers un contact qui me bloque")
            .await
            .unwrap();

        // Give the outbox pump (500ms tick) several cycles to actually attempt delivery.
        tokio::time::sleep(Duration::from_secs(2)).await;

        let alice_history = alice.get_messages(&conv_id).unwrap();
        let stored = alice_history
            .iter()
            .find(|m| m.id == sent.id)
            .expect("the message must still be in alice's own history");
        assert_ne!(
            stored.status,
            DbMessageStatus::Delivered,
            "a message to a contact who blocks the sender must never show as Delivered"
        );

        // And Bob must genuinely never have it in his own visible history either.
        assert!(bob.get_messages(&conv_id).unwrap().is_empty());
    }

    /// Regression test for the 2026-08-22 audit's "silent packet drop" finding: when the
    /// recipient's engine cannot process a packet at all (here: an oversized message that trips
    /// `nova_protocol::MAX_PACKET_SIZE` once wrapped in CBOR/ratchet framing), the sender must
    /// never see it marked "Delivered" — the old unconditional transport-level ack did exactly
    /// that, discarding the message with no signal to either side. It must instead remain queued
    /// in the outbox for retry, not vanish.
    #[tokio::test]
    async fn test_oversized_message_is_never_marked_delivered_over_the_real_network() {
        let alice_mnemonic = MnemonicPhrase::generate().unwrap();
        let bob_mnemonic = MnemonicPhrase::generate().unwrap();
        let alice_id_for_net = nova_crypto::DeviceIdentity::from_mnemonic(&alice_mnemonic, "alex").unwrap();
        let bob_id_for_net = nova_crypto::DeviceIdentity::from_mnemonic(&bob_mnemonic, "bob").unwrap();
        let bob_pub = bob_id_for_net.public_id_hex();

        let alice_node = nova_transport::P2PNode::start(alice_id_for_net, "/ip4/127.0.0.1/udp/0/quic-v1").await.unwrap();
        let bob_node = nova_transport::P2PNode::start(bob_id_for_net, "/ip4/127.0.0.1/udp/0/quic-v1").await.unwrap();
        alice_node.bootstrap_dial(bob_node.listen_addr().clone()).await.unwrap();
        tokio::time::sleep(Duration::from_millis(500)).await;
        bob_node.announce_presence().await.unwrap();
        tokio::time::sleep(Duration::from_millis(500)).await;

        let alice = Arc::new(NovaEngine::new(":memory:", "alice-storage-pass").unwrap());
        let bob = Arc::new(NovaEngine::new(":memory:", "bob-storage-pass").unwrap());

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

        let conv_id = format!("conv_{bob_pub}");
        // Large enough that the final CBOR/ratchet-framed wire packet exceeds
        // nova_protocol::MAX_PACKET_SIZE (900 KiB) even accounting for framing overhead.
        let oversized_text = "A".repeat(2 * 1024 * 1024);
        let sent = alice.send_message(&conv_id, &bob_pub, &oversized_text).await.unwrap();

        tokio::time::sleep(Duration::from_secs(2)).await;

        let alice_history = alice.get_messages(&conv_id).unwrap();
        let stored = alice_history.iter().find(|m| m.id == sent.id).unwrap();
        assert_ne!(
            stored.status,
            DbMessageStatus::Delivered,
            "a packet the recipient's engine could not even decode must never show as Delivered"
        );

        let pending = alice.storage.get_pending_outbox().unwrap();
        assert!(
            pending.iter().any(|i| i.message_id == sent.id),
            "a rejected message must remain queued for retry, not be silently dropped"
        );

        assert!(bob.get_messages(&conv_id).unwrap().is_empty());
    }

    /// The pure backoff formula, independent of any storage/network setup: grows
    /// exponentially, respects the 5-minute cap, and the first two attempts (where 20% jitter
    /// rounds down to zero) are exactly deterministic.
    #[test]
    fn test_next_retry_delay_grows_and_caps() {
        assert_eq!(next_retry_delay_secs(0), 1);
        assert_eq!(next_retry_delay_secs(1), 2);

        let mid = next_retry_delay_secs(5); // doubled = 32s, +/-20% jitter => [26, 38]
        assert!((26..=38).contains(&mid), "got {mid}");

        for _ in 0..20 {
            let capped = next_retry_delay_secs(15); // far past the cap even before jitter
            assert!(
                (OUTBOX_RETRY_CAP_SECS - OUTBOX_RETRY_CAP_SECS / 5..=OUTBOX_RETRY_CAP_SECS + OUTBOX_RETRY_CAP_SECS / 5)
                    .contains(&capped),
                "got {capped}, must respect the {OUTBOX_RETRY_CAP_SECS}s cap (+/-20% jitter)"
            );
        }
    }

    /// Regression test for the 2026-08-22 audit's "no backoff / no give-up" finding: an outbox
    /// item that has been failing for longer than the retry window must be removed from the
    /// outbox and marked `Failed` — not retried forever with no user-visible signal.
    #[tokio::test]
    async fn test_outbox_gives_up_and_marks_failed_after_the_retry_window() {
        let alice = NovaEngine::new(":memory:", "alice-pass").unwrap();
        let bob = NovaEngine::new(":memory:", "bob-pass").unwrap();
        alice.create_account("alice").await.unwrap();
        let (bob_pub, _) = bob.create_account("bob").await.unwrap();
        let bob_bundle = bob.get_own_prekey_bundle_bytes().await.unwrap();
        alice.add_contact("bob", "Bob", &bob_bundle).await.unwrap();

        let conv_id = format!("conv_{bob_pub}");
        let sent = alice.send_message(&conv_id, &bob_pub, "will never arrive").await.unwrap();

        // Simulate this item having already been failing for 8 days (past the 7-day window),
        // without actually waiting 8 real days for the test to run.
        let mut item = alice
            .storage
            .get_pending_outbox()
            .unwrap()
            .into_iter()
            .find(|i| i.message_id == sent.id)
            .unwrap();
        item.first_attempt_utc = chrono::Utc::now().timestamp() - 8 * 24 * 3600;

        alice.back_off_or_give_up(&item, chrono::Utc::now().timestamp()).unwrap();

        assert!(
            alice.storage.get_pending_outbox().unwrap().iter().all(|i| i.message_id != sent.id),
            "a message that gave up must leave the outbox — no more silent retries"
        );
        let stored = alice.get_messages(&conv_id).unwrap().into_iter().find(|m| m.id == sent.id).unwrap();
        assert_eq!(stored.status, DbMessageStatus::Failed, "the UI must be able to show this as a definite failure, not eternal 'Sent'");
    }

    /// J4 extension of the J2 retry mechanism: a `Failed` MEDIA message must retry with its
    /// actual attachment bytes (re-fetched from storage and re-chunked), not just its caption —
    /// the naive version of `retry_failed_message` (text-only) would have silently dropped the
    /// file and re-sent an empty message.
    #[tokio::test]
    async fn test_retry_failed_media_message_resends_the_actual_attachment() {
        let alice = NovaEngine::new(":memory:", "alice-pass").unwrap();
        let bob = NovaEngine::new(":memory:", "bob-pass").unwrap();
        alice.create_account("alice").await.unwrap();
        let (bob_pub, _) = bob.create_account("bob").await.unwrap();
        let bob_bundle = bob.get_own_prekey_bundle_bytes().await.unwrap();
        alice.add_contact("bob", "Bob", &bob_bundle).await.unwrap();

        let conv_id = format!("conv_{bob_pub}");
        let file_bytes = vec![0x42u8; 500_000];
        let sent = alice
            .send_media(
                &conv_id,
                &bob_pub,
                nova_protocol::MessageContentType::Audio,
                "note.ogg".into(),
                "audio/ogg".into(),
                file_bytes.clone(),
                "🎤 Note vocale".into(),
            )
            .await
            .unwrap();

        let mut item = alice
            .storage
            .get_pending_outbox()
            .unwrap()
            .into_iter()
            .find(|i| i.message_id == sent.id)
            .unwrap();
        item.first_attempt_utc = chrono::Utc::now().timestamp() - 8 * 24 * 3600;
        alice.back_off_or_give_up(&item, chrono::Utc::now().timestamp()).unwrap();
        assert_eq!(
            alice.get_messages(&conv_id).unwrap().into_iter().find(|m| m.id == sent.id).unwrap().status,
            DbMessageStatus::Failed
        );

        let retried = alice.retry_failed_message(&conv_id, &bob_pub, &sent.id).await.unwrap();
        assert_eq!(retried.id, sent.id);
        assert!(retried.attachment.is_some());

        let pending = alice.storage.get_pending_outbox().unwrap();
        let requeued = pending.iter().find(|i| i.message_id == sent.id).expect("must be back in the outbox");
        let chunks = unwrap_chunks_for_outbox(&requeued.payload).unwrap();

        let mut final_outcome = None;
        for chunk in &chunks {
            final_outcome = Some(bob.receive_packet(chunk).await.unwrap());
        }
        let received = final_outcome.unwrap().new_message().expect("retried media must still reassemble on the other end");
        assert_eq!(received.id, sent.id);
        let fetched = bob.get_attachment_data(&sent.id).unwrap().unwrap();
        assert_eq!(fetched, file_bytes, "retry must resend the real attachment bytes, not an empty/caption-only message");
    }

    /// Regression test for the 2026-08-22 audit's plan (J2): the UI's manual "Réessayer" action
    /// on a `Failed` message must re-queue it under its ORIGINAL id (not duplicate it in the
    /// conversation) with a fresh attempt counter.
    #[tokio::test]
    async fn test_retry_failed_message_requeues_under_the_same_id_with_a_fresh_clock() {
        let alice = NovaEngine::new(":memory:", "alice-pass").unwrap();
        let bob = NovaEngine::new(":memory:", "bob-pass").unwrap();
        alice.create_account("alice").await.unwrap();
        let (bob_pub, _) = bob.create_account("bob").await.unwrap();
        let bob_bundle = bob.get_own_prekey_bundle_bytes().await.unwrap();
        alice.add_contact("bob", "Bob", &bob_bundle).await.unwrap();

        let conv_id = format!("conv_{bob_pub}");
        let sent = alice.send_message(&conv_id, &bob_pub, "retry me").await.unwrap();

        let mut item = alice
            .storage
            .get_pending_outbox()
            .unwrap()
            .into_iter()
            .find(|i| i.message_id == sent.id)
            .unwrap();
        item.first_attempt_utc = chrono::Utc::now().timestamp() - 8 * 24 * 3600;
        alice.back_off_or_give_up(&item, chrono::Utc::now().timestamp()).unwrap();
        assert_eq!(
            alice.get_messages(&conv_id).unwrap().into_iter().find(|m| m.id == sent.id).unwrap().status,
            DbMessageStatus::Failed
        );

        let retried = alice.retry_failed_message(&conv_id, &bob_pub, &sent.id).await.unwrap();
        assert_eq!(retried.id, sent.id, "retry must keep the original message id, not create a new one");
        assert_eq!(retried.text_content, "retry me");

        let pending = alice.storage.get_pending_outbox().unwrap();
        let requeued = pending.iter().find(|i| i.message_id == sent.id).expect("message must be back in the outbox after a manual retry");
        assert_eq!(requeued.attempt_count, 0, "a manual retry must get a fresh attempt counter, not resume the old one");
    }

    #[tokio::test]
    async fn test_group_end_to_end_lifecycle() {
        let alice = NovaEngine::new(":memory:", "alice-pass").unwrap();
        let bob = NovaEngine::new(":memory:", "bob-pass").unwrap();

        let (_alice_pub, _) = alice.create_account("alice").await.unwrap();
        let (bob_pub, _) = bob.create_account("bob").await.unwrap();

        let bob_bundle = bob.get_own_prekey_bundle_bytes().await.unwrap();
        alice.add_contact("bob", "Bob", &bob_bundle).await.unwrap();

        // Alice creates a group with Bob
        let group = alice
            .create_group(
                "Nova Team",
                Some("Development chat".to_string()),
                None,
                vec![bob_pub.clone()],
            )
            .await
            .unwrap();

        assert_eq!(group.name, "Nova Team");

        // Alice sends a message to the group
        let msg = alice.send_group_message(&group.id, "Hello Nova Team!").await.unwrap();
        assert_eq!(msg.text_content, "Hello Nova Team!");

        // Alice creates an invitation URI for the group
        let uri = alice.create_group_invitation(&group.id, 86400).await.unwrap();
        assert!(uri.starts_with("nova://group-invite?d="));

        // A third peer Charlie joins via the invitation URI
        let charlie = NovaEngine::new(":memory:", "charlie-pass").unwrap();
        let (_charlie_pub, _) = charlie.create_account("charlie").await.unwrap();

        let joined_group = charlie.join_group_by_invitation_uri(&uri).await.unwrap();
        assert_eq!(joined_group.id, group.id);
        assert_eq!(joined_group.name, "Nova Team");

        let charlie_members = charlie.get_group_members(&group.id).unwrap();
        assert_eq!(charlie_members.len(), 2); // Alice + Charlie

        // Charlie leaves the group
        charlie.leave_group(&group.id).await.unwrap();
        assert!(charlie.get_group(&group.id).unwrap().is_none());
    }
}

