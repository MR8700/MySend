//! A minimal, transport-level-unauthenticated point-to-point channel over Tor SOCKS5 — the
//! actual send/receive primitive `TorStrict`/`Hybrid` mode falls back to when a peer cannot (or,
//! in `TorStrict`, must not) be reached via the DHT/QUIC path in `dht_node`.
//!
//! ## Why not a libp2p `Transport`
//!
//! Deliberately NOT integrated as a `libp2p::core::Transport` registered on the same
//! `SwarmBuilder` as everything else: doing so would require encoding onion addresses into
//! `Multiaddr` values flowing through the same pipeline as every other transport — including the
//! `.with_dns()` wrapper already active (for `/dns4/...` bootstrap addresses), which transparently
//! intercepts and resolves any address in that same family through the REAL system DNS resolver
//! before a custom transport would ever see it. Encoding a `.onion` address that way would
//! reintroduce, in the very code meant to fix it, the exact class of leak the 2026-08-22 audit
//! flagged. Plain strings, dialed directly by this module, never touch a `Multiaddr` at all.
//!
//! ## Why no additional handshake on top of the raw SOCKS5 stream
//!
//! The payload sent over this channel is always an already Double-Ratchet-encrypted `NovaPacket`
//! (see `nova-engine`), and a Tor v3 `.onion` address is itself self-certifying: Tor's own
//! rendezvous protocol already proves you're talking to the holder of that address's private key
//! before a single byte of application data crosses the circuit. A second handshake on top would
//! protect against nothing this design doesn't already cover, so this module trusts the stream
//! `TorManager::connect_onion_stream` hands back and just frames raw bytes over it.
//!
//! ## Scope: outbound dial only
//!
//! This node being reachable AT its own `.onion` address requires an externally-configured Tor
//! hidden service (`ADD_ONION` via the Tor control port, or a `torrc` `HiddenServiceDir`)
//! forwarding to [`run_onion_listener`]'s bound port — genuinely out of scope here, since it needs
//! a live Tor process under external control, not just a SOCKS5 proxy to dial out through. The
//! recommended path to remove that external-process dependency entirely (letting this crate embed
//! its own Tor client instead of dialing through a system SOCKS5 proxy) is `arti-client`
//! (<https://docs.rs/arti-client>), the Tor Project's own pure-Rust implementation — a substantial
//! follow-up integration, not attempted here.

use crate::error::TransportError;
use crate::tor::TorManager;
use crate::DeliveryOutcome;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, warn};

const FRAME_TIMEOUT: Duration = Duration::from_secs(20);

/// The onion-channel listener's default port by convention — used both as the default bind port
/// (see `P2PNode::start`'s `NOVA_ONION_LISTEN_ADDR` handling) and as the port `nova-engine` dials
/// when sending to a contact's onion address, since a `PreKeyBundle` carries only the address,
/// not a port. Bound to loopback only by default (`127.0.0.1`), so a *fixed* port here is exactly
/// as safe as a random one would be — both are unreachable except through a local Tor process's
/// hidden-service forwarding — while being far more usable: two devices agree on where to dial
/// without needing to exchange a port number out of band.
pub const DEFAULT_ONION_CHANNEL_PORT: u16 = 4434;

const OUTCOME_PROCESSED: u8 = 0;
const OUTCOME_BLOCKED: u8 = 1;
const OUTCOME_REJECTED: u8 = 2;

fn outcome_to_byte(outcome: DeliveryOutcome) -> u8 {
    match outcome {
        DeliveryOutcome::Processed => OUTCOME_PROCESSED,
        DeliveryOutcome::Blocked => OUTCOME_BLOCKED,
        DeliveryOutcome::Rejected => OUTCOME_REJECTED,
    }
}

fn byte_to_outcome(b: u8) -> DeliveryOutcome {
    match b {
        OUTCOME_PROCESSED => DeliveryOutcome::Processed,
        OUTCOME_BLOCKED => DeliveryOutcome::Blocked,
        _ => DeliveryOutcome::Rejected,
    }
}

async fn write_frame<W: AsyncWrite + Unpin>(w: &mut W, payload: &[u8]) -> std::io::Result<()> {
    w.write_all(&(payload.len() as u32).to_be_bytes()).await?;
    w.write_all(payload).await?;
    w.flush().await
}

async fn read_frame<R: AsyncRead + Unpin>(r: &mut R) -> std::io::Result<Vec<u8>> {
    let mut len_buf = [0u8; 4];
    r.read_exact(&mut len_buf).await?;
    let len = u32::from_be_bytes(len_buf) as usize;
    if len > nova_protocol::MAX_PACKET_SIZE {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "onion channel frame exceeds MAX_PACKET_SIZE",
        ));
    }
    let mut buf = vec![0u8; len];
    r.read_exact(&mut buf).await?;
    Ok(buf)
}

/// Dials `onion_address:port` through the configured Tor SOCKS5 proxy, sends `payload` as one
/// length-prefixed frame, and waits for the single-byte [`DeliveryOutcome`] the remote
/// [`run_onion_listener`] reports back — the onion-channel equivalent of
/// `P2PNode::send_to_peer`'s live request/response round-trip.
pub async fn send_via_onion(
    tor_manager: &TorManager,
    onion_address: &str,
    port: u16,
    payload: Vec<u8>,
) -> Result<DeliveryOutcome, TransportError> {
    let mut stream = tor_manager.connect_onion_stream(onion_address, port).await?;

    tokio::time::timeout(FRAME_TIMEOUT, write_frame(&mut stream, &payload))
        .await
        .map_err(|_| TransportError::Timeout)??;

    let mut outcome_buf = [0u8; 1];
    tokio::time::timeout(FRAME_TIMEOUT, stream.read_exact(&mut outcome_buf))
        .await
        .map_err(|_| TransportError::Timeout)??;

    Ok(byte_to_outcome(outcome_buf[0]))
}

/// One inbound frame received over the onion listener, still awaiting an application-level
/// verdict from `nova-engine` — the onion-channel counterpart of `dht_node::IncomingMessage`.
pub struct OnionIncoming {
    pub bytes: Vec<u8>,
    respond_tx: oneshot::Sender<DeliveryOutcome>,
}

impl OnionIncoming {
    /// Reports how `nova-engine` handled this frame back to the peer that sent it, over the same
    /// TCP connection their `send_via_onion` call is still holding open and waiting on.
    pub fn respond(self, outcome: DeliveryOutcome) {
        let _ = self.respond_tx.send(outcome);
    }
}

/// Binds `bind_addr` and serves the onion-channel protocol forever — see [`run_onion_listener`]
/// for the convenience entry point. Split out the same way `nova_server::bind`/`serve_forever`
/// is, so a caller that needs the actual bound port (a test binding to port 0) can read it before
/// serving starts.
pub async fn bind(bind_addr: SocketAddr) -> std::io::Result<TcpListener> {
    TcpListener::bind(bind_addr).await
}

/// Accepts plain TCP connections on `listener` forever, each carrying exactly one length-prefixed
/// frame (see [`send_via_onion`]), and forwards them to `incoming_tx` for `nova-engine` to
/// process. `listener`'s bound address is whatever local port an externally-configured Tor hidden
/// service should forward this node's `.onion` traffic to (see this module's docs on why running
/// that hidden service itself is out of scope here).
pub async fn serve_forever(listener: TcpListener, incoming_tx: mpsc::UnboundedSender<OnionIncoming>) {
    loop {
        let (stream, peer_addr) = match listener.accept().await {
            Ok(v) => v,
            Err(e) => {
                warn!("Onion listener accept error: {e}");
                continue;
            }
        };
        let incoming_tx = incoming_tx.clone();
        tokio::spawn(handle_onion_connection(stream, peer_addr, incoming_tx));
    }
}

/// Binds `bind_addr` and serves forever — the one-call entry point for production use (see
/// [`bind`] + [`serve_forever`] for the split version tests use).
pub async fn run_onion_listener(
    bind_addr: SocketAddr,
    incoming_tx: mpsc::UnboundedSender<OnionIncoming>,
) -> std::io::Result<()> {
    let listener = bind(bind_addr).await?;
    serve_forever(listener, incoming_tx).await;
    Ok(())
}

async fn handle_onion_connection(
    mut stream: TcpStream,
    peer_addr: SocketAddr,
    incoming_tx: mpsc::UnboundedSender<OnionIncoming>,
) {
    let bytes = match tokio::time::timeout(FRAME_TIMEOUT, read_frame(&mut stream)).await {
        Ok(Ok(b)) => b,
        Ok(Err(e)) => {
            debug!("Onion listener: failed to read frame from {peer_addr}: {e}");
            return;
        }
        Err(_) => {
            debug!("Onion listener: timed out reading frame from {peer_addr}");
            return;
        }
    };

    let (respond_tx, respond_rx) = oneshot::channel();
    if incoming_tx.send(OnionIncoming { bytes, respond_tx }).is_err() {
        // Nobody is listening for incoming frames at all (network not attached) — answer
        // immediately rather than leaving the sender hanging until its own FRAME_TIMEOUT.
        let _ = write_outcome(&mut stream, DeliveryOutcome::Rejected).await;
        return;
    }

    let outcome = respond_rx.await.unwrap_or(DeliveryOutcome::Rejected);
    let _ = write_outcome(&mut stream, outcome).await;
}

async fn write_outcome(stream: &mut TcpStream, outcome: DeliveryOutcome) -> std::io::Result<()> {
    stream.write_all(&[outcome_to_byte(outcome)]).await?;
    stream.flush().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tor::TorConfig;

    /// A minimal SOCKS5 server good enough to drive `TorManager::connect_onion_stream` end to
    /// end: accepts the greeting/CONNECT handshake exactly like Tor's SOCKS5 endpoint would, then
    /// forwards the resulting stream to a real local TCP listener — standing in for "Tor connects
    /// you to the hidden service" so this test can prove the onion-channel wire protocol
    /// (framing, outcome byte, listener plumbing) end to end without a live Tor process.
    async fn spawn_mock_socks5_forwarder(forward_to: SocketAddr) -> SocketAddr {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            loop {
                let (client, _) = match listener.accept().await {
                    Ok(v) => v,
                    Err(_) => return,
                };
                tokio::spawn(handle_mock_socks5_client(client, forward_to));
            }
        });
        addr
    }

    async fn handle_mock_socks5_client(mut client: TcpStream, forward_to: SocketAddr) {
        let mut greeting = [0u8; 3];
        if client.read_exact(&mut greeting).await.is_err() {
            return;
        }
        if client.write_all(&[0x05, 0x00]).await.is_err() {
            return;
        }

        // CONNECT request: VER CMD RSV ATYP LEN domain PORT(2)
        let mut head = [0u8; 4];
        if client.read_exact(&mut head).await.is_err() {
            return;
        }
        let mut len_buf = [0u8; 1];
        if client.read_exact(&mut len_buf).await.is_err() {
            return;
        }
        let mut domain = vec![0u8; len_buf[0] as usize];
        if client.read_exact(&mut domain).await.is_err() {
            return;
        }
        let mut port_buf = [0u8; 2];
        if client.read_exact(&mut port_buf).await.is_err() {
            return;
        }

        // Success reply: VER REP RSV ATYP BND.ADDR(4) BND.PORT(2)
        let reply = [0x05, 0x00, 0x00, 0x01, 0, 0, 0, 0, 0, 0];
        if client.write_all(&reply).await.is_err() {
            return;
        }

        let Ok(mut upstream) = TcpStream::connect(forward_to).await else { return };
        let _ = tokio::io::copy_bidirectional(&mut client, &mut upstream).await;
    }

    #[tokio::test]
    async fn test_send_via_onion_full_roundtrip_through_mock_socks5() {
        let (incoming_tx, mut incoming_rx) = mpsc::unbounded_channel();
        let listener = bind("127.0.0.1:0".parse().unwrap()).await.unwrap();
        let listener_addr = listener.local_addr().unwrap();
        tokio::spawn(serve_forever(listener, incoming_tx));

        let socks_addr = spawn_mock_socks5_forwarder(listener_addr).await;
        let tor_manager = TorManager::new(TorConfig {
            enabled: true,
            mode: crate::tor::TorMode::Hybrid,
            socks_proxy: socks_addr.to_string(),
            onion_address: None,
            bridge_type: None,
        });

        let send_task = tokio::spawn(async move {
            send_via_onion(&tor_manager, "irrelevanttestaddress.onion", 4433, b"hello via onion".to_vec()).await
        });

        let incoming = tokio::time::timeout(Duration::from_secs(5), incoming_rx.recv())
            .await
            .expect("listener should receive the frame before the timeout")
            .expect("channel should not be closed");
        assert_eq!(incoming.bytes, b"hello via onion");
        incoming.respond(DeliveryOutcome::Processed);

        let outcome = tokio::time::timeout(Duration::from_secs(5), send_task)
            .await
            .expect("send_via_onion should complete before the timeout")
            .expect("send task must not panic")
            .expect("send_via_onion must succeed");
        assert_eq!(outcome, DeliveryOutcome::Processed);
    }

    #[tokio::test]
    async fn test_onion_listener_reports_no_receiver_as_rejected() {
        let (incoming_tx, incoming_rx) = mpsc::unbounded_channel();
        drop(incoming_rx); // nobody is listening — the send-side path this test exercises

        let listener = bind("127.0.0.1:0".parse().unwrap()).await.unwrap();
        let listener_addr = listener.local_addr().unwrap();
        tokio::spawn(serve_forever(listener, incoming_tx));

        let socks_addr = spawn_mock_socks5_forwarder(listener_addr).await;
        let tor_manager = TorManager::new(TorConfig {
            enabled: true,
            mode: crate::tor::TorMode::Hybrid,
            socks_proxy: socks_addr.to_string(),
            onion_address: None,
            bridge_type: None,
        });

        let outcome = send_via_onion(&tor_manager, "nobodylistening.onion", 4433, b"anyone there?".to_vec())
            .await
            .unwrap();
        assert_eq!(outcome, DeliveryOutcome::Rejected);
    }
}
