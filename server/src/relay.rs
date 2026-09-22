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
    pool: Option<sqlx::PgPool>,
}

impl Default for BlindRelay {
    fn default() -> Self {
        Self::new()
    }
}

impl BlindRelay {
    pub fn new() -> Self {
        Self::with_pool(None)
    }

    pub fn with_pool(pool: Option<sqlx::PgPool>) -> Self {
        Self {
            queues: Arc::new(RwLock::new(HashMap::new())),
            pool,
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

        if let Some(ref pool) = self.pool {
            let _ = sqlx::query("INSERT INTO relay_queue (target_peer_id, payload, received_at) VALUES ($1, $2, $3)")
                .bind(target_peer_id)
                .bind(&payload)
                .bind(now as i64)
                .execute(pool)
                .await;
        }

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

    /// Retrieves a bounded batch of encrypted packets for the peer.
    /// Remaining packets are preserved in memory / database for subsequent drain requests.
    pub async fn drain_for_peer(&self, target_peer_id: &str) -> Vec<Vec<u8>> {
        let mut batch = Vec::new();
        let mut total_bytes = 0;
        const MAX_BATCH_BYTES: usize = 16 * 1024 * 1024; // 16 MB WebSocket batch ceiling

        if let Some(ref pool) = self.pool {
            use sqlx::Row;
            if let Ok(rows) = sqlx::query(
                "DELETE FROM relay_queue WHERE id IN (SELECT id FROM relay_queue WHERE target_peer_id = $1 ORDER BY id ASC LIMIT 50) RETURNING payload"
            )
            .bind(target_peer_id)
            .fetch_all(pool)
            .await
            {
                for row in rows {
                    let payload: Vec<u8> = row.get("payload");
                    total_bytes += payload.len();
                    batch.push(payload);
                    if total_bytes >= MAX_BATCH_BYTES {
                        break;
                    }
                }
            }
        }

        let mut lock = self.queues.write().await;
        if let Some(queue) = lock.get_mut(target_peer_id) {
            while let Some(front) = queue.front() {
                if !batch.is_empty() && total_bytes + front.payload.len() > MAX_BATCH_BYTES {
                    break;
                }
                if let Some(packet) = queue.pop_front() {
                    total_bytes += packet.payload.len();
                    batch.push(packet.payload);
                }
            }
            if queue.is_empty() {
                lock.remove(target_peer_id);
            }
        }

        batch
    }

    /// Returns the number of pending packets currently waiting.
    pub async fn pending_count(&self, target_peer_id: &str) -> usize {
        let mut count = {
            let lock = self.queues.read().await;
            lock.get(target_peer_id).map(|q| q.len()).unwrap_or(0)
        };
        if let Some(ref pool) = self.pool {
            let db_count: Result<i64, _> = sqlx::query_scalar("SELECT COUNT(*) FROM relay_queue WHERE target_peer_id = $1")
                .bind(target_peer_id)
                .fetch_one(pool)
                .await;
            if let Ok(c) = db_count {
                count += c as usize;
            }
        }
        count
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
