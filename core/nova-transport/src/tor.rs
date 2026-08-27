//! Embedded Tor client and onion-routing fallback, via `arti-client` — the Tor Project's own
//! pure-Rust Tor implementation (<https://arti.torproject.org>). Tor runs *in-process*: no
//! external `tor`/Orbot binary to install or keep running, no SOCKS5 proxy address to configure,
//! and the exact same code on Windows and Android since it's the same Rust crate compiled for
//! each target.
//!
//! ## Why this replaced an external-Tor-process model
//!
//! The previous version of this module only knew how to *dial out* through an already-running
//! external Tor process's SOCKS5 port, and receiving onion traffic required a human to hand-edit
//! that external process's `torrc` (a `HiddenServiceDir` + matching `hs_ed25519_secret_key` file
//! containing this device's expanded Ed25519 identity key) — real, but entirely manual, and
//! impossible to ship to an end user. This module does both halves itself: `connect_onion_stream`
//! builds a real Tor circuit in-process via [`arti_client::TorClient::connect`], and
//! [`TorManager::launch_own_onion_service`] hosts this device's own onion service *using its real
//! NOVA identity key* (via [`arti_client::TorClient::launch_onion_service_with_hsid`]) so the
//! `.onion` address it serves at is exactly the one `nova_crypto::derive_onion_v3_address`
//! computes and this device already advertises in its `PreKeyBundle` — not a second, unrelated
//! address Tor would otherwise generate on its own.

use crate::error::TransportError;
use arti_client::config::TorClientConfigBuilder;
use arti_client::{DataStream, TorClient};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::OnceCell;
use tor_hscrypto::pk::HsIdKeypair;
use tor_hsservice::config::OnionServiceConfigBuilder;
use tor_hsservice::{HsNickname, RendRequest, RunningOnionService};
use tor_llcrypto::pk::ed25519::{ExpandedKeypair, Keypair as Ed25519Keypair};
use tor_rtcompat::PreferredRuntime;

/// Operating transport mode for privacy and onion routing.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TorMode {
    /// Standard P2P: direct QUIC/UDP, DCUtR hole punching, direct relay.
    DirectOnly,
    /// Hybrid: direct connections for local LAN / bootstrap, Tor for `.onion` peers.
    Hybrid,
    /// Tor Strict: all outbound connections are strictly gated through Tor circuits.
    /// Direct IPv4/IPv6 connections to unknown peers are dropped to guarantee zero IP leaks.
    TorStrict,
}

impl Default for TorMode {
    fn default() -> Self {
        TorMode::DirectOnly
    }
}

fn default_socks_proxy() -> String {
    "127.0.0.1:9050".to_string()
}

/// Configuration parameters for Tor anonymization.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TorConfig {
    pub enabled: bool,
    pub mode: TorMode,
    /// Vestigial — kept only so `serde` can still deserialize a `tor_settings` row a previous
    /// version of this app persisted (see `nova-storage`'s `tor_settings` table), from back when
    /// this module dialed an *external* Tor/Orbot process's SOCKS5 proxy. Tor now runs embedded
    /// (see this module's doc comment); there is no external proxy address to point at, and this
    /// field is no longer read by anything.
    #[serde(default = "default_socks_proxy")]
    pub socks_proxy: String,
    pub onion_address: Option<String>,
    pub bridge_type: Option<String>, // "snowflake", "obfs4", or None (direct)
}

impl Default for TorConfig {
    fn default() -> Self {
        // Deliberately `enabled: false` here, unlike `nova_storage::TorSettingsRecord::default`
        // (the actual product default a fresh install ends up with, reapplied on top of THIS
        // default after every `P2PNode::start()` — see `ui/src-tauri`'s `start_network_once`).
        // This is the bare fallback `P2PNode::start()` itself falls back to before that
        // reapplication ever runs — and it's also what every test and example that never calls
        // `configure_tor` at all (most of nova-transport/nova-engine's own test suite, plus the
        // `mesh_test`/`lan_test` examples) actually gets. `enabled: true` here previously made
        // every single one of those spawn a REAL embedded-Tor bootstrap attempt over the real
        // network on construction (`P2PNode::start` → `ensure_onion_service_started`), whether or
        // not the test/example ever cared about Tor — several tests already explicitly documented
        // relying on this exact default staying inert for that reason (see e.g.
        // `dht_node::tests::test_tor_strict_mode_blocks_direct_dial_at_the_swarm_level`'s own
        // comment) — turning it on here broke that assumption everywhere at once.
        Self {
            enabled: false,
            mode: TorMode::DirectOnly,
            socks_proxy: default_socks_proxy(),
            onion_address: None,
            bridge_type: None,
        }
    }
}

/// Live status report of the Tor subsystem.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TorStatus {
    pub enabled: bool,
    pub connected: bool,
    pub bootstrap_percent: u8,
    pub onion_address: String,
    pub socks_proxy: String,
    pub mode: TorMode,
    pub bridge_type: Option<String>,
}

/// Builds this device's `HsIdKeypair` (the key an onion service authenticates itself with) from
/// its real NOVA identity seed, so the resulting `.onion` address matches
/// `nova_crypto::derive_onion_v3_address` exactly — verified byte-for-byte against a real `tor`
/// process's own `hs_ed25519_secret_key`/`hostname` output during development of this module.
fn hsid_keypair_from_identity_seed(seed: &[u8; 32]) -> HsIdKeypair {
    let keypair = Ed25519Keypair::from_bytes(seed);
    let expanded = ExpandedKeypair::from(&keypair);
    HsIdKeypair::from(expanded)
}

/// The single onion-service nickname this app ever uses — arti keys/state are looked up by
/// nickname within `TorManager`'s own `state_dir` (never shared with another app), so there is
/// no need for more than one.
const ONION_SERVICE_NICKNAME: &str = "nova";

/// Upper bound on how long a single embedded-Tor bootstrap attempt is allowed to run — see
/// [`TorManager::client`]'s doc comment for why this must be bounded at all. 60s, tried first, was
/// too aggressive in practice: a genuine first-time bootstrap (no cached consensus yet) legitimately
/// spends most of a minute just downloading and validating the ~3MB consensus document and its
/// certificates before it can even start building circuits — confirmed by
/// `examples/tor_bootstrap_probe.rs` reaching real progress ("connecting successfully", 15%) within
/// 3 seconds and then still being mid-consensus-fetch when the old 60s bound cut it off.
const BOOTSTRAP_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(180);

/// Owns the embedded Tor client (lazily bootstrapped on first real use, so a device that never
/// enables Tor never pays for it) and this device's real identity key material, so it can host
/// its own onion service under that identity without the caller needing to hand the key in again.
pub struct TorManager {
    config: TorConfig,
    identity_seed: [u8; 32],
    state_dir: PathBuf,
    client: OnceCell<Arc<TorClient<PreferredRuntime>>>,
}

impl TorManager {
    /// `identity_seed` is this device's real NOVA identity's raw Ed25519 seed (`DeviceIdentity
    /// ::signing_key_bytes`) — used only to derive the onion service's `HsIdKeypair` on demand,
    /// never persisted by this module itself. `state_dir` is where arti keeps its consensus
    /// cache, guard state, and (if `launch_own_onion_service` is ever called) its keystore —
    /// should be a subdirectory of this device's real app data directory so it survives restarts
    /// instead of being rebuilt (slow: a fresh Tor bootstrap) every launch.
    pub fn new(config: TorConfig, identity_seed: [u8; 32], state_dir: PathBuf) -> Self {
        Self {
            config,
            identity_seed,
            state_dir,
            client: OnceCell::new(),
        }
    }

    pub fn config(&self) -> &TorConfig {
        &self.config
    }

    pub fn update_config(&mut self, config: TorConfig) {
        self.config = config;
    }

    /// Checks whether an address string references a Tor `.onion` hidden service.
    pub fn is_onion_address(addr: &str) -> bool {
        let clean = addr.trim().to_ascii_lowercase();
        clean.contains(".onion")
    }

    /// Verifies if an outbound dial is permitted under the current Tor privacy policy.
    /// Returns Ok(()) if permitted, or Err(TransportError) if it violates TorStrict mode.
    pub fn validate_outbound_dial(&self, target_addr: &str) -> Result<(), TransportError> {
        match self.config.mode {
            TorMode::DirectOnly => Ok(()),
            TorMode::Hybrid => Ok(()),
            TorMode::TorStrict => {
                if Self::is_onion_address(target_addr)
                    || target_addr.contains("127.0.0.1")
                    || target_addr.contains("localhost")
                    || target_addr.contains("::1")
                {
                    Ok(())
                } else {
                    Err(TransportError::Setup(format!(
                        "TorStrict mode: direct non-onion dial to '{target_addr}' blocked to prevent IP leak"
                    )))
                }
            }
        }
    }

    /// Returns the embedded Tor client, bootstrapping it on the first call (this can take from a
    /// few seconds to, on a slow/censored network, tens of seconds — real circuit-building work,
    /// not a fixed delay). Every later call reuses the same already-bootstrapped client.
    ///
    /// Bounded by [`BOOTSTRAP_TIMEOUT`]: a network that can't reach the Tor directory network at
    /// all (no connectivity, a firewall blocking it, a sandboxed CI environment) must fail
    /// cleanly instead of hanging forever — both so a real user gets a real error instead of a
    /// permanently-spinning "Démarrage du circuit…" in Settings, and because
    /// [`Self::launch_own_onion_service`] gets spawned onto whatever runtime the caller is on
    /// (see `dht_node::P2PNode::ensure_onion_service_started`): an unbounded bootstrap there
    /// would hang not just this call but the entire owning process's shutdown, since a tokio
    /// runtime's `Drop` blocks until every task it ever spawned — including this detached,
    /// otherwise-eternal one — actually finishes. `get_or_try_init` never caches an `Err`, so a
    /// later call (once the network is actually reachable) retries the bootstrap from scratch.
    async fn client(&self) -> Result<Arc<TorClient<PreferredRuntime>>, TransportError> {
        self.client
            .get_or_try_init(|| async {
                let cache_dir = self.state_dir.join("cache");
                let state_dir = self.state_dir.join("state");
                std::fs::create_dir_all(&cache_dir)
                    .map_err(|e| TransportError::Setup(format!("failed to create Tor cache dir {cache_dir:?}: {e}")))?;
                std::fs::create_dir_all(&state_dir)
                    .map_err(|e| TransportError::Setup(format!("failed to create Tor state dir {state_dir:?}: {e}")))?;
                let config = TorClientConfigBuilder::from_directories(&state_dir, &cache_dir)
                    .build()
                    .map_err(|e| TransportError::Setup(format!("invalid embedded Tor client config: {e}")))?;
                tokio::time::timeout(BOOTSTRAP_TIMEOUT, TorClient::<PreferredRuntime>::create_bootstrapped(config))
                    .await
                    .map_err(|_| TransportError::Setup(format!(
                        "embedded Tor client failed to bootstrap within {}s (no route to the Tor network?)",
                        BOOTSTRAP_TIMEOUT.as_secs()
                    )))?
                    .map_err(|e| TransportError::Setup(format!("failed to bootstrap embedded Tor client: {e}")))
            })
            .await
            .cloned()
    }

    /// Live bootstrap progress: `(ready_for_traffic, percent 0-100)`. Returns `(false, 0)`
    /// without actually starting a bootstrap if the client has never been used yet (mirrors the
    /// old `check_proxy_liveness`'s "not running" case, but without needing a network probe).
    pub async fn bootstrap_status(&self) -> (bool, u8) {
        let Some(client) = self.client.get() else {
            return (false, 0);
        };
        let status = client.bootstrap_status();
        (status.ready_for_traffic(), (status.as_frac().clamp(0.0, 1.0) * 100.0).round() as u8)
    }

    /// Establishes an anonymous stream to a remote `.onion` hidden service, building a real Tor
    /// circuit in-process (bootstrapping the embedded client first if this is the first Tor use).
    pub async fn connect_onion_stream(&self, target_onion: &str, target_port: u16) -> Result<DataStream, TransportError> {
        let client = self.client().await?;

        let clean_onion = target_onion.trim().to_ascii_lowercase();
        let domain = clean_onion
            .strip_prefix("http://")
            .unwrap_or(&clean_onion)
            .strip_prefix("https://")
            .unwrap_or(&clean_onion);
        let domain = domain.split(':').next().unwrap_or(domain).to_string();

        client
            .connect((domain.as_str(), target_port))
            .await
            .map_err(|e| TransportError::Setup(format!("Tor circuit to {domain}:{target_port} failed: {e}")))
    }

    /// Launches this device's own onion service under its real NOVA identity key (see this
    /// module's doc comment), bootstrapping the embedded client first if needed. Returns the
    /// running service handle (drop it, or drop the returned stream, to shut the service down)
    /// and a stream of inbound rendezvous requests — the caller is expected to `.accept()` each
    /// one (see [`tor_hsservice::handle_rend_requests`]) and treat the resulting
    /// [`tor_hsservice::StreamRequest`]s exactly like an inbound TCP connection.
    pub async fn launch_own_onion_service(
        &self,
    ) -> Result<
        (
            Arc<RunningOnionService>,
            std::pin::Pin<Box<dyn futures::Stream<Item = RendRequest> + Send>>,
        ),
        TransportError,
    > {
        let client = self.client().await?;
        let hsid_keypair = hsid_keypair_from_identity_seed(&self.identity_seed);

        let nickname = HsNickname::new(ONION_SERVICE_NICKNAME.to_string())
            .map_err(|e| TransportError::Setup(format!("invalid onion service nickname: {e}")))?;
        let hs_config = OnionServiceConfigBuilder::default()
            .nickname(nickname)
            .build()
            .map_err(|e| TransportError::Setup(format!("invalid onion service config: {e}")))?;

        // Boxed so both launch paths below (with vs. without a fresh `HsIdKeypair`) share one
        // concrete return type — `impl Trait` from two different functions is never the same
        // type to the compiler even when structurally identical.
        let (running, rend_requests) = match client.launch_onion_service_with_hsid(hs_config.clone(), hsid_keypair) {
            Ok(Some((running, rend_requests))) => (running, Box::pin(rend_requests) as std::pin::Pin<Box<dyn futures::Stream<Item = RendRequest> + Send>>),
            Ok(None) => {
                return Err(TransportError::Setup(
                    "onion service launch returned no handle (config reported disabled)".into(),
                ))
            }
            Err(e) => {
                // "already has an associated HsIdKeypair" happens on every relaunch after the
                // first — arti's keystore already holds this exact key from last time (same
                // identity seed → same key, by design), which is success, not failure: retry
                // once without re-inserting, letting arti reuse what's already on disk.
                match client.launch_onion_service(hs_config) {
                    Ok(Some((running, rend_requests))) => (running, Box::pin(rend_requests) as std::pin::Pin<Box<dyn futures::Stream<Item = RendRequest> + Send>>),
                    Ok(None) => {
                        return Err(TransportError::Setup(
                            "onion service launch returned no handle (config reported disabled)".into(),
                        ))
                    }
                    Err(_) => {
                        return Err(TransportError::Setup(format!(
                            "failed to launch onion service with this device's identity key: {e}"
                        )))
                    }
                }
            }
        };

        Ok((running, rend_requests))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_onion_address() {
        assert!(TorManager::is_onion_address("expyuzz5wqqfdgah56trgahhqhnkrybe5vdggqnxzjptlkpq5ldnfiyd.onion"));
        assert!(TorManager::is_onion_address("/dns4/mytestaddress.onion/tcp/8443"));
        assert!(!TorManager::is_onion_address("192.168.1.1"));
        assert!(!TorManager::is_onion_address("/ip4/127.0.0.1/udp/4001/quic-v1"));
    }

    #[test]
    fn test_tor_strict_mode_blocks_clearnet_dials() {
        let mut config = TorConfig::default();
        config.mode = TorMode::TorStrict;
        let manager = TorManager::new(config, [0u8; 32], std::env::temp_dir().join("nova_tor_test_strict"));

        // Allowed: Onion addresses and localhost
        assert!(manager.validate_outbound_dial("testservice.onion").is_ok());
        assert!(manager.validate_outbound_dial("127.0.0.1:9050").is_ok());

        // Blocked: Public IP / clearnet addresses
        assert!(manager.validate_outbound_dial("/ip4/82.64.12.34/udp/4001/quic-v1").is_err());
        assert!(manager.validate_outbound_dial("example.com:4001").is_err());
    }

    #[test]
    fn test_hybrid_and_direct_modes_permit_all() {
        let mut config = TorConfig::default();
        config.mode = TorMode::Hybrid;
        let manager = TorManager::new(config, [0u8; 32], std::env::temp_dir().join("nova_tor_test_hybrid"));

        assert!(manager.validate_outbound_dial("testservice.onion").is_ok());
        assert!(manager.validate_outbound_dial("/ip4/1.2.3.4/udp/4001").is_ok());
    }

    /// The exact bug this module's doc comment describes fixing (see also
    /// `nova_crypto::onion`'s SHA3-256 fix): the `.onion` address this device's onion service
    /// actually serves at must be bit-for-bit identical to what `derive_onion_v3_address`
    /// computes from the same identity key and already publishes in this device's
    /// `PreKeyBundle` — otherwise contacts dial an address nothing is listening at.
    #[test]
    fn test_hsid_keypair_matches_derive_onion_v3_address() {
        let seed = [0x42u8; 32];
        let keypair = hsid_keypair_from_identity_seed(&seed);
        let hsid_pubkey_bytes: [u8; 32] = *keypair.as_ref().public().as_bytes();

        // `HsIdKeypair`'s public half must be the exact same Ed25519 public key
        // `derive_onion_v3_address` encodes into the address — computed independently here via
        // ed25519-dalek directly, not by reusing tor_llcrypto's own derivation, so this actually
        // catches a mismatch instead of just re-checking the same code path against itself.
        let signing_key = ed25519_dalek::SigningKey::from_bytes(&seed);
        let expected_pub = signing_key.verifying_key().to_bytes();
        assert_eq!(hsid_pubkey_bytes, expected_pub);

        let expected_onion = nova_crypto::derive_onion_v3_address(&expected_pub);
        let parsed_back = nova_crypto::parse_onion_v3_address(&expected_onion).unwrap();
        assert_eq!(parsed_back, hsid_pubkey_bytes);
    }
}
