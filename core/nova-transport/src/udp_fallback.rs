//! Discovery/signaling/relay fallback — a thin client for `nova-server`, the
//! presence-registry-and-blind-relay protocol that predates the Kademlia DHT migration (see
//! `dht_node`'s module docs). Re-wired here as a genuine production backup path rather than left
//! orphaned: when `nova-server` was found to have no remaining callers anywhere in the codebase
//! (2026-08-22 audit), the fix chosen was to reconnect it as a fallback, not delete it.
//!
//! ## Why WebSocket, not raw UDP (module name kept for continuity)
//!
//! This module (and the `NOVA_UDP_FALLBACK_ADDR` env var, and the `UdpFallbackClient` name) date
//! from when `nova-server` spoke raw UDP directly. It was rewritten to run over WebSocket
//! (`nova-server`'s `service.rs`) so it can be deployed on ordinary HTTP-only PaaS hosts (Render,
//! and effectively every other such host) that expose no raw UDP ingress at all — the whole point
//! of this path is to be the one well-known, centrally-reachable fallback that works even when a
//! device's network filters everything else, so it needs to be dead simple to keep running
//! somewhere. The names were left as-is rather than renamed throughout `dht_node.rs` purely to
//! keep this change mechanical; nothing about the actual wire behavior below is UDP anymore.
//!
//! ## Why keep a second path at all
//!
//! The DHT + circuit-relay-v2 + DCUtR path in `dht_node` is the primary transport and covers the
//! overwhelming majority of networks. But it depends on reaching at least one other peer first
//! (mDNS on the LAN, or a known bootstrap/relay node) to even begin Kademlia routing-table
//! discovery. A device that cannot reach ANY libp2p peer at all yet (a fresh install whose only
//! configured bootstrap address happens to be unreachable right now, or a network where QUIC/UDP
//! is filtered outright but ordinary outbound HTTPS/WebSocket traffic is not) has no path forward
//! on the DHT alone. `nova-server`'s single well-known WebSocket endpoint is a second, independent
//! way to both announce reachability and hand off opaque ciphertext — exactly the
//! centrally-reachable fallback a "small list of well-known bootstrap peers" already assumes
//! exists for first contact, just over a different, simpler protocol that degrades better under
//! aggressive UDP filtering (the same class of restrictive network that also broke embedded Tor
//! bootstrap for this project — see `tor.rs` — since outbound WebSocket-over-443 traffic is
//! indistinguishable at the network level from any other HTTPS request).
//!
//! ## What this path can and cannot promise
//!
//! Unlike the DHT/relay-circuit path (a live QUIC request/response round-trip, so the sender
//! learns a real [`crate::DeliveryOutcome`] from the recipient's own engine), `RelayForward` is
//! fire-and-forget store-and-forward: the server accepts custody of the bytes and purges them
//! after 24h if never drained (see `nova_server::relay::RELAY_EXPIRY_SECONDS`), with no signal
//! back to the sender either way. `nova-engine`'s outbox therefore never treats a successful
//! `relay_forward` call as delivery confirmation — see `P2PNode::send_to_peer`'s use of this
//! module — only as "one more way the bytes might get through," attempted alongside, not instead
//! of, the primary path.

use crate::error::TransportError;
use futures_util::{SinkExt, StreamExt};
use nova_crypto::DeviceIdentity;
use nova_protocol::{PeerEndpoint, ServerRequest, ServerResponse, SignedDrainRequest, SignedPresenceRegistration};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio_tungstenite::tungstenite::Message;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(6);
/// Matches `nova_server::service::MAX_DATAGRAM_SIZE` — the signaling endpoint is control-plane
/// traffic, not bulk transfer, so a reply never needs to be larger than this.
const MAX_RESPONSE_SIZE: usize = 16 * 1024;
/// Also matches `nova_server::service::MAX_DATAGRAM_SIZE`: `relay_forward` checks this up front
/// so an oversized `RelayForward { payload, .. }` — e.g. one of `nova-engine`'s
/// `MEDIA_CHUNK_SIZE` chunks, sized for the primary DHT/QUIC transport's ~1 MiB request ceiling,
/// not this control-plane one — fails cleanly and immediately instead of being rejected by the
/// server mid-flight.
const MAX_RELAY_PAYLOAD_SIZE: usize = 15 * 1024;

/// Client for the discovery/signaling/blind-relay fallback server (`nova-server`), spoken over a
/// WebSocket connection — see this module's doc comment for why.
#[derive(Clone, Debug)]
pub struct UdpFallbackClient {
    /// A full `ws://` or `wss://` URL, e.g. `wss://nova-discovery.onrender.com`. Kept as the
    /// original string (not a parsed `Url`) since `tokio_tungstenite::connect_async` accepts one
    /// directly and this client never needs to inspect its components.
    server_url: String,
}

impl UdpFallbackClient {
    /// `server_url` must include an explicit `ws://` or `wss://` scheme (e.g.
    /// `wss://nova-discovery.onrender.com` for a real deployment, or `ws://127.0.0.1:8080` for a
    /// local/LAN one) — there is no implicit default, since guessing wrong between plaintext and
    /// TLS silently either leaks this signaling traffic or fails to connect at all.
    pub fn new(server_url: impl Into<String>) -> Self {
        Self { server_url: server_url.into() }
    }

    pub fn server_url(&self) -> &str {
        &self.server_url
    }

    /// Opens a fresh WebSocket connection for exactly one request/response pair, then closes it —
    /// mirrors the previous UDP client's "fresh ephemeral socket per call" design, which kept
    /// this trivially safe to share across concurrent callers (announce heartbeat, outbox
    /// fallback sends, the drain poll) without needing connection pooling or a demultiplexing
    /// layer for replies. This is a deliberately simple/low-frequency fallback path, not the hot
    /// path, so a full connection handshake per call is an acceptable cost for that simplicity.
    async fn roundtrip(&self, request: &ServerRequest) -> Result<ServerResponse, TransportError> {
        let (mut ws, _response) = tokio::time::timeout(REQUEST_TIMEOUT, tokio_tungstenite::connect_async(&self.server_url))
            .await
            .map_err(|_| TransportError::Timeout)?
            .map_err(|e| TransportError::Setup(format!("failed to connect to fallback server at {}: {e}", self.server_url)))?;

        let bytes = request.to_bytes()?;
        tokio::time::timeout(REQUEST_TIMEOUT, ws.send(Message::Binary(bytes)))
            .await
            .map_err(|_| TransportError::Timeout)?
            .map_err(|e| TransportError::Send(e.to_string()))?;

        let msg = tokio::time::timeout(REQUEST_TIMEOUT, ws.next())
            .await
            .map_err(|_| TransportError::Timeout)?
            .ok_or_else(|| TransportError::Setup("fallback server closed the connection without a response".to_string()))?
            .map_err(|e| TransportError::Setup(format!("WebSocket error: {e}")))?;

        // Best-effort graceful close — the response has already been read either way, so a
        // failure here (e.g. the server already dropped the connection) is not itself an error.
        let _ = ws.close(None).await;

        match msg {
            Message::Binary(bytes) if bytes.len() <= MAX_RESPONSE_SIZE => Ok(ServerResponse::from_bytes(&bytes)?),
            Message::Binary(bytes) => Err(TransportError::Setup(format!(
                "fallback server response of {} bytes exceeds the {MAX_RESPONSE_SIZE}-byte limit",
                bytes.len()
            ))),
            other => Err(TransportError::Setup(format!("unexpected WebSocket message from fallback server: {other:?}"))),
        }
    }

    /// Announces (or refreshes) this device's reachability with the fallback server. `endpoint`
    /// should be this node's best-known direct address (its own claim); the server overwrites
    /// the public IP with whatever it actually observed the connection arrive from (see
    /// `nova_server::registry::PresenceRegistry::register`), so a stale or lying `public_ip`
    /// here can never misdirect anyone — the signature only proves identity, not the address.
    pub async fn register_presence(&self, identity: &DeviceIdentity, endpoint: PeerEndpoint) -> Result<(), TransportError> {
        self.register_presence_signed(SignedPresenceRegistration::sign(identity, endpoint, now_secs())).await
    }

    /// Same as [`register_presence`](Self::register_presence), but takes an already-signed
    /// registration — for callers (the `P2PNode` background task) that must not hold or clone a
    /// `DeviceIdentity` across the `.await` of a slow network round-trip; they sign synchronously
    /// first, then hand the result to a spawned task that only needs this method.
    pub async fn register_presence_signed(&self, registration: SignedPresenceRegistration) -> Result<(), TransportError> {
        match self.roundtrip(&ServerRequest::Register(registration)).await? {
            ServerResponse::Registered => Ok(()),
            ServerResponse::Error(e) => Err(TransportError::Setup(format!("fallback server rejected registration: {e}"))),
            other => Err(unexpected_response(other)),
        }
    }

    /// Looks up a peer's last-announced reachability on the fallback server — independent of
    /// whether they are currently reachable via the DHT.
    pub async fn lookup(&self, peer_id: &str) -> Result<Option<PeerEndpoint>, TransportError> {
        match self.roundtrip(&ServerRequest::Lookup { peer_id: peer_id.to_string() }).await? {
            ServerResponse::LookupResult(endpoint) => Ok(endpoint),
            ServerResponse::Error(e) => Err(TransportError::Setup(format!("fallback server rejected lookup: {e}"))),
            other => Err(unexpected_response(other)),
        }
    }

    /// Hands an already-encrypted packet to the fallback server's blind relay for
    /// `target_peer_id` to pick up later via [`drain_incoming`](Self::drain_incoming).
    /// Best-effort and unconfirmed — see this module's docs on what this path can promise.
    ///
    /// Rejects `payload` up front if it exceeds [`MAX_RELAY_PAYLOAD_SIZE`] — see that constant's
    /// docs for why this path cannot carry a full-sized media chunk the way the primary transport
    /// can.
    pub async fn relay_forward(&self, target_peer_id: &str, payload: Vec<u8>) -> Result<(), TransportError> {
        if payload.len() > MAX_RELAY_PAYLOAD_SIZE {
            return Err(TransportError::Send(format!(
                "payload of {} bytes exceeds the fallback relay's {MAX_RELAY_PAYLOAD_SIZE}-byte ceiling",
                payload.len()
            )));
        }
        let request = ServerRequest::RelayForward { target_peer_id: target_peer_id.to_string(), payload };
        match self.roundtrip(&request).await? {
            ServerResponse::RelayForwarded => Ok(()),
            ServerResponse::Error(e) => Err(TransportError::Setup(format!("fallback relay rejected forward: {e}"))),
            other => Err(unexpected_response(other)),
        }
    }

    /// Fetches and purges every packet currently queued for this device on the fallback server.
    pub async fn drain_incoming(&self, identity: &DeviceIdentity) -> Result<Vec<Vec<u8>>, TransportError> {
        self.drain_incoming_signed(SignedDrainRequest::sign(identity, now_secs())).await
    }

    /// Same as [`drain_incoming`](Self::drain_incoming), but takes an already-signed drain
    /// request — see [`register_presence_signed`](Self::register_presence_signed) for why.
    pub async fn drain_incoming_signed(&self, drain_request: SignedDrainRequest) -> Result<Vec<Vec<u8>>, TransportError> {
        match self.roundtrip(&ServerRequest::RelayDrain(drain_request)).await? {
            ServerResponse::RelayDrained(items) => Ok(items),
            ServerResponse::Error(e) => Err(TransportError::Setup(format!("fallback relay rejected drain: {e}"))),
            other => Err(unexpected_response(other)),
        }
    }
}

fn unexpected_response(response: ServerResponse) -> TransportError {
    TransportError::Setup(format!("unexpected fallback server response: {response:?}"))
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

    /// Starts a real `nova-server` bound to an OS-assigned port and returns a client pointed at
    /// it over `ws://` — every test in this module runs against the genuine server
    /// implementation, not a mock of the wire protocol.
    async fn start_test_server() -> UdpFallbackClient {
        let (listener, registry, relay) = nova_server::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(nova_server::serve_forever(listener, registry, relay));
        // Give the server a moment to start accepting connections before the first request.
        tokio::time::sleep(Duration::from_millis(150)).await;
        UdpFallbackClient::new(format!("ws://{addr}/"))
    }

    #[tokio::test]
    async fn test_register_and_lookup_roundtrip() {
        let client = start_test_server().await;
        let alice = identity("alice");

        let endpoint = PeerEndpoint {
            peer_id: String::new(), // overwritten by SignedPresenceRegistration::sign
            public_ip: "0.0.0.0".to_string(), // ignored too — server uses the observed source IP
            public_port: 4433,
            local_ip: Some("10.0.0.5".to_string()),
            local_port: Some(4433),
        };
        client.register_presence(&alice, endpoint).await.unwrap();

        let found = client.lookup(&alice.public_id_hex()).await.unwrap().expect("just-registered peer must be found");
        assert_eq!(found.public_port, 4433);
        assert_eq!(found.public_ip, "127.0.0.1", "server must report the observed loopback source, not the claimed 0.0.0.0");

        assert!(client.lookup(&hex::encode([0x99u8; 32])).await.unwrap().is_none());
    }

    /// The core promise of the fallback relay path: a packet handed off via `relay_forward`
    /// really does reach a `drain_incoming` call from the intended recipient, through the real
    /// server's in-memory queue — and only that recipient's signed drain request can retrieve it.
    #[tokio::test]
    async fn test_relay_forward_and_drain_roundtrip() {
        let client = start_test_server().await;
        let alice = identity("alice");
        let bob = identity("bob");

        client.relay_forward(&bob.public_id_hex(), b"opaque ciphertext for bob".to_vec()).await.unwrap();

        // Alice draining her own queue must not see Bob's packet.
        let alice_drained = client.drain_incoming(&alice).await.unwrap();
        assert!(alice_drained.is_empty());

        let bob_drained = client.drain_incoming(&bob).await.unwrap();
        assert_eq!(bob_drained, vec![b"opaque ciphertext for bob".to_vec()]);

        // Draining is destructive — a second drain finds nothing left.
        assert!(client.drain_incoming(&bob).await.unwrap().is_empty());
    }
}
