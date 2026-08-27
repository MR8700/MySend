use nova_crypto::{verify_signature, DeviceIdentity};
use serde::{Deserialize, Serialize};
use thiserror::Error;

const REGISTRATION_SIG_DOMAIN: &[u8] = b"NOVA_PRESENCE_REGISTER_V1";

#[derive(Error, Debug)]
pub enum PresenceError {
    #[error("peer_id is not a valid 32-byte hex-encoded Ed25519 public key")]
    InvalidPeerId,
    #[error("registration signature does not match the claimed peer_id")]
    InvalidSignature,
}

/// A peer's network reachability info, as announced to the discovery/signaling server.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PeerEndpoint {
    /// Hex-encoded Ed25519 identity public key — the peer's canonical id.
    pub peer_id: String,
    pub public_ip: String,
    pub public_port: u16,
    pub local_ip: Option<String>,
    pub local_port: Option<u16>,
}

/// A presence registration, signed by the peer's own identity key, so the server (or any
/// relay) can verify the registrant genuinely controls `peer_id` before trusting the announced
/// address — without this, anyone could announce presence under someone else's identity and
/// hijack their discovery entry.
/// `#[serde(with = "serde_bytes")]` on `signature` is load-bearing, not cosmetic: without it,
/// plain `serde::Serialize` encodes a `Vec<u8>` as a CBOR array of one integer item per byte
/// instead of a compact CBOR byte string — see the identical footgun documented on
/// `MessagePayload::chunk_bytes` in packet.rs and on `SignedContactInvitation`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SignedPresenceRegistration {
    pub endpoint: PeerEndpoint,
    pub timestamp_utc: u64,
    #[serde(with = "serde_bytes")]
    pub signature: Vec<u8>,
}

fn signing_input(endpoint: &PeerEndpoint, timestamp_utc: u64) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(REGISTRATION_SIG_DOMAIN);
    buf.extend_from_slice(endpoint.peer_id.as_bytes());
    buf.extend_from_slice(endpoint.public_ip.as_bytes());
    buf.extend_from_slice(&endpoint.public_port.to_be_bytes());
    buf.extend_from_slice(endpoint.local_ip.as_deref().unwrap_or("").as_bytes());
    buf.extend_from_slice(&endpoint.local_port.unwrap_or(0).to_be_bytes());
    buf.extend_from_slice(&timestamp_utc.to_be_bytes());
    buf
}

const DRAIN_SIG_DOMAIN: &[u8] = b"NOVA_RELAY_DRAIN_V1";

fn drain_signing_input(peer_id: &str, timestamp_utc: u64) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(DRAIN_SIG_DOMAIN);
    buf.extend_from_slice(peer_id.as_bytes());
    buf.extend_from_slice(&timestamp_utc.to_be_bytes());
    buf
}

/// Proves ownership of `peer_id` when asking the relay to hand over (and purge) its queued
/// opaque packets — without this, anyone who guesses or observes a peer_id could drain and
/// discard another peer's pending relayed messages (a delivery-availability attack), even
/// though the relay's zero-knowledge design already prevents them from reading the contents.
/// See `SignedPresenceRegistration`'s doc comment for why `#[serde(with = "serde_bytes")]` on
/// `signature` is load-bearing here too.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SignedDrainRequest {
    pub peer_id: String,
    pub timestamp_utc: u64,
    #[serde(with = "serde_bytes")]
    pub signature: Vec<u8>,
}

impl SignedDrainRequest {
    pub fn sign(identity: &DeviceIdentity, timestamp_utc: u64) -> Self {
        let peer_id = identity.public_id_hex();
        let signature = identity.sign(&drain_signing_input(&peer_id, timestamp_utc)).to_vec();
        Self {
            peer_id,
            timestamp_utc,
            signature,
        }
    }

    pub fn verify(&self) -> Result<[u8; 32], PresenceError> {
        let peer_pub: [u8; 32] = hex::decode(&self.peer_id)
            .map_err(|_| PresenceError::InvalidPeerId)?
            .try_into()
            .map_err(|_| PresenceError::InvalidPeerId)?;

        let sig_bytes: [u8; 64] = self
            .signature
            .as_slice()
            .try_into()
            .map_err(|_| PresenceError::InvalidSignature)?;

        let msg = drain_signing_input(&self.peer_id, self.timestamp_utc);
        verify_signature(&peer_pub, &msg, &sig_bytes).map_err(|_| PresenceError::InvalidSignature)?;
        Ok(peer_pub)
    }
}

impl SignedPresenceRegistration {
    /// Builds and signs a registration for `identity`'s own peer_id. `endpoint.peer_id` is
    /// overwritten with the identity's canonical hex id so callers cannot accidentally (or
    /// maliciously) sign a registration under the wrong id.
    pub fn sign(identity: &DeviceIdentity, mut endpoint: PeerEndpoint, timestamp_utc: u64) -> Self {
        endpoint.peer_id = identity.public_id_hex();
        let signature = identity.sign(&signing_input(&endpoint, timestamp_utc)).to_vec();
        Self {
            endpoint,
            timestamp_utc,
            signature,
        }
    }

    /// Verifies the signature proves possession of the private key behind `endpoint.peer_id`.
    /// Returns the decoded 32-byte public key on success. Does NOT check timestamp freshness —
    /// that is a server-side replay/staleness policy, not a cryptographic property.
    pub fn verify(&self) -> Result<[u8; 32], PresenceError> {
        let peer_pub: [u8; 32] = hex::decode(&self.endpoint.peer_id)
            .map_err(|_| PresenceError::InvalidPeerId)?
            .try_into()
            .map_err(|_| PresenceError::InvalidPeerId)?;

        let sig_bytes: [u8; 64] = self
            .signature
            .as_slice()
            .try_into()
            .map_err(|_| PresenceError::InvalidSignature)?;

        let msg = signing_input(&self.endpoint, self.timestamp_utc);
        verify_signature(&peer_pub, &msg, &sig_bytes).map_err(|_| PresenceError::InvalidSignature)?;

        Ok(peer_pub)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nova_crypto::MnemonicPhrase;

    fn identity() -> DeviceIdentity {
        let mnemonic = MnemonicPhrase::generate().unwrap();
        DeviceIdentity::from_mnemonic(&mnemonic, "alex").unwrap()
    }

    #[test]
    fn test_valid_registration_verifies() {
        let id = identity();
        let endpoint = PeerEndpoint {
            peer_id: String::new(), // overwritten by sign()
            public_ip: "198.51.100.12".into(),
            public_port: 9000,
            local_ip: Some("10.0.0.5".into()),
            local_port: Some(9000),
        };
        let reg = SignedPresenceRegistration::sign(&id, endpoint, 1_800_000_000);
        assert_eq!(reg.endpoint.peer_id, id.public_id_hex());
        assert_eq!(reg.verify().unwrap(), id.verifying_key_bytes);
    }

    #[test]
    fn test_spoofed_peer_id_is_rejected() {
        let alice = identity();
        let bob = identity();

        let endpoint = PeerEndpoint {
            peer_id: String::new(),
            public_ip: "203.0.113.9".into(),
            public_port: 51820,
            local_ip: None,
            local_port: None,
        };
        // Alice signs a legitimate registration for herself...
        let mut reg = SignedPresenceRegistration::sign(&alice, endpoint, 1_800_000_000);
        // ...but an attacker then relabels it as Bob's entry, hoping to hijack his presence.
        reg.endpoint.peer_id = bob.public_id_hex();

        assert!(reg.verify().is_err());
    }

    #[test]
    fn test_tampered_endpoint_is_rejected() {
        let id = identity();
        let endpoint = PeerEndpoint {
            peer_id: String::new(),
            public_ip: "198.51.100.12".into(),
            public_port: 9000,
            local_ip: None,
            local_port: None,
        };
        let mut reg = SignedPresenceRegistration::sign(&id, endpoint, 1_800_000_000);
        reg.endpoint.public_port = 4444; // attacker redirects traffic post-signature
        assert!(reg.verify().is_err());
    }

    #[test]
    fn test_drain_request_proves_ownership() {
        let alice = identity();
        let bob = identity();

        let alice_drain = SignedDrainRequest::sign(&alice, 1_800_000_000);
        assert_eq!(alice_drain.verify().unwrap(), alice.verifying_key_bytes);

        // Bob cannot forge a drain request claiming Alice's peer_id.
        let mut forged = SignedDrainRequest::sign(&bob, 1_800_000_000);
        forged.peer_id = alice.public_id_hex();
        assert!(forged.verify().is_err());
    }
}
