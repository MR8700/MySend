use nova_crypto::DeviceIdentity;
use nova_protocol::{
    PeerEndpoint, ServerRequest, ServerResponse, SignedDrainRequest, SignedPresenceRegistration,
};
use std::net::SocketAddr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thiserror::Error;
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use tokio::time::timeout;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(3);
const RESPONSE_BUF_LEN: usize = 16 * 1024;

#[derive(Debug, Error)]
pub enum RendezvousError {
    #[error("network I/O error talking to rendezvous server: {0}")]
    Io(#[from] std::io::Error),
    #[error("rendezvous server did not respond in time")]
    Timeout,
    #[error("rendezvous server returned an error: {0}")]
    ServerError(String),
    #[error("unexpected response variant from rendezvous server")]
    UnexpectedResponse,
    #[error("protocol encode/decode error: {0}")]
    Protocol(#[from] nova_protocol::ProtocolError),
}

/// Thin client for the discovery/relay rendezvous server (`server/`). This server is
/// deliberately minimal and is not a "cloud platform" that owns your data: it never stores
/// messages beyond a short-lived, authenticated relay queue (drained-and-deleted, see
/// `relay_drain`) and it never sees plaintext — everything it relays already left the sender as
/// Double Ratchet ciphertext. It exists purely as a rendezvous point, because two devices behind
/// independent NATs (e.g. two different mobile carriers in two different cities) have no other
/// way to learn each other's current public address. Every P2P messenger needs *some* rendezvous
/// mechanism for this (a signaling server, a DHT bootstrap node, a Tor directory) — it is not a
/// design shortcut, and it can be run by anyone, not necessarily permanently by us.
pub struct RendezvousClient {
    socket: UdpSocket,
    server_addr: SocketAddr,
    // UDP is connectionless, so nothing pairs a response datagram with the request that
    // triggered it except send-then-immediately-recv ordering. `P2PNode` calls into this client
    // concurrently (a presence heartbeat, on-demand lookups, relay sends) from independent
    // tasks, so without this lock two overlapping round trips can each read the *other's*
    // response off the shared socket — this actually happened in testing, manifesting as
    // sporadic `UnexpectedResponse` errors and spurious relay fallbacks. Serializing here trades
    // a small amount of latency for correctness.
    request_lock: Mutex<()>,
}

impl RendezvousClient {
    /// Binds a local UDP socket (use `"0.0.0.0:0"` to let the OS pick a port) and points it at
    /// the rendezvous server's address.
    pub async fn connect(bind_addr: &str, server_addr: SocketAddr) -> Result<Self, RendezvousError> {
        let socket = UdpSocket::bind(bind_addr).await?;
        Ok(Self {
            socket,
            server_addr,
            request_lock: Mutex::new(()),
        })
    }

    pub fn local_addr(&self) -> std::io::Result<SocketAddr> {
        self.socket.local_addr()
    }

    async fn roundtrip(&self, request: &ServerRequest) -> Result<ServerResponse, RendezvousError> {
        let _guard = self.request_lock.lock().await;

        let bytes = request.to_bytes()?;
        self.socket.send_to(&bytes, self.server_addr).await?;

        let mut buf = vec![0u8; RESPONSE_BUF_LEN];
        let (len, _) = timeout(REQUEST_TIMEOUT, self.socket.recv_from(&mut buf))
            .await
            .map_err(|_| RendezvousError::Timeout)??;
        Ok(ServerResponse::from_bytes(&buf[..len])?)
    }

    /// Announces (or refreshes) this device's current reachability, signed so the server can
    /// verify the caller genuinely controls `identity`'s peer_id.
    pub async fn register(
        &self,
        identity: &DeviceIdentity,
        endpoint: PeerEndpoint,
    ) -> Result<(), RendezvousError> {
        let registration = SignedPresenceRegistration::sign(identity, endpoint, now_secs());
        match self.roundtrip(&ServerRequest::Register(registration)).await? {
            ServerResponse::Registered => Ok(()),
            ServerResponse::Error(e) => Err(RendezvousError::ServerError(e)),
            _ => Err(RendezvousError::UnexpectedResponse),
        }
    }

    /// Looks up a peer's last-announced address, so we know where to attempt a direct
    /// connection (and, failing that, where the peer will be checking for relayed traffic).
    pub async fn lookup(&self, peer_id: &str) -> Result<Option<PeerEndpoint>, RendezvousError> {
        match self
            .roundtrip(&ServerRequest::Lookup {
                peer_id: peer_id.to_string(),
            })
            .await?
        {
            ServerResponse::LookupResult(found) => Ok(found),
            ServerResponse::Error(e) => Err(RendezvousError::ServerError(e)),
            _ => Err(RendezvousError::UnexpectedResponse),
        }
    }

    /// Deposits an opaque, already end-to-end-encrypted blob for `target_peer_id`, used when a
    /// direct connection could not be established (e.g. both sides are behind carrier-grade
    /// NAT). The server cannot decrypt this payload.
    pub async fn relay_forward(
        &self,
        target_peer_id: &str,
        payload: Vec<u8>,
    ) -> Result<(), RendezvousError> {
        match self
            .roundtrip(&ServerRequest::RelayForward {
                target_peer_id: target_peer_id.to_string(),
                payload,
            })
            .await?
        {
            ServerResponse::RelayForwarded => Ok(()),
            ServerResponse::Error(e) => Err(RendezvousError::ServerError(e)),
            _ => Err(RendezvousError::UnexpectedResponse),
        }
    }

    /// Fetches and purges every packet currently queued for `identity`'s own peer_id, proving
    /// ownership via a fresh signature.
    pub async fn relay_drain(&self, identity: &DeviceIdentity) -> Result<Vec<Vec<u8>>, RendezvousError> {
        let drain = SignedDrainRequest::sign(identity, now_secs());
        match self.roundtrip(&ServerRequest::RelayDrain(drain)).await? {
            ServerResponse::RelayDrained(items) => Ok(items),
            ServerResponse::Error(e) => Err(RendezvousError::ServerError(e)),
            _ => Err(RendezvousError::UnexpectedResponse),
        }
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is before UNIX epoch")
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use nova_crypto::MnemonicPhrase;
    use nova_server::run_server;

    fn identity(name: &str) -> DeviceIdentity {
        let mnemonic = MnemonicPhrase::generate().unwrap();
        DeviceIdentity::from_mnemonic(&mnemonic, name).unwrap()
    }

    async fn start_test_server() -> SocketAddr {
        let probe = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let addr = probe.local_addr().unwrap();
        drop(probe);
        let bind_addr = addr.to_string();

        let spawn_addr = bind_addr.clone();
        tokio::spawn(async move {
            let _ = run_server(&spawn_addr).await;
        });
        tokio::time::sleep(Duration::from_millis(150)).await;
        addr
    }

    #[tokio::test]
    async fn test_register_and_lookup_roundtrip_through_client() {
        let server_addr = start_test_server().await;
        let alice = identity("alice");
        let client = RendezvousClient::connect("127.0.0.1:0", server_addr).await.unwrap();

        // The claimed IP is discarded by the server, which records the UDP source IP it
        // actually observed instead (see `PresenceRegistry::register`) — a device behind NAT
        // cannot know its own public IP by itself. The claimed port IS kept (it names the
        // caller's own QUIC endpoint port, a different socket than this signaling client).
        let endpoint = PeerEndpoint {
            peer_id: String::new(),
            public_ip: "203.0.113.7".into(),
            public_port: 4242,
            local_ip: None,
            local_port: None,
        };
        let client_local_addr = client.local_addr().unwrap();
        client.register(&alice, endpoint).await.unwrap();

        let found = client.lookup(&alice.public_id_hex()).await.unwrap();
        let found = found.expect("registered peer should be discoverable");
        assert_eq!(found.public_ip, client_local_addr.ip().to_string());
        assert_ne!(found.public_ip, "203.0.113.7");
        assert_eq!(found.public_port, 4242);
    }

    #[tokio::test]
    async fn test_relay_forward_and_drain_roundtrip_through_client() {
        let server_addr = start_test_server().await;
        let bob = identity("bob");
        let client = RendezvousClient::connect("127.0.0.1:0", server_addr).await.unwrap();

        client
            .relay_forward(&bob.public_id_hex(), b"ciphertext_blob".to_vec())
            .await
            .unwrap();

        let drained = client.relay_drain(&bob).await.unwrap();
        assert_eq!(drained, vec![b"ciphertext_blob".to_vec()]);

        // A second drain finds nothing left — the queue was purged, not just read.
        let drained_again = client.relay_drain(&bob).await.unwrap();
        assert!(drained_again.is_empty());
    }
}
