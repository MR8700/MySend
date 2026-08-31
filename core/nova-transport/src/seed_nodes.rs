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
use nova_protocol::{
    DirectorySearchResult, PeerEndpoint, SignedDrainRequest, SignedPresenceRegistration,
};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
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

    /// **Multipath Chunk Striping**: Distributes file chunks or message packets across all available
    /// relay servers in parallel using round-robin striping with automatic failover!
    ///
    /// - Chunk $0 \to \text{Server } 0$
    /// - Chunk $1 \to \text{Server } 1$
    /// - Chunk $2 \to \text{Server } 2$
    /// - Chunk $3 \to \text{Server } 0$ ...
    ///
    /// Multiplies transfer bandwidth across $N$ servers while ensuring delivery even if one fails.
    pub async fn send_chunks_multipath(&self, target_peer_id: &str, chunks: Vec<Vec<u8>>) {
        if self.clients.is_empty() || chunks.is_empty() {
            return;
        }

        let num_clients = self.clients.len();
        let target_peer_id = target_peer_id.to_string();
        let clients = self.clients.clone();

        for (i, chunk) in chunks.into_iter().enumerate() {
            let primary_idx = (self.round_robin_counter.fetch_add(1, Ordering::Relaxed) + i) % num_clients;
            let target = target_peer_id.clone();
            let pool_clients = clients.clone();

            tokio::spawn(async move {
                // Try assigned striped client first
                let primary_client = &pool_clients[primary_idx];
                if let Err(e) = primary_client.relay_forward(&target, chunk.clone()).await {
                    debug!(
                        "Striped relay_forward failed on {} ({e}) — attempting failover...",
                        primary_client.server_url()
                    );
                    // Failover: try remaining clients in the pool
                    for (alt_idx, alt_client) in pool_clients.iter().enumerate() {
                        if alt_idx == primary_idx {
                            continue;
                        }
                        if let Ok(()) = alt_client.relay_forward(&target, chunk.clone()).await {
                            debug!("Failover relay_forward succeeded on {}", alt_client.server_url());
                            break;
                        }
                    }
                }
            });
        }
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
