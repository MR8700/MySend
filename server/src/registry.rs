use nova_protocol::{PeerEndpoint, SignedPresenceRegistration};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;
use tokio::sync::RwLock;

const DEFAULT_TTL_SECONDS: u64 = 60;
/// A registration's signed timestamp must fall within this many seconds of "now" (either
/// direction) to be accepted — bounds how long a captured registration could be replayed to
/// resurrect a stale presence entry after the real peer has moved or gone offline.
const REGISTRATION_FRESHNESS_WINDOW_SECS: u64 = 30;
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

#[derive(Clone)]
pub struct PresenceRegistry {
    entries: Arc<RwLock<HashMap<String, TrackedEndpoint>>>,
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
        }
    }

    /// Registers (or refreshes) a peer's presence. The registration MUST be signed by the
    /// Ed25519 identity key matching its own `peer_id` — this is the fix for the previous,
    /// unauthenticated version of this registry, where any client could announce presence
    /// under an arbitrary peer_id and hijack another peer's discovery entry.
    ///
    /// `observed_addr` is the UDP source address the datagram actually arrived from (stamped by
    /// the OS/NAT, not the client). A device behind NAT cannot know its own post-NAT public *IP*
    /// by itself — this is the same reflexive-address trick STUN servers use — so the
    /// client-claimed `public_ip` is discarded and replaced with what was actually observed
    /// before storing the entry. This closes an obvious spoofing vector: without it, any
    /// registrant could claim to be reachable at an arbitrary IP, hijacking traffic meant for
    /// someone else.
    ///
    /// `public_port`, by contrast, is kept as the client-claimed value (the port its own QUIC
    /// endpoint is actually listening on) rather than the observed source port. The registration
    /// datagram is sent from a separate signaling socket, not from the QUIC endpoint itself, so
    /// its source port does not reflect where the QUIC endpoint is reachable — overriding it
    /// would silently break every direct connection attempt. Trusting the claimed port is safe:
    /// a peer can only misroute traffic *to itself* by lying about it (callers simply fail to
    /// connect and fall back to the relay), never hijack another peer's port. On NATs that
    /// preserve the local port externally (common on home routers and some carrier networks)
    /// the claimed port is also the correct one; on NATs that rewrite it unpredictably (e.g.
    /// symmetric NAT), direct connection attempts fail closed into the relay path rather than
    /// silently misdirecting traffic.
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

        // Verifies the registrant genuinely controls the private key behind `peer_id`, over the
        // fields as originally signed — the reflexive-address override below happens only after
        // this check passes, so it never invalidates the signature it is layered on top of.
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
        let mut lock = self.entries.write().await;
        lock.retain(|_, tracked| now.saturating_sub(tracked.last_seen_utc) <= DEFAULT_TTL_SECONDS);
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
}
