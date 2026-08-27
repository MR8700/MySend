//! DHT-based peer-to-peer node, built on rust-libp2p's Kademlia implementation rather than a
//! hand-rolled DHT or a single centralized rendezvous server. Kademlia is battle-tested in
//! production by IPFS, Ethereum, Polkadot, and Filecoin — a home-grown DHT is exactly the kind
//! of subtle, hard-to-audit mistake (Sybil/eclipse resistance, routing-table poisoning) a
//! sovereign encrypted messenger should not be reinventing from scratch.
//!
//! There is still no single server that owns anyone's data: presence is a self-authenticating,
//! signed record (see `nova_protocol::SignedDhtPeerRecord`) published into the DHT, replicated
//! across whichever peers happen to be closest to its key in the Kademlia keyspace — no one of
//! them needs to be trusted, because the record's own Ed25519 signature is what makes it
//! trustworthy, not who happens to be storing it. Bootstrapping into the DHT for the very first
//! time (or discovering peers on the same LAN) uses mDNS locally and, for wide-area use, would
//! use a small list of well-known bootstrap peers — the same bootstrapping reality every DHT
//! network (BitTorrent, IPFS, Tox) has, not a design shortcut specific to this app.
//!
//! Kademlia discovery + mDNS (LAN) + a direct QUIC connection (via libp2p's own QUIC transport,
//! which — unlike the previous hand-rolled transport — ties the QUIC-layer TLS certificate to
//! the node's real libp2p identity, so the transport handshake is cryptographically meaningful
//! rather than skipped) is the happy path. For peers that cannot establish a direct connection
//! (the common case on carrier-grade/symmetric mobile NAT — see the crate's original design
//! notes on Ouagadougou/Bobo-Dioulasso connectivity), every node also runs circuit-relay-v2 +
//! DCUtR: any reachable peer can act as a relay for another (opt-in, resource-limited by
//! `relay::Config`'s defaults), and once a relayed connection exists, DCUtR automatically
//! attempts to upgrade it to a direct one via coordinated hole-punching. The relay only ever
//! forwards opaque bytes — it is not a trusted party, exactly like the discovery DHT above it.

use crate::error::TransportError;
use crate::udp_fallback::UdpFallbackClient;
use crate::{P2PTransportMode, TransportSupervisor};
use futures_util::StreamExt;
use libp2p::core::multiaddr::Protocol;
use libp2p::kad::store::MemoryStore;
use libp2p::request_response::{OutboundRequestId, ProtocolSupport};
use libp2p::swarm::{NetworkBehaviour, SwarmEvent};
use libp2p::{
    dcutr, identify, identity, kad, mdns, relay, request_response, Multiaddr, PeerId, StreamProtocol, Swarm,
};
use nova_crypto::DeviceIdentity;
use nova_protocol::SignedDhtPeerRecord;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::{mpsc, oneshot, Mutex};
use tracing::{debug, info, warn};

const NOVA_MSG_PROTOCOL: &str = "/nova-chat/msg/1.0.0";
const DIRECT_CONNECT_TIMEOUT: Duration = Duration::from_secs(6);
const LOOKUP_TIMEOUT: Duration = Duration::from_secs(6);
const SEND_TIMEOUT: Duration = Duration::from_secs(6);
const PRESENCE_HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);
/// How often to poll the UDP fallback relay (if configured) for anything queued for this node —
/// see `crate::udp_fallback`. More frequent than the DHT presence heartbeat: this is this node's
/// only way to receive anything over that path at all (unlike the DHT/QUIC path, nothing pushes
/// to it), so responsiveness on this path depends entirely on poll frequency.
const UDP_FALLBACK_DRAIN_INTERVAL: Duration = Duration::from_secs(5);

#[derive(Clone, Debug, Serialize, Deserialize)]
struct NovaMessageRequest(Vec<u8>);

/// Application-level outcome of processing one incoming message, carried back to the sender as
/// the request/response reply — replaces what used to be an unconditional empty ack sent the
/// instant raw bytes were received, before `nova-engine` ever attempted to decode them.
///
/// This is what lets `nova-engine::pump_outbox_once` tell a genuinely-delivered-and-shown
/// message apart from one dropped because the recipient blocks the sender (never mark it
/// "Delivered") or one the recipient's engine simply failed to process at all (keep retrying
/// instead of losing it silently — see the 2026-08-22 audit's "silent packet drop" finding).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeliveryOutcome {
    /// Decoded, decrypted, and persisted (or was an exact duplicate of an already-persisted
    /// message, or a handshake this node deliberately yielded on due to a simultaneous-
    /// initiation collision) — the recipient's engine considers this packet fully handled.
    Processed,
    /// Decrypted fine (the Double Ratchet stayed in sync) but dropped because the sender is a
    /// contact the recipient has blocked.
    Blocked,
    /// The recipient's engine could not process this packet at all (malformed, oversized,
    /// unknown session, or any other decode/protocol failure).
    Rejected,
}

#[derive(NetworkBehaviour)]
struct NovaBehaviour {
    kademlia: kad::Behaviour<MemoryStore>,
    mdns: mdns::tokio::Behaviour,
    identify: identify::Behaviour,
    messaging: request_response::cbor::Behaviour<NovaMessageRequest, DeliveryOutcome>,
    /// Lets this node act as a relay for other peers who can't reach each other directly.
    relay: relay::Behaviour,
    /// Lets this node reserve a slot on (and dial through) another peer's relay.
    relay_client: relay::client::Behaviour,
    /// Attempts to upgrade an established relayed connection to a direct one via hole-punching.
    dcutr: dcutr::Behaviour,
}

/// A resolved peer: their libp2p identity plus the addresses their signed DHT record claims to
/// be reachable at.
type ResolvedPeer = (PeerId, Vec<Multiaddr>);
type LookupResponder = oneshot::Sender<Result<Option<ResolvedPeer>, TransportError>>;
type PendingLookups = HashMap<kad::QueryId, (String, LookupResponder)>;

enum Command {
    Announce(oneshot::Sender<Result<(), TransportError>>),
    Lookup {
        nova_peer_id: String,
        respond: LookupResponder,
    },
    Dial {
        peer_id: PeerId,
        addrs: Vec<Multiaddr>,
        respond: oneshot::Sender<Result<(), TransportError>>,
    },
    SendRequest {
        peer_id: PeerId,
        bytes: Vec<u8>,
        respond: oneshot::Sender<Result<DeliveryOutcome, TransportError>>,
    },
    /// Delivers the [`DeliveryOutcome`] `nova-engine` computed for a previously-received
    /// [`IncomingMessage`] back to whichever peer sent it, via the pending `ResponseChannel`
    /// this swarm task is still holding for it (see `pending_incoming_acks`).
    RespondIncoming {
        channel: request_response::ResponseChannel<DeliveryOutcome>,
        outcome: DeliveryOutcome,
    },
    /// Dials a bare multiaddr without knowing the remote peer's `PeerId` in advance — for manual
    /// "first contact" bootstrapping (e.g. a known bootstrap node's address) rather than a DHT
    /// lookup. Responds as soon as the dial is *submitted*, not once connected: the resulting
    /// connection (and the `identify` exchange that follows it) populates the Kademlia routing
    /// table asynchronously via the normal event handlers.
    DialAddr {
        addr: Multiaddr,
        respond: oneshot::Sender<Result<(), TransportError>>,
    },
    /// Reserves a slot on `relay_addr` (a relay-capable peer's address), making this node
    /// reachable at a `/p2p-circuit` address through it. Responds with the resulting circuit
    /// multiaddr once the reservation is confirmed by the relay — include that address in the
    /// next `announce_presence()` call so other peers can actually find and dial it.
    ReserveRelay {
        relay_addr: Multiaddr,
        respond: oneshot::Sender<Result<Multiaddr, TransportError>>,
    },
    /// Publishes a DHT record advertising exactly the given addresses, bypassing the normal
    /// "collect every known address" behavior of `Command::Announce`.
    AnnounceAddrs {
        addrs: Vec<Multiaddr>,
        respond: oneshot::Sender<Result<(), TransportError>>,
    },
}

pub struct P2PNode {
    own_peer_id: String,
    libp2p_peer_id: PeerId,
    listen_addrs: Vec<Multiaddr>,
    command_tx: mpsc::UnboundedSender<Command>,
    incoming_rx: Mutex<mpsc::UnboundedReceiver<IncomingMessage>>,
    /// The sending half feeding `incoming_rx` — kept as a field (in addition to being moved into
    /// `run_swarm_task` and cloned into the onion bridge at construction time) so
    /// [`Self::ensure_onion_service_started`] can bridge the onion listener's own channel into it
    /// later too, from [`Self::configure_tor`], not only at `start()` time.
    incoming_tx: mpsc::UnboundedSender<IncomingMessage>,
    pub supervisor: Arc<TransportSupervisor>,
    pub tor_manager: Arc<tokio::sync::RwLock<crate::tor::TorManager>>,
    /// Guards [`Self::ensure_onion_service_started`] against launching the onion service twice —
    /// once true (via `swap`), it stays true for the node's lifetime; the service itself is never
    /// torn down once started (see that method's doc comment for why).
    onion_service_started: AtomicBool,
    /// Secondary delivery path for when the DHT + relay-circuit path above cannot reach a peer
    /// at all — see `crate::udp_fallback`'s module docs. `None` unless `NOVA_UDP_FALLBACK_ADDR`
    /// is set: this path depends on a specific well-known `nova-server` instance being
    /// configured, unlike the DHT path which needs no central service at all.
    udp_fallback: Option<Arc<UdpFallbackClient>>,
}

/// Where an [`IncomingMessage`] arrived from — determines whether [`IncomingMessage::respond`]
/// has anyone live to report back to.
enum IncomingSource {
    /// A live QUIC request/response round-trip: the sender is actually waiting on the other end
    /// of `channel` for a [`DeliveryOutcome`].
    LibP2p {
        channel: request_response::ResponseChannel<DeliveryOutcome>,
        command_tx: mpsc::UnboundedSender<Command>,
    },
    /// Pulled from `nova-server`'s blind relay via [`crate::udp_fallback::UdpFallbackClient::drain_incoming`]
    /// — store-and-forward, not a live round-trip, so there is no sender waiting for an outcome.
    UdpFallback,
    /// A live connection over `crate::onion_channel`'s embedded onion service: the sender's
    /// `send_via_onion` call is holding the Tor stream open, waiting on this outcome.
    OnionDirect(crate::onion_channel::OnionIncoming),
}

/// One raw wire packet received from a peer, still awaiting an application-level verdict from
/// `nova-engine` on whether it was actually processed, blocked, or rejected — see
/// [`DeliveryOutcome`]. The caller MUST eventually call [`IncomingMessage::respond`] exactly
/// once; for a live libp2p-sourced message, until then the sender's `send_to_peer` call is left
/// waiting on the wire.
pub struct IncomingMessage {
    pub bytes: Vec<u8>,
    source: IncomingSource,
}

impl IncomingMessage {
    /// Reports how `nova-engine` handled this packet back to the peer that sent it, over the
    /// same request/response round-trip their `send_to_peer` call is awaiting — a no-op for a
    /// message that arrived via the UDP fallback relay, which has no live sender to report to.
    pub fn respond(self, outcome: DeliveryOutcome) {
        match self.source {
            IncomingSource::LibP2p { channel, command_tx } => {
                let _ = command_tx.send(Command::RespondIncoming { channel, outcome });
            }
            IncomingSource::UdpFallback => {}
            IncomingSource::OnionDirect(onion_incoming) => onion_incoming.respond(outcome),
        }
    }
}

impl P2PNode {
    /// Starts a DHT-backed node: brings up a libp2p `Swarm` over QUIC, joins the local mDNS
    /// discovery group, and spawns the background task that owns and polls the swarm for the
    /// lifetime of the node.
    pub async fn start(identity: DeviceIdentity, listen_addr: &str) -> Result<Arc<Self>, TransportError> {
        let keypair = identity::Keypair::ed25519_from_bytes(identity.signing_key_bytes.to_vec())
            .map_err(|e| TransportError::Setup(format!("invalid Ed25519 seed for libp2p identity: {e}")))?;
        let local_peer_id = keypair.public().to_peer_id();
        let own_peer_id = identity.public_id_hex();

        let mut swarm = libp2p::SwarmBuilder::with_existing_identity(keypair)
            .with_tokio()
            .with_quic()
            // Lets a bootstrap/rendezvous address use a `/dns4/<hostname>/...` multiaddr (e.g. a
            // free Dynamic DNS hostname) instead of a bare IP — the hostname is re-resolved on
            // every dial, so an operator whose ISP reassigns their public IP on router restart
            // doesn't need to re-share a new address with every other device each time.
            .with_dns()
            .map_err(|e| TransportError::Setup(e.to_string()))?
            .with_relay_client(libp2p::noise::Config::new, libp2p::yamux::Config::default)
            .map_err(|e| TransportError::Setup(e.to_string()))?
            .with_behaviour(|key, relay_client| {
                let peer_id = key.public().to_peer_id();
                let kademlia = kad::Behaviour::new(peer_id, MemoryStore::new(peer_id));
                let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), peer_id)?;
                let identify = identify::Behaviour::new(identify::Config::new(
                    "nova-chat/1.0.0".to_string(),
                    key.public(),
                ));
                let messaging = request_response::cbor::Behaviour::new(
                    [(StreamProtocol::new(NOVA_MSG_PROTOCOL), ProtocolSupport::Full)],
                    request_response::Config::default(),
                );
                let relay = relay::Behaviour::new(peer_id, relay::Config::default());
                let dcutr = dcutr::Behaviour::new(peer_id);
                Ok(NovaBehaviour {
                    kademlia,
                    mdns,
                    identify,
                    messaging,
                    relay,
                    relay_client,
                    dcutr,
                })
            })
            .map_err(|e| TransportError::Setup(e.to_string()))?
            // libp2p's default idle_connection_timeout is effectively zero: a connection with no
            // behaviour actively signaling keep-alive gets reaped within milliseconds of
            // establishment — often before a just-opened Kademlia query even gets to use it, which
            // otherwise shows up as spurious "KeepAliveTimeout" connection closures during
            // discovery. Give connections real breathing room instead.
            .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(30)))
            .build();

        swarm.behaviour_mut().kademlia.set_mode(Some(kad::Mode::Server));

        let listen_multiaddr: Multiaddr = listen_addr
            .parse()
            .map_err(|e| TransportError::Setup(format!("invalid listen multiaddr {listen_addr}: {e}")))?;
        swarm
            .listen_on(listen_multiaddr.clone())
            .map_err(|e| TransportError::Setup(e.to_string()))?;
        let mut expected_listeners: usize = 1;

        // IPv6 has no NAT — each device gets a real globally-routable address, so two peers
        // whose ISPs give them working IPv6 (increasingly common, including on mobile/4G) can
        // reach each other directly with no port forwarding, UPnP, or DDNS at all. Only attempted
        // for a real "listen on every interface" deployment address (`0.0.0.0`, used by
        // nova-desktop/nova-bootstrap) — not for the explicit loopback addresses the test suite
        // uses, where a second stack would be pointless. Best-effort: some hosts/networks have no
        // usable IPv6 at all, which is not an error, just IPv4-only like before.
        let requests_dual_stack = listen_multiaddr
            .iter()
            .any(|p| matches!(p, Protocol::Ip4(ip) if ip.is_unspecified()));
        if requests_dual_stack {
            let port = listen_multiaddr
                .iter()
                .find_map(|p| if let Protocol::Udp(port) = p { Some(port) } else { None })
                .unwrap_or(0);
            let ipv6_multiaddr: Multiaddr = format!("/ip6/::/udp/{port}/quic-v1")
                .parse()
                .expect("hardcoded IPv6 wildcard multiaddr is always valid");
            match swarm.listen_on(ipv6_multiaddr) {
                Ok(_) => expected_listeners += 1,
                Err(e) => info!("IPv6 dual-stack listen not available ({e}) — continuing IPv4-only"),
            }
        }

        // On a real "listen on every interface" deployment (0.0.0.0 / ::), libp2p's QUIC
        // transport emits one NewListenAddr per local network interface — not one for the
        // wildcard address itself, which isn't dialable by anyone. On a machine with a virtual
        // adapter (WSL's vEthernet, a VPN, Hyper-V, ...) alongside the real Wi-Fi/Ethernet NIC,
        // that means several candidate addresses arrive, in OS-dependent order, and some are
        // useless to advertise to a peer:
        //   - loopback (127.0.0.0/8, ::1) is never reachable by another device, ever.
        // The old code kept only the *first* NewListenAddr per stack (`expected_listeners`
        // capped collection at exactly 1 for IPv4) — on such a machine this could silently pick
        // the virtual adapter's address (or, for the IPv6 dual-stack leg, loopback itself, since
        // binding `::` also opens a loopback listener) as the ONLY address ever published in this
        // device's own invitation/DHT record, while the real, reachable LAN address was dropped
        // on the floor. A peer dialing that record then reaches nothing (or itself, for ::1) —
        // this was reproduced directly: a real invitation ticket generated on this kind of
        // machine advertised `/ip6/::1/udp/.../quic-v1`, a loopback address, as a rendezvous addr.
        // Fixed by collecting and advertising *every* non-loopback address that shows up within
        // the window instead of just the first: multi-address dialing is exactly what libp2p is
        // designed to do (try each candidate, use whichever succeeds), so including a
        // non-reachable virtual-adapter address alongside the real one is harmless — the peer's
        // dial to it simply fails while the real one succeeds — whereas dropping the real address
        // and keeping only a bogus one is fatal to connectivity.
        //
        // `requests_dual_stack` doubles as "this is a real wildcard deployment listen, not the
        // test suite's explicit loopback address" — tests bind directly to 127.0.0.1 to talk to
        // themselves on purpose, and must keep working exactly as before.
        let is_wildcard_deployment = requests_dual_stack;
        fn is_loopback(addr: &Multiaddr) -> bool {
            addr.iter().any(|p| match p {
                Protocol::Ip4(ip) => ip.is_loopback(),
                Protocol::Ip6(ip) => ip.is_loopback(),
                _ => false,
            })
        }

        // Block briefly with timeout until the transport resolves concrete listen address(es).
        let first_addr = match tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                match swarm.select_next_some().await {
                    SwarmEvent::NewListenAddr { address, .. } => {
                        if is_wildcard_deployment && is_loopback(&address) {
                            continue;
                        }
                        return Ok(address);
                    }
                    SwarmEvent::ListenerClosed { .. } | SwarmEvent::ListenerError { .. } => {
                        return Err(TransportError::Setup("listener failed to bind".into()));
                    }
                    _ => {}
                }
            }
        }).await {
            Ok(Ok(addr)) => addr,
            Ok(Err(e)) => return Err(e),
            Err(_) => return Err(TransportError::Setup("timed out waiting for initial listen address".into())),
        };

        let mut resolved_listen_addrs: Vec<Multiaddr> = vec![first_addr];
        let collect_deadline = tokio::time::Instant::now() + Duration::from_secs(3);
        loop {
            if !is_wildcard_deployment && resolved_listen_addrs.len() >= expected_listeners {
                break;
            }
            let remaining = collect_deadline.saturating_duration_since(tokio::time::Instant::now());
            if remaining.is_zero() {
                break;
            }
            match tokio::time::timeout(remaining, swarm.select_next_some()).await {
                Ok(SwarmEvent::NewListenAddr { address, .. }) => {
                    if is_wildcard_deployment && is_loopback(&address) {
                        continue;
                    }
                    if !resolved_listen_addrs.contains(&address) {
                        resolved_listen_addrs.push(address);
                    }
                }
                Ok(SwarmEvent::ListenerClosed { .. } | SwarmEvent::ListenerError { .. }) => {
                    expected_listeners = expected_listeners.saturating_sub(1);
                }
                Ok(_) => {}
                Err(_) => break,
            }
        }

        // A pinned (non-zero) port is the signal that this device is meant to be reachable from
        // outside its own network (see `full_listen_addrs`'s docs) — attempt to open it
        // automatically via UPnP so an operator with no router admin access (a consumer ISP
        // login only, not the NAT settings) still has a chance without anyone else's help. IPv6
        // needs no such mapping (no NAT to traverse), so this only ever targets the IPv4 port.
        if let Some(Protocol::Udp(port)) = listen_multiaddr.iter().find(|p| matches!(p, Protocol::Udp(_))) {
            let upnp_opt_in = std::env::var("NOVA_ENABLE_UPNP").map(|v| v == "1" || v.eq_ignore_ascii_case("true")).unwrap_or(false);
            if port != 0 && upnp_opt_in {
                spawn_upnp_port_mapping(port);
            }
        }

        let (command_tx, command_rx) = mpsc::unbounded_channel();
        let (incoming_tx, incoming_rx) = mpsc::unbounded_channel();
        let supervisor = Arc::new(TransportSupervisor::new());

        if resolved_listen_addrs.is_empty() {
            return Err(TransportError::Setup("failed to resolve any listen address".into()));
        }

        let own_onion_addr = nova_crypto::derive_onion_v3_address(&identity.verifying_key_bytes);
        let mut initial_tor_config = crate::tor::TorConfig::default();
        initial_tor_config.onion_address = Some(own_onion_addr);
        // Where the embedded Tor client keeps its consensus cache, guard state, and onion-service
        // keystore — a real, persistent directory is expected (set via `NOVA_TOR_STATE_DIR`, the
        // same pattern as `NOVA_BOOTSTRAP_ADDR`/`NOVA_LISTEN_ADDR`/`NOVA_ONION_LISTEN_ADDR`), since
        // rebuilding it from scratch means a fresh, slow Tor bootstrap on every launch. Falls back
        // to a fixed subdirectory of the OS temp dir (fine for tests and ad-hoc runs, but not
        // meant for a real install — `ui/src-tauri` sets this to a real app-data subdirectory).
        let tor_state_dir = std::env::var("NOVA_TOR_STATE_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| std::env::temp_dir().join("nova_tor_state"));
        let tor_manager = Arc::new(tokio::sync::RwLock::new(crate::tor::TorManager::new(
            initial_tor_config.clone(),
            identity.signing_key_bytes,
            tor_state_dir,
        )));

        // Opt-in secondary delivery path (see `crate::udp_fallback`) for networks where this
        // node cannot reach the DHT/relay-circuit path at all yet — set by whoever operates a
        // `nova-server` instance this deployment should fall back to, the same way
        // `NOVA_BOOTSTRAP_ADDR` names a DHT bootstrap peer. Must be a full `ws://`/`wss://` URL
        // (e.g. `wss://nova-discovery.onrender.com`) now that the fallback server speaks
        // WebSocket rather than raw UDP — see `udp_fallback.rs`'s module doc comment for why.
        let udp_fallback = std::env::var("NOVA_UDP_FALLBACK_ADDR")
            .ok()
            .and_then(|s| {
                let parsed = url::Url::parse(&s).ok()?;
                (parsed.scheme() == "ws" || parsed.scheme() == "wss").then_some(s)
            })
            .map(|url| Arc::new(UdpFallbackClient::new(url)));
        if udp_fallback.is_none() {
            if let Ok(raw) = std::env::var("NOVA_UDP_FALLBACK_ADDR") {
                warn!("NOVA_UDP_FALLBACK_ADDR={raw} is not a valid ws:// or wss:// URL (expected e.g. wss://nova-discovery.onrender.com) — fallback relay disabled");
            }
        }

        let node = Arc::new(Self {
            own_peer_id,
            libp2p_peer_id: local_peer_id,
            listen_addrs: resolved_listen_addrs,
            command_tx,
            incoming_rx: Mutex::new(incoming_rx),
            incoming_tx: incoming_tx.clone(),
            supervisor,
            tor_manager: tor_manager.clone(),
            onion_service_started: AtomicBool::new(false),
            udp_fallback: udp_fallback.clone(),
        });

        // Onion service (see `crate::onion_channel`) — the receiving half of the Tor fallback
        // path, hosted in-process under this device's real identity key (see `crate::tor`'s doc
        // comment) so its `.onion` address matches `own_onion_addr` above exactly. Only launched
        // when Tor is enabled in the device's saved settings: bootstrapping a real Tor circuit
        // and publishing a hidden-service descriptor has real bandwidth/CPU/battery cost, which a
        // user who never opted into Tor shouldn't pay on every launch. `configure_tor` calls
        // `ensure_onion_service_started` too, so enabling Tor later (without restarting) also
        // starts receiving over it, not just sending.
        if initial_tor_config.enabled {
            node.ensure_onion_service_started();
        }

        // `identity` moves into the background task by value rather than being cloned: it holds
        // zeroized private key material and deliberately does not implement `Clone`. Only the
        // task itself needs it, for signing DHT presence records — the public `P2PNode` handle
        // only ever needs the already-derived, non-secret `own_peer_id`.
        let command_tx_for_task = node.command_tx.clone();
        tokio::spawn(run_swarm_task(swarm, identity, local_peer_id, command_rx, command_tx_for_task, incoming_tx, udp_fallback, tor_manager));

        Ok(node)
    }

    pub fn peer_id(&self) -> &str {
        &self.own_peer_id
    }

    /// Returns the deterministic Tor Onion v3 address associated with this node's identity.
    pub async fn own_onion_address(&self) -> String {
        let mgr = self.tor_manager.read().await;
        mgr.config().onion_address.clone().unwrap_or_default()
    }

    /// Returns the live status of the Tor subsystem.
    pub async fn get_tor_status(&self) -> crate::tor::TorStatus {
        let mgr = self.tor_manager.read().await;
        let (connected, bootstrap_percent) = mgr.bootstrap_status().await;
        let cfg = mgr.config();
        crate::tor::TorStatus {
            enabled: cfg.enabled,
            connected,
            bootstrap_percent,
            onion_address: cfg.onion_address.clone().unwrap_or_default(),
            socks_proxy: cfg.socks_proxy.clone(),
            mode: cfg.mode,
            bridge_type: cfg.bridge_type.clone(),
        }
    }

    /// Reconfigures Tor operating mode, SOCKS proxy endpoint, and bridges. If this turns Tor on
    /// for the first time (it was off, or never started, when this node was constructed), also
    /// starts the onion-service receiving side right now — see
    /// [`Self::ensure_onion_service_started`] — so enabling Tor from Settings makes this device
    /// reachable via `.onion` immediately, with no app restart required.
    pub async fn configure_tor(&self, config: crate::tor::TorConfig) {
        let enabled = config.enabled;
        {
            let mut mgr = self.tor_manager.write().await;
            mgr.update_config(config);
        }
        if enabled {
            self.ensure_onion_service_started();
        }
    }

    /// Launches the onion-service receiving side (see
    /// [`crate::onion_channel::spawn_onion_service`]) exactly once for this node's lifetime — a
    /// second or later call is a cheap no-op, guarded by `onion_service_started`. Once started,
    /// the service is never torn down again even if Tor is later disabled in settings: turning it
    /// off only stops new *outbound* dials from using Tor (`connect_onion_stream` isn't called);
    /// leaving an already-launched onion service listening costs nothing extra worth the
    /// complexity of tearing down and re-launching it, and a peer with this device's onion
    /// address on file should still be able to reach it — being *listed* under `TorStrict`/
    /// `Hybrid` mode is a sending-side policy, not a promise that this device stops receiving.
    ///
    /// Called both from `start()` (if Tor was already enabled when the node was constructed) and
    /// from `configure_tor()` (if the user enables Tor afterward, without restarting) — the two
    /// only ways `enabled` can ever become true.
    fn ensure_onion_service_started(&self) {
        if self.onion_service_started.swap(true, Ordering::SeqCst) {
            return;
        }
        let (onion_incoming_tx, mut onion_incoming_rx) = mpsc::unbounded_channel::<crate::onion_channel::OnionIncoming>();
        crate::onion_channel::spawn_onion_service(self.tor_manager.clone(), onion_incoming_tx);
        // Bridges the onion listener's own channel into the unified `incoming_tx` stream
        // `nova-engine::attach_network` reads from, so it has exactly one receive loop to run
        // regardless of which of the three paths (libp2p, UDP fallback, onion) a packet arrived
        // on — see `IncomingSource::OnionDirect`.
        let incoming_tx_for_onion_bridge = self.incoming_tx.clone();
        tokio::spawn(async move {
            while let Some(onion_incoming) = onion_incoming_rx.recv().await {
                let bytes = onion_incoming.bytes.clone();
                let msg = IncomingMessage { bytes, source: IncomingSource::OnionDirect(onion_incoming) };
                if incoming_tx_for_onion_bridge.send(msg).is_err() {
                    break;
                }
            }
        });
    }

    /// This node's primary actual (OS-resolved) listen multiaddr, e.g. for out-of-band bootstrap
    /// / first-contact exchange (QR code, manual entry) when the peer isn't discoverable via
    /// mDNS or the DHT yet. "Primary" is whichever was resolved first — in practice the IPv4
    /// address when dual-stack is active, since it's always listened on first. Use
    /// `listen_addrs()` to see every address (e.g. the IPv6 one too), which matters because it
    /// may be reachable with zero NAT-traversal effort even when the IPv4 one needs a manual
    /// port forward or UPnP.
    pub fn listen_addr(&self) -> &Multiaddr {
        &self.listen_addrs[0]
    }

    /// Every address this node is actually listening on (IPv4, and IPv6 too when dual-stack was
    /// available on this host — see `P2PNode::start`).
    pub fn listen_addrs(&self) -> &[Multiaddr] {
        &self.listen_addrs
    }

    /// This node's primary listen address with its libp2p peer id appended (`.../p2p/<id>`) —
    /// the form needed to address this node unambiguously as a specific relay
    /// (`reserve_relay_slot` requires this, not the bare `listen_addr()`).
    pub fn full_listen_addr(&self) -> Multiaddr {
        self.listen_addr().clone().with(Protocol::P2p(self.libp2p_peer_id))
    }

    /// Every listen address this node has, each with its libp2p peer id appended — for
    /// advertising every way to reach this device (e.g. showing both an IPv4 and an IPv6 option
    /// to an operator sharing this device's address with another).
    pub fn full_listen_addrs(&self) -> Vec<Multiaddr> {
        self.listen_addrs
            .iter()
            .map(|a| a.clone().with(Protocol::P2p(self.libp2p_peer_id)))
            .collect()
    }

    /// This node's libp2p `PeerId` (distinct from `peer_id()`, which is the nova identity's
    /// hex-encoded Ed25519 public key) — useful for constructing a full multiaddr by hand, e.g.
    /// when overriding the address a bootstrap node advertises with an operator-supplied public
    /// IP that auto-detection cannot know.
    pub fn local_libp2p_peer_id(&self) -> PeerId {
        self.libp2p_peer_id
    }

    /// Dials a peer directly by address, without a prior DHT lookup — for manual bootstrapping
    /// (see `Command::DialAddr`). Once connected, the `identify` protocol exchange populates
    /// both sides' Kademlia routing tables automatically, after which normal DHT discovery works
    /// between them.
    pub async fn bootstrap_dial(&self, addr: Multiaddr) -> Result<(), TransportError> {
        {
            let mgr = self.tor_manager.read().await;
            mgr.validate_outbound_dial(&addr.to_string())?;
        }
        let (tx, rx) = oneshot::channel();
        self.command_tx
            .send(Command::DialAddr { addr, respond: tx })
            .map_err(|_| TransportError::SwarmTaskGone)?;
        rx.await.map_err(|_| TransportError::SwarmTaskGone)?
    }

    /// Publishes a DHT presence record advertising exactly `addrs` (no automatic collection of
    /// this node's other known addresses) — mainly useful to make a node reachable *only*
    /// through a specific path (e.g. a relay circuit) when testing or diagnosing that path in
    /// isolation.
    pub async fn announce_addresses_only(&self, addrs: Vec<Multiaddr>) -> Result<(), TransportError> {
        let (tx, rx) = oneshot::channel();
        self.command_tx
            .send(Command::AnnounceAddrs { addrs, respond: tx })
            .map_err(|_| TransportError::SwarmTaskGone)?;
        rx.await.map_err(|_| TransportError::SwarmTaskGone)?
    }

    /// Reserves a slot on a relay-capable peer, so this node becomes reachable at a
    /// `/p2p-circuit` address through it even if it has no usable direct address of its own
    /// (the common case behind carrier-grade or symmetric NAT). Include the returned address in
    /// the next `announce_presence()` call. Once a peer actually connects through it, DCUtR
    /// automatically attempts to upgrade the connection to a direct one.
    pub async fn reserve_relay_slot(&self, relay_addr: Multiaddr) -> Result<Multiaddr, TransportError> {
        let (tx, rx) = oneshot::channel();
        self.command_tx
            .send(Command::ReserveRelay { relay_addr, respond: tx })
            .map_err(|_| TransportError::SwarmTaskGone)?;
        tokio::time::timeout(DIRECT_CONNECT_TIMEOUT, rx)
            .await
            .map_err(|_| TransportError::Timeout)?
            .map_err(|_| TransportError::SwarmTaskGone)?
    }

    /// Signs and publishes this device's current libp2p reachability into the DHT under a key
    /// derived from its own nova peer_id. Give mDNS/bootstrap discovery a moment to populate the
    /// local routing table before calling this the first time, or `put_record` may have no
    /// peers to route the record through yet.
    pub async fn announce_presence(&self) -> Result<(), TransportError> {
        let (tx, rx) = oneshot::channel();
        self.command_tx
            .send(Command::Announce(tx))
            .map_err(|_| TransportError::SwarmTaskGone)?;
        rx.await.map_err(|_| TransportError::SwarmTaskGone)?
    }

    /// A single Kademlia query can come back empty even when the record genuinely exists,
    /// simply because the one or two peers it happened to ask were mid-reconnect (routing-table
    /// entries are best-effort, not guaranteed-live connections — ordinary for any DHT, not
    /// specific to this one). A few retries absorb that without the caller needing to know
    /// anything about it.
    async fn lookup(&self, nova_peer_id: &str) -> Result<Option<ResolvedPeer>, TransportError> {
        const ATTEMPTS: u32 = 3;
        let mut last_result = Ok(None);
        for attempt in 0..ATTEMPTS {
            last_result = self.lookup_once(nova_peer_id).await;
            if matches!(last_result, Ok(Some(_))) {
                return last_result;
            }
            if attempt + 1 < ATTEMPTS {
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }
        last_result
    }

    async fn lookup_once(&self, nova_peer_id: &str) -> Result<Option<ResolvedPeer>, TransportError> {
        let (tx, rx) = oneshot::channel();
        self.command_tx
            .send(Command::Lookup {
                nova_peer_id: nova_peer_id.to_string(),
                respond: tx,
            })
            .map_err(|_| TransportError::SwarmTaskGone)?;
        tokio::time::timeout(LOOKUP_TIMEOUT, rx)
            .await
            .map_err(|_| TransportError::Timeout)?
            .map_err(|_| TransportError::SwarmTaskGone)?
    }

    async fn dial(&self, peer_id: PeerId, addrs: Vec<Multiaddr>) -> Result<(), TransportError> {
        let filtered_addrs: Vec<Multiaddr> = {
            let mgr = self.tor_manager.read().await;
            addrs
                .into_iter()
                .filter(|addr| mgr.validate_outbound_dial(&addr.to_string()).is_ok())
                .collect()
        };

        if filtered_addrs.is_empty() {
            return Err(TransportError::Setup(
                "no dialable addresses permitted under current Tor privacy policy".into(),
            ));
        }

        let (tx, rx) = oneshot::channel();
        self.command_tx
            .send(Command::Dial { peer_id, addrs: filtered_addrs, respond: tx })
            .map_err(|_| TransportError::SwarmTaskGone)?;
        rx.await.map_err(|_| TransportError::SwarmTaskGone)?
    }

    /// Looks the peer up in the DHT and connects to their currently-published address(es) —
    /// which may be a direct address, a relay circuit address, or both (in which case libp2p
    /// tries them in order). Returns `Disconnected` (not an error) if the DHT has no record for
    /// them or every address fails; the caller's message simply stays queued for retry.
    pub async fn connect_to_peer(&self, nova_peer_id: &str) -> Result<P2PTransportMode, TransportError> {
        // A DHT lookup that times out (e.g. this node has no routing-table peers yet to query)
        // is not meaningfully different from one that completes and finds nothing: either way
        // the peer isn't reachable right now, not a fatal error the caller needs to handle
        // specially — nova-engine's outbox pump already treats "not delivered yet" as routine
        // and simply retries later.
        let found = match self.lookup(nova_peer_id).await {
            Ok(found) => found,
            Err(e) => {
                debug!("DHT lookup for {nova_peer_id} did not complete cleanly: {e}");
                None
            }
        };
        let Some((peer_id, addrs)) = found else {
            self.supervisor
                .update_peer_state(nova_peer_id.to_string(), P2PTransportMode::Disconnected, 0, String::new())
                .await;
            return Ok(P2PTransportMode::Disconnected);
        };

        let dial_started = tokio::time::Instant::now();
        match tokio::time::timeout(DIRECT_CONNECT_TIMEOUT, self.dial(peer_id, addrs.clone())).await {
            Ok(Ok(())) => {
                let latency_ms = dial_started.elapsed().as_millis().min(u128::from(u32::MAX)) as u32;
                let remote = addrs.first().map(|a| a.to_string()).unwrap_or_default();
                let mode = connection_mode_for(&addrs);
                info!("Connected to {nova_peer_id} at {remote} ({mode:?}, {latency_ms}ms to establish)");
                self.supervisor
                    .update_peer_state(nova_peer_id.to_string(), mode, latency_ms, remote)
                    .await;
                Ok(mode)
            }
            Ok(Err(e)) => {
                debug!("Direct connection to {nova_peer_id} failed: {e}");
                self.supervisor
                    .update_peer_state(nova_peer_id.to_string(), P2PTransportMode::Disconnected, 0, String::new())
                    .await;
                Ok(P2PTransportMode::Disconnected)
            }
            Err(_) => {
                debug!("Direct connection to {nova_peer_id} timed out");
                self.supervisor
                    .update_peer_state(nova_peer_id.to_string(), P2PTransportMode::Disconnected, 0, String::new())
                    .await;
                Ok(P2PTransportMode::Disconnected)
            }
        }
    }

    /// Sends already-encrypted wire bytes (a serialized `NovaPacket`) to `nova_peer_id` — the
    /// single-packet case of [`send_chunks_to_peer`](Self::send_chunks_to_peer); see there for
    /// the full behavior (this is `send_chunks_to_peer(nova_peer_id, vec![bytes])`).
    pub async fn send_to_peer(
        &self,
        nova_peer_id: &str,
        bytes: Vec<u8>,
    ) -> Result<(P2PTransportMode, DeliveryOutcome), TransportError> {
        self.send_chunks_to_peer(nova_peer_id, vec![bytes]).await
    }

    /// Sends one or more already-encrypted wire packets (each a serialized `NovaPacket`) to
    /// `nova_peer_id` over a SINGLE connection — looking the peer up and connecting once (direct
    /// or via relay circuit, see `connect_to_peer`), then delivering every chunk in order over
    /// that one connection. Used both for a plain single-packet text message (see
    /// [`send_to_peer`](Self::send_to_peer)) and for a multi-chunk media message (see
    /// `nova-engine::send_media`), where all chunks travel together so a partial failure retries
    /// the whole message rather than needing per-chunk delivery bookkeeping.
    ///
    /// The returned [`DeliveryOutcome`] is only meaningful when `mode != Disconnected`: it
    /// reflects what the *recipient's* `nova-engine` did with the LAST chunk sent (for a media
    /// message, that's the chunk that actually completes reassembly) — not just whether the
    /// bytes reached them. When `mode == Disconnected`, the accompanying outcome is a placeholder
    /// (`Rejected`) and must not be interpreted; the peer was never reached, or a chunk partway
    /// through the batch failed (in which case the whole batch is reported as undelivered, even
    /// if earlier chunks were individually accepted — the recipient's engine cannot reassemble a
    /// media message missing a chunk anyway).
    pub async fn send_chunks_to_peer(
        &self,
        nova_peer_id: &str,
        chunks: Vec<Vec<u8>>,
    ) -> Result<(P2PTransportMode, DeliveryOutcome), TransportError> {
        let found = match self.lookup(nova_peer_id).await {
            Ok(found) => found,
            Err(e) => {
                debug!("DHT lookup for {nova_peer_id} did not complete cleanly: {e}");
                None
            }
        };
        let Some((peer_id, addrs)) = found else {
            self.spawn_udp_fallback_send_chunks(nova_peer_id, chunks);
            return Ok((P2PTransportMode::Disconnected, DeliveryOutcome::Rejected));
        };
        let mode = connection_mode_for(&addrs);

        if tokio::time::timeout(DIRECT_CONNECT_TIMEOUT, self.dial(peer_id, addrs))
            .await
            .is_err()
        {
            self.spawn_udp_fallback_send_chunks(nova_peer_id, chunks);
            return Ok((P2PTransportMode::Disconnected, DeliveryOutcome::Rejected));
        }

        let request_started = tokio::time::Instant::now();
        let mut last_outcome = DeliveryOutcome::Rejected;
        for chunk in chunks {
            let (tx, rx) = oneshot::channel();
            self.command_tx
                .send(Command::SendRequest {
                    peer_id,
                    bytes: chunk,
                    respond: tx,
                })
                .map_err(|_| TransportError::SwarmTaskGone)?;

            match tokio::time::timeout(SEND_TIMEOUT, rx).await {
                Ok(Ok(Ok(outcome))) => last_outcome = outcome,
                _ => return Ok((P2PTransportMode::Disconnected, DeliveryOutcome::Rejected)),
            }
        }

        // The request/response round trip (send → peer's ack) is a more meaningful "latency" for
        // messaging purposes than connection-establishment time alone: a connection can be up
        // while the peer is slow or backlogged. For a multi-chunk send this is the total time for
        // every chunk, which is the honest end-to-end figure for how long this message took.
        let latency_ms = request_started.elapsed().as_millis().min(u128::from(u32::MAX)) as u32;
        self.supervisor
            .update_peer_state(nova_peer_id.to_string(), mode, latency_ms, String::new())
            .await;
        Ok((mode, last_outcome))
    }

    /// Sends `bytes` to a peer via their `.onion` address through a real, in-process Tor circuit
    /// (see `crate::onion_channel`), bypassing the DHT/QUIC path entirely. The caller (`nova-
    /// engine`, which knows a contact's onion address from their `PreKeyBundle`) decides when to
    /// use this — as a fallback once `send_to_peer` reports `Disconnected`, or exclusively in
    /// `TorStrict` mode. `port` should be the recipient's onion-service port, conventionally
    /// `crate::onion_channel::DEFAULT_ONION_CHANNEL_PORT`.
    pub async fn send_via_onion(
        &self,
        onion_address: &str,
        port: u16,
        bytes: Vec<u8>,
    ) -> Result<DeliveryOutcome, TransportError> {
        let mgr = self.tor_manager.read().await;
        crate::onion_channel::send_via_onion(&mgr, onion_address, port, bytes).await
    }

    /// Waits for the next raw packet received from any peer over a direct connection. Returns
    /// `None` only if the node's background task has stopped. The caller MUST call
    /// [`IncomingMessage::respond`] on the result exactly once, or the sender's `send_to_peer`
    /// call will hang until its own timeout.
    pub async fn recv_next(&self) -> Option<IncomingMessage> {
        self.incoming_rx.lock().await.recv().await
    }

    /// Best-effort, fire-and-forget hand-off of each of `chunks` to the UDP fallback relay (see
    /// `crate::udp_fallback`) for `nova_peer_id` — called from `send_chunks_to_peer`'s failure
    /// paths so a peer unreachable through the DHT still gets a second chance. Each chunk is
    /// relayed independently (out-of-order delivery/reassembly on the receiving end is already
    /// handled at the Double Ratchet level — see `send_chunks_to_peer`'s docs); relaying one
    /// chunk failing does not stop the others from being attempted. Spawned rather than awaited:
    /// this must never add latency to a call that has already decided the primary path failed,
    /// and a slow/unreachable fallback server must not block it either. A no-op if no fallback
    /// server is configured.
    fn spawn_udp_fallback_send_chunks(&self, nova_peer_id: &str, chunks: Vec<Vec<u8>>) {
        let Some(fallback) = self.udp_fallback.clone() else { return };
        let nova_peer_id = nova_peer_id.to_string();
        tokio::spawn(async move {
            for chunk in chunks {
                if let Err(e) = fallback.relay_forward(&nova_peer_id, chunk).await {
                    debug!("UDP fallback relay_forward to {nova_peer_id} failed: {e}");
                }
            }
        });
    }
}

/// Best-effort automatic port forwarding via UPnP IGD — lets a device act as a reachable
/// rendezvous/relay point without needing admin access to its router's configuration UI, which
/// many residential ISP customers simply don't have (a consumer login only, not the NAT/port
/// forwarding settings). Runs in the background and never blocks node startup or fails it: UPnP
/// is commonly unavailable (disabled by the router, or no IGD reachable at all — e.g. CGNAT or a
/// mobile network), in which case a manual forward remains the fallback for whoever does have
/// router admin access.
fn spawn_upnp_port_mapping(port: u16) {
    tokio::spawn(async move {
        let gateway = match igd_next::aio::tokio::search_gateway(igd_next::SearchOptions::default()).await {
            Ok(gw) => gw,
            Err(e) => {
                info!(
                    "UPnP: no gateway found ({e}) — if this device needs to be reachable from \
                     outside its network, port {port}/udp needs a manual forward instead"
                );
                return;
            }
        };

        // Connecting a UDP socket doesn't send anything — it just asks the OS to pick the local
        // interface/IP it would use to route toward `gateway.addr`, which is exactly the LAN IP
        // the router needs to forward this port to.
        let local_ip = match std::net::UdpSocket::bind("0.0.0.0:0")
            .and_then(|s| s.connect(gateway.addr).map(|_| s))
            .and_then(|s| s.local_addr())
        {
            Ok(addr) => addr.ip(),
            Err(e) => {
                warn!("UPnP: found a gateway but could not determine this device's own LAN IP: {e}");
                return;
            }
        };
        let local_addr = std::net::SocketAddr::new(local_ip, port);

        match gateway
            .add_port(igd_next::PortMappingProtocol::UDP, port, local_addr, 0, "nova-chat")
            .await
        {
            Ok(()) => info!(
                "UPnP: mapped external UDP port {port} -> {local_addr} automatically — no router \
                 admin access was needed"
            ),
            Err(e) => info!(
                "UPnP: router rejected the mapping request ({e}) — port {port}/udp needs a manual \
                 forward instead if this device needs to be reachable from outside its network"
            ),
        }
    });
}

async fn run_swarm_task(
    mut swarm: Swarm<NovaBehaviour>,
    identity: DeviceIdentity,
    local_peer_id: PeerId,
    mut command_rx: mpsc::UnboundedReceiver<Command>,
    command_tx: mpsc::UnboundedSender<Command>,
    incoming_tx: mpsc::UnboundedSender<IncomingMessage>,
    udp_fallback: Option<Arc<UdpFallbackClient>>,
    tor_manager: Arc<tokio::sync::RwLock<crate::tor::TorManager>>,
) {
    let mut pending_lookups: PendingLookups = HashMap::new();
    let mut pending_dials: HashMap<PeerId, Vec<oneshot::Sender<Result<(), TransportError>>>> = HashMap::new();
    let mut pending_requests: HashMap<OutboundRequestId, oneshot::Sender<Result<DeliveryOutcome, TransportError>>> = HashMap::new();
    let mut pending_relay_reservations: VecDeque<oneshot::Sender<Result<Multiaddr, TransportError>>> = VecDeque::new();
    let mut heartbeat = tokio::time::interval(PRESENCE_HEARTBEAT_INTERVAL);
    // Ticks unconditionally even when `udp_fallback` is `None` (the branch body below just
    // no-ops in that case) — simpler than threading an `Option<Interval>` through `select!`, at
    // the cost of one negligible no-op wakeup every UDP_FALLBACK_DRAIN_INTERVAL.
    let mut udp_fallback_drain = tokio::time::interval(UDP_FALLBACK_DRAIN_INTERVAL);

    loop {
        tokio::select! {
            _ = heartbeat.tick() => {
                if let Err(e) = do_announce(&mut swarm, &identity, local_peer_id) {
                    warn!("Presence heartbeat failed: {e}");
                }
            }
            _ = udp_fallback_drain.tick() => {
                // Signing is fast/synchronous (no I/O) so it happens inline here; the actual
                // network round-trip is spawned off so a slow/unreachable fallback server can
                // never stall this event loop (which also drives the primary DHT/QUIC path).
                if let Some(fallback) = udp_fallback.clone() {
                    let drain_request = nova_protocol::SignedDrainRequest::sign(&identity, now_secs());
                    let incoming_tx = incoming_tx.clone();
                    tokio::spawn(async move {
                        match fallback.drain_incoming_signed(drain_request).await {
                            Ok(items) => {
                                for bytes in items {
                                    let _ = incoming_tx.send(IncomingMessage { bytes, source: IncomingSource::UdpFallback });
                                }
                            }
                            Err(e) => debug!("UDP fallback drain failed: {e}"),
                        }
                    });
                }
            }
            maybe_cmd = command_rx.recv() => {
                let Some(cmd) = maybe_cmd else { break };
                match cmd {
                    Command::Announce(respond) => {
                        let _ = respond.send(do_announce(&mut swarm, &identity, local_peer_id));
                    }
                    Command::Lookup { nova_peer_id, respond } => {
                        let key_bytes = match hex::decode(&nova_peer_id) {
                            Ok(b) => b,
                            Err(_) => {
                                let _ = respond.send(Err(TransportError::Dht("nova_peer_id is not valid hex".into())));
                                continue;
                            }
                        };
                        let query_id = swarm.behaviour_mut().kademlia.get_record(kad::RecordKey::new(&key_bytes));
                        pending_lookups.insert(query_id, (nova_peer_id, respond));
                    }
                    Command::Dial { peer_id, addrs, respond } => {
                        // Hard gate, not just the advisory pre-filter `P2PNode::dial` already
                        // applies before ever sending this command: this is the actual point
                        // `swarm.dial` gets called, so it is the one place a TorStrict violation
                        // cannot slip through regardless of which caller constructed this
                        // command. See the 2026-08-22 audit's "TorStrict mode is not actually
                        // enforced" finding — the pre-filter alone was exactly that weakness.
                        let addrs: Vec<Multiaddr> = {
                            let mgr = tor_manager.read().await;
                            addrs.into_iter().filter(|a| mgr.validate_outbound_dial(&a.to_string()).is_ok()).collect()
                        };
                        if addrs.is_empty() {
                            let _ = respond.send(Err(TransportError::Setup(
                                "no dialable addresses permitted under the current Tor privacy policy".into(),
                            )));
                            continue;
                        }
                        // `PeerCondition::Always` is load-bearing: Kademlia's own routing-table
                        // maintenance can already have an outbound connection attempt in flight
                        // to this peer_id (e.g. triggered the moment mDNS discovered them and
                        // fed their address into `kademlia.add_address`), using addresses we
                        // never chose. With the default condition, our dial for the address we
                        // actually resolved from the peer's signed DHT record can get silently
                        // folded into — or skipped in favor of — that unrelated attempt, so our
                        // caller ends up waiting on somebody else's doomed connection instead of
                        // the one we asked for. Forcing a fresh attempt with exactly our
                        // addresses avoids that.
                        let opts = libp2p::swarm::dial_opts::DialOpts::peer_id(peer_id)
                            .condition(libp2p::swarm::dial_opts::PeerCondition::Always)
                            .addresses(addrs)
                            .build();
                        match swarm.dial(opts) {
                            Ok(()) => {
                                if swarm.is_connected(&peer_id) {
                                    let _ = respond.send(Ok(()));
                                } else {
                                    pending_dials.entry(peer_id).or_default().push(respond);
                                }
                            }
                            Err(e) => {
                                let _ = respond.send(Err(TransportError::Dial(e.to_string())));
                            }
                        }
                    }
                    Command::SendRequest { peer_id, bytes, respond } => {
                        let request_id = swarm
                            .behaviour_mut()
                            .messaging
                            .send_request(&peer_id, NovaMessageRequest(bytes));
                        pending_requests.insert(request_id, respond);
                    }
                    Command::DialAddr { addr, respond } => {
                        let result = swarm.dial(addr).map_err(|e| TransportError::Dial(e.to_string()));
                        let _ = respond.send(result);
                    }
                    Command::ReserveRelay { relay_addr, respond } => {
                        let circuit_addr = relay_addr.with(Protocol::P2pCircuit);
                        match swarm.listen_on(circuit_addr) {
                            Ok(_listener_id) => pending_relay_reservations.push_back(respond),
                            Err(e) => {
                                let _ = respond.send(Err(TransportError::Dial(e.to_string())));
                            }
                        }
                    }
                    Command::AnnounceAddrs { addrs, respond } => {
                        let addrs = addrs.into_iter().map(|a| a.to_string()).collect();
                        let _ = respond.send(publish_record(&mut swarm, &identity, local_peer_id, addrs));
                    }
                    Command::RespondIncoming { channel, outcome } => {
                        let _ = swarm.behaviour_mut().messaging.send_response(channel, outcome);
                    }
                }
            }
            event = swarm.select_next_some() => {
                handle_event(
                    event,
                    &mut swarm,
                    &incoming_tx,
                    &command_tx,
                    &mut pending_lookups,
                    &mut pending_dials,
                    &mut pending_requests,
                    &mut pending_relay_reservations,
                );
            }
        }
    }
}

fn do_announce(swarm: &mut Swarm<NovaBehaviour>, identity: &DeviceIdentity, local_peer_id: PeerId) -> Result<(), TransportError> {
    let mut addrs: Vec<String> = swarm
        .external_addresses()
        .chain(swarm.listeners())
        .map(|a| a.to_string())
        .collect();
    addrs.sort();
    addrs.dedup();
    publish_record(swarm, identity, local_peer_id, addrs)
}

fn publish_record(
    swarm: &mut Swarm<NovaBehaviour>,
    identity: &DeviceIdentity,
    local_peer_id: PeerId,
    addrs: Vec<String>,
) -> Result<(), TransportError> {
    if addrs.is_empty() {
        return Err(TransportError::Setup("no known listen/external address to announce yet".into()));
    }

    let record = SignedDhtPeerRecord::sign(identity, local_peer_id.to_bytes(), addrs, now_secs());
    let key_bytes = hex::decode(&record.nova_peer_id).map_err(|_| TransportError::Setup("bad peer id hex".into()))?;
    let value = record.to_bytes()?;
    let kad_record = kad::Record {
        key: kad::RecordKey::new(&key_bytes),
        value,
        publisher: None,
        expires: None,
    };
    swarm
        .behaviour_mut()
        .kademlia
        .put_record(kad_record, kad::Quorum::One)
        .map_err(|e| TransportError::Dht(e.to_string()))?;
    Ok(())
}

fn handle_event(
    event: SwarmEvent<NovaBehaviourEvent>,
    swarm: &mut Swarm<NovaBehaviour>,
    incoming_tx: &mpsc::UnboundedSender<IncomingMessage>,
    command_tx: &mpsc::UnboundedSender<Command>,
    pending_lookups: &mut PendingLookups,
    pending_dials: &mut HashMap<PeerId, Vec<oneshot::Sender<Result<(), TransportError>>>>,
    pending_requests: &mut HashMap<OutboundRequestId, oneshot::Sender<Result<DeliveryOutcome, TransportError>>>,
    pending_relay_reservations: &mut VecDeque<oneshot::Sender<Result<Multiaddr, TransportError>>>,
) {
    match event {
        SwarmEvent::ConnectionEstablished { peer_id, .. } => {
            if let Some(waiters) = pending_dials.remove(&peer_id) {
                for w in waiters {
                    let _ = w.send(Ok(()));
                }
            }
        }
        SwarmEvent::OutgoingConnectionError { peer_id: Some(peer_id), error, .. } => {
            if let Some(waiters) = pending_dials.remove(&peer_id) {
                for w in waiters {
                    let _ = w.send(Err(TransportError::Dial(error.to_string())));
                }
            }
        }
        SwarmEvent::NewListenAddr { address, .. } if address.iter().any(|p| matches!(p, Protocol::P2pCircuit)) => {
            info!("Relay reservation confirmed at {address}");
            if let Some(respond) = pending_relay_reservations.pop_front() {
                let _ = respond.send(Ok(address));
            }
        }
        SwarmEvent::Behaviour(NovaBehaviourEvent::RelayClient(relay::client::Event::ReservationReqAccepted {
            relay_peer_id,
            ..
        })) => {
            info!("Relay reservation accepted by {relay_peer_id}");
        }
        SwarmEvent::Behaviour(NovaBehaviourEvent::Dcutr(dcutr::Event { remote_peer_id, result })) => match result {
            Ok(_) => info!("DCUtR upgraded {remote_peer_id} to a direct connection"),
            Err(e) => debug!("DCUtR could not upgrade the connection to {remote_peer_id} to direct: {e}"),
        },
        SwarmEvent::Behaviour(NovaBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
            for (peer_id, addr) in list {
                debug!("mDNS discovered {peer_id} at {addr}");
                swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
            }
        }
        SwarmEvent::Behaviour(NovaBehaviourEvent::Identify(identify::Event::Received { peer_id, info, .. })) => {
            swarm.add_external_address(info.observed_addr.clone());
            for addr in info.listen_addrs {
                swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
            }
        }
        SwarmEvent::Behaviour(NovaBehaviourEvent::Kademlia(kad::Event::OutboundQueryProgressed {
            id,
            result: kad::QueryResult::GetRecord(get_result),
            step,
            ..
        })) => {
            if let Ok(kad::GetRecordOk::FoundRecord(peer_record)) = get_result {
                if let Some((expected_nova_peer_id, respond)) = pending_lookups.remove(&id) {
                    match resolve_record(&peer_record.record.value, &expected_nova_peer_id) {
                        Ok(resolved) => {
                            let _ = respond.send(Ok(Some(resolved)));
                        }
                        Err(e) => {
                            let _ = respond.send(Err(e));
                        }
                    }
                }
            } else if step.last {
                if let Some((_id, respond)) = pending_lookups.remove(&id) {
                    let _ = respond.send(Ok(None));
                }
            }
        }
        SwarmEvent::Behaviour(NovaBehaviourEvent::Messaging(request_response::Event::Message { message, .. })) => match message {
            request_response::Message::Request { request, channel, .. } => {
                // The response is deliberately NOT sent here: it now carries the real
                // DeliveryOutcome nova-engine computes from actually decoding this packet (see
                // `IncomingMessage::respond`), not an unconditional "received the bytes" ack.
                // Sending an ack before that verdict exists is exactly what let a packet the
                // recipient could never even decrypt still show as "Delivered" to the sender.
                let incoming = IncomingMessage {
                    bytes: request.0,
                    source: IncomingSource::LibP2p { channel, command_tx: command_tx.clone() },
                };
                if let Err(unsent) = incoming_tx.send(incoming) {
                    // nova-engine's receive loop is gone (e.g. network never attached, or the
                    // engine shut down) — nothing will ever call `.respond()` on this packet, so
                    // answer here to avoid leaving the sender hanging until SEND_TIMEOUT for a
                    // packet nobody is actually listening for.
                    warn!("No receiver attached for incoming packets — rejecting immediately");
                    if let IncomingSource::LibP2p { channel, .. } = unsent.0.source {
                        let _ = swarm.behaviour_mut().messaging.send_response(channel, DeliveryOutcome::Rejected);
                    }
                }
            }
            request_response::Message::Response { request_id, response } => {
                if let Some(respond) = pending_requests.remove(&request_id) {
                    let _ = respond.send(Ok(response));
                }
            }
        },
        SwarmEvent::Behaviour(NovaBehaviourEvent::Messaging(request_response::Event::OutboundFailure {
            request_id,
            error,
            ..
        })) => {
            if let Some(respond) = pending_requests.remove(&request_id) {
                let _ = respond.send(Err(TransportError::Send(error.to_string())));
            }
        }
        _ => {}
    }
}

/// A record whose *only* usable addresses are relay circuits means the connection, if it
/// succeeds, went through a relay — report that honestly rather than always claiming
/// `DirectQuic`. If a direct address is also present, libp2p tries addresses in order and
/// direct ones are placed first by `resolve_record`, so `DirectQuic` is the accurate label
/// whenever at least one non-circuit address was offered (DCUtR may still be upgrading a
/// circuit-only connection to direct behind the scenes; that transition isn't reflected here).
fn connection_mode_for(addrs: &[Multiaddr]) -> P2PTransportMode {
    let has_direct = addrs
        .iter()
        .any(|a| !a.iter().any(|p| matches!(p, Protocol::P2pCircuit)));
    if has_direct {
        P2PTransportMode::DirectQuic
    } else {
        P2PTransportMode::RelayedOpaque
    }
}

fn resolve_record(raw_value: &[u8], expected_nova_peer_id: &str) -> Result<ResolvedPeer, TransportError> {
    let record = SignedDhtPeerRecord::from_bytes(raw_value)?;
    // Validate signature and freshness (5 minutes tolerance) to prevent stale record replay attacks
    let now = now_secs();
    record.verify_fresh(now, 300)?;
    if record.nova_peer_id != expected_nova_peer_id {
        return Err(TransportError::Dht("DHT record key/content mismatch".into()));
    }
    let peer_id = PeerId::from_bytes(&record.libp2p_peer_id_bytes)
        .map_err(|e| TransportError::Dht(format!("invalid libp2p peer id in record: {e}")))?;
    let addrs: Vec<Multiaddr> = record
        .addresses
        .iter()
        .filter_map(|a| a.parse().ok())
        .collect();
    Ok((peer_id, addrs))
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

    /// The core claim of this module: two independently-started nodes with no shared process
    /// state discover each other purely through the distributed DHT (not a hardcoded address),
    /// verify each other's signed presence record, and exchange real bytes over a direct QUIC
    /// connection whose TLS layer is bound to their actual libp2p identity.
    #[tokio::test]
    async fn test_two_independent_nodes_discover_via_dht_and_exchange_bytes() {
        let alice_id = identity("alice");
        let bob_id = identity("bob");
        let bob_peer_id = bob_id.public_id_hex();

        let alice = P2PNode::start(alice_id, "/ip4/127.0.0.1/udp/0/quic-v1").await.unwrap();
        let bob = P2PNode::start(bob_id, "/ip4/127.0.0.1/udp/0/quic-v1").await.unwrap();

        // mDNS multicast is not reliably available in every sandboxed/CI network environment, so
        // this test does not depend on it: Alice bootstraps her Kademlia routing table with one
        // explicit, known address (Bob's) — exactly what a small list of well-known bootstrap
        // peers provides at wide-area scale for real first contact. Once connected, the
        // `identify` exchange populates both routing tables, after which all further discovery
        // (the actual thing under test) goes through the real DHT, not this bootstrap dial.
        alice.bootstrap_dial(bob.listen_addr().clone()).await.unwrap();
        tokio::time::sleep(Duration::from_millis(500)).await;

        bob.announce_presence().await.unwrap();
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Bob's receive side: exactly what nova-engine::attach_network's background loop does in
        // production — pull the next incoming packet and report how it was "processed" over the
        // same request/response round-trip Alice's send_to_peer below is waiting on.
        let bob_recv = bob.clone();
        let recv_task = tokio::spawn(async move {
            // A generous window: unlike the original sequential version of this test (send fully
            // completes, then drain an already-buffered channel), this task now runs concurrently
            // with send_to_peer below, so it must also cover that call's own DHT-lookup retries
            // (up to 3 x 500ms) and dial timeout, not just the final channel recv. Bumped from 12s
            // to 20s: this test is one of the last to run in a full `cargo test --workspace` pass,
            // where residual CPU/IO contention from every prior crate's tests made the original
            // 12s budget an intermittent flake (observed panicking with `Elapsed(())` even though
            // the same test passes reliably in isolation) — this is machine-load headroom, not a
            // change in what the test actually verifies.
            let incoming = tokio::time::timeout(Duration::from_secs(20), bob_recv.recv_next())
                .await
                .expect("bob should receive the message before the timeout")
                .expect("incoming channel should not be closed");
            let bytes = incoming.bytes.clone();
            incoming.respond(DeliveryOutcome::Processed);
            bytes
        });

        let (mode, outcome) = alice
            .send_to_peer(&bob_peer_id, b"hello bob via dht".to_vec())
            .await
            .unwrap();
        assert_eq!(mode, P2PTransportMode::DirectQuic);
        assert_eq!(outcome, DeliveryOutcome::Processed);

        let received = recv_task.await.expect("bob's receive task must not panic");
        assert_eq!(received, b"hello bob via dht");
    }

    /// Regression test for the 2026-08-22 audit's "silent packet drop" / false-"Delivered"
    /// findings: whatever `DeliveryOutcome` the receiving side reports via `IncomingMessage::
    /// respond` — not just "the bytes arrived" — must be exactly what `send_to_peer` returns to
    /// the sender. `nova-engine` relies on this to never mark a message "Delivered" when the
    /// recipient dropped it as blocked, and to keep retrying one it failed to process at all.
    #[tokio::test]
    async fn test_delivery_outcome_is_faithfully_reported_to_the_sender() {
        let alice_id = identity("alice");
        let bob_id = identity("bob");
        let bob_peer_id = bob_id.public_id_hex();

        let alice = P2PNode::start(alice_id, "/ip4/127.0.0.1/udp/0/quic-v1").await.unwrap();
        let bob = P2PNode::start(bob_id, "/ip4/127.0.0.1/udp/0/quic-v1").await.unwrap();

        alice.bootstrap_dial(bob.listen_addr().clone()).await.unwrap();
        tokio::time::sleep(Duration::from_millis(500)).await;
        bob.announce_presence().await.unwrap();
        tokio::time::sleep(Duration::from_millis(500)).await;

        for outcome in [DeliveryOutcome::Blocked, DeliveryOutcome::Rejected, DeliveryOutcome::Processed] {
            let bob_recv = bob.clone();
            let recv_task = tokio::spawn(async move {
                let incoming = tokio::time::timeout(Duration::from_secs(12), bob_recv.recv_next())
                    .await
                    .expect("bob should receive the message before the timeout")
                    .expect("incoming channel should not be closed");
                incoming.respond(outcome);
            });

            let (mode, reported) = alice
                .send_to_peer(&bob_peer_id, b"probe".to_vec())
                .await
                .unwrap();
            assert_eq!(mode, P2PTransportMode::DirectQuic);
            assert_eq!(reported, outcome, "sender must see exactly the outcome the recipient reported");
            recv_task.await.expect("bob's receive task must not panic");
        }
    }

    /// Regression test for the 2026-08-22 audit's core "TorStrict does not actually enforce
    /// anything" finding. Alice successfully resolves Bob's real direct QUIC address via the DHT
    /// (proving the lookup path works), then enables TorStrict — the dial itself must still be
    /// refused, because the enforcement now lives at the actual `swarm.dial` call site in
    /// `run_swarm_task`, not just an advisory pre-filter a caller could bypass.
    #[tokio::test]
    async fn test_tor_strict_mode_blocks_direct_dial_at_the_swarm_level() {
        let alice_id = identity("alice");
        let bob_id = identity("bob");
        let bob_peer_id = bob_id.public_id_hex();

        let alice = P2PNode::start(alice_id, "/ip4/127.0.0.1/udp/0/quic-v1").await.unwrap();
        let bob = P2PNode::start(bob_id, "/ip4/127.0.0.1/udp/0/quic-v1").await.unwrap();

        alice.bootstrap_dial(bob.listen_addr().clone()).await.unwrap();
        tokio::time::sleep(Duration::from_millis(500)).await;
        bob.announce_presence().await.unwrap();
        tokio::time::sleep(Duration::from_millis(500)).await;

        // `enabled: false` is deliberate, not an oversight: `validate_outbound_dial` — the thing
        // this test actually exercises, at the real `swarm.dial` call site — is driven purely by
        // `mode`, never by `enabled` (see `TorManager::validate_outbound_dial`). `enabled: true`
        // would additionally make `configure_tor` launch a real, in-process onion service (see
        // `P2PNode::ensure_onion_service_started`), which tries to bootstrap a genuine Tor
        // circuit over the real network — pointless network I/O this test doesn't need, and, on a
        // host with no route to the Tor network (e.g. a sandboxed CI runner), a real multi-minute
        // delay this test shouldn't be paying for just to prove a direct dial gets refused.
        alice
            .configure_tor(crate::tor::TorConfig {
                enabled: false,
                mode: crate::tor::TorMode::TorStrict,
                socks_proxy: "127.0.0.1:9050".to_string(),
                onion_address: None,
                bridge_type: None,
            })
            .await;

        let (mode, _) = alice
            .send_to_peer(&bob_peer_id, b"should never leave loopback".to_vec())
            .await
            .unwrap();
        assert_eq!(
            mode,
            P2PTransportMode::Disconnected,
            "TorStrict must refuse the direct dial even though Bob's real address was successfully resolved via the DHT"
        );
    }

    /// Looking up a peer nobody has ever announced must resolve to "not found", not hang or
    /// error out — the outbox-retry logic in `nova-engine` depends on this being a clean,
    /// non-fatal outcome.
    #[tokio::test]
    async fn test_lookup_of_unknown_peer_returns_disconnected() {
        let alice_id = identity("alice");
        let alice = P2PNode::start(alice_id, "/ip4/127.0.0.1/udp/0/quic-v1").await.unwrap();

        let fake_peer_id = hex::encode([0x42u8; 32]);
        let mode = alice.connect_to_peer(&fake_peer_id).await.unwrap();
        assert_eq!(mode, P2PTransportMode::Disconnected);
    }

    /// Proves the relay/DCUtR path specifically: Bob is made discoverable *only* through a
    /// relay circuit address (his direct address is deliberately withheld from his DHT record),
    /// standing in for a peer behind NAT so restrictive that no direct address of theirs is
    /// usable at all — the case circuit-relay-v2 exists for. Alice must reach him purely by
    /// dialing through the relay; if that path were not wired up correctly, this would time out
    /// rather than merely "prefer" the wrong path.
    #[tokio::test]
    async fn test_peer_reachable_only_through_relay_circuit() {
        let relay_id = identity("relay");
        let alice_id = identity("alice");
        let bob_id = identity("bob");
        let bob_peer_id = bob_id.public_id_hex();

        let relay_node = P2PNode::start(relay_id, "/ip4/127.0.0.1/udp/0/quic-v1").await.unwrap();
        let alice = P2PNode::start(alice_id, "/ip4/127.0.0.1/udp/0/quic-v1").await.unwrap();
        let bob = P2PNode::start(bob_id, "/ip4/127.0.0.1/udp/0/quic-v1").await.unwrap();

        // Both Alice and Bob connect to the relay directly (this is the one bootstrap step real
        // deployments also need: a small list of known relay/bootstrap peers). This alone lets
        // DHT lookups for Bob route through the relay, since the relay is the only peer Alice
        // knows — but it does NOT give Alice a way to reach Bob directly.
        alice.bootstrap_dial(relay_node.listen_addr().clone()).await.unwrap();
        bob.bootstrap_dial(relay_node.listen_addr().clone()).await.unwrap();
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Bob reserves a slot on the relay and publishes *only* the resulting circuit address —
        // his real direct address is never announced, so any successful connection below must
        // have gone through the relay.
        let circuit_addr = bob.reserve_relay_slot(relay_node.full_listen_addr()).await.unwrap();
        assert!(
            circuit_addr.iter().any(|p| matches!(p, libp2p::core::multiaddr::Protocol::P2pCircuit)),
            "expected a /p2p-circuit address, got {circuit_addr}"
        );
        bob.announce_addresses_only(vec![circuit_addr]).await.unwrap();
        tokio::time::sleep(Duration::from_millis(500)).await;

        let bob_recv = bob.clone();
        let recv_task = tokio::spawn(async move {
            // Extra margin for this path specifically: it also involves relay circuit
            // reservation/dial, not just a DHT lookup + direct dial.
            let incoming = tokio::time::timeout(Duration::from_secs(15), bob_recv.recv_next())
                .await
                .expect("bob should receive the relayed message before the timeout")
                .expect("incoming channel should not be closed");
            let bytes = incoming.bytes.clone();
            incoming.respond(DeliveryOutcome::Processed);
            bytes
        });

        let (mode, outcome) = alice
            .send_to_peer(&bob_peer_id, b"hello bob via relay only".to_vec())
            .await
            .unwrap();
        assert_ne!(mode, P2PTransportMode::Disconnected, "message must actually be delivered through the relay");
        assert_eq!(outcome, DeliveryOutcome::Processed);

        let received = recv_task.await.expect("bob's receive task must not panic");
        assert_eq!(received, b"hello bob via relay only");
    }
}
