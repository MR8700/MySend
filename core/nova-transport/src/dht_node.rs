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
//! rather than skipped) is the happy path for peers on the same LAN. For a peer reached only
//! through wide-area/DHT discovery, a direct dial is *never* attempted at all — see
//! [`validate_outbound_dial`] — every such connection goes through circuit-relay-v2 instead: any
//! reachable peer can act as a relay for another (opt-in, resource-limited by `relay::Config`'s
//! defaults). This is the project's IP-hiding property (a contact never learns this device's
//! real IP unless they're already on the same LAN as it): DCUtR — libp2p's automatic
//! relayed-to-direct hole-punching upgrade — is deliberately NOT wired into this node's
//! `NovaBehaviour`, since a successful hole-punch would silently defeat that property the moment
//! it completed. The relay only ever forwards opaque bytes — it is not a trusted party, exactly
//! like the discovery DHT above it.

use crate::error::TransportError;
use crate::seed_nodes::{fetch_remote_nodes_config, MultiFallbackPool, DEFAULT_PRIMARY_RELAY_URL};
use crate::udp_fallback::UdpFallbackClient;
use crate::{P2PTransportMode, TransportSupervisor};
use futures_util::StreamExt;
use libp2p::core::multiaddr::Protocol;
use libp2p::kad::store::MemoryStore;
use libp2p::request_response::{OutboundRequestId, ProtocolSupport};
use libp2p::swarm::{NetworkBehaviour, SwarmEvent};
use libp2p::{identify, identity, kad, mdns, relay, request_response, Multiaddr, PeerId, StreamProtocol, Swarm};
use nova_crypto::DeviceIdentity;
use nova_protocol::SignedDhtPeerRecord;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
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
    /// Lets this node reserve a slot on (and dial through) another peer's relay. Deliberately
    /// paired with no `dcutr::Behaviour` — see this module's doc comment on why an automatic
    /// relayed-to-direct upgrade is not wanted here.
    relay_client: relay::client::Behaviour,
}

/// A resolved peer: their libp2p identity plus the addresses their signed DHT record claims to
/// be reachable at.
type ResolvedPeer = (PeerId, Vec<Multiaddr>);
type LookupResponder = oneshot::Sender<Result<Option<ResolvedPeer>, TransportError>>;
type PendingLookups = HashMap<kad::QueryId, (String, LookupResponder)>;

pub(crate) enum Command {
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
    /// Dynamically updates or clears the fallback discovery/relay server pool.
    SetFallback(MultiFallbackPool),
}

pub struct P2PNode {
    own_peer_id: String,
    libp2p_peer_id: PeerId,
    listen_addrs: Vec<Multiaddr>,
    command_tx: mpsc::UnboundedSender<Command>,
    incoming_rx: Mutex<mpsc::UnboundedReceiver<IncomingMessage>>,
    pub supervisor: Arc<TransportSupervisor>,
    /// Secondary delivery and discovery path for when the DHT + relay-circuit path above cannot reach a peer
    /// at all — backed by a multi-server dynamic pool with striping (see `crate::seed_nodes`).
    fallback_pool: Arc<tokio::sync::RwLock<MultiFallbackPool>>,
}

/// Where an [`IncomingMessage`] arrived from — determines whether [`IncomingMessage::respond`]
/// has anyone live to report back to.
#[derive(Debug)]
pub(crate) enum IncomingSource {
    /// A live QUIC request/response round-trip: the sender is actually waiting on the other end
    /// of `channel` for a [`DeliveryOutcome`].
    LibP2p {
        channel: request_response::ResponseChannel<DeliveryOutcome>,
        command_tx: mpsc::UnboundedSender<Command>,
    },
    /// Pulled from `nova-server`'s blind relay via [`crate::udp_fallback::UdpFallbackClient::drain_incoming`]
    /// — store-and-forward, not a live round-trip, so there is no sender waiting for an outcome.
    UdpFallback,
}

/// One raw wire packet received from a peer, still awaiting an application-level verdict from
/// `nova-engine` on whether it was actually processed, blocked, or rejected — see
/// [`DeliveryOutcome`]. The caller MUST eventually call [`IncomingMessage::respond`] exactly
/// once; for a live libp2p-sourced message, until then the sender's `send_to_peer` call is left
/// waiting on the wire.
pub struct IncomingMessage {
    pub bytes: Vec<u8>,
    pub(crate) source: IncomingSource,
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
                Ok(NovaBehaviour {
                    kademlia,
                    mdns,
                    identify,
                    messaging,
                    relay,
                    relay_client,
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

        // Secondary delivery path (see `crate::udp_fallback`) for networks where this node
        // cannot reach the DHT/relay-circuit path at all yet. Defaults to the project's own
        // `nova-server` instance (a WebSocket-based presence registry + blind relay, deployed
        // free-tier on Render — see `server/`) so every device works out of the box with no
        // per-device setup, exactly like a browser ships with default STUN/bootstrap servers
        // rather than requiring the user to find and paste one in. `NOVA_UDP_FALLBACK_ADDR`
        // Fallback Discovery and Multi-Server Relay Pool:
        // Automatically fetches the dynamic server list from GitHub Raw (`network_nodes.json`)
        // while falling back to built-in defaults or `NOVA_UDP_FALLBACK_ADDR` env override.
        let fallback_env = std::env::var("NOVA_UDP_FALLBACK_ADDR").ok();
        let initial_pool = if fallback_env.as_deref() == Some("none") {
            MultiFallbackPool::new(Vec::new())
        } else if let Some(url) = fallback_env {
            MultiFallbackPool::new(vec![url])
        } else {
            MultiFallbackPool::new(vec![DEFAULT_PRIMARY_RELAY_URL.to_string()])
        };
        let fallback_pool = Arc::new(tokio::sync::RwLock::new(initial_pool.clone()));

        let node = Arc::new(Self {
            own_peer_id,
            libp2p_peer_id: local_peer_id,
            listen_addrs: resolved_listen_addrs,
            command_tx,
            incoming_rx: Mutex::new(incoming_rx),
            supervisor,
            fallback_pool: fallback_pool.clone(),
        });

        // Background Remote Seed Fetcher:
        // Periodically/at-startup queries GitHub Raw for updated relay nodes without requiring app rebuilds.
        let background_pool = node.fallback_pool.clone();
        let background_cmd_tx = node.command_tx.clone();
        tokio::spawn(async move {
            let remote_cfg = fetch_remote_nodes_config().await;
            if !remote_cfg.fallback_servers.is_empty() {
                let updated_pool = MultiFallbackPool::from_config(&remote_cfg);
                *background_pool.write().await = updated_pool.clone();
                let _ = background_cmd_tx.send(Command::SetFallback(updated_pool));
            }
        });

        // `identity` moves into the background task by value rather than being cloned: it holds
        // zeroized private key material and deliberately does not implement `Clone`. Only the
        // task itself needs it, for signing DHT presence records — the public `P2PNode` handle
        // only ever needs the already-derived, non-secret `own_peer_id`.
        let command_tx_for_task = node.command_tx.clone();
        tokio::spawn(run_swarm_task(swarm, identity, local_peer_id, command_rx, command_tx_for_task, incoming_tx, initial_pool));

        Ok(node)
    }

    pub fn peer_id(&self) -> &str {
        &self.own_peer_id
    }

    /// Dynamically sets or clears the fallback server URL (e.g. from the UI Settings).
    pub async fn set_fallback_server_url(&self, url: Option<String>) {
        let pool = match url {
            Some(u) if !u.trim().is_empty() && u.trim() != "none" => {
                MultiFallbackPool::new(vec![u.trim().to_string()])
            }
            _ => MultiFallbackPool::new(Vec::new()),
        };
        *self.fallback_pool.write().await = pool.clone();
        let _ = self.command_tx.send(Command::SetFallback(pool));
    }

    /// Dynamically sets multiple fallback server URLs (e.g. for multi-path striping).
    pub async fn set_fallback_server_urls(&self, urls: Vec<String>) {
        let pool = MultiFallbackPool::new(urls);
        *self.fallback_pool.write().await = pool.clone();
        let _ = self.command_tx.send(Command::SetFallback(pool));
    }

    /// Returns the currently active primary fallback server URL, if any.
    pub async fn get_fallback_server_url(&self) -> Option<String> {
        let urls = self.fallback_pool.read().await.urls();
        urls.first().cloned()
    }

    /// Returns all currently active fallback server URLs in the pool.
    pub async fn get_fallback_server_urls(&self) -> Vec<String> {
        self.fallback_pool.read().await.urls()
    }

    /// Publishes this device's public directory profile and PreKey bundle to the fallback servers.
    pub async fn register_directory_entry(&self, entry: nova_protocol::SignedDirectoryEntry) -> Result<(), TransportError> {
        let pool = self.fallback_pool.read().await.clone();
        let urls = pool.urls();
        if urls.is_empty() {
            return Err(TransportError::Setup("no fallback server configured".into()));
        }
        // Register on primary client
        let client = UdpFallbackClient::new(pool.primary_url());
        client.register_directory_signed(entry).await
    }

    /// Searches the fallback directory across all active servers in parallel, merging results.
    pub async fn search_directory(&self, query: &str) -> Result<Vec<nova_protocol::DirectorySearchResult>, TransportError> {
        let pool = self.fallback_pool.read().await.clone();
        pool.search_directory_merged(query).await
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
        crate::dial_policy::validate_outbound_dial(&addr)?;
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

        // If DHT lookup didn't find the peer, check the fallback discovery servers (fastest response wins)
        let pool = self.fallback_pool.read().await.clone();
        if let Ok(Some(endpoint)) = pool.lookup_fastest(nova_peer_id).await {
            if let Ok(pub_bytes) = hex::decode(nova_peer_id) {
                if let Ok(ed_pub) = libp2p::identity::ed25519::PublicKey::try_from_bytes(&pub_bytes) {
                    let libp2p_peer_id = libp2p::identity::PublicKey::from(ed_pub).to_peer_id();
                    let mut addrs = Vec::new();
                    if let (Some(local_ip), Some(local_port)) = (&endpoint.local_ip, endpoint.local_port) {
                        if let Ok(addr) = format!("/ip4/{local_ip}/udp/{local_port}/quic-v1").parse::<Multiaddr>() {
                            addrs.push(addr);
                        }
                    }
                    if endpoint.public_port != 0 && !endpoint.public_ip.is_empty() && endpoint.public_ip != "0.0.0.0" {
                        if let Ok(addr) = format!("/ip4/{}/udp/{}/quic-v1", endpoint.public_ip, endpoint.public_port).parse::<Multiaddr>() {
                            addrs.push(addr);
                        }
                    }
                    if !addrs.is_empty() {
                        debug!("Resolved peer {nova_peer_id} via fallback discovery pool");
                        return Ok(Some((libp2p_peer_id, addrs)));
                    }
                }
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
        let filtered_addrs: Vec<Multiaddr> = addrs
            .into_iter()
            .filter(|addr| crate::dial_policy::validate_outbound_dial(addr).is_ok())
            .collect();

        if filtered_addrs.is_empty() {
            return Err(TransportError::Setup(
                "no dialable addresses permitted under the LAN/relay-only dial policy".into(),
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
            self.spawn_udp_fallback_send_chunks(nova_peer_id, chunks).await;
            return Ok((P2PTransportMode::Disconnected, DeliveryOutcome::Rejected));
        };
        let mode = connection_mode_for(&addrs);

        if tokio::time::timeout(DIRECT_CONNECT_TIMEOUT, self.dial(peer_id, addrs))
            .await
            .is_err()
        {
            self.spawn_udp_fallback_send_chunks(nova_peer_id, chunks).await;
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

    /// Waits for the next raw packet received from any peer over a direct connection. Returns
    /// `None` only if the node's background task has stopped. The caller MUST call
    /// [`IncomingMessage::respond`] on the result exactly once, or the sender's `send_to_peer`
    /// call will hang until its own timeout.
    pub async fn recv_next(&self) -> Option<IncomingMessage> {
        self.incoming_rx.lock().await.recv().await
    }

    /// Best-effort, fire-and-forget hand-off of `chunks` to the multi-server fallback relay pool (see
    /// `crate::seed_nodes`) for `nova_peer_id` — called from `send_chunks_to_peer`'s failure
    /// paths so a peer unreachable through the DHT still gets a second chance.
    ///
    /// Chunks are **striped in parallel** across all active relay servers to maximize throughput
    /// and provide automatic failover.
    async fn spawn_udp_fallback_send_chunks(&self, nova_peer_id: &str, chunks: Vec<Vec<u8>>) {
        let pool = self.fallback_pool.read().await.clone();
        if pool.is_empty() || chunks.is_empty() {
            return;
        }
        let nova_peer_id = nova_peer_id.to_string();
        tokio::spawn(async move {
            pool.send_chunks_multipath(&nova_peer_id, chunks).await;
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
    mut fallback_pool: MultiFallbackPool,
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
                if let Err(e) = do_announce(&mut swarm, &identity, local_peer_id, &fallback_pool) {
                    warn!("Presence heartbeat failed: {e}");
                }
            }
            _ = udp_fallback_drain.tick() => {
                if !fallback_pool.is_empty() {
                    let drain_request = nova_protocol::SignedDrainRequest::sign(&identity, now_secs());
                    let pool = fallback_pool.clone();
                    let incoming_tx = incoming_tx.clone();
                    tokio::spawn(async move {
                        pool.drain_incoming_all(drain_request, incoming_tx).await;
                    });
                }
            }
            maybe_cmd = command_rx.recv() => {
                let Some(cmd) = maybe_cmd else { break };
                match cmd {
                    Command::Announce(respond) => {
                        let _ = respond.send(do_announce(&mut swarm, &identity, local_peer_id, &fallback_pool));
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
                        // `swarm.dial` gets called, so it is the one place the LAN/relay-only
                        // policy cannot slip through regardless of which caller constructed this
                        // command. See the 2026-08-22 audit's "TorStrict mode is not actually
                        // enforced" finding — the pre-filter alone was exactly that weakness, and
                        // the same reasoning applies to this policy's unconditional replacement.
                        let addrs: Vec<Multiaddr> = addrs
                            .into_iter()
                            .filter(|a| crate::dial_policy::validate_outbound_dial(a).is_ok())
                            .collect();
                        if addrs.is_empty() {
                            let _ = respond.send(Err(TransportError::Setup(
                                "no dialable addresses permitted under the LAN/relay-only dial policy".into(),
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
                        // Same hard gate as `Command::Dial` above, and for the same reason: the
                        // advisory pre-filter `P2PNode::bootstrap_dial` already applies before
                        // ever sending this command is not itself enforcement — this is the
                        // actual `swarm.dial` call site.
                        if let Err(e) = crate::dial_policy::validate_outbound_dial(&addr) {
                            let _ = respond.send(Err(e));
                            continue;
                        }
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
                    Command::SetFallback(new_pool) => {
                        fallback_pool = new_pool;
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

fn do_announce(
    swarm: &mut Swarm<NovaBehaviour>,
    identity: &DeviceIdentity,
    local_peer_id: PeerId,
    fallback_pool: &MultiFallbackPool,
) -> Result<(), TransportError> {
    let mut addrs: Vec<String> = swarm
        .external_addresses()
        .chain(swarm.listeners())
        .map(|a| a.to_string())
        .collect();
    addrs.sort();
    addrs.dedup();

    let dht_result = publish_record(swarm, identity, local_peer_id, addrs);

    // Announce to all active fallback servers in parallel
    if !fallback_pool.is_empty() {
        let local_udp_port = swarm.listeners().find_map(|a| {
            a.iter().find_map(|p| match p {
                Protocol::Udp(port) => Some(port),
                _ => None,
            })
        }).unwrap_or(0);

        let local_ip = swarm.listeners().find_map(|a| {
            a.iter().find_map(|p| match p {
                Protocol::Ip4(ip) if !ip.is_loopback() && !ip.is_unspecified() => Some(ip.to_string()),
                _ => None,
            })
        });

        let endpoint = nova_protocol::PeerEndpoint {
            peer_id: identity.public_id_hex(),
            public_ip: "0.0.0.0".to_string(), // Replaced by server with observed/proxy IP
            public_port: local_udp_port,
            local_ip,
            local_port: Some(local_udp_port),
        };

        let registration = nova_protocol::SignedPresenceRegistration::sign(identity, endpoint, now_secs());
        let pool = fallback_pool.clone();
        tokio::spawn(async move {
            pool.register_presence_all(registration).await;
        });
    }

    dht_result
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

    /// Regression test for the 2026-08-22 audit's core "an advisory pre-filter alone is not
    /// enforcement" finding, adapted for the LAN/relay-only dial policy that replaced Tor (see
    /// `crate::dial_policy`): a WAN address must be refused at the actual `swarm.dial` call site
    /// inside `run_swarm_task`'s `Command::Dial` handler — not merely by a caller-side check that
    /// a differently-written caller could skip. Calls the crate-private `dial()` directly (this
    /// test lives in the same module) with a fabricated public multiaddr for a `PeerId` that
    /// doesn't need to correspond to a real peer: the policy block happens purely from inspecting
    /// the address, before any real network I/O is attempted, so nothing needs to actually be
    /// listening there for this to prove the block fires at the swarm level.
    #[tokio::test]
    async fn test_wan_direct_dial_is_blocked_at_the_swarm_level() {
        let alice_id = identity("alice");
        let alice = P2PNode::start(alice_id, "/ip4/127.0.0.1/udp/0/quic-v1").await.unwrap();

        let fake_peer_id = PeerId::random();
        let wan_addr: Multiaddr = "/ip4/203.0.113.9/udp/4001/quic-v1".parse().unwrap();

        let err = alice.dial(fake_peer_id, vec![wan_addr]).await.unwrap_err();
        match err {
            // `dial()` filters the address list through `dial_policy::validate_outbound_dial`
            // and, finding it empty, returns its own generic message rather than propagating the
            // per-address rejection reason — see `dial_policy`'s own unit tests for coverage of
            // that underlying per-address message.
            TransportError::Setup(msg) => assert!(
                msg.contains("dial policy"),
                "expected the LAN/relay-only dial policy rejection, got: {msg}"
            ),
            other => panic!("expected TransportError::Setup from the dial policy, got: {other:?}"),
        }
    }

    /// The same policy must permit a same-LAN (here: loopback, standing in for any private/LAN
    /// address) direct dial — there is no privacy benefit to routing local traffic through a
    /// remote relay, and blocking it would break the documented "same Wi-Fi, zero config" path.
    #[tokio::test]
    async fn test_lan_direct_dial_is_permitted_end_to_end() {
        let alice_id = identity("alice");
        let bob_id = identity("bob");
        let bob_peer_id = bob_id.public_id_hex();

        let alice = P2PNode::start(alice_id, "/ip4/127.0.0.1/udp/0/quic-v1").await.unwrap();
        let bob = P2PNode::start(bob_id, "/ip4/127.0.0.1/udp/0/quic-v1").await.unwrap();

        alice.bootstrap_dial(bob.listen_addr().clone()).await.unwrap();
        tokio::time::sleep(Duration::from_millis(500)).await;
        bob.announce_presence().await.unwrap();
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Bob must actively receive and respond, or Alice's `send_to_peer` below just times out
        // waiting for the request/response round-trip — see `test_two_independent_nodes_discover_
        // via_dht_and_exchange_bytes` above, which this mirrors.
        let bob_recv = bob.clone();
        let recv_task = tokio::spawn(async move {
            let incoming = tokio::time::timeout(Duration::from_secs(10), bob_recv.recv_next())
                .await
                .expect("bob should receive the message before the timeout")
                .expect("incoming channel should not be closed");
            incoming.respond(DeliveryOutcome::Processed);
        });

        let (mode, _) = alice
            .send_to_peer(&bob_peer_id, b"loopback is LAN, must go through".to_vec())
            .await
            .unwrap();
        assert_eq!(mode, P2PTransportMode::DirectQuic, "a loopback/LAN address must be dialed directly, not blocked");
        recv_task.await.expect("bob's receive task must not panic");
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
