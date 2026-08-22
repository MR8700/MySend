use crate::packet::{ProtocolError, MAX_PACKET_SIZE};
use nova_crypto::{verify_signature, DeviceIdentity};
use serde::{Deserialize, Serialize};
use thiserror::Error;

const DHT_RECORD_SIG_DOMAIN: &[u8] = b"NOVA_DHT_PEER_RECORD_V1";

#[derive(Error, Debug)]
pub enum DhtRecordError {
    #[error("nova_peer_id is not a valid 32-byte hex-encoded Ed25519 public key")]
    InvalidPeerId,
    #[error("DHT record signature does not match the claimed peer_id")]
    InvalidSignature,
    #[error("DHT record timestamp is stale or too far in the future")]
    StaleTimestamp,
}

/// A peer's current libp2p reachability, published into the Kademlia DHT under a key derived
/// from `nova_peer_id`. Kademlia's `put_record` has no authentication of its own — without a
/// signature here, anyone could publish a record under another peer's key and hijack their
/// discovery entry, exactly the vulnerability the old centralized rendezvous server's
/// `SignedPresenceRegistration` closed. This is the same fix, expressed for DHT storage instead
/// of a single trusted registry: the DHT nodes storing this record do not need to be trusted,
/// because the record authenticates itself.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SignedDhtPeerRecord {
    /// Hex-encoded Ed25519 identity public key — also the record's DHT key.
    pub nova_peer_id: String,
    /// The libp2p `PeerId` bytes this nova identity is currently reachable at.
    pub libp2p_peer_id_bytes: Vec<u8>,
    /// Multiaddrs the libp2p node believes it is reachable at (from listen addresses and/or
    /// addresses observed and confirmed by other peers via the `identify` protocol).
    pub addresses: Vec<String>,
    pub timestamp_utc: u64,
    pub signature: Vec<u8>,
}

fn signing_input(
    nova_peer_id: &str,
    libp2p_peer_id_bytes: &[u8],
    addresses: &[String],
    timestamp_utc: u64,
) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(DHT_RECORD_SIG_DOMAIN);
    buf.extend_from_slice(nova_peer_id.as_bytes());
    buf.extend_from_slice(libp2p_peer_id_bytes);
    for addr in addresses {
        // Length-prefixed rather than just concatenated, so ["ab", "c"] cannot be confused with
        // ["a", "bc"] under signing — a classic ambiguous-concatenation pitfall.
        buf.extend_from_slice(&(addr.len() as u32).to_be_bytes());
        buf.extend_from_slice(addr.as_bytes());
    }
    buf.extend_from_slice(&timestamp_utc.to_be_bytes());
    buf
}

impl SignedDhtPeerRecord {
    pub fn sign(
        identity: &DeviceIdentity,
        libp2p_peer_id_bytes: Vec<u8>,
        addresses: Vec<String>,
        timestamp_utc: u64,
    ) -> Self {
        let nova_peer_id = identity.public_id_hex();
        let signature = identity
            .sign(&signing_input(&nova_peer_id, &libp2p_peer_id_bytes, &addresses, timestamp_utc))
            .to_vec();
        Self {
            nova_peer_id,
            libp2p_peer_id_bytes,
            addresses,
            timestamp_utc,
            signature,
        }
    }

    /// Verifies the signature proves possession of the private key behind `nova_peer_id`.
    /// Returns the decoded 32-byte public key on success. Does not check timestamp freshness —
    /// callers reading a record from the DHT should apply their own freshness policy, since a
    /// malicious or simply stale storing node could otherwise serve back an old-but-validly-signed
    /// record for a peer who has since moved.
    pub fn verify(&self) -> Result<[u8; 32], DhtRecordError> {
        let peer_pub: [u8; 32] = hex::decode(&self.nova_peer_id)
            .map_err(|_| DhtRecordError::InvalidPeerId)?
            .try_into()
            .map_err(|_| DhtRecordError::InvalidPeerId)?;

        let sig_bytes: [u8; 64] = self
            .signature
            .as_slice()
            .try_into()
            .map_err(|_| DhtRecordError::InvalidSignature)?;

        let msg = signing_input(&self.nova_peer_id, &self.libp2p_peer_id_bytes, &self.addresses, self.timestamp_utc);
        verify_signature(&peer_pub, &msg, &sig_bytes).map_err(|_| DhtRecordError::InvalidSignature)?;
        Ok(peer_pub)
    }

    /// Verifies both the signature and the freshness of the record against the current timestamp.
    /// Rejects records older than `max_age_secs` or timestamped in the future by more than `max_age_secs`.
    pub fn verify_fresh(&self, now_utc: u64, max_age_secs: u64) -> Result<[u8; 32], DhtRecordError> {
        let pub_key = self.verify()?;
        let is_fresh = now_utc.saturating_sub(self.timestamp_utc) <= max_age_secs
            && self.timestamp_utc.saturating_sub(now_utc) <= max_age_secs;
        if !is_fresh {
            return Err(DhtRecordError::StaleTimestamp);
        }
        Ok(pub_key)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, ProtocolError> {
        let mut buf = Vec::new();
        ciborium::into_writer(self, &mut buf).map_err(|e| ProtocolError::SerializationFailed(e.to_string()))?;
        Ok(buf)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ProtocolError> {
        if bytes.len() > MAX_PACKET_SIZE {
            return Err(ProtocolError::PacketTooLarge(bytes.len()));
        }
        ciborium::from_reader(bytes).map_err(|e| ProtocolError::DeserializationFailed(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nova_crypto::MnemonicPhrase;

    fn identity(name: &str) -> DeviceIdentity {
        let mnemonic = MnemonicPhrase::generate().unwrap();
        DeviceIdentity::from_mnemonic(&mnemonic, name).unwrap()
    }

    #[test]
    fn test_valid_record_verifies_and_roundtrips() {
        let id = identity("alex");
        let record = SignedDhtPeerRecord::sign(
            &id,
            vec![1, 2, 3, 4],
            vec!["/ip4/198.51.100.4/udp/4433/quic-v1".to_string()],
            1_800_000_000,
        );
        assert_eq!(record.nova_peer_id, id.public_id_hex());
        assert_eq!(record.verify().unwrap(), id.verifying_key_bytes);

        let bytes = record.to_bytes().unwrap();
        let decoded = SignedDhtPeerRecord::from_bytes(&bytes).unwrap();
        assert_eq!(decoded, record);
        assert!(decoded.verify().is_ok());
    }

    #[test]
    fn test_hijacked_record_is_rejected() {
        let alice = identity("alice");
        let bob = identity("bob");

        // Bob signs a legitimate record for himself...
        let mut hijacked = SignedDhtPeerRecord::sign(
            &bob,
            vec![9, 9, 9],
            vec!["/ip4/203.0.113.9/udp/1234/quic-v1".to_string()],
            1_800_000_000,
        );
        // ...but an attacker relabels it as Alice's, hoping the DHT will serve it back to anyone
        // looking her up and misdirect them to an address Bob (or the attacker) controls.
        hijacked.nova_peer_id = alice.public_id_hex();

        assert!(hijacked.verify().is_err());
    }

    #[test]
    fn test_tampered_address_is_rejected() {
        let id = identity("alex");
        let mut record = SignedDhtPeerRecord::sign(
            &id,
            vec![1, 2, 3],
            vec!["/ip4/198.51.100.4/udp/4433/quic-v1".to_string()],
            1_800_000_000,
        );
        record.addresses = vec!["/ip4/203.0.113.66/udp/1/quic-v1".to_string()];
        assert!(record.verify().is_err());
    }
}
