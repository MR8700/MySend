use thiserror::Error;

#[derive(Debug, Error)]
pub enum TransportError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("TLS/QUIC setup error: {0}")]
    Tls(#[from] crate::quic_config::TlsSetupError),
    #[error("rendezvous server error: {0}")]
    Rendezvous(#[from] crate::rendezvous::RendezvousError),
    #[error("no async runtime available for the QUIC endpoint")]
    NoAsyncRuntime,
    #[error("peer address returned by the rendezvous server is invalid")]
    InvalidPeerAddress,
    #[error("QUIC connect error: {0}")]
    Connect(#[from] quinn::ConnectError),
    #[error("QUIC connection error: {0}")]
    Connection(#[from] quinn::ConnectionError),
    #[error("QUIC stream write error: {0}")]
    Write(#[from] quinn::WriteError),
    #[error("QUIC stream read error: {0}")]
    Read(#[from] quinn::ReadToEndError),
    #[error("QUIC stream close error: {0}")]
    Closed(#[from] quinn::ClosedStream),
}
