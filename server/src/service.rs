use crate::relay::BlindRelay;
use crate::registry::PresenceRegistry;
use nova_protocol::{ServerRequest, ServerResponse};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::UdpSocket;
use tracing::{error, info, warn};

/// UDP datagrams are capped well below the 65507-byte practical UDP ceiling: this endpoint is
/// signaling/control-plane traffic (presence registration, lookups, small relayed blobs), not a
/// bulk transfer channel — large payloads belong in chunked messages over the future QUIC
/// transport, not a single unfragmented datagram here.
pub const MAX_DATAGRAM_SIZE: usize = 16 * 1024;

/// Binds a UDP socket at `bind_addr` and serves the discovery/signaling protocol forever
/// (register / lookup / relay). Also spawns the periodic housekeeping task that expires stale
/// presence entries and relay queues. Returns only if the initial bind fails.
pub async fn run_server(bind_addr: &str) -> std::io::Result<()> {
    let registry = Arc::new(PresenceRegistry::new());
    let relay = Arc::new(BlindRelay::new());
    info!("Presence Registry & Blind Relay initialized (In-Memory / Zero-Persistent-Storage)");

    let reg_clone = registry.clone();
    let relay_clone = relay.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            reg_clone.cleanup_expired().await;
            relay_clone.purge_expired().await;
        }
    });

    let socket = UdpSocket::bind(bind_addr).await.map_err(|e| {
        error!("Failed to bind UDP socket on {bind_addr}: {e}");
        e
    })?;
    info!("NOVA Server listening on udp://{bind_addr} (register / lookup / relay signaling)");

    serve_forever(socket, registry, relay).await;
    Ok(())
}

async fn serve_forever(socket: UdpSocket, registry: Arc<PresenceRegistry>, relay: Arc<BlindRelay>) {
    let mut buf = vec![0u8; MAX_DATAGRAM_SIZE];
    loop {
        let (len, src) = match socket.recv_from(&mut buf).await {
            Ok(v) => v,
            Err(e) => {
                warn!("UDP recv error: {e}");
                continue;
            }
        };

        let response = handle_datagram(&registry, &relay, &buf[..len], src).await;
        match response.to_bytes() {
            Ok(bytes) => {
                if let Err(e) = socket.send_to(&bytes, src).await {
                    warn!("UDP send error to {src}: {e}");
                }
            }
            Err(e) => warn!("Failed to encode response for {src}: {e}"),
        }
    }
}

/// Pure request handler (no I/O beyond the registry/relay state) — the piece worth unit- and
/// integration-testing directly, independent of a real socket. `src` is the UDP source address
/// the datagram actually arrived from, used to correct (STUN-style) the reflexive public
/// address on `Register` requests — see [`PresenceRegistry::register`].
pub async fn handle_datagram(
    registry: &PresenceRegistry,
    relay: &BlindRelay,
    bytes: &[u8],
    src: SocketAddr,
) -> ServerResponse {
    let request = match ServerRequest::from_bytes(bytes) {
        Ok(r) => r,
        Err(e) => return ServerResponse::Error(format!("malformed request: {e}")),
    };

    match request {
        ServerRequest::Register(registration) => match registry.register(registration, src).await {
            Ok(()) => ServerResponse::Registered,
            Err(e) => ServerResponse::Error(e.to_string()),
        },
        ServerRequest::Lookup { peer_id } => ServerResponse::LookupResult(registry.get_peer(&peer_id).await),
        ServerRequest::RelayForward {
            target_peer_id,
            payload,
        } => match relay.forward_opaque(&target_peer_id, payload).await {
            Ok(()) => ServerResponse::RelayForwarded,
            Err(e) => ServerResponse::Error(e.to_string()),
        },
        ServerRequest::RelayDrain(drain_request) => match drain_request.verify() {
            Ok(pubkey) => {
                let peer_id = hex::encode(pubkey);
                ServerResponse::RelayDrained(relay.drain_for_peer(&peer_id).await)
            }
            Err(e) => ServerResponse::Error(e.to_string()),
        },
    }
}
