//! Dynamic Seed Node & Fallback Relay Pool.
//!
//! Fetches and parses `network_nodes.json` dynamically from GitHub Raw CDN, allowing
//! discovery/relay nodes to be added, swapped, or scaled without requiring any client
//! rebuilds or application reinstalls.
//!
//! Also implements **Multipath Data Striping**: file chunks and messages are distributed
//! concurrently across all active relay servers in parallel, multiplying transfer throughput
//! and providing automatic failover if any server is slow or sleeping.

use crate::error::TransportError;
use crate::udp_fallback::UdpFallbackClient;
use futures_util::{SinkExt, StreamExt};
use nova_protocol::{
    DirectorySearchResult, PeerEndpoint, ServerRequest, ServerResponse, SignedDrainRequest,
    SignedPresenceRegistration,
};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;
use tracing::{debug, info, warn};

/// GitHub Raw URL for the dynamic node list.
pub const GITHUB_RAW_NODES_URL: &str =
    "https://raw.githubusercontent.com/MR8700/MySend/main/config/network_nodes.json";

/// Hardcoded built-in default relay URL used if remote fetch fails or device is offline at first launch.
pub const DEFAULT_PRIMARY_RELAY_URL: &str = "wss://nova-discovery-jllv.onrender.com";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FallbackServerEntry {
    pub url: String,
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_priority")]
    pub priority: u32,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_priority() -> u32 {
    1
}

fn default_enabled() -> bool {
    true
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetworkNodesConfig {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default)]
    pub updated_at: String,
    #[serde(default)]
    pub fallback_servers: Vec<FallbackServerEntry>,
    #[serde(default)]
    pub bootstrap_dht_nodes: Vec<String>,
}

fn default_version() -> u32 {
    1
}

impl Default for NetworkNodesConfig {
    fn default() -> Self {
        Self {
            version: 1,
            updated_at: "default".to_string(),
            fallback_servers: vec![FallbackServerEntry {
                url: DEFAULT_PRIMARY_RELAY_URL.to_string(),
                name: "Render Default".to_string(),
                priority: 1,
                enabled: true,
            }],
            bootstrap_dht_nodes: Vec::new(),
        }
    }
}

/// Asynchronously fetches the latest `network_nodes.json` from GitHub Raw with a short timeout.
/// Falls back safely to default built-in configuration if offline or unavailable.
pub async fn fetch_remote_nodes_config() -> NetworkNodesConfig {
    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(4))
        .build()
    {
        Ok(c) => c,
        Err(_) => return NetworkNodesConfig::default(),
    };

    match client.get(GITHUB_RAW_NODES_URL).send().await {
        Ok(resp) if resp.status().is_success() => match resp.json::<NetworkNodesConfig>().await {
            Ok(config) => {
                info!(
                    "Successfully fetched dynamic network nodes ({} fallback servers, {} bootstrap nodes)",
                    config.fallback_servers.len(),
                    config.bootstrap_dht_nodes.len()
                );
                config
            }
            Err(e) => {
                warn!("Failed to parse remote network_nodes.json: {e} — using defaults");
                NetworkNodesConfig::default()
            }
        },
        Ok(resp) => {
            debug!("Remote network_nodes.json returned status: {} — using defaults", resp.status());
            NetworkNodesConfig::default()
        }
        Err(e) => {
            debug!("Could not reach remote network_nodes.json ({e}) — using defaults");
            NetworkNodesConfig::default()
        }
    }
}

/// A pool of concurrent fallback relay clients supporting Multi-Server Striping and Failover.
#[derive(Clone, Debug)]
pub struct MultiFallbackPool {
    clients: Arc<Vec<Arc<UdpFallbackClient>>>,
    round_robin_counter: Arc<AtomicUsize>,
}

impl MultiFallbackPool {
    /// Creates a pool from a list of `ws://` or `wss://` URLs.
    pub fn new(urls: Vec<String>) -> Self {
        let mut valid_clients = Vec::new();
        for url in urls {
            let trimmed = url.trim();
            if trimmed.starts_with("ws://") || trimmed.starts_with("wss://") {
                valid_clients.push(Arc::new(UdpFallbackClient::new(trimmed.to_string())));
            }
        }
        if valid_clients.is_empty() {
            valid_clients.push(Arc::new(UdpFallbackClient::new(DEFAULT_PRIMARY_RELAY_URL)));
        }
        Self {
            clients: Arc::new(valid_clients),
            round_robin_counter: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Creates a pool from a `NetworkNodesConfig`.
    pub fn from_config(config: &NetworkNodesConfig) -> Self {
        let mut urls: Vec<String> = config
            .fallback_servers
            .iter()
            .filter(|s| s.enabled)
            .map(|s| s.url.clone())
            .collect();

        if urls.is_empty() {
            urls.push(DEFAULT_PRIMARY_RELAY_URL.to_string());
        }
        Self::new(urls)
    }

    /// Returns the list of active server URLs in the pool.
    pub fn urls(&self) -> Vec<String> {
        self.clients.iter().map(|c| c.server_url().to_string()).collect()
    }

    /// Returns the primary/first server URL in the pool.
    pub fn primary_url(&self) -> String {
        self.clients
            .first()
            .map(|c| c.server_url().to_string())
            .unwrap_or_else(|| DEFAULT_PRIMARY_RELAY_URL.to_string())
    }

    /// Returns true if the pool has no valid clients.
    pub fn is_empty(&self) -> bool {
        self.clients.is_empty()
    }

    /// Returns the number of active relay servers in the pool.
    pub fn len(&self) -> usize {
        self.clients.len()
    }

    /// Registers directory entry on ALL fallback servers in parallel.
    pub async fn register_directory_all(&self, entry: nova_protocol::SignedDirectoryEntry) -> Result<(), TransportError> {
        if self.clients.is_empty() {
            return Err(TransportError::Setup("no fallback server configured in pool".into()));
        }

        let mut handles = Vec::new();
        for client in self.clients.iter() {
            let client = client.clone();
            let entry = entry.clone();
            handles.push(tokio::spawn(async move {
                client.register_directory_signed(entry).await
            }));
        }

        let mut last_err = None;
        let mut any_success = false;
        for handle in handles {
            match handle.await {
                Ok(Ok(())) => any_success = true,
                Ok(Err(e)) => last_err = Some(e),
                Err(_) => {}
            }
        }

        if any_success {
            Ok(())
        } else {
            Err(last_err.unwrap_or_else(|| TransportError::Setup("all fallback directory registrations failed".into())))
        }
    }

    /// Registers presence on ALL fallback servers in parallel so any peer on any relay can locate us.
    pub async fn register_presence_all(&self, registration: SignedPresenceRegistration) {
        let clients = self.clients.clone();
        for client in clients.iter() {
            let client = client.clone();
            let reg = registration.clone();
            tokio::spawn(async move {
                if let Err(e) = client.register_presence_signed(reg).await {
                    debug!("Presence registration failed on {}: {e}", client.server_url());
                } else {
                    debug!("Presence registered on {}", client.server_url());
                }
            });
        }
    }

    /// Drains incoming queued packets from ALL fallback servers concurrently in parallel tasks,
    /// multiplexing all received packets into `incoming_tx`.
    pub async fn drain_incoming_all(
        &self,
        drain_request: SignedDrainRequest,
        incoming_tx: mpsc::UnboundedSender<crate::dht_node::IncomingMessage>,
    ) {
        let clients = self.clients.clone();
        for client in clients.iter() {
            let client = client.clone();
            let drain_req = drain_request.clone();
            let tx = incoming_tx.clone();
            tokio::spawn(async move {
                match client.drain_incoming_signed(drain_req).await {
                    Ok(items) => {
                        for bytes in items {
                            let _ = tx.send(crate::dht_node::IncomingMessage {
                                bytes,
                                source: crate::dht_node::IncomingSource::UdpFallback,
                            });
                        }
                    }
                    Err(e) => {
                        debug!("Drain failed on {}: {e}", client.server_url());
                    }
                }
            });
        }
    }

    /// Starts a persistent real-time streaming WebSocket connection to the server(s) in the pool.
    /// Whenever an incoming message is pushed by the server, it is immediately emitted to `incoming_tx`.
    pub fn start_realtime_listener(
        &self,
        identity: Arc<nova_crypto::DeviceIdentity>,
        incoming_tx: mpsc::UnboundedSender<crate::dht_node::IncomingMessage>,
    ) {
        let clients = self.clients.clone();
        for client in clients.iter() {
            let server_url = client.server_url().to_string();
            let identity = identity.clone();
            let tx = incoming_tx.clone();
            tokio::spawn(async move {
                let mut retry_delay = Duration::from_millis(500);
                loop {
                    debug!("Connecting persistent real-time messaging WebSocket to {server_url}...");
                    match tokio::time::timeout(
                        Duration::from_secs(15),
                        tokio_tungstenite::connect_async(&server_url),
                    ).await {
                        Ok(Ok((mut ws, _))) => {
                            retry_delay = Duration::from_millis(500);
                            info!("Connected persistent real-time messaging WebSocket to {server_url}");

                            // 1. Announce presence to bind this WebSocket connection to our identity
                            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                            let endpoint = PeerEndpoint {
                                peer_id: identity.public_id_hex(),
                                public_ip: "0.0.0.0".to_string(),
                                public_port: 0,
                                local_ip: None,
                                local_port: None,
                            };
                            let registration = SignedPresenceRegistration::sign(&identity, endpoint, now);
                            if let Ok(bytes) = ServerRequest::Register(registration).to_bytes() {
                                let _ = ws.send(Message::Binary(bytes)).await;
                            }

                            // 2. Initial drain of any offline queue
                            let drain_req = SignedDrainRequest::sign(&identity, now);
                            if let Ok(bytes) = ServerRequest::RelayDrain(drain_req).to_bytes() {
                                let _ = ws.send(Message::Binary(bytes)).await;
                            }

                            // 3. Heartbeat + Receive loop
                            let mut heartbeat = tokio::time::interval(Duration::from_secs(20));
                            heartbeat.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

                            loop {
                                tokio::select! {
                                    _ = heartbeat.tick() => {
                                        if ws.send(Message::Ping(Vec::new())).await.is_err() {
                                            break;
                                        }
                                    }
                                    msg = ws.next() => {
                                        match msg {
                                            Some(Ok(Message::Binary(bytes))) => {
                                                if let Ok(response) = ServerResponse::from_bytes(&bytes) {
                                                    match response {
                                                        ServerResponse::RelayDrained(items) => {
                                                            for payload in items {
                                                                let _ = tx.send(crate::dht_node::IncomingMessage {
                                                                    bytes: payload,
                                                                    source: crate::dht_node::IncomingSource::UdpFallback,
                                                                });
                                                            }
                                                        }
                                                        _ => {}
                                                    }
                                                }
                                            }
                                            Some(Ok(Message::Ping(payload))) => {
                                                let _ = ws.send(Message::Pong(payload)).await;
                                            }
                                            Some(Ok(Message::Pong(_))) => {}
                                            Some(Ok(Message::Close(_))) | None => {
                                                debug!("Real-time WebSocket connection to {server_url} closed");
                                                break;
                                            }
                                            Some(Err(e)) => {
                                                debug!("Real-time WebSocket error from {server_url}: {e}");
                                                break;
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                            }
                        }
                        Ok(Err(e)) => {
                            debug!("Failed to connect real-time WebSocket to {server_url}: {e}");
                        }
                        Err(_) => {
                            debug!("Timeout connecting real-time WebSocket to {server_url}");
                        }
                    }

                    tokio::time::sleep(retry_delay).await;
                    retry_delay = (retry_delay * 2).min(Duration::from_secs(10));
                }
            });
        }
    }

    /// **Multipath Chunk Striping**: Distributes file chunks or message packets across all available
    /// relay servers in parallel using round-robin striping with automatic failover!
    ///
    /// - Chunk $0 \to \text{Server } 0$
    /// - Chunk $1 \to \text{Server } 1$
    /// - Chunk $2 \to \text{Server } 2$
    /// - Chunk $3 \to \text{Server } 0$ ...
    ///
    pub async fn send_chunks_multipath(&self, target_peer_id: &str, chunks: Vec<Vec<u8>>) -> bool {
        if self.clients.is_empty() || chunks.is_empty() {
            return false;
        }

        let num_clients = self.clients.len();
        let target_peer_id = target_peer_id.to_string();
        let clients = self.clients.clone();
        let mut handles = Vec::new();

        for (i, chunk) in chunks.into_iter().enumerate() {
            let primary_idx = (self.round_robin_counter.fetch_add(1, Ordering::Relaxed) + i) % num_clients;
            let target = target_peer_id.clone();
            let pool_clients = clients.clone();

            handles.push(tokio::spawn(async move {
                // Try assigned striped client first
                let primary_client = &pool_clients[primary_idx];
                if primary_client.relay_forward(&target, chunk.clone()).await.is_ok() {
                    return true;
                }
                debug!(
                    "Striped relay_forward failed on {} — attempting failover...",
                    primary_client.server_url()
                );
                // Failover: try remaining clients in the pool
                for (alt_idx, alt_client) in pool_clients.iter().enumerate() {
                    if alt_idx == primary_idx {
                        continue;
                    }
                    if alt_client.relay_forward(&target, chunk.clone()).await.is_ok() {
                        debug!("Failover relay_forward succeeded on {}", alt_client.server_url());
                        return true;
                    }
                }
                false
            }));
        }

        let mut all_ok = true;
        for handle in handles {
            match handle.await {
                Ok(true) => {}
                _ => all_ok = false,
            }
        }
        all_ok
    }

    /// Queries all active relays in parallel for a peer's endpoint and returns the fastest response.
    pub async fn lookup_fastest(&self, peer_id: &str) -> Result<Option<PeerEndpoint>, TransportError> {
        if self.clients.is_empty() {
            return Ok(None);
        }

        let mut handles = Vec::new();
        for client in self.clients.iter() {
            let client = client.clone();
            let pid = peer_id.to_string();
            handles.push(tokio::spawn(async move {
                client.lookup(&pid).await
            }));
        }

        for handle in handles {
            if let Ok(Ok(Some(endpoint))) = handle.await {
                return Ok(Some(endpoint));
            }
        }

        Ok(None)
    }

    /// Searches directory across all relays in parallel, merging and deduplicating results by `peer_id`.
    pub async fn search_directory_merged(&self, query: &str) -> Result<Vec<DirectorySearchResult>, TransportError> {
        if self.clients.is_empty() {
            return Ok(Vec::new());
        }

        let mut handles = Vec::new();
        for client in self.clients.iter() {
            let client = client.clone();
            let q = query.to_string();
            handles.push(tokio::spawn(async move {
                client.search_directory(&q).await
            }));
        }

        let mut merged = Vec::new();
        let mut seen_ids = std::collections::HashSet::new();

        for handle in handles {
            if let Ok(Ok(results)) = handle.await {
                for entry in results {
                    if seen_ids.insert(entry.peer_id.clone()) {
                        merged.push(entry);
                    }
                }
            }
        }

        Ok(merged)
    }
}
