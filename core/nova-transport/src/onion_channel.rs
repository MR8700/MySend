//! A minimal, transport-level-unauthenticated point-to-point channel over Tor — the actual
//! send/receive primitive `TorStrict`/`Hybrid` mode falls back to when a peer cannot (or, in
//! `TorStrict`, must not) be reached via the DHT/QUIC path in `dht_node`.
//!
//! Both halves run entirely in-process via the embedded `arti-client` Tor implementation (see
//! `crate::tor`): [`send_via_onion`] builds a real Tor circuit and dials out through it, and
//! [`serve_onion_service_forever`] hosts this device's own onion service and accepts inbound
//! rendezvous streams — no external Tor/Orbot process, and no local TCP port forwarded by one, is
//! involved on either side.
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
//! ## Why no additional handshake on top of the raw stream
//!
//! The payload sent over this channel is always an already Double-Ratchet-encrypted `NovaPacket`
//! (see `nova-engine`), and a Tor v3 `.onion` address is itself self-certifying: Tor's own
//! rendezvous protocol already proves you're talking to the holder of that address's private key
//! before a single byte of application data crosses the circuit. A second handshake on top would
//! protect against nothing this design doesn't already cover, so this module trusts the stream
//! `TorManager` hands back (either direction) and just frames raw bytes over it.

use crate::error::TransportError;
use crate::tor::TorManager;
use crate::DeliveryOutcome;
use futures::StreamExt;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::sync::{mpsc, oneshot, RwLock};
use tor_cell::relaycell::msg::Connected;
use tor_hsservice::StreamRequest;
use tracing::{debug, warn};

const FRAME_TIMEOUT: Duration = Duration::from_secs(20);

/// The port this device's onion service listens on, and the port every contact's
/// [`send_via_onion`] dials by default — a `PreKeyBundle` carries only the `.onion` address, not
/// a port, so both sides must agree on this out of band. Purely a logical rendezvous-protocol
/// port (arti's onion service, not a real bound TCP socket) — kept as a named constant only so
/// callers never hardcode the number.
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

async fn write_outcome<W: AsyncWrite + Unpin>(w: &mut W, outcome: DeliveryOutcome) -> std::io::Result<()> {
    w.write_all(&[outcome_to_byte(outcome)]).await?;
    w.flush().await
}

/// Dials `onion_address:port` through a real, in-process Tor circuit, sends `payload` as one
/// length-prefixed frame, and waits for the single-byte [`DeliveryOutcome`] the remote
/// [`serve_onion_service_forever`] reports back — the onion-channel equivalent of
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
    /// Tor stream their `send_via_onion` call is still holding open and waiting on.
    pub fn respond(self, outcome: DeliveryOutcome) {
        let _ = self.respond_tx.send(outcome);
    }
}

/// Launches this device's own onion service (see [`TorManager::launch_own_onion_service`]) and
/// serves the onion-channel protocol over every inbound rendezvous stream forever, forwarding
/// decoded frames to `incoming_tx` — the receiving half of the Tor fallback path. Returns only if
/// launching the service itself fails (e.g. Tor bootstrap failure); once running, this loops
/// until the rendezvous-request stream ends, which in practice means "never" for a live service.
pub async fn serve_onion_service_forever(
    tor_manager: Arc<RwLock<TorManager>>,
    incoming_tx: mpsc::UnboundedSender<OnionIncoming>,
) -> Result<(), TransportError> {
    let (running_service, rend_requests) = {
        let mgr = tor_manager.read().await;
        mgr.launch_own_onion_service().await?
    };

    // `running_service` must stay alive for exactly as long as this loop runs — dropping it
    // would tear the onion service down. Keeping it bound here (rather than discarding it) does
    // that implicitly: it only drops once this function returns.
    let mut stream_requests = Box::pin(tor_hsservice::handle_rend_requests(rend_requests));
    while let Some(stream_request) = stream_requests.next().await {
        let incoming_tx = incoming_tx.clone();
        tokio::spawn(handle_onion_connection(stream_request, incoming_tx));
    }

    drop(running_service);
    Ok(())
}

async fn handle_onion_connection(stream_request: StreamRequest, incoming_tx: mpsc::UnboundedSender<OnionIncoming>) {
    let mut stream = match stream_request.accept(Connected::new_empty()).await {
        Ok(s) => s,
        Err(e) => {
            debug!("Onion listener: failed to accept inbound rendezvous stream: {e}");
            return;
        }
    };

    let bytes = match tokio::time::timeout(FRAME_TIMEOUT, read_frame(&mut stream)).await {
        Ok(Ok(b)) => b,
        Ok(Err(e)) => {
            debug!("Onion listener: failed to read frame: {e}");
            return;
        }
        Err(_) => {
            debug!("Onion listener: timed out reading frame");
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

/// Spawns [`serve_onion_service_forever`] as a background task, logging (rather than propagating)
/// a launch failure — matching every other best-effort optional path in `P2PNode::start` (a
/// failure here means this device isn't reachable via Tor until fixed, not that startup fails).
/// Delay between retry attempts after a failed bootstrap — matches roughly how long a real
/// bootstrap itself can legitimately take (see `tor::BOOTSTRAP_TIMEOUT`'s doc comment), so a
/// transient failure (temporary loss of connectivity, a flaky guard relay, switching from
/// mobile data to Wi-Fi mid-attempt) gets retried on a similar cadence rather than either
/// hammering the network or leaving the device unreachable for an arbitrarily long time.
const ONION_SERVICE_RETRY_DELAY: Duration = Duration::from_secs(30);

/// Keeps retrying [`serve_onion_service_forever`] forever on failure instead of giving up after
/// one attempt. `TorManager::client`'s `OnceCell` never caches a bootstrap error (see its doc
/// comment), so each retry here genuinely attempts a fresh bootstrap — but nothing previously
/// *drove* that retry: `ensure_onion_service_started` (this function's only caller) is itself
/// guarded to run at most once per node lifetime, so a single failed attempt — e.g. a 180s
/// bootstrap timeout on a network that took a moment to come up, or a mobile carrier that
/// blocked the first guard relay dial but not a later one — previously left the onion service
/// permanently unreachable and `get_tor_status` permanently reporting `bootstrap_percent: 0`
/// with no way to distinguish "still trying" from "gave up 40 minutes ago", which is exactly the
/// stuck-at-0%-forever symptom this fixes.
pub fn spawn_onion_service(tor_manager: Arc<RwLock<TorManager>>, incoming_tx: mpsc::UnboundedSender<OnionIncoming>) {
    tokio::spawn(async move {
        loop {
            match serve_onion_service_forever(tor_manager.clone(), incoming_tx.clone()).await {
                Ok(()) => break, // the rendezvous-request stream ended cleanly — nothing left to retry
                Err(e) => {
                    warn!("Onion service failed to launch: {e} — retrying in {}s", ONION_SERVICE_RETRY_DELAY.as_secs());
                    tokio::time::sleep(ONION_SERVICE_RETRY_DELAY).await;
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_outcome_byte_roundtrip() {
        for outcome in [DeliveryOutcome::Processed, DeliveryOutcome::Blocked, DeliveryOutcome::Rejected] {
            assert_eq!(byte_to_outcome(outcome_to_byte(outcome)), outcome);
        }
    }

    #[tokio::test]
    async fn test_frame_roundtrip_over_an_in_memory_duplex_stream() {
        let (mut a, mut b) = tokio::io::duplex(4096);
        let payload = b"hello via onion".to_vec();
        write_frame(&mut a, &payload).await.unwrap();
        let received = read_frame(&mut b).await.unwrap();
        assert_eq!(received, payload);
    }

    #[tokio::test]
    async fn test_oversized_frame_is_rejected_before_allocating() {
        let (mut a, mut b) = tokio::io::duplex(4096);
        let huge_len = (nova_protocol::MAX_PACKET_SIZE as u32) + 1;
        a.write_all(&huge_len.to_be_bytes()).await.unwrap();
        let err = read_frame(&mut b).await.unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    }
}
