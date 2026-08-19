use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;
use tokio::sync::RwLock;

const MAX_RELAY_QUEUE_PER_PEER: usize = 100;
const RELAY_EXPIRY_SECONDS: u64 = 86400; // 24 hours max in volatile memory
/// Hard cap on the number of distinct target peer_ids with a queue at once. Without this, an
/// attacker could grow the outer map without bound simply by depositing one packet each for an
/// unlimited number of fabricated peer_ids — the per-peer cap alone does not prevent that.
const MAX_TOTAL_QUEUES: usize = 20_000;

#[derive(Error, Debug)]
pub enum RelayError {
    #[error("target_peer_id is not a valid 32-byte hex-encoded Ed25519 public key")]
    InvalidPeerId,
    #[error("relay is at capacity (too many distinct pending recipients)")]
    AtCapacity,
}

fn validate_peer_id(peer_id: &str) -> Result<(), RelayError> {
    let bytes = hex::decode(peer_id).map_err(|_| RelayError::InvalidPeerId)?;
    if bytes.len() != 32 {
        return Err(RelayError::InvalidPeerId);
    }
    Ok(())
}

#[derive(Clone, Debug)]
pub struct OpaqueRelayPacket {
    pub payload: Vec<u8>,
    pub received_at: u64,
}

#[derive(Clone)]
pub struct BlindRelay {
    queues: Arc<RwLock<HashMap<String, VecDeque<OpaqueRelayPacket>>>>,
}

impl Default for BlindRelay {
    fn default() -> Self {
        Self::new()
    }
}

impl BlindRelay {
    pub fn new() -> Self {
        Self {
            queues: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Stores an opaque encrypted blob destined for an offline or unreachable peer.
    /// The relay DOES NOT and CANNOT decrypt or inspect the contents.
    pub async fn forward_opaque(&self, target_peer_id: &str, payload: Vec<u8>) -> Result<(), RelayError> {
        validate_peer_id(target_peer_id)?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let packet = OpaqueRelayPacket {
            payload,
            received_at: now,
        };

        let mut lock = self.queues.write().await;
        if !lock.contains_key(target_peer_id) && lock.len() >= MAX_TOTAL_QUEUES {
            return Err(RelayError::AtCapacity);
        }

        let queue = lock
            .entry(target_peer_id.to_string())
            .or_insert_with(VecDeque::new);

        if queue.len() >= MAX_RELAY_QUEUE_PER_PEER {
            queue.pop_front(); // Evict oldest if limit reached
        }
        queue.push_back(packet);
        Ok(())
    }

    /// Retrieves and IMMEDIATELY PURGES all queued encrypted packets for the peer.
    pub async fn drain_for_peer(&self, target_peer_id: &str) -> Vec<Vec<u8>> {
        let mut lock = self.queues.write().await;
        if let Some(queue) = lock.remove(target_peer_id) {
            queue.into_iter().map(|p| p.payload).collect()
        } else {
            Vec::new()
        }
    }

    /// Returns the number of pending packets currently waiting in RAM.
    pub async fn pending_count(&self, target_peer_id: &str) -> usize {
        let lock = self.queues.read().await;
        lock.get(target_peer_id).map(|q| q.len()).unwrap_or(0)
    }

    /// Housekeeping: purges packets older than expiry threshold.
    pub async fn purge_expired(&self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut lock = self.queues.write().await;
        lock.retain(|_, queue| {
            queue.retain(|p| now.saturating_sub(p.received_at) <= RELAY_EXPIRY_SECONDS);
            !queue.is_empty()
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_blind_relay_forward_and_drain() {
        let relay = BlindRelay::new();
        let target = "aa".repeat(32);
        let target = target.as_str();

        relay
            .forward_opaque(target, b"opaque_encrypted_chunk_1".to_vec())
            .await
            .unwrap();
        relay
            .forward_opaque(target, b"opaque_encrypted_chunk_2".to_vec())
            .await
            .unwrap();

        assert_eq!(relay.pending_count(target).await, 2);

        let packets = relay.drain_for_peer(target).await;
        assert_eq!(packets.len(), 2);
        assert_eq!(packets[0], b"opaque_encrypted_chunk_1");
        assert_eq!(packets[1], b"opaque_encrypted_chunk_2");

        // After drain, memory is immediately cleared
        assert_eq!(relay.pending_count(target).await, 0);
        assert_eq!(relay.drain_for_peer(target).await.len(), 0);
    }

    #[tokio::test]
    async fn test_malformed_peer_id_is_rejected() {
        let relay = BlindRelay::new();
        let result = relay.forward_opaque("not-a-hex-peer-id", b"payload".to_vec()).await;
        assert!(matches!(result, Err(RelayError::InvalidPeerId)));
    }
}
