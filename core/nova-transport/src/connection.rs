use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum P2PTransportMode {
    DirectQuic,
    RelayedOpaque,
    Disconnected,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PeerConnectionInfo {
    pub peer_id: String,
    pub mode: P2PTransportMode,
    pub latency_ms: u32,
    pub is_connected: bool,
    pub remote_endpoint: String,
}
