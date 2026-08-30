use nova_protocol::{
    DirectoryProfile, DirectorySearchResult, PeerEndpoint, SignedDirectoryEntry,
    SignedPresenceRegistration,
};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;
use tokio::sync::RwLock;

const DEFAULT_TTL_SECONDS: u64 = 60;
const DIRECTORY_TTL_SECONDS: u64 = 7 * 24 * 3600; // 7 days retention
/// A registration's signed timestamp must fall within this many seconds of "now" (either
/// direction) to be accepted — bounds how long a captured registration could be replayed to
/// resurrect a stale presence entry after the real peer has moved or gone offline.
const REGISTRATION_FRESHNESS_WINDOW_SECS: u64 = 120;
/// Hard cap on distinct peer_ids tracked at once, independent of the per-entry TTL, so a flood
/// of registrations under many fabricated identities cannot grow this table without bound
/// between housekeeping sweeps.
const MAX_TRACKED_PEERS: usize = 50_000;

#[derive(Error, Debug)]
pub enum RegistryError {
    #[error("registration failed cryptographic verification: {0}")]
    InvalidRegistration(#[from] nova_protocol::PresenceError),
    #[error("registration timestamp is stale or too far in the future")]
    StaleTimestamp,
    #[error("presence registry is at capacity")]
    AtCapacity,
}

struct TrackedEndpoint {
    endpoint: PeerEndpoint,
    last_seen_utc: u64,
}

struct TrackedDirectoryEntry {
    profile: DirectoryProfile,
    last_updated_utc: u64,
}

#[derive(Clone)]
pub struct PresenceRegistry {
    entries: Arc<RwLock<HashMap<String, TrackedEndpoint>>>,
    directory: Arc<RwLock<HashMap<String, TrackedDirectoryEntry>>>,
}

impl Default for PresenceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl PresenceRegistry {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            directory: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Registers (or refreshes) a peer's presence. The registration MUST be signed by the
    /// Ed25519 identity key matching its own `peer_id`.
    pub async fn register(
        &self,
        registration: SignedPresenceRegistration,
        observed_addr: SocketAddr,
    ) -> Result<(), RegistryError> {
        let now = now_secs();
        let ts = registration.timestamp_utc;
        let within_window = now.saturating_sub(ts) <= REGISTRATION_FRESHNESS_WINDOW_SECS
            && ts.saturating_sub(now) <= REGISTRATION_FRESHNESS_WINDOW_SECS;
        if !within_window {
            return Err(RegistryError::StaleTimestamp);
        }

        registration.verify()?;

        let mut endpoint = registration.endpoint;
        endpoint.public_ip = observed_addr.ip().to_string();

        let mut lock = self.entries.write().await;
        if !lock.contains_key(&endpoint.peer_id) && lock.len() >= MAX_TRACKED_PEERS {
            return Err(RegistryError::AtCapacity);
        }

        lock.insert(
            endpoint.peer_id.clone(),
            TrackedEndpoint {
                endpoint,
                last_seen_utc: now,
            },
        );
        Ok(())
    }

    /// Registers or updates a peer's public directory profile and PreKey bundle.
    /// Cryptographically verified against the peer's Ed25519 identity key.
    pub async fn register_directory(
        &self,
        entry: SignedDirectoryEntry,
    ) -> Result<(), RegistryError> {
        let now = now_secs();
        let ts = entry.timestamp_utc;
        let within_window = now.saturating_sub(ts) <= REGISTRATION_FRESHNESS_WINDOW_SECS
            && ts.saturating_sub(now) <= REGISTRATION_FRESHNESS_WINDOW_SECS;
        if !within_window {
            return Err(RegistryError::StaleTimestamp);
        }

        entry.verify()?;

        let mut lock = self.directory.write().await;
        if !lock.contains_key(&entry.profile.peer_id) && lock.len() >= MAX_TRACKED_PEERS {
            return Err(RegistryError::AtCapacity);
        }

        lock.insert(
            entry.profile.peer_id.clone(),
            TrackedDirectoryEntry {
                profile: entry.profile,
                last_updated_utc: now,
            },
        );
        Ok(())
    }

    /// Searches the public directory by peer_id (full or prefix), @username, or display name.
    pub async fn search_directory(&self, query: &str) -> Vec<DirectorySearchResult> {
        let now = now_secs();
        let q = query.trim().trim_start_matches('@').to_lowercase();
        if q.is_empty() {
            return Vec::new();
        }

        let dir_lock = self.directory.read().await;
        let presence_lock = self.entries.read().await;

        let mut results = Vec::new();

        for (peer_id, entry) in dir_lock.iter() {
            let pid_lower = peer_id.to_lowercase();
            let uname_lower = entry.profile.username.to_lowercase();
            let dname_lower = entry.profile.display_name.to_lowercase();

            let matches = pid_lower.starts_with(&q)
                || pid_lower == q
                || uname_lower == q
                || uname_lower.starts_with(&q)
                || uname_lower.contains(&q)
                || dname_lower.contains(&q);

            if matches {
                let is_online = presence_lock
                    .get(peer_id)
                    .map(|p| now.saturating_sub(p.last_seen_utc) <= DEFAULT_TTL_SECONDS)
                    .unwrap_or(false);

                results.push(DirectorySearchResult {
                    peer_id: entry.profile.peer_id.clone(),
                    username: entry.profile.username.clone(),
                    display_name: entry.profile.display_name.clone(),
                    avatar_data_url: entry.profile.avatar_data_url.clone(),
                    prekey_bundle_hex: entry.profile.prekey_bundle_hex.clone(),
                    is_online,
                    last_seen_utc: entry.last_updated_utc,
                });

                if results.len() >= 20 {
                    break;
                }
            }
        }

        // Sort: exact matches first, then online peers, then alphabetical
        results.sort_by(|a, b| {
            let a_exact = a.peer_id.to_lowercase() == q || a.username.to_lowercase() == q;
            let b_exact = b.peer_id.to_lowercase() == q || b.username.to_lowercase() == q;
            b_exact.cmp(&a_exact)
                .then_with(|| b.is_online.cmp(&a_online(a)))
                .then_with(|| a.display_name.to_lowercase().cmp(&b.display_name.to_lowercase()))
        });

        fn a_online(a: &DirectorySearchResult) -> bool {
            a.is_online
        }

        results
    }

    pub async fn get_peer(&self, peer_id: &str) -> Option<PeerEndpoint> {
        let now = now_secs();
        let lock = self.entries.read().await;
        lock.get(peer_id).and_then(|tracked| {
            if now.saturating_sub(tracked.last_seen_utc) <= DEFAULT_TTL_SECONDS {
                Some(tracked.endpoint.clone())
            } else {
                None
            }
        })
    }

    pub async fn cleanup_expired(&self) {
        let now = now_secs();
        {
            let mut lock = self.entries.write().await;
            lock.retain(|_, tracked| now.saturating_sub(tracked.last_seen_utc) <= DEFAULT_TTL_SECONDS);
        }
        {
            let mut lock = self.directory.write().await;
            lock.retain(|_, tracked| now.saturating_sub(tracked.last_updated_utc) <= DIRECTORY_TTL_SECONDS);
        }
    }

    pub async fn count(&self) -> usize {
        let lock = self.entries.read().await;
        lock.len()
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use nova_crypto::{DeviceIdentity, MnemonicPhrase};

    fn identity(name: &str) -> DeviceIdentity {
        let mnemonic = MnemonicPhrase::generate().unwrap();
        DeviceIdentity::from_mnemonic(&mnemonic, name).unwrap()
    }

    fn endpoint() -> PeerEndpoint {
        PeerEndpoint {
            peer_id: String::new(),
            public_ip: "192.0.2.1".into(),
            public_port: 9000,
            local_ip: Some("10.0.0.5".into()),
            local_port: Some(9000),
        }
    }

    fn loopback_addr() -> SocketAddr {
        "127.0.0.1:51820".parse().unwrap()
    }

    #[tokio::test]
    async fn test_presence_registry_lifecycle() {
        let registry = PresenceRegistry::new();
        let alice = identity("alice");
        let now = now_secs();

        let reg = SignedPresenceRegistration::sign(&alice, endpoint(), now);
        registry.register(reg, loopback_addr()).await.unwrap();
        assert_eq!(registry.count().await, 1);

        let retrieved = registry.get_peer(&alice.public_id_hex()).await.unwrap();
        // The client claimed IP "192.0.2.1", but the registry must trust only the observed UDP
        // source IP, never the client's self-report. The claimed port (9000) IS kept — see the
        // doc comment on `register` for why that half of the address is trusted.
        assert_eq!(retrieved.public_ip, "127.0.0.1");
        assert_eq!(retrieved.public_port, 9000);

        assert!(registry.get_peer("non_existent").await.is_none());
    }

    #[tokio::test]
    async fn test_unsigned_or_spoofed_registration_is_rejected() {
        let registry = PresenceRegistry::new();
        let alice = identity("alice");
        let bob = identity("bob");
        let now = now_secs();

        // Bob cannot register an entry claiming to be Alice.
        let mut reg = SignedPresenceRegistration::sign(&bob, endpoint(), now);
        reg.endpoint.peer_id = alice.public_id_hex();

        let result = registry.register(reg, loopback_addr()).await;
        assert!(matches!(result, Err(RegistryError::InvalidRegistration(_))));
        assert_eq!(registry.count().await, 0);
    }

    #[tokio::test]
    async fn test_stale_timestamp_is_rejected() {
        let registry = PresenceRegistry::new();
        let alice = identity("alice");
        let ancient = now_secs().saturating_sub(3600);

        let reg = SignedPresenceRegistration::sign(&alice, endpoint(), ancient);
        let result = registry.register(reg, loopback_addr()).await;
        assert!(matches!(result, Err(RegistryError::StaleTimestamp)));
    }

    #[tokio::test]
    async fn test_client_claimed_public_ip_is_never_trusted_but_claimed_port_is_kept() {
        // Even a perfectly valid signature cannot make the registry store an attacker-chosen
        // public IP: only the actually-observed UDP source IP is ever recorded. The claimed
        // port, however, is deliberately kept (it names the QUIC endpoint's own listening port,
        // not the signaling socket's) — see the doc comment on `register`.
        let registry = PresenceRegistry::new();
        let alice = identity("alice");
        let now = now_secs();

        let mut lying_endpoint = endpoint();
        lying_endpoint.public_ip = "203.0.113.99".into();
        lying_endpoint.public_port = 4433;
        let reg = SignedPresenceRegistration::sign(&alice, lying_endpoint, now);

        let real_addr: SocketAddr = "198.51.100.42:33221".parse().unwrap();
        registry.register(reg, real_addr).await.unwrap();

        let retrieved = registry.get_peer(&alice.public_id_hex()).await.unwrap();
        assert_eq!(retrieved.public_ip, "198.51.100.42");
        assert_eq!(retrieved.public_port, 4433);
    }

    #[tokio::test]
    async fn test_directory_registration_and_search() {
        let registry = PresenceRegistry::new();
        let bob = identity("bob");
        let now = now_secs();

        let profile = DirectoryProfile {
            peer_id: bob.public_id_hex(),
            username: "bob".into(),
            display_name: "Bob Martin".into(),
            avatar_data_url: None,
            prekey_bundle_hex: "deadbeef0102".into(),
        };

        let entry = SignedDirectoryEntry::sign(&bob, profile, now);
        registry.register_directory(entry).await.unwrap();

        // Search by username
        let res1 = registry.search_directory("bob").await;
        assert_eq!(res1.len(), 1);
        assert_eq!(res1[0].display_name, "Bob Martin");
        assert_eq!(res1[0].peer_id, bob.public_id_hex());

        // Search by peer_id prefix
        let prefix = &bob.public_id_hex()[..8];
        let res2 = registry.search_directory(prefix).await;
        assert_eq!(res2.len(), 1);
        assert_eq!(res2[0].username, "bob");

        // Search by display name
        let res3 = registry.search_directory("Martin").await;
        assert_eq!(res3.len(), 1);

        // Search non-existent
        let res4 = registry.search_directory("alice").await;
        assert_eq!(res4.len(), 0);
    }
}
