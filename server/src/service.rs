use crate::relay::BlindRelay;
use crate::registry::PresenceRegistry;
use axum::extract::connect_info::ConnectInfo;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::http::HeaderMap;
use axum::routing::get;
use axum::Router;
use nova_protocol::{ServerRequest, ServerResponse};
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tracing::{error, info, warn};

/// Ceiling for control-plane datagrams (presence registration, lookups, directory profiles with avatars, small relayed blobs).
pub const MAX_DATAGRAM_SIZE: usize = 64 * 1024;

#[derive(Clone)]
struct AppState {
    registry: Arc<PresenceRegistry>,
    relay: Arc<BlindRelay>,
}

/// Binds a TCP listener at `bind_addr` and serves the discovery/signaling protocol forever as a
/// WebSocket upgrade over plain HTTP (see this module's doc comment on why: no raw UDP ingress on
/// a host like Render). Also spawns the periodic housekeeping task that expires stale presence
/// entries and relay queues. Returns only if the initial bind fails.
pub async fn run_server(bind_addr: &str) -> std::io::Result<()> {
    let (listener, registry, relay) = bind(bind_addr).await?;
    serve_forever(listener, registry, relay).await;
    Ok(())
}

/// Binds the TCP listener and starts the registry/relay housekeeping task without serving yet —
/// split out from [`run_server`] so a caller that needs the actual bound address (e.g. a test
/// binding to port 0, or `nova-transport`'s fallback client wiring up a same-process server for
/// its own integration tests) can read `listener.local_addr()` before handing it off to
/// [`serve_forever`].
pub async fn bind(bind_addr: &str) -> std::io::Result<(TcpListener, Arc<PresenceRegistry>, Arc<BlindRelay>)> {
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

    let listener = TcpListener::bind(bind_addr).await.map_err(|e| {
        error!("Failed to bind TCP socket on {bind_addr}: {e}");
        e
    })?;
    let bound_addr = listener.local_addr().map(|a| a.to_string()).unwrap_or_else(|_| bind_addr.to_string());
    info!("NOVA Server listening on ws://{bound_addr} (register / lookup / relay signaling)");

    Ok((listener, registry, relay))
}

fn router(registry: Arc<PresenceRegistry>, relay: Arc<BlindRelay>) -> Router {
    Router::new()
        .route("/", get(ws_handler))
        .route("/health", get(|| async { "ok" }))
        .with_state(AppState { registry, relay })
}

/// Serves the discovery/signaling protocol forever over an already-bound listener (see [`bind`]).
pub async fn serve_forever(listener: TcpListener, registry: Arc<PresenceRegistry>, relay: Arc<BlindRelay>) {
    let app = router(registry, relay).into_make_service_with_connect_info::<SocketAddr>();
    if let Err(e) = axum::serve(listener, app).await {
        error!("WebSocket server exited: {e}");
    }
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    ConnectInfo(peer_addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    State(state): State<AppState>,
) -> axum::response::Response {
    let observed_ip = client_ip(&headers, peer_addr);
    ws.on_upgrade(move |socket| handle_socket(socket, state, observed_ip))
}

/// The IP a `Register` request's reflexive-address correction should trust (see
/// `PresenceRegistry::register`'s doc comment). Render — like effectively every PaaS host —
/// terminates the client's real TCP/TLS connection at its own edge proxy and forwards to this
/// process over an internal network, so `peer_addr` as seen here is the *proxy's* address, not
/// the real client's, once deployed; the proxy instead reports the original client in
/// `X-Forwarded-For`. Trusting that header is safe specifically *because* this process is only
/// ever reachable through Render's proxy in production (never directly from the internet) — the
/// same assumption every app behind a PaaS load balancer already makes. Locally (no proxy in
/// front, e.g. running the server directly for a test or a LAN deployment) there is no such
/// header and this falls back to the real socket peer address exactly as the old UDP path did.
fn client_ip(headers: &HeaderMap, peer_addr: SocketAddr) -> IpAddr {
    headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .map(str::trim)
        .and_then(|s| s.parse::<IpAddr>().ok())
        .unwrap_or_else(|| peer_addr.ip())
}

async fn handle_socket(mut socket: WebSocket, state: AppState, observed_ip: IpAddr) {
    // Only `.ip()` of this address is ever read (see `PresenceRegistry::register`) — the port is
    // irrelevant for the reflexive-address correction, so a dummy `0` here is exactly as
    // meaningful as the never-preserved-through-a-proxy real source port would have been anyway.
    let observed_addr = SocketAddr::new(observed_ip, 0);

    while let Some(msg) = socket.recv().await {
        let msg = match msg {
            Ok(m) => m,
            Err(e) => {
                warn!("WebSocket recv error from {observed_ip}: {e}");
                break;
            }
        };

        let bytes = match msg {
            Message::Binary(b) => b,
            Message::Close(_) => break,
            Message::Ping(_) | Message::Pong(_) | Message::Text(_) => continue,
        };

        if bytes.len() > MAX_DATAGRAM_SIZE {
            let _ = socket
                .send(Message::Binary(
                    ServerResponse::Error(format!("request exceeds {MAX_DATAGRAM_SIZE}-byte limit"))
                        .to_bytes()
                        .unwrap_or_default(),
                ))
                .await;
            continue;
        }

        let response = handle_request(&state.registry, &state.relay, &bytes, observed_addr).await;
        let encoded = match response.to_bytes() {
            Ok(b) => b,
            Err(e) => {
                warn!("Failed to encode response for {observed_ip}: {e}");
                continue;
            }
        };
        if socket.send(Message::Binary(encoded)).await.is_err() {
            break;
        }
    }
}

/// Pure request handler (no I/O beyond the registry/relay state) — the piece worth unit- and
/// integration-testing directly, independent of the real WebSocket transport. `observed_addr` is
/// the caller's real address as this process trusts it (see [`client_ip`]), used to correct
/// (STUN-style) the reflexive public address on `Register` requests — see
/// [`PresenceRegistry::register`].
pub async fn handle_request(
    registry: &PresenceRegistry,
    relay: &BlindRelay,
    bytes: &[u8],
    observed_addr: SocketAddr,
) -> ServerResponse {
    let request = match ServerRequest::from_bytes(bytes) {
        Ok(r) => r,
        Err(e) => return ServerResponse::Error(format!("malformed request: {e}")),
    };

    match request {
        ServerRequest::Register(registration) => match registry.register(registration, observed_addr).await {
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
        ServerRequest::RegisterDirectory(entry) => match registry.register_directory(entry).await {
            Ok(()) => ServerResponse::DirectoryRegistered,
            Err(e) => ServerResponse::Error(e.to_string()),
        },
        ServerRequest::SearchDirectory { query } => {
            let results = registry.search_directory(&query).await;
            ServerResponse::DirectorySearchResults(results)
        }
    }
}
