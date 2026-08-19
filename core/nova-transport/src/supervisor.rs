use crate::connection::{P2PTransportMode, PeerConnectionInfo};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct TransportSupervisor {
    peers: Arc<RwLock<HashMap<String, PeerConnectionInfo>>>,
}

impl Default for TransportSupervisor {
    fn default() -> Self {
        Self::new()
    }
}

impl TransportSupervisor {
    pub fn new() -> Self {
        Self {
            peers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn update_peer_state(
        &self,
        peer_id: String,
        mode: P2PTransportMode,
        latency_ms: u32,
        remote_endpoint: String,
    ) {
        let is_connected = mode != P2PTransportMode::Disconnected;
        let mut lock = self.peers.write().await;
        lock.insert(
            peer_id.clone(),
            PeerConnectionInfo {
                peer_id,
                mode,
                latency_ms,
                is_connected,
                remote_endpoint,
            },
        );
    }

    pub async fn get_peer_info(&self, peer_id: &str) -> Option<PeerConnectionInfo> {
        let lock = self.peers.read().await;
        lock.get(peer_id).cloned()
    }

    pub async fn list_active_connections(&self) -> Vec<PeerConnectionInfo> {
        let lock = self.peers.read().await;
        lock.values().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_transport_supervisor() {
        let supervisor = TransportSupervisor::new();
        supervisor
            .update_peer_state(
                "peer_emma".into(),
                P2PTransportMode::DirectQuic,
                42,
                "198.51.100.12:9000".into(),
            )
            .await;

        let info = supervisor.get_peer_info("peer_emma").await.unwrap();
        assert_eq!(info.mode, P2PTransportMode::DirectQuic);
        assert_eq!(info.latency_ms, 42);
        assert!(info.is_connected);

        // Fallback to relay
        supervisor
            .update_peer_state(
                "peer_emma".into(),
                P2PTransportMode::RelayedOpaque,
                120,
                "relay.novachat.net:443".into(),
            )
            .await;

        let info_relay = supervisor.get_peer_info("peer_emma").await.unwrap();
        assert_eq!(info_relay.mode, P2PTransportMode::RelayedOpaque);
        assert_eq!(info_relay.latency_ms, 120);
    }
}
