use thiserror::Error;

#[derive(Debug, Error)]
pub enum TransportError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("protocol encode/decode error: {0}")]
    Protocol(#[from] nova_protocol::ProtocolError),
    #[error("DHT record signature invalid: {0}")]
    DhtRecord(#[from] nova_protocol::DhtRecordError),
    #[error("Kademlia DHT operation failed: {0}")]
    Dht(String),
    #[error("failed to dial peer: {0}")]
    Dial(String),
    #[error("failed to send request to peer: {0}")]
    Send(String),
    #[error("the P2P background task is no longer running")]
    SwarmTaskGone,
    #[error("operation timed out")]
    Timeout,
    #[error("peer is not reachable: no DHT record found")]
    PeerNotFound,
    #[error("libp2p transport setup failed: {0}")]
    Setup(String),
    #[error("invalid address: {0}")]
    InvalidAddress(String),
}
