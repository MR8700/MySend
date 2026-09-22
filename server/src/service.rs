use crate::relay::BlindRelay;
use crate::registry::{AdminOverview, FeedbackRecord, PresenceRegistry};
use axum::extract::connect_info::ConnectInfo;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, post};
use axum::{Json, Router};
use nova_protocol::{ServerRequest, ServerResponse};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tracing::{error, info, warn};

/// Ceiling for control-plane datagrams and relayed media blobs (up to 1 MiB).
pub const MAX_DATAGRAM_SIZE: usize = 1024 * 1024;

static NEXT_SESSION_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

#[derive(Clone)]
struct AppState {
    registry: Arc<PresenceRegistry>,
    relay: Arc<BlindRelay>,
    sessions: Arc<tokio::sync::RwLock<HashMap<String, (u64, tokio::sync::mpsc::UnboundedSender<ServerResponse>)>>>,
}

#[derive(Deserialize)]
pub struct ReportPayload {
    pub reporter_peer_id: String,
    pub target_peer_id: String,
    pub reason: String,
    pub category: Option<String>,
    pub comment: Option<String>,
}

#[derive(Serialize)]
pub struct ReportResponse {
    pub success: bool,
    pub auto_quarantined: bool,
    pub message: String,
}

#[derive(Deserialize)]
pub struct FeedbackPayload {
    pub sender_peer_id: Option<String>,
    pub rating: u8,
    pub category: Option<String>,
    pub comment: Option<String>,
}

#[derive(Serialize)]
pub struct FeedbackResponse {
    pub success: bool,
    pub record: FeedbackRecord,
}

#[derive(Deserialize)]
pub struct BanUserPayload {
    pub peer_id: String,
    pub reason: Option<String>,
}

#[derive(Deserialize)]
pub struct UnbanUserPayload {
    pub peer_id: String,
}

#[derive(Deserialize)]
pub struct SettingsPayload {
    pub auto_ban_threshold: usize,
}

#[derive(Serialize)]
pub struct SimpleStatusResponse {
    pub success: bool,
    pub message: String,
}

/// Binds a TCP listener at `bind_addr` and serves the discovery/signaling protocol forever as a
/// WebSocket upgrade over plain HTTP. Also spawns the periodic housekeeping task.
pub async fn run_server(bind_addr: &str) -> std::io::Result<()> {
    let (listener, registry, relay) = bind(bind_addr).await?;
    serve_forever(listener, registry, relay).await;
    Ok(())
}

pub async fn bind(bind_addr: &str) -> std::io::Result<(TcpListener, Arc<PresenceRegistry>, Arc<BlindRelay>)> {
    let db_url = std::env::var("DATABASE_URL")
        .or_else(|_| std::env::var("POSTGRES_URL"))
        .or_else(|_| std::env::var("POSTGRES_PRISMA_URL"))
        .or_else(|_| std::env::var("POSTGRES_URL_NON_POOLING"))
        .ok();

    let db_pool = match db_url {
        Some(url) if !url.trim().is_empty() => {
            info!("PostgreSQL connection string detected, initializing Supabase / PostgreSQL connection pool...");
            match crate::db::init_db(&url).await {
                Ok(pool) => {
                    info!("PostgreSQL / Supabase connection pool and automated schema initialized successfully.");
                    Some(pool)
                }
                Err(e) => {
                    warn!("Failed to initialize PostgreSQL / Supabase database: {e}. Falling back to in-memory mode.");
                    None
                }
            }
        }
        _ => {
            info!("No DATABASE_URL or POSTGRES_URL configured. Running in-memory mode.");
            None
        }
    };

    let registry = Arc::new(PresenceRegistry::with_pool(db_pool.clone()));
    let relay = Arc::new(BlindRelay::with_pool(db_pool));
    info!("Presence Registry & Blind Relay initialized");

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
    info!("NOVA Server listening on ws://{bound_addr} (register / lookup / relay signaling + HTTP API)");

    Ok((listener, registry, relay))
}

fn router(
    registry: Arc<PresenceRegistry>,
    relay: Arc<BlindRelay>,
    sessions: Arc<tokio::sync::RwLock<HashMap<String, (u64, tokio::sync::mpsc::UnboundedSender<ServerResponse>)>>>,
) -> Router {
    Router::new()
        .route("/", get(ws_handler))
        .route("/health", get(|| async { "ok" }))
        // Public Consumer Endpoints (Reports & App Feedback)
        .route("/report", post(handle_report))
        .route("/feedback", post(handle_feedback))
        // Admin & Moderation Endpoints (Protected by Admin Token)
        .route("/admin/overview", get(handle_admin_overview))
        .route("/admin/ban", post(handle_admin_ban))
        .route("/admin/unban", post(handle_admin_unban))
        .route("/admin/settings", post(handle_admin_settings))
        .with_state(AppState { registry, relay, sessions })
}

/// Serves the discovery/signaling protocol forever over an already-bound listener.
pub async fn serve_forever(listener: TcpListener, registry: Arc<PresenceRegistry>, relay: Arc<BlindRelay>) {
    let sessions = Arc::new(tokio::sync::RwLock::new(HashMap::new()));
    let app = router(registry, relay, sessions).into_make_service_with_connect_info::<SocketAddr>();
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
    ws.max_message_size(16 * 1024 * 1024)
        .max_frame_size(16 * 1024 * 1024)
        .on_upgrade(move |socket| handle_socket(socket, state, observed_ip))
}

fn client_ip(headers: &HeaderMap, peer_addr: SocketAddr) -> IpAddr {
    headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .map(str::trim)
        .and_then(|s| s.parse::<IpAddr>().ok())
        .unwrap_or_else(|| peer_addr.ip())
}

fn check_admin_auth(headers: &HeaderMap, params: Option<&HashMap<String, String>>) -> bool {
    let expected = std::env::var("NOVA_ADMIN_SECRET").unwrap_or_else(|_| "nova_admin_master_secret".to_string());
    if let Some(h) = headers.get("x-admin-token").and_then(|v| v.to_str().ok()) {
        if h.trim() == expected.trim() {
            return true;
        }
    }
    if let Some(p) = params.and_then(|m| m.get("admin_token")) {
        if p.trim() == expected.trim() {
            return true;
        }
    }
    false
}

// -------------------------------------------------------------
// HTTP REST Handlers for Moderation, Reporting and Feedback
// -------------------------------------------------------------

async fn handle_report(
    State(state): State<AppState>,
    Json(payload): Json<ReportPayload>,
) -> Result<Json<ReportResponse>, (StatusCode, String)> {
    let cat = payload.category.unwrap_or_else(|| "other".into());
    let cmt = payload.comment.unwrap_or_default();

    match state.registry.submit_report(
        payload.reporter_peer_id,
        payload.target_peer_id,
        payload.reason,
        cat,
        cmt,
    ).await {
        Ok(auto_quarantined) => Ok(Json(ReportResponse {
            success: true,
            auto_quarantined,
            message: if auto_quarantined {
                "Signalement enregistré. Le compte a été placé en quarantaine automatique.".into()
            } else {
                "Signalement enregistré avec succès. Merci de contribuer à la sécurité.".into()
            },
        })),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}

async fn handle_feedback(
    State(state): State<AppState>,
    Json(payload): Json<FeedbackPayload>,
) -> Result<Json<FeedbackResponse>, (StatusCode, String)> {
    let cat = payload.category.unwrap_or_else(|| "general".into());
    let cmt = payload.comment.unwrap_or_default();

    match state.registry.submit_feedback(payload.sender_peer_id, payload.rating, cat, cmt).await {
        Ok(record) => Ok(Json(FeedbackResponse {
            success: true,
            record,
        })),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}

async fn handle_admin_overview(
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
    State(state): State<AppState>,
) -> Result<Json<AdminOverview>, (StatusCode, String)> {
    if !check_admin_auth(&headers, Some(&params)) {
        return Err((StatusCode::UNAUTHORIZED, "Accès administrateur refusé : jeton invalide.".into()));
    }
    let overview = state.registry.get_admin_overview().await;
    Ok(Json(overview))
}

async fn handle_admin_ban(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<BanUserPayload>,
) -> Result<Json<SimpleStatusResponse>, (StatusCode, String)> {
    if !check_admin_auth(&headers, None) {
        return Err((StatusCode::UNAUTHORIZED, "Accès administrateur refusé : jeton invalide.".into()));
    }
    let reason = payload.reason.unwrap_or_else(|| "Bannissement administratif".into());
    match state.registry.ban_user(payload.peer_id, reason).await {
        Ok(()) => Ok(Json(SimpleStatusResponse {
            success: true,
            message: "Utilisateur banni avec succès.".into(),
        })),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn handle_admin_unban(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<UnbanUserPayload>,
) -> Result<Json<SimpleStatusResponse>, (StatusCode, String)> {
    if !check_admin_auth(&headers, None) {
        return Err((StatusCode::UNAUTHORIZED, "Accès administrateur refusé : jeton invalide.".into()));
    }
    match state.registry.unban_user(&payload.peer_id).await {
        Ok(unbanned) => Ok(Json(SimpleStatusResponse {
            success: true,
            message: if unbanned {
                "Utilisateur débanni avec succès.".into()
            } else {
                "Cet utilisateur n'était pas banni.".into()
            },
        })),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn handle_admin_settings(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<SettingsPayload>,
) -> Result<Json<SimpleStatusResponse>, (StatusCode, String)> {
    if !check_admin_auth(&headers, None) {
        return Err((StatusCode::UNAUTHORIZED, "Accès administrateur refusé : jeton invalide.".into()));
    }
    state.registry.set_auto_ban_threshold(payload.auto_ban_threshold).await;
    Ok(Json(SimpleStatusResponse {
        success: true,
        message: format!("Seuil de bannissement automatique mis à jour à {}.", payload.auto_ban_threshold),
    }))
}

// -------------------------------------------------------------
// WebSocket Signaling & Request Handling
// -------------------------------------------------------------

async fn handle_socket(socket: WebSocket, state: AppState, observed_ip: IpAddr) {
    use futures_util::{SinkExt, StreamExt};
    use std::sync::atomic::Ordering;

    let session_id = NEXT_SESSION_ID.fetch_add(1, Ordering::Relaxed);
    let observed_addr = SocketAddr::new(observed_ip, 0);
    let (mut ws_sender, mut ws_receiver) = socket.split();
    let (push_tx, mut push_rx) = tokio::sync::mpsc::unbounded_channel::<ServerResponse>();

    let mut authenticated_peer_id: Option<String> = None;

    loop {
        tokio::select! {
            Some(push_msg) = push_rx.recv() => {
                if let Ok(encoded) = push_msg.to_bytes() {
                    if ws_sender.send(Message::Binary(encoded)).await.is_err() {
                        break;
                    }
                }
            }
            msg = ws_receiver.next() => {
                let msg = match msg {
                    Some(Ok(m)) => m,
                    Some(Err(e)) => {
                        warn!("WebSocket recv error from {observed_ip}: {e}");
                        break;
                    }
                    None => break,
                };

                match msg {
                    Message::Binary(bytes) => {
                        if bytes.len() > MAX_DATAGRAM_SIZE {
                            let _ = ws_sender
                                .send(Message::Binary(
                                    ServerResponse::Error(format!("request exceeds {MAX_DATAGRAM_SIZE}-byte limit"))
                                        .to_bytes()
                                        .unwrap_or_default(),
                                 ))
                                .await;
                            continue;
                        }

                        let (response, register_peer) = handle_request_with_state(
                            &state,
                            &bytes,
                            observed_addr,
                        ).await;

                        if let Some(peer_id) = register_peer {
                            authenticated_peer_id = Some(peer_id.clone());
                            state.sessions.write().await.insert(peer_id.clone(), (session_id, push_tx.clone()));

                            // Instantly drain any pending packets in BlindRelay for this peer!
                            if state.relay.pending_count(&peer_id).await > 0 {
                                let pending = state.relay.drain_for_peer(&peer_id).await;
                                if !pending.is_empty() {
                                    let _ = push_tx.send(ServerResponse::RelayDrained(pending));
                                }
                            }
                        }

                        let encoded = match response.to_bytes() {
                            Ok(b) => b,
                            Err(e) => {
                                warn!("Failed to encode response for {observed_ip}: {e}");
                                continue;
                            }
                        };
                        if ws_sender.send(Message::Binary(encoded)).await.is_err() {
                            break;
                        }
                    }
                    Message::Ping(payload) => {
                        if ws_sender.send(Message::Pong(payload)).await.is_err() {
                            break;
                        }
                    }
                    Message::Close(_) => break,
                    Message::Pong(_) | Message::Text(_) => continue,
                }
            }
        }
    }

    if let Some(peer_id) = authenticated_peer_id {
        let mut sessions = state.sessions.write().await;
        if let Some((curr_id, _)) = sessions.get(&peer_id) {
            if *curr_id == session_id {
                sessions.remove(&peer_id);
            }
        }
    }
}

async fn handle_request_with_state(
    state: &AppState,
    bytes: &[u8],
    observed_addr: SocketAddr,
) -> (ServerResponse, Option<String>) {
    let request = match ServerRequest::from_bytes(bytes) {
        Ok(r) => r,
        Err(e) => return (ServerResponse::Error(format!("malformed request: {e}")), None),
    };

    match request {
        ServerRequest::Register(registration) => {
            let peer_id = registration.endpoint.peer_id.clone();
            match state.registry.register(registration, observed_addr).await {
                Ok(()) => (ServerResponse::Registered, Some(peer_id)),
                Err(e) => (ServerResponse::Error(e.to_string()), None),
            }
        }
        ServerRequest::Lookup { peer_id } => {
            (ServerResponse::LookupResult(state.registry.get_peer(&peer_id).await), None)
        }
        ServerRequest::RelayForward {
            target_peer_id,
            payload,
        } => {
            if state.registry.is_banned(&target_peer_id).await {
                return (ServerResponse::Error("L'utilisateur cible est suspendu.".into()), None);
            }
            match state.relay.forward_opaque(&target_peer_id, payload).await {
                Ok(()) => {
                    // Real-Time Direct Push: if recipient is currently connected via WebSocket, deliver immediately!
                    let sessions = state.sessions.read().await;
                    if let Some((_, target_tx)) = sessions.get(&target_peer_id) {
                        let pending = state.relay.drain_for_peer(&target_peer_id).await;
                        if !pending.is_empty() {
                            let _ = target_tx.send(ServerResponse::RelayDrained(pending));
                        }
                    }
                    (ServerResponse::RelayForwarded, None)
                }
                Err(e) => (ServerResponse::Error(e.to_string()), None),
            }
        }
        ServerRequest::RelayDrain(drain_request) => match drain_request.verify() {
            Ok(pubkey) => {
                let peer_id = hex::encode(pubkey);
                if state.registry.is_banned(&peer_id).await {
                    return (ServerResponse::Error("Votre compte est suspendu.".into()), None);
                }
                (ServerResponse::RelayDrained(state.relay.drain_for_peer(&peer_id).await), None)
            }
            Err(e) => (ServerResponse::Error(e.to_string()), None),
        },
        ServerRequest::RegisterDirectory(entry) => match state.registry.register_directory(entry).await {
            Ok(()) => (ServerResponse::DirectoryRegistered, None),
            Err(e) => (ServerResponse::Error(e.to_string()), None),
        },
        ServerRequest::SearchDirectory { query } => {
            let results = state.registry.search_directory(&query).await;
            (ServerResponse::DirectorySearchResults(results), None)
        }
    }
}

pub async fn handle_request(
    registry: &PresenceRegistry,
    relay: &BlindRelay,
    bytes: &[u8],
    observed_addr: SocketAddr,
) -> ServerResponse {
    let dummy_sessions = Arc::new(tokio::sync::RwLock::new(HashMap::new()));
    let state = AppState {
        registry: Arc::new(registry.clone()),
        relay: Arc::new(relay.clone()),
        sessions: dummy_sessions,
    };
    let (resp, _) = handle_request_with_state(&state, bytes, observed_addr).await;
    resp
}
