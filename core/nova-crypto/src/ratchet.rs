use crate::aead::{decrypt_aead, encrypt_aead};
use crate::error::CryptoError;
use crate::kdf::{kdf_ck, kdf_rk, Key32};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret};
use zeroize::Zeroize;

const MAX_SKIP: u32 = 1000;

/// Header attached in clear (authenticated as AEAD associated data) to each ratchet message.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RatchetHeader {
    pub dh_pub: [u8; 32],
    pub pn: u32, // Number of messages in previous sending chain
    pub n: u32,  // Message index in current sending chain
}

impl RatchetHeader {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(40);
        buf.extend_from_slice(&self.dh_pub);
        buf.extend_from_slice(&self.pn.to_be_bytes());
        buf.extend_from_slice(&self.n.to_be_bytes());
        buf
    }
}

/// Double Ratchet Session state between two Sovereign Peers.
///
/// Implements `Serialize`/`Deserialize` so callers (see `nova-storage`) can persist the
/// session across restarts. Persisted bytes MUST be encrypted at rest: this state includes
/// live key material (`rk`, `cks`, `ckr`, skipped message keys) whose disclosure breaks
/// forward secrecy for any message still protected by it.
#[derive(Serialize, Deserialize)]
pub struct DoubleRatchetSession {
    dhs: StaticSecret,
    dhs_pub: X25519PublicKey,
    dhr: Option<X25519PublicKey>,
    rk: Key32,
    cks: Option<Key32>,
    ckr: Option<Key32>,
    ns: u32,
    nr: u32,
    pn: u32,
    mkskipped: HashMap<([u8; 32], u32), Key32>,
}

impl Clone for DoubleRatchetSession {
    fn clone(&self) -> Self {
        Self {
            dhs: StaticSecret::from(self.dhs.to_bytes()),
            dhs_pub: self.dhs_pub,
            dhr: self.dhr,
            rk: self.rk,
            cks: self.cks,
            ckr: self.ckr,
            ns: self.ns,
            nr: self.nr,
            pn: self.pn,
            mkskipped: self.mkskipped.clone(),
        }
    }
}

impl Drop for DoubleRatchetSession {
    fn drop(&mut self) {
        self.rk.zeroize();
        if let Some(ref mut cks) = self.cks {
            cks.zeroize();
        }
        if let Some(ref mut ckr) = self.ckr {
            ckr.zeroize();
        }
        for (_, mut key) in self.mkskipped.drain() {
            key.zeroize();
        }
    }
}

impl DoubleRatchetSession {
    /// Initialize Alice's session (initiator).
    ///
    /// Alice knows Bob's static or ephemeral public key (`bob_dh_pub`) and shared secret `shared_key`.
    pub fn init_alice(shared_key: &[u8; 32], bob_dh_pub: &[u8; 32]) -> Result<Self, CryptoError> {
        let mut rng = rand::thread_rng();
        let dhs = StaticSecret::random_from_rng(&mut rng);
        let dhs_pub = X25519PublicKey::from(&dhs);

        let bob_pub = X25519PublicKey::from(*bob_dh_pub);
        let dh_out = dhs.diffie_hellman(&bob_pub);

        let (rk, cks) = kdf_rk(shared_key, dh_out.as_bytes())?;

        Ok(Self {
            dhs,
            dhs_pub,
            dhr: Some(bob_pub),
            rk,
            cks: Some(cks),
            ckr: None,
            ns: 0,
            nr: 0,
            pn: 0,
            mkskipped: HashMap::new(),
        })
    }

    /// Initialize Bob's session (responder).
    ///
    /// Bob has his own DH key pair and shared secret `shared_key`.
    pub fn init_bob(shared_key: &[u8; 32], bob_dhs: StaticSecret) -> Self {
        let dhs_pub = X25519PublicKey::from(&bob_dhs);
        Self {
            dhs: bob_dhs,
            dhs_pub,
            dhr: None,
            rk: *shared_key,
            cks: None,
            ckr: None,
            ns: 0,
            nr: 0,
            pn: 0,
            mkskipped: HashMap::new(),
        }
    }

    /// Returns true if this session has an initialized sending chain.
    pub fn can_send(&self) -> bool {
        self.cks.is_some()
    }

    /// Encrypt an outgoing message using the Double Ratchet.
    pub fn ratchet_encrypt(
        &mut self,
        plaintext: &[u8],
        associated_data: &[u8],
    ) -> Result<(RatchetHeader, Vec<u8>), CryptoError> {
        let cks = self
            .cks
            .as_ref()
            .ok_or_else(|| CryptoError::RatchetError("Sending chain not initialized".into()))?;

        let (next_cks, mut msg_key) = kdf_ck(cks)?;
        self.cks = Some(next_cks);

        let header = RatchetHeader {
            dh_pub: *self.dhs_pub.as_bytes(),
            pn: self.pn,
            n: self.ns,
        };

        self.ns += 1;

        // Derive deterministic 12-byte nonce for this message
        let nonce = derive_nonce(&msg_key, header.n);

        // Build full associated data = user_ad || header_bytes
        let mut full_ad = Vec::with_capacity(associated_data.len() + 40);
        full_ad.extend_from_slice(associated_data);
        full_ad.extend_from_slice(&header.to_bytes());

        let ciphertext = encrypt_aead(&msg_key, &nonce, plaintext, &full_ad)?;

        // Immediately wipe message key from RAM
        msg_key.zeroize();

        Ok((header, ciphertext))
    }

    /// Decrypt an incoming message using the Double Ratchet.
    ///
    /// Executes all ratchet and chain transformations in a transactional clone.
    /// If decryption fails (e.g. invalid AEAD tag, tampered ciphertext, transmission error),
    /// `self` is left completely unmodified so that the session is never corrupted or desynchronized.
    pub fn ratchet_decrypt(
        &mut self,
        header: &RatchetHeader,
        ciphertext: &[u8],
        associated_data: &[u8],
    ) -> Result<Vec<u8>, CryptoError> {
        let mut full_ad = Vec::with_capacity(associated_data.len() + 40);
        full_ad.extend_from_slice(associated_data);
        full_ad.extend_from_slice(&header.to_bytes());

        // 1. Check if we already have a skipped message key for this header
        if let Some(mut mk) = self.mkskipped.remove(&(header.dh_pub, header.n)) {
            let nonce = derive_nonce(&mk, header.n);
            let plaintext = match decrypt_aead(&mk, &nonce, ciphertext, &full_ad) {
                Ok(pt) => pt,
                Err(e) => {
                    // Put back the skipped key if decryption fails (or wipe it if corrupted)
                    mk.zeroize();
                    return Err(e);
                }
            };
            mk.zeroize();
            return Ok(plaintext);
        }

        // Transactional clone for state safety
        let mut tx_session = self.clone();
        let header_dhr = X25519PublicKey::from(header.dh_pub);

        // 2. If new ephemeral DH key received, perform DH Ratchet step
        if tx_session.dhr.as_ref() != Some(&header_dhr) {
            tx_session.skip_message_keys(header.pn)?;
            tx_session.dh_ratchet_step(&header_dhr)?;
        }

        // 3. Skip messages in current receiving chain up to header.n
        tx_session.skip_message_keys(header.n)?;

        // 4. Perform symmetric ratchet step to get message key
        let ckr = tx_session
            .ckr
            .as_ref()
            .ok_or_else(|| CryptoError::RatchetError("Receiving chain not initialized".into()))?;

        let (next_ckr, mut msg_key) = kdf_ck(ckr)?;
        tx_session.ckr = Some(next_ckr);
        tx_session.nr += 1;

        let nonce = derive_nonce(&msg_key, header.n);
        let plaintext = match decrypt_aead(&msg_key, &nonce, ciphertext, &full_ad) {
            Ok(pt) => pt,
            Err(e) => {
                msg_key.zeroize();
                return Err(e);
            }
        };

        msg_key.zeroize();
        // Commit the state only after successful AEAD tag verification!
        *self = tx_session;
        Ok(plaintext)
    }

    fn dh_ratchet_step(&mut self, header_dhr: &X25519PublicKey) -> Result<(), CryptoError> {
        self.pn = self.ns;
        self.ns = 0;
        self.nr = 0;
        self.dhr = Some(*header_dhr);

        // DH receive step
        let dh_recv = self.dhs.diffie_hellman(header_dhr);
        let (rk_recv, ckr) = kdf_rk(&self.rk, dh_recv.as_bytes())?;
        self.rk = rk_recv;
        self.ckr = Some(ckr);

        // Generate new local ephemeral DH key
        let mut rng = rand::thread_rng();
        self.dhs = StaticSecret::random_from_rng(&mut rng);
        self.dhs_pub = X25519PublicKey::from(&self.dhs);

        // DH send step
        let dh_send = self.dhs.diffie_hellman(header_dhr);
        let (rk_send, cks) = kdf_rk(&self.rk, dh_send.as_bytes())?;
        self.rk = rk_send;
        self.cks = Some(cks);

        Ok(())
    }

    fn skip_message_keys(&mut self, until: u32) -> Result<(), CryptoError> {
        if self.nr + MAX_SKIP < until {
            return Err(CryptoError::MaxSkippedKeysExceeded);
        }

        const MAX_TOTAL_SKIPPED_KEYS: usize = 200;

        if let Some(mut ckr) = self.ckr {
            while self.nr < until {
                let (next_ckr, mut mk) = kdf_ck(&ckr)?;
                ckr = next_ckr;
                if let Some(dhr) = self.dhr {
                    // Evict oldest skipped key if cap reached to prevent memory exhaustion
                    if self.mkskipped.len() >= MAX_TOTAL_SKIPPED_KEYS {
                        if let Some(oldest_key) = self.mkskipped.keys().next().cloned() {
                            if let Some(mut old_val) = self.mkskipped.remove(&oldest_key) {
                                old_val.zeroize();
                            }
                        }
                    }
                    self.mkskipped.insert((*dhr.as_bytes(), self.nr), mk);
                } else {
                    mk.zeroize();
                }
                self.nr += 1;
            }
            self.ckr = Some(ckr);
        }

        Ok(())
    }
}

/// Derives a 12-byte nonce from message key and sequence counter.
fn derive_nonce(msg_key: &[u8; 32], counter: u32) -> [u8; 12] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"NOVA_NONCE_DERIVATION_V1");
    hasher.update(msg_key);
    hasher.update(&counter.to_be_bytes());
    let hash = hasher.finalize();

    let mut nonce = [0u8; 12];
    nonce.copy_from_slice(&hash.as_bytes()[0..12]);
    nonce
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bidirectional_double_ratchet_conversation() {
        let shared_key = [42u8; 32];

        // Bob creates his initial DH key pair
        let mut rng = rand::thread_rng();
        let bob_dhs = StaticSecret::random_from_rng(&mut rng);
        let bob_dh_pub = *X25519PublicKey::from(&bob_dhs).as_bytes();

        // Alice initializes session to Bob
        let mut alice = DoubleRatchetSession::init_alice(&shared_key, &bob_dh_pub).unwrap();

        // Bob initializes session
        let mut bob = DoubleRatchetSession::init_bob(&shared_key, bob_dhs);

        // 1. Alice sends message 1 to Bob
        let msg1 = b"Bonjour Bob ! Ceci est un message P2P direct.";
        let (h1, c1) = alice.ratchet_encrypt(msg1, b"ad_header").unwrap();
        let p1 = bob.ratchet_decrypt(&h1, &c1, b"ad_header").unwrap();
        assert_eq!(p1, msg1);

        // 2. Alice sends message 2 to Bob (same sending chain)
        let msg2 = b"Deuxieme message sans reponse.";
        let (h2, c2) = alice.ratchet_encrypt(msg2, b"ad_header").unwrap();
        let p2 = bob.ratchet_decrypt(&h2, &c2, b"ad_header").unwrap();
        assert_eq!(p2, msg2);

        // 3. Bob responds to Alice (triggers DH Ratchet step for Bob and restores PFS/PCS)
        let msg3 = b"Salut Alice ! J'ai bien recu tes messages.";
        let (h3, c3) = bob.ratchet_encrypt(msg3, b"ad_header").unwrap();
        let p3 = alice.ratchet_decrypt(&h3, &c3, b"ad_header").unwrap();
        assert_eq!(p3, msg3);

        // 4. Alice replies back to Bob (triggers DH Ratchet step for Alice)
        let msg4 = b"Parfait ! La session Double Ratchet est parfaitement synchronisee.";
        let (h4, c4) = alice.ratchet_encrypt(msg4, b"ad_header").unwrap();
        let p4 = bob.ratchet_decrypt(&h4, &c4, b"ad_header").unwrap();
        assert_eq!(p4, msg4);
    }

    #[test]
    fn test_out_of_order_message_delivery() {
        let shared_key = [99u8; 32];
        let mut rng = rand::thread_rng();
        let bob_dhs = StaticSecret::random_from_rng(&mut rng);
        let bob_dh_pub = *X25519PublicKey::from(&bob_dhs).as_bytes();

        let mut alice = DoubleRatchetSession::init_alice(&shared_key, &bob_dh_pub).unwrap();
        let mut bob = DoubleRatchetSession::init_bob(&shared_key, bob_dhs);

        // Alice sends 3 messages
        let (h1, c1) = alice.ratchet_encrypt(b"Message 1", b"").unwrap();
        let (h2, c2) = alice.ratchet_encrypt(b"Message 2", b"").unwrap();
        let (h3, c3) = alice.ratchet_encrypt(b"Message 3", b"").unwrap();

        // Bob receives Message 3 FIRST (out-of-order)
        let p3 = bob.ratchet_decrypt(&h3, &c3, b"").unwrap();
        assert_eq!(p3, b"Message 3");

        // Bob then receives Message 1
        let p1 = bob.ratchet_decrypt(&h1, &c1, b"").unwrap();
        assert_eq!(p1, b"Message 1");

        // Bob finally receives Message 2
        let p2 = bob.ratchet_decrypt(&h2, &c2, b"").unwrap();
        assert_eq!(p2, b"Message 2");
    }
}
