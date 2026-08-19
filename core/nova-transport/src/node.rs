//! The real P2P networking node: this is what actually lets two independent devices — say, one
//! phone on a mobile network in Ouagadougou and another on a different mobile network in
//! Bobo-Dioulasso — find each other and exchange bytes, instead of the crypto/storage layers
//! talking only to themselves in a test.
//!
//! There is no permanent cloud service holding accounts, contacts, or messages here. The only
//! external component involved (`RendezvousClient` / `server/`) is a minimal, replaceable
//! rendezvous point: it tells a peer where another peer was last seen (like a STUN server) and,
//! failing a direct connection, relays already end-to-end-encrypted bytes until they can be
//! picked up. It is not a design shortcut — no two NAT'd devices with no prior contact can find
//! each other over the public internet without *some* rendezvous mechanism, whether that's a
//! signaling server, a DHT, or a Tor directory.
//!
//! Connection strategy, in order:
//! 1. Reuse an already-open direct QUIC connection to the peer, if one exists.
//! 2. Look the peer up via the rendezvous server and attempt a direct QUIC connection to their
//!    last-announced (server-observed, not self-reported) address. On non-symmetric NATs and
//!    same-network peers this succeeds outright.
//! 3. If that fails or times out (common on carrier-grade NAT, which many mobile operators in
//!    West Africa use), fall back to relaying the already-encrypted payload through the
//!    rendezvous server, to be drained by the recipient the next time it polls.

use crate::error::TransportError;
use crate::quic_config::{build_client_config, build_server_config};
use crate::rendezvous::RendezvousClient;
use crate::{P2PTransportMode, TransportSupervisor};
use nova_crypto::DeviceIdentity;
use nova_protocol::{PeerEndpoint, MAX_PACKET_SIZE};
use quinn::Endpoint;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};
use tracing::{debug, info, warn};

/// How long a direct QUIC connection attempt gets before we give up and fall back to relaying.
const DIRECT_CONNECT_TIMEOUT: Duration = Duration::from_secs(4);
/// How often the node re-announces its presence. Comfortably under the registry's 60s entry TTL.
const PRESENCE_HEARTBEAT_INTERVAL: Duration = Duration::from_secs(20);
/// How often the node polls the rendezvous server for relayed packets addressed to it.
const RELAY_POLL_INTERVAL: Duration = Duration::from_secs(2);
/// TLS server name presented during the QUIC handshake. It is not used for authentication (see
/// `quic_config` module docs) — it only satisfies TLS's requirement for *a* server name.
const QUIC_SERVER_NAME: &str = "nova-p2p.local";

pub struct P2PNode {
    identity: DeviceIdentity,
    endpoint: Endpoint,
    rendezvous: RendezvousClient,
    incoming_tx: mpsc::UnboundedSender<Vec<u8>>,
    incoming_rx: Mutex<mpsc::UnboundedReceiver<Vec<u8>>>,
    direct_conns: Mutex<HashMap<String, quinn::Connection>>,
    pub supervisor: Arc<TransportSupervisor>,
}

impl P2PNode {
    /// Starts a node: binds a UDP socket for direct QUIC traffic, connects to the rendezvous
    /// server, and spawns the background tasks that keep presence fresh, accept inbound direct
    /// connections, and drain any relayed packets. Returns an `Arc` since the background tasks
    /// need to keep the node alive independent of the caller's own handle.
    pub async fn start(
        identity: DeviceIdentity,
        quic_bind_addr: &str,
        rendezvous_addr: SocketAddr,
    ) -> Result<Arc<Self>, TransportError> {
        let socket = std::net::UdpSocket::bind(quic_bind_addr)?;
        let server_config = build_server_config()?;
        let client_config = build_client_config()?;

        let runtime = quinn::default_runtime().ok_or(TransportError::NoAsyncRuntime)?;
        let mut endpoint = Endpoint::new(quinn::EndpointConfig::default(), Some(server_config), socket, runtime)?;
        endpoint.set_default_client_config(client_config);

        let rendezvous = RendezvousClient::connect("0.0.0.0:0", rendezvous_addr).await?;
        let (incoming_tx, incoming_rx) = mpsc::unbounded_channel();

        let node = Arc::new(Self {
            identity,
            endpoint,
            rendezvous,
            incoming_tx,
            incoming_rx: Mutex::new(incoming_rx),
            direct_conns: Mutex::new(HashMap::new()),
            supervisor: Arc::new(TransportSupervisor::new()),
        });

        node.clone().spawn_accept_loop();
        node.clone().spawn_relay_poll_loop();
        node.clone().spawn_presence_heartbeat();

        Ok(node)
    }

    pub fn peer_id(&self) -> String {
        self.identity.public_id_hex()
    }

    pub fn local_addr(&self) -> std::io::Result<SocketAddr> {
        self.endpoint.local_addr()
    }

    /// Announces this device's current reachability to the rendezvous server. The public
    /// address stored is whatever the server actually observed the request arrive from — this
    /// call cannot lie about it even if it tried.
    pub async fn announce_presence(&self) -> Result<(), TransportError> {
        let local_addr = self.endpoint.local_addr()?;
        let endpoint = PeerEndpoint {
            peer_id: String::new(),
            public_ip: local_addr.ip().to_string(),
            public_port: local_addr.port(),
            local_ip: None,
            local_port: None,
        };
        self.rendezvous.register(&self.identity, endpoint).await?;
        Ok(())
    }

    /// Ensures a usable connection to `peer_id` exists, attempting a direct QUIC connection
    /// first and reporting whether direct or relay must be used. Does not itself send anything.
    pub async fn connect_to_peer(&self, peer_id: &str) -> Result<P2PTransportMode, TransportError> {
        if self.direct_conns.lock().await.contains_key(peer_id) {
            return Ok(P2PTransportMode::DirectQuic);
        }

        let Some(peer_endpoint) = self.rendezvous.lookup(peer_id).await? else {
            self.supervisor
                .update_peer_state(peer_id.to_string(), P2PTransportMode::Disconnected, 0, String::new())
                .await;
            return Ok(P2PTransportMode::Disconnected);
        };

        let addr: SocketAddr = format!("{}:{}", peer_endpoint.public_ip, peer_endpoint.public_port)
            .parse()
            .map_err(|_| TransportError::InvalidPeerAddress)?;

        let start = tokio::time::Instant::now();
        match tokio::time::timeout(DIRECT_CONNECT_TIMEOUT, self.try_direct_connect(addr)).await {
            Ok(Ok(conn)) => {
                let latency_ms = start.elapsed().as_millis() as u32;
                info!("Direct QUIC connection established to {peer_id} at {addr}");
                self.direct_conns.lock().await.insert(peer_id.to_string(), conn);
                self.supervisor
                    .update_peer_state(peer_id.to_string(), P2PTransportMode::DirectQuic, latency_ms, addr.to_string())
                    .await;
                Ok(P2PTransportMode::DirectQuic)
            }
            Ok(Err(e)) => {
                debug!("Direct connection to {peer_id} at {addr} failed, falling back to relay: {e}");
                self.supervisor
                    .update_peer_state(peer_id.to_string(), P2PTransportMode::RelayedOpaque, 0, "relay".into())
                    .await;
                Ok(P2PTransportMode::RelayedOpaque)
            }
            Err(_) => {
                debug!("Direct connection to {peer_id} at {addr} timed out, falling back to relay");
                self.supervisor
                    .update_peer_state(peer_id.to_string(), P2PTransportMode::RelayedOpaque, 0, "relay".into())
                    .await;
                Ok(P2PTransportMode::RelayedOpaque)
            }
        }
    }

    async fn try_direct_connect(&self, addr: SocketAddr) -> Result<quinn::Connection, TransportError> {
        let connecting = self.endpoint.connect(addr, QUIC_SERVER_NAME)?;
        let conn = connecting.await?;
        Ok(conn)
    }

    /// Sends already-encrypted wire bytes (a serialized `NovaPacket`) to `peer_id`, preferring a
    /// direct connection and transparently falling back to the rendezvous relay. The relay never
    /// sees plaintext: whatever is passed here already left the caller as Double Ratchet
    /// ciphertext.
    pub async fn send_to_peer(&self, peer_id: &str, bytes: Vec<u8>) -> Result<P2PTransportMode, TransportError> {
        if bytes.len() > MAX_PACKET_SIZE {
            return Err(TransportError::InvalidPeerAddress);
        }

        let mode = self.connect_to_peer(peer_id).await?;
        if mode == P2PTransportMode::DirectQuic {
            let conn = self.direct_conns.lock().await.get(peer_id).cloned();
            if let Some(conn) = conn {
                match send_direct(&conn, &bytes).await {
                    Ok(()) => return Ok(P2PTransportMode::DirectQuic),
                    Err(e) => {
                        warn!("Direct send to {peer_id} failed, falling back to relay: {e}");
                        self.direct_conns.lock().await.remove(peer_id);
                    }
                }
            }
        }

        self.rendezvous.relay_forward(peer_id, bytes).await?;
        self.supervisor
            .update_peer_state(peer_id.to_string(), P2PTransportMode::RelayedOpaque, 0, "relay".into())
            .await;
        Ok(P2PTransportMode::RelayedOpaque)
    }

    /// Waits for the next raw packet received from any peer, whether it arrived over a direct
    /// QUIC stream or was drained from the rendezvous relay. Returns `None` only if the node has
    /// been fully shut down.
    pub async fn recv_next(&self) -> Option<Vec<u8>> {
        self.incoming_rx.lock().await.recv().await
    }

    fn spawn_accept_loop(self: Arc<Self>) {
        tokio::spawn(async move {
            while let Some(incoming) = self.endpoint.accept().await {
                let node = self.clone();
                tokio::spawn(async move {
                    match incoming.await {
                        Ok(conn) => node.handle_incoming_connection(conn).await,
                        Err(e) => warn!("Incoming QUIC handshake failed: {e}"),
                    }
                });
            }
        });
    }

    async fn handle_incoming_connection(&self, conn: quinn::Connection) {
        while let Ok(mut recv_stream) = conn.accept_uni().await {
            match recv_stream.read_to_end(MAX_PACKET_SIZE).await {
                Ok(bytes) => {
                    let _ = self.incoming_tx.send(bytes);
                }
                Err(e) => warn!("Failed reading incoming direct stream: {e}"),
            }
        }
    }

    fn spawn_relay_poll_loop(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(RELAY_POLL_INTERVAL);
            loop {
                interval.tick().await;
                match self.rendezvous.relay_drain(&self.identity).await {
                    Ok(items) => {
                        for item in items {
                            let _ = self.incoming_tx.send(item);
                        }
                    }
                    Err(e) => debug!("Relay drain failed: {e}"),
                }
            }
        });
    }

    fn spawn_presence_heartbeat(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(PRESENCE_HEARTBEAT_INTERVAL);
            loop {
                if let Err(e) = self.announce_presence().await {
                    warn!("Presence heartbeat failed: {e}");
                }
                interval.tick().await;
            }
        });
    }
}

async fn send_direct(conn: &quinn::Connection, bytes: &[u8]) -> Result<(), TransportError> {
    let mut send_stream = conn.open_uni().await?;
    send_stream.write_all(bytes).await?;
    send_stream.finish()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use nova_crypto::MnemonicPhrase;
    use tokio::net::UdpSocket as TokioUdpSocket;

    fn identity(name: &str) -> DeviceIdentity {
        let mnemonic = MnemonicPhrase::generate().unwrap();
        DeviceIdentity::from_mnemonic(&mnemonic, name).unwrap()
    }

    async fn start_test_rendezvous_server() -> SocketAddr {
        let probe = TokioUdpSocket::bind("127.0.0.1:0").await.unwrap();
        let addr = probe.local_addr().unwrap();
        drop(probe);
        let bind_addr = addr.to_string();
        tokio::spawn(async move {
            let _ = nova_server::run_server(&bind_addr).await;
        });
        tokio::time::sleep(Duration::from_millis(150)).await;
        addr
    }

    /// The core claim this crate exists to prove: two independently-started nodes — standing in
    /// for, say, a phone in Ouagadougou and a phone in Bobo-Dioulasso — with no shared process
    /// state, discover each other purely through the rendezvous server and exchange real bytes
    /// over a direct QUIC connection.
    #[tokio::test]
    async fn test_two_independent_nodes_connect_directly_and_exchange_bytes() {
        let server_addr = start_test_rendezvous_server().await;

        let alice_id = identity("alice");
        let bob_id = identity("bob");
        let bob_peer_id = bob_id.public_id_hex();

        let alice = P2PNode::start(alice_id, "127.0.0.1:0", server_addr).await.unwrap();
        let bob = P2PNode::start(bob_id, "127.0.0.1:0", server_addr).await.unwrap();

        bob.announce_presence().await.unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;

        let mode = alice
            .send_to_peer(&bob_peer_id, b"hello bob".to_vec())
            .await
            .unwrap();
        assert_eq!(mode, P2PTransportMode::DirectQuic);

        let received = tokio::time::timeout(Duration::from_secs(3), bob.recv_next())
            .await
            .expect("bob should receive the message before the timeout")
            .expect("incoming channel should not be closed");
        assert_eq!(received, b"hello bob");
    }

    /// When a peer's registered address does not actually speak QUIC (standing in for a NAT
    /// direct-connection failure — e.g. carrier-grade NAT on a mobile network), sending must
    /// transparently fall back to the rendezvous relay rather than simply failing.
    #[tokio::test]
    async fn test_falls_back_to_relay_when_peer_is_unreachable_directly() {
        let server_addr = start_test_rendezvous_server().await;

        let alice_id = identity("alice");
        let bob_id = identity("bob");
        let bob_peer_id = bob_id.public_id_hex();

        let alice = P2PNode::start(alice_id, "127.0.0.1:0", server_addr).await.unwrap();

        // Register bob's presence from a socket that is dropped immediately afterward, so the
        // registered address genuinely has nothing listening on it — standing in for a peer
        // whose NAT drops the direct connection attempt. Reusing that same socket afterward
        // would risk Alice's failed QUIC dial packets sitting in its receive queue and being
        // misread as the server's later relay_drain response.
        {
            let throwaway = RendezvousClient::connect("127.0.0.1:0", server_addr).await.unwrap();
            let claimed_endpoint = PeerEndpoint {
                peer_id: String::new(),
                public_ip: "0.0.0.0".into(),
                public_port: 0,
                local_ip: None,
                local_port: None,
            };
            throwaway.register(&bob_id, claimed_endpoint).await.unwrap();
        }

        let mode = alice
            .send_to_peer(&bob_peer_id, b"relayed hello".to_vec())
            .await
            .unwrap();
        assert_eq!(mode, P2PTransportMode::RelayedOpaque);

        let bob_rendezvous = RendezvousClient::connect("127.0.0.1:0", server_addr).await.unwrap();
        let drained = bob_rendezvous.relay_drain(&bob_id).await.unwrap();
        assert_eq!(drained, vec![b"relayed hello".to_vec()]);
    }
}
