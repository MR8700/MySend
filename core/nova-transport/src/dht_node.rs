//! Backend server-based transport node for NOVA Messenger.
//!
//! All communications (presence, messaging, media chunks, directory search)
//! pass securely through the backend server pool (`nova-server`), preserving
//! End-to-End Encryption (E2EE) with zero P2P/ad-hoc socket requirements.
//! This ensures 100% reliable connectivity on mobile devices, CGNAT networks,
//! and restricted environments without firewall/UPnP/mDNS issues.

use crate::error::TransportError;
use crate::seed_nodes::{fetch_remote_nodes_config, MultiFallbackPool, DEFAULT_PRIMARY_RELAY_URL};
use crate::{P2PTransportMode, TransportSupervisor};
use libp2p::core::multiaddr::Protocol;
use libp2p::{identity, Multiaddr, PeerId};
use nova_crypto::DeviceIdentity;
use nova_protocol::{PeerEndpoint, SignedDrainRequest, SignedPresenceRegistration};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::{mpsc, Mutex};
use tracing::info;

const PRESENCE_HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);
const SERVER_DRAIN_INTERVAL: Duration = Duration::from_secs(30);

/// Application-level outcome of processing one incoming message.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeliveryOutcome {
    Processed,
    Blocked,
    Rejected,
}

#[allow(dead_code)]
#[derive(Debug)]
pub(crate) enum IncomingSource {
    ServerRelay,
    UdpFallback,
}

/// One raw wire packet received from a peer via the backend server relay.
pub struct IncomingMessage {
    pub bytes: Vec<u8>,
    #[allow(dead_code)]
    pub(crate) source: IncomingSource,
}

impl IncomingMessage {
    /// Reports how `nova-engine` handled this packet.
    /// In the server relay model, custody is already acknowledged by the server.
    pub fn respond(self, _outcome: DeliveryOutcome) {}
}

pub struct P2PNode {
    own_peer_id: String,
    libp2p_peer_id: PeerId,
    listen_addrs: Vec<Multiaddr>,
    incoming_rx: Mutex<mpsc::UnboundedReceiver<IncomingMessage>>,
    pub supervisor: Arc<TransportSupervisor>,
    fallback_pool: Arc<tokio::sync::RwLock<MultiFallbackPool>>,
    identity: Arc<DeviceIdentity>,
}

impl P2PNode {
    /// Starts a backend-server-backed transport node: connects persistent real-time
    /// streaming WebSocket connection to the server, starts presence announcements,
    /// and listens for incoming messages.
    pub async fn start(identity: DeviceIdentity, listen_addr: &str) -> Result<Arc<Self>, TransportError> {
        let keypair = identity::Keypair::ed25519_from_bytes(identity.signing_key_bytes.to_vec())
            .map_err(|e| TransportError::Setup(format!("invalid Ed25519 seed: {e}")))?;
        let libp2p_peer_id = keypair.public().to_peer_id();
        let own_peer_id = identity.public_id_hex();

        // Server URL configuration:
        // 1. Explicit env override `NOVA_UDP_FALLBACK_ADDR`
        // 2. Explicit ws:// or wss:// URL passed in `listen_addr`
        // 3. In-process test server if listen_addr contains 127.0.0.1
        // 4. Default primary relay URL (Render)
        let initial_pool = if listen_addr.starts_with("ws://") || listen_addr.starts_with("wss://") {
            MultiFallbackPool::new(vec![listen_addr.to_string()])
        } else if let Ok(url) = std::env::var("NOVA_FALLBACK_SERVER") {
            if url.contains(',') {
                MultiFallbackPool::new(url.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
            } else {
                MultiFallbackPool::new(vec![url.trim().to_string()])
            }
        } else {
            MultiFallbackPool::new(vec![DEFAULT_PRIMARY_RELAY_URL.to_string()])
        };

        let fallback_pool = Arc::new(tokio::sync::RwLock::new(initial_pool.clone()));
        let (incoming_tx, incoming_rx) = mpsc::unbounded_channel();
        let supervisor = Arc::new(TransportSupervisor::new());

        let dummy_listen: Multiaddr = "/dns4/nova-discovery-jllv.onrender.com/tcp/443/wss"
            .parse()
            .unwrap_or_else(|_| Multiaddr::empty());

        let identity_arc = Arc::new(identity);

        let node = Arc::new(Self {
            own_peer_id,
            libp2p_peer_id,
            listen_addrs: vec![dummy_listen],
            incoming_rx: Mutex::new(incoming_rx),
            supervisor,
            fallback_pool: fallback_pool.clone(),
            identity: identity_arc.clone(),
        });

        // 1. Start persistent real-time streaming WebSocket connection to the server pool
        if !initial_pool.is_empty() {
            initial_pool.start_realtime_listener(identity_arc.clone(), incoming_tx.clone());
        }

        // 2. Background Heartbeat & Drain Loop
        let bg_pool = node.fallback_pool.clone();
        let bg_identity = identity_arc.clone();
        let bg_tx = incoming_tx.clone();
        tokio::spawn(async move {
            let mut heartbeat = tokio::time::interval(PRESENCE_HEARTBEAT_INTERVAL);
            heartbeat.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
            heartbeat.tick().await;
            let mut drain = tokio::time::interval(SERVER_DRAIN_INTERVAL);
            drain.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
            drain.tick().await;

            loop {
                tokio::select! {
                    _ = heartbeat.tick() => {
                        let pool = bg_pool.read().await.clone();
                        if !pool.is_empty() {
                            let endpoint = PeerEndpoint {
                                peer_id: bg_identity.public_id_hex(),
                                public_ip: "0.0.0.0".to_string(),
                                public_port: 0,
                                local_ip: None,
                                local_port: None,
                            };
                            let registration = SignedPresenceRegistration::sign(&bg_identity, endpoint, now_secs());
                            tokio::spawn(async move {
                                pool.register_presence_all(registration).await;
                            });
                        }
                    }
                    _ = drain.tick() => {
                        let pool = bg_pool.read().await.clone();
                        if !pool.is_empty() {
                            let drain_req = SignedDrainRequest::sign(&bg_identity, now_secs());
                            let tx = bg_tx.clone();
                            tokio::spawn(async move {
                                pool.drain_incoming_all(drain_req, tx).await;
                            });
                        }
                    }
                }
            }
        });

        // 3. Background Remote Seed Fetcher (Production only):
        // Automatically fetches dynamic server list from GitHub Raw (`network_nodes.json`)
        let is_custom_or_test = listen_addr.starts_with("ws://") || listen_addr.starts_with("wss://") || std::env::var("NOVA_FALLBACK_SERVER").is_ok();
        if !is_custom_or_test {
            let fetcher_pool = node.fallback_pool.clone();
            let fetcher_identity = identity_arc.clone();
            let fetcher_tx = incoming_tx.clone();
            tokio::spawn(async move {
                let remote_cfg = fetch_remote_nodes_config().await;
                if !remote_cfg.fallback_servers.is_empty() {
                    let updated_pool = MultiFallbackPool::from_config(&remote_cfg);
                    *fetcher_pool.write().await = updated_pool.clone();
                    updated_pool.start_realtime_listener(fetcher_identity, fetcher_tx);
                }
            });
        }

        info!("Backend transport node started for peer {}", node.own_peer_id);
        Ok(node)
    }

    pub fn peer_id(&self) -> &str {
        &self.own_peer_id
    }

    /// Dynamically sets or clears the server URL.
    pub async fn set_fallback_server_url(&self, url: Option<String>) {
        let pool = match url {
            Some(u) if !u.trim().is_empty() && u.trim() != "none" => {
                MultiFallbackPool::new(vec![u.trim().to_string()])
            }
            _ => MultiFallbackPool::new(Vec::new()),
        };
        *self.fallback_pool.write().await = pool.clone();
        if !pool.is_empty() {
            pool.start_realtime_listener(self.identity.clone(), mpsc::unbounded_channel().0);
        }
    }

    /// Dynamically sets multiple server URLs for multi-path striping.
    pub async fn set_fallback_server_urls(&self, urls: Vec<String>) {
        let pool = MultiFallbackPool::new(urls);
        *self.fallback_pool.write().await = pool.clone();
        if !pool.is_empty() {
            pool.start_realtime_listener(self.identity.clone(), mpsc::unbounded_channel().0);
        }
    }

    /// Returns the currently active primary server URL, if any.
    pub async fn get_fallback_server_url(&self) -> Option<String> {
        let urls = self.fallback_pool.read().await.urls();
        urls.first().cloned()
    }

    /// Returns all currently active server URLs in the pool.
    pub async fn get_fallback_server_urls(&self) -> Vec<String> {
        self.fallback_pool.read().await.urls()
    }

    /// Publishes this device's public directory profile and PreKey bundle to all servers in parallel.
    pub async fn register_directory_entry(&self, entry: nova_protocol::SignedDirectoryEntry) -> Result<(), TransportError> {
        let pool = self.fallback_pool.read().await.clone();
        pool.register_directory_all(entry).await
    }

    /// Searches the directory across all active servers in parallel, merging results.
    pub async fn search_directory(&self, query: &str) -> Result<Vec<nova_protocol::DirectorySearchResult>, TransportError> {
        let pool = self.fallback_pool.read().await.clone();
        pool.search_directory_merged(query).await
    }

    pub fn listen_addr(&self) -> &Multiaddr {
        &self.listen_addrs[0]
    }

    pub fn listen_addrs(&self) -> &[Multiaddr] {
        &self.listen_addrs
    }

    pub fn full_listen_addr(&self) -> Multiaddr {
        self.listen_addr().clone().with(Protocol::P2p(self.libp2p_peer_id))
    }

    pub fn full_listen_addrs(&self) -> Vec<Multiaddr> {
        self.listen_addrs
            .iter()
            .map(|a| a.clone().with(Protocol::P2p(self.libp2p_peer_id)))
            .collect()
    }

    pub fn local_libp2p_peer_id(&self) -> PeerId {
        self.libp2p_peer_id
    }

    /// API compatibility no-op.
    pub async fn bootstrap_dial(&self, _addr: Multiaddr) -> Result<(), TransportError> {
        Ok(())
    }

    /// API compatibility no-op.
    pub async fn announce_addresses_only(&self, _addrs: Vec<Multiaddr>) -> Result<(), TransportError> {
        Ok(())
    }

    /// API compatibility: returns input address.
    pub async fn reserve_relay_slot(&self, relay_addr: Multiaddr) -> Result<Multiaddr, TransportError> {
        Ok(relay_addr)
    }

    /// Signs and publishes this device's presence to the backend server pool.
    pub async fn announce_presence(&self) -> Result<(), TransportError> {
        let pool = self.fallback_pool.read().await.clone();
        if !pool.is_empty() {
            let endpoint = PeerEndpoint {
                peer_id: self.own_peer_id.clone(),
                public_ip: "0.0.0.0".to_string(),
                public_port: 0,
                local_ip: None,
                local_port: None,
            };
            let registration = SignedPresenceRegistration::sign(&self.identity, endpoint, now_secs());
            pool.register_presence_all(registration).await;
        }
        Ok(())
    }

    /// Looks up whether a peer is known on the backend server.
    pub async fn connect_to_peer(&self, nova_peer_id: &str) -> Result<P2PTransportMode, TransportError> {
        let pool = self.fallback_pool.read().await.clone();
        if let Ok(Some(_endpoint)) = pool.lookup_fastest(nova_peer_id).await {
            self.supervisor
                .update_peer_state(nova_peer_id.to_string(), P2PTransportMode::RelayedOpaque, 50, "backend-server".to_string())
                .await;
            Ok(P2PTransportMode::RelayedOpaque)
        } else {
            self.supervisor
                .update_peer_state(nova_peer_id.to_string(), P2PTransportMode::Disconnected, 0, String::new())
                .await;
            Ok(P2PTransportMode::Disconnected)
        }
    }

    /// Sends already-encrypted wire bytes to `nova_peer_id` via backend server relay.
    pub async fn send_to_peer(
        &self,
        nova_peer_id: &str,
        bytes: Vec<u8>,
    ) -> Result<(P2PTransportMode, DeliveryOutcome), TransportError> {
        self.send_chunks_to_peer(nova_peer_id, vec![bytes]).await
    }

    /// Sends one or more encrypted wire packets to `nova_peer_id` via the backend server relay.
    pub async fn send_chunks_to_peer(
        &self,
        nova_peer_id: &str,
        chunks: Vec<Vec<u8>>,
    ) -> Result<(P2PTransportMode, DeliveryOutcome), TransportError> {
        let pool = self.fallback_pool.read().await.clone();
        if !pool.is_empty() {
            let request_started = tokio::time::Instant::now();
            if pool.send_chunks_multipath(nova_peer_id, chunks).await {
                let latency_ms = request_started.elapsed().as_millis().min(u128::from(u32::MAX)) as u32;
                self.supervisor
                    .update_peer_state(
                        nova_peer_id.to_string(),
                        P2PTransportMode::RelayedOpaque,
                        latency_ms,
                        "backend-server".to_string(),
                    )
                    .await;
                return Ok((P2PTransportMode::RelayedOpaque, DeliveryOutcome::Processed));
            }
        }

        self.supervisor
            .update_peer_state(nova_peer_id.to_string(), P2PTransportMode::Disconnected, 0, String::new())
            .await;
        Ok((P2PTransportMode::Disconnected, DeliveryOutcome::Rejected))
    }

    /// Waits for the next raw packet received from any peer over the real-time server stream or drain.
    pub async fn recv_next(&self) -> Option<IncomingMessage> {
        self.incoming_rx.lock().await.recv().await
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

    fn identity(name: &str) -> DeviceIdentity {
        let mnemonic = MnemonicPhrase::generate().unwrap();
        DeviceIdentity::from_mnemonic(&mnemonic, name).unwrap()
    }

    async fn start_test_server() -> String {
        let (listener, registry, relay) = nova_server::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(nova_server::serve_forever(listener, registry, relay));
        tokio::time::sleep(Duration::from_millis(50)).await;
        format!("ws://{addr}/")
    }

    #[tokio::test]
    async fn test_two_independent_nodes_exchange_bytes_via_server() {
        let server_url = start_test_server().await;
        let alice_id = identity("alice");
        let bob_id = identity("bob");
        let bob_peer_id = bob_id.public_id_hex();

        let alice = P2PNode::start(alice_id, &server_url).await.unwrap();
        let bob = P2PNode::start(bob_id, &server_url).await.unwrap();

        // Give the WebSocket connection a moment to connect and register
        tokio::time::sleep(Duration::from_millis(300)).await;

        let bob_recv = bob.clone();
        let recv_task = tokio::spawn(async move {
            let incoming = tokio::time::timeout(Duration::from_secs(5), bob_recv.recv_next())
                .await
                .expect("bob should receive the message before timeout")
                .expect("incoming channel should not be closed");
            incoming.bytes
        });

        let (mode, outcome) = alice
            .send_to_peer(&bob_peer_id, b"hello bob via server".to_vec())
            .await
            .unwrap();
        assert_eq!(mode, P2PTransportMode::RelayedOpaque);
        assert_eq!(outcome, DeliveryOutcome::Processed);

        let received = recv_task.await.expect("bob's receive task must not panic");
        assert_eq!(received, b"hello bob via server");
    }

    #[tokio::test]
    async fn test_lookup_of_unknown_peer_returns_disconnected() {
        let server_url = start_test_server().await;
        let alice_id = identity("alice");
        let alice = P2PNode::start(alice_id, &server_url).await.unwrap();

        let fake_peer_id = hex::encode([0x42u8; 32]);
        let mode = alice.connect_to_peer(&fake_peer_id).await.unwrap();
        assert_eq!(mode, P2PTransportMode::Disconnected);
    }
}
