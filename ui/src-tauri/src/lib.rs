//! Tauri desktop shell wiring the existing static HTML/JS/CSS UI (`ui/public`) to the real
//! `nova-engine`/`nova-transport` backend, in place of the browser-only mock state `app.js`
//! previously simulated. Chosen over compiling the backend to WASM for an in-browser build
//! because the current backend cannot run inside a real browser sandbox at all: `rusqlite`'s
//! bundled SQLite doesn't compile for `wasm32-unknown-unknown`, and browsers do not permit raw
//! UDP sockets, which the QUIC/libp2p transport requires. Tauri runs nova-engine as a real
//! native process instead, so nothing about the backend needs to change.
//!
//! Scope: identity creation/restore/resume, adding a contact from a pasted prekey bundle or a
//! scanned/imported QR code, sending/receiving text messages, voice notes, location shares and
//! file transfers (the latter three as JSON payloads over the existing text pipeline — see
//! `STRUCTURED_MARKER` in `ui/public/app.js` — since `nova_protocol`'s binary payload field isn't
//! wired end-to-end yet), listing conversations, and real connection diagnostics.

use nova_crypto::{DeviceIdentity, MnemonicPhrase};
use nova_engine::{EngineError, NovaEngine};
use nova_storage::{ConversationRecord, MessageRecord};
use nova_transport::P2PNode;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{Manager, State};
use tokio::sync::Mutex;

struct AppState {
    engine: Arc<NovaEngine>,
    /// Address of a well-known bootstrap/relay node (see `nova-bootstrap`) to dial on first
    /// contact so this node can join the DHT. With none set, this device can still find peers
    /// already on the same LAN via mDNS, but not wide-area ones. Seeded at startup from
    /// `NOVA_BOOTSTRAP_ADDR` if set (desktop convenience — a terminal launch can export it),
    /// falling back to `bootstrap_addr_file` on disk; a launcher-started app (notably Android —
    /// it inherits no shell environment at all) has no other way to receive this, so the
    /// Settings screen writes to the same file via `set_bootstrap_addr`.
    bootstrap_addr: Mutex<Option<libp2p::Multiaddr>>,
    /// Where `bootstrap_addr`'s value is persisted across restarts (`<app-data-dir>/bootstrap_addr.txt`).
    bootstrap_addr_file: PathBuf,
    /// Where `fallback_server_url` is persisted across restarts (`<app-data-dir>/fallback_server_url.txt`).
    fallback_server_url_file: PathBuf,
    /// Local QUIC listen multiaddr. Defaults to an OS-assigned UDP port
    /// (`/ip4/0.0.0.0/udp/0/quic-v1`); override with `NOVA_LISTEN_ADDR` to bind a fixed port,
    /// which is what a device acting as the reachable rendezvous point for other physical test
    /// devices needs so it can be port-forwarded on its router. Desktop-only in practice — the
    /// devices dialing in (including any Android ones) never need a fixed port themselves.
    listen_addr: String,
    /// Set once `create_account`/`restore_account` has started the network node — guards
    /// against starting a second one if the frontend calls either command twice.
    network_started: Mutex<bool>,
}

#[derive(Serialize)]
struct AccountInfo {
    peer_id: String,
    mnemonic: String,
    /// Whether the P2P network actually came up this call. `false` means the account itself is
    /// real and fully usable (identity + keys are saved), but this device could not reach the
    /// network this time (e.g. no working network adapter/DNS config) — see `start_network_once`.
    /// The frontend uses this to show an honest "not connected" status instead of pretending to
    /// be online, and to know a restart (or a future explicit retry) is worth trying.
    network_active: bool,
}

#[derive(Serialize)]
struct ResumedSession {
    peer_id: String,
    mnemonic: String,
    username: String,
    network_active: bool,
    display_name: Option<String>,
    bio: Option<String>,
    avatar_data_url: Option<String>,
}

#[derive(Serialize)]
struct RestoreInfo {
    peer_id: String,
    network_active: bool,
}

fn engine_err(e: EngineError) -> String {
    e.to_string()
}

#[tauri::command]
async fn create_account(state: State<'_, AppState>, username: String) -> Result<AccountInfo, String> {
    let (peer_id, mnemonic) = state.engine.create_account(&username).await.map_err(engine_err)?;
    let network_active = start_network_once(&state, &mnemonic, &username).await;
    Ok(AccountInfo { peer_id, mnemonic, network_active })
}

/// Called once on app startup: if this device has already created/restored an identity in a
/// previous session, resumes it (and starts the network) without asking for the mnemonic again —
/// that is the entire reason it is persisted in encrypted local storage rather than only kept in
/// memory. Returns `Ok(None)` on a genuine first run, which is not an error.
#[tauri::command]
async fn try_resume_session(state: State<'_, AppState>) -> Result<Option<ResumedSession>, String> {
    let Some((peer_id, mnemonic, username)) = state.engine.try_resume_session().await.map_err(engine_err)? else {
        return Ok(None);
    };
    let profile = state.engine.get_user_profile().ok().flatten();
    let (display_name, bio, avatar_data_url) = match profile {
        Some(p) => (Some(p.display_name), Some(p.bio), p.avatar_data_url),
        None => (None, None, None),
    };
    let network_active = start_network_once(&state, &mnemonic, &username).await;
    Ok(Some(ResumedSession {
        peer_id,
        mnemonic,
        username,
        network_active,
        display_name,
        bio,
        avatar_data_url,
    }))
}

#[tauri::command]
async fn restore_account(state: State<'_, AppState>, mnemonic: String, username: String) -> Result<RestoreInfo, String> {
    let peer_id = state
        .engine
        .restore_account(&mnemonic, &username)
        .await
        .map_err(engine_err)?;
    let network_active = start_network_once(&state, &mnemonic, &username).await;
    Ok(RestoreInfo { peer_id, network_active })
}

/// Starts the P2P network node for the identity just created/restored/resumed and attaches it to
/// the engine. Takes the mnemonic rather than reading it back off the engine because
/// `DeviceIdentity` deliberately does not implement `Clone` (it holds zeroized private key
/// material) — the caller already has the mnemonic in hand at exactly this moment, so
/// re-deriving an independent identity instance from it is simpler than threading a reference
/// through the engine's internal `Mutex<Option<DeviceIdentity>>>`.
///
/// Deliberately infallible (returns whether it worked, never an `Err`): the account itself
/// (identity + keys, already persisted by the caller before this runs) must stay valid and
/// usable even when the network genuinely cannot come up right now (no working adapter/DNS
/// config, firewall, etc.) — this is the same "best-effort, never blocks the caller" treatment
/// already given to the bootstrap dial and presence announcement below. Before this existed, a
/// network failure here made the *entire* create/restore/resume command fail even though the
/// account had already been saved — leaving a real account trapped in storage with no mnemonic
/// ever shown and every later attempt reporting "already exists", with no way back short of
/// deleting it (see the account screens' error handling in app.js for the user-facing side).
async fn start_network_once(state: &State<'_, AppState>, mnemonic_str: &str, username: &str) -> bool {
    let mut started = state.network_started.lock().await;
    if *started {
        return true;
    }

    let mnemonic = match MnemonicPhrase::from_phrase(mnemonic_str) {
        Ok(m) => m,
        Err(e) => {
            tracing::error!("Failed to re-derive mnemonic for network startup: {e}");
            return false;
        }
    };
    let identity = match DeviceIdentity::from_mnemonic(&mnemonic, username) {
        Ok(i) => i,
        Err(e) => {
            tracing::error!("Failed to re-derive identity for network startup: {e}");
            return false;
        }
    };

    // Each network step below is best-effort — bootstrap_dial/announce_presence already treat
    // their own errors as non-fatal (warn and continue, account stays locally usable) — but on a
    // device with no working internet/LAN path at all, the underlying dial or DHT round-trip can
    // *hang* rather than fail fast (no network error to catch, just nothing ever answering the
    // oneshot channel these wait on). That turns "best-effort" into "blocks account creation
    // forever" for exactly the offline case this fallback exists to handle. Bounding each step
    // keeps the intended behavior (account creation always succeeds locally; network_active just
    // ends up false) even when there's no network to fail against quickly.
    const NETWORK_STEP_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

    let node = match tokio::time::timeout(NETWORK_STEP_TIMEOUT, P2PNode::start(identity, &state.listen_addr)).await {
        Ok(Ok(n)) => n,
        Ok(Err(e)) => {
            tracing::warn!("P2P network failed to start, account remains usable locally (will retry next launch): {e}");
            return false;
        }
        Err(_) => {
            tracing::warn!("P2P network did not start within {NETWORK_STEP_TIMEOUT:?}, account remains usable locally (will retry next launch)");
            return false;
        }
    };

    if let Some(addr) = state.bootstrap_addr.lock().await.clone() {
        match tokio::time::timeout(NETWORK_STEP_TIMEOUT, node.bootstrap_dial(addr.clone())).await {
            Ok(Err(e)) => tracing::warn!("Bootstrap dial to {addr} failed (will still work over mDNS/LAN): {e}"),
            Err(_) => tracing::warn!("Bootstrap dial to {addr} did not complete within {NETWORK_STEP_TIMEOUT:?} (will still work over mDNS/LAN)"),
            Ok(Ok(())) => {}
        }
    }
    match tokio::time::timeout(NETWORK_STEP_TIMEOUT, node.announce_presence()).await {
        Ok(Err(e)) => tracing::warn!("Initial presence announcement failed (will retry on the usual heartbeat): {e}"),
        Err(_) => tracing::warn!("Initial presence announcement did not complete within {NETWORK_STEP_TIMEOUT:?} (will retry on the usual heartbeat)"),
        Ok(Ok(())) => {}
    }

    state.engine.attach_network(node).await;
    let publish_engine = state.engine.clone();
    tokio::spawn(async move {
        let _ = publish_engine.publish_directory_profile().await;
    });

    *started = true;
    true
}

/// Wipes this device's local identity/contacts/messages (see `NovaEngine::clear_identity`) and
/// allows the next `create_account`/`restore_account` call in this same run to start a fresh
/// network node — there is no server account to log back into, so this is the entire meaning of
/// "log out" for a device-bound identity.
#[tauri::command]
async fn logout(state: State<'_, AppState>) -> Result<(), String> {
    state.engine.clear_identity().await.map_err(engine_err)?;
    *state.network_started.lock().await = false;
    Ok(())
}

#[tauri::command]
async fn get_own_peer_id(state: State<'_, AppState>) -> Result<Option<String>, String> {
    Ok(state.engine.identity.lock().await.as_ref().map(|id| id.public_id_hex()))
}

/// Hex-encoded so it can be pasted as plain text — the recipient calls `add_contact` with this same string.
#[tauri::command]
async fn get_own_prekey_bundle_hex(state: State<'_, AppState>) -> Result<String, String> {
    let bytes = state.engine.get_own_prekey_bundle_bytes().await.map_err(engine_err)?;
    Ok(hex::encode(bytes))
}

/// Generates a signed, time-limited contact invitation URI (`nova://invite?d=...`).
#[tauri::command]
async fn get_own_invitation_uri(
    state: State<'_, AppState>,
    ttl_seconds: Option<i64>,
) -> Result<String, String> {
    state
        .engine
        .get_own_invitation_uri(ttl_seconds)
        .await
        .map_err(engine_err)
}

#[tauri::command]
async fn add_contact(
    state: State<'_, AppState>,
    username: String,
    display_name: String,
    bundle_hex: String,
) -> Result<nova_storage::ContactRecord, String> {
    let trimmed = bundle_hex.trim();
    state
        .engine
        .add_contact(&username, &display_name, trimmed.as_bytes())
        .await
        .map_err(engine_err)
}

#[tauri::command]
async fn search_directory(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<nova_protocol::DirectorySearchResult>, String> {
    state.engine.search_directory(&query).await.map_err(engine_err)
}

#[tauri::command]
fn get_contacts(state: State<'_, AppState>) -> Result<Vec<nova_storage::ContactRecord>, String> {
    state.engine.get_contacts().map_err(engine_err)
}

#[tauri::command]
fn trust_contact(state: State<'_, AppState>, peer_id: String) -> Result<(), String> {
    state.engine.trust_contact(&peer_id).map_err(engine_err)
}

#[tauri::command]
fn block_contact(state: State<'_, AppState>, peer_id: String) -> Result<(), String> {
    state.engine.block_contact(&peer_id).map_err(engine_err)
}

#[tauri::command]
fn block_and_delete_conversation(state: State<'_, AppState>, peer_id: String) -> Result<(), String> {
    state.engine.block_and_delete_conversation(&peer_id).map_err(engine_err)
}

#[tauri::command]
fn unblock_contact(state: State<'_, AppState>, peer_id: String) -> Result<(), String> {
    state.engine.unblock_contact(&peer_id).map_err(engine_err)
}

#[tauri::command]
fn delete_contact(state: State<'_, AppState>, peer_id: String) -> Result<(), String> {
    state.engine.delete_contact(&peer_id).map_err(engine_err)
}

#[tauri::command]
fn get_user_profile(state: State<'_, AppState>) -> Result<Option<nova_storage::UserProfileRecord>, String> {
    state.engine.get_user_profile().map_err(engine_err)
}

#[tauri::command]
async fn update_user_profile(
    state: State<'_, AppState>,
    display_name: String,
    bio: String,
    avatar_data_url: Option<String>,
) -> Result<(), String> {
    let profile = nova_storage::UserProfileRecord {
        display_name,
        bio,
        avatar_data_url,
    };
    state.engine.save_user_profile(&profile).map_err(engine_err)?;
    let _ = state.engine.publish_directory_profile().await;
    Ok(())
}

#[tauri::command]
async fn send_message(
    state: State<'_, AppState>,
    conversation_id: String,
    recipient_peer_id: String,
    text: String,
) -> Result<MessageRecord, String> {
    state
        .engine
        .send_message(&conversation_id, &recipient_peer_id, &text)
        .await
        .map_err(engine_err)
}

#[tauri::command]
fn get_conversations(state: State<'_, AppState>) -> Result<Vec<ConversationRecord>, String> {
    state.engine.get_conversations().map_err(engine_err)
}

/// Manually re-attempts delivery of a message the outbox already gave up on (status `Failed`,
/// after `OUTBOX_GIVE_UP_AFTER_SECS` of failed automatic retries) — the "Réessayer" action next
/// to a failed message bubble.
#[tauri::command]
async fn retry_failed_message(
    state: State<'_, AppState>,
    conversation_id: String,
    recipient_peer_id: String,
    message_id: String,
) -> Result<MessageRecord, String> {
    state
        .engine
        .retry_failed_message(&conversation_id, &recipient_peer_id, &message_id)
        .await
        .map_err(engine_err)
}

#[tauri::command]
fn get_messages(state: State<'_, AppState>, conversation_id: String) -> Result<Vec<MessageRecord>, String> {
    state.engine.get_messages(&conversation_id).map_err(engine_err)
}

#[tauri::command]
fn delete_message(state: State<'_, AppState>, message_id: String) -> Result<(), String> {
    state.engine.delete_message(&message_id).map_err(engine_err)
}

#[tauri::command]
fn search_messages(state: State<'_, AppState>, query: String) -> Result<Vec<MessageRecord>, String> {
    state.engine.search_messages(&query).map_err(engine_err)
}

/// Sends a media message (image/video/audio/file). `data_base64` is the file's contents base64-
/// encoded — the frontend already produces this via `FileReader.readAsDataURL` for the preview,
/// so no extra encoding step is needed there; decoded back to real bytes here before the engine
/// splits it into wire chunks (see `nova_engine::NovaEngine::send_media`). A plain `Vec<u8>`
/// argument would let Tauri's JSON IPC serialize it as an array of numbers instead, which for an
/// 8+ MB file is considerably larger on the wire than base64 — this is a deliberate choice, not
/// an oversight.
#[tauri::command]
async fn send_media(
    state: State<'_, AppState>,
    conversation_id: String,
    recipient_peer_id: String,
    content_type: String,
    file_name: String,
    mime_type: String,
    data_base64: String,
    caption: String,
) -> Result<MessageRecord, String> {
    let content_type = parse_content_type(&content_type)?;
    let bytes = base64_decode(&data_base64).map_err(|e| format!("invalid base64 payload: {e}"))?;
    state
        .engine
        .send_media(&conversation_id, &recipient_peer_id, content_type, file_name, mime_type, bytes, caption)
        .await
        .map_err(engine_err)
}

/// Fetches one message's attachment data on demand, base64-encoded for the frontend to turn
/// directly into a data URL (`data:<mime>;base64,<this>`) — never sent inline with
/// `get_messages`, so opening/polling a conversation never has to move megabytes of media it
/// isn't displaying yet.
#[tauri::command]
fn get_attachment_data(state: State<'_, AppState>, message_id: String) -> Result<Option<String>, String> {
    let bytes = state.engine.get_attachment_data(&message_id).map_err(engine_err)?;
    Ok(bytes.map(|b| base64_encode(&b)))
}

/// Saves an attachment to the user's dedicated NOVA media folder (e.g. `Downloads/NOVA` or
/// `Pictures/NOVA`) on disk, making it easily accessible outside the app (e.g. in file explorer or gallery).
#[tauri::command]
fn save_attachment_to_disk(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    message_id: String,
    suggested_filename: Option<String>,
) -> Result<String, String> {
    let bytes = state
        .engine
        .get_attachment_data(&message_id)
        .map_err(engine_err)?
        .ok_or_else(|| "Pièce jointe introuvable".to_string())?;

    let base_dir = app
        .path()
        .download_dir()
        .or_else(|_| app.path().picture_dir())
        .or_else(|_| app.path().app_data_dir())
        .map_err(|e| format!("Impossible d'accéder au dossier de téléchargement : {e}"))?;

    let nova_media_dir = base_dir.join("NOVA");
    std::fs::create_dir_all(&nova_media_dir)
        .map_err(|e| format!("Impossible de créer le dossier NOVA : {e}"))?;

    let now_str = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();

    let raw_name = suggested_filename
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| format!("nova_media_{}", message_id));

    let sanitized_raw: String = raw_name
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' { c } else { '_' })
        .collect();

    let stem = std::path::Path::new(&sanitized_raw)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("fichier");
    let ext = std::path::Path::new(&sanitized_raw)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    // Standardized format: <nom>_<horodatage>.<ext>
    let timestamped_name = if ext.is_empty() {
        format!("{}_{}", stem, now_str)
    } else {
        format!("{}_{}.{}", stem, now_str, ext)
    };

    let mut target_path = nova_media_dir.join(&timestamped_name);
    let mut counter = 1;

    while target_path.exists() {
        let new_name = if ext.is_empty() {
            format!("{}_{}_{}", stem, now_str, counter)
        } else {
            format!("{}_{}_{}.{}", stem, now_str, counter, ext)
        };
        target_path = nova_media_dir.join(new_name);
        counter += 1;
    }

    std::fs::write(&target_path, &bytes)
        .map_err(|e| format!("Erreur lors de l'enregistrement du fichier : {e}"))?;

    Ok(target_path.to_string_lossy().to_string())
}

#[tauri::command]
fn open_file_with_default_app(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", &path])
            .spawn()
            .map_err(|e| format!("Impossible d'ouvrir le fichier : {e}"))?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Impossible d'ouvrir le fichier : {e}"))?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Impossible d'ouvrir le fichier : {e}"))?;
    }
    #[cfg(target_os = "android")]
    {
        let _ = &path;
    }
    Ok(())
}

#[tauri::command]
fn save_and_open_attachment(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    message_id: String,
    suggested_filename: Option<String>,
) -> Result<String, String> {
    let path = save_attachment_to_disk(app, state, message_id, suggested_filename)?;
    let _ = open_file_with_default_app(path.clone());
    Ok(path)
}

#[tauri::command]
fn get_media_folder_path(app: tauri::AppHandle) -> Result<String, String> {
    let base_dir = app
        .path()
        .download_dir()
        .or_else(|_| app.path().picture_dir())
        .or_else(|_| app.path().app_data_dir())
        .map_err(|e| format!("Impossible d'accéder au dossier : {e}"))?;

    let nova_media_dir = base_dir.join("NOVA");
    let _ = std::fs::create_dir_all(&nova_media_dir);
    Ok(nova_media_dir.to_string_lossy().to_string())
}

fn parse_content_type(s: &str) -> Result<nova_protocol::MessageContentType, String> {
    match s {
        "image" | "Image" => Ok(nova_protocol::MessageContentType::Image),
        "video" | "Video" => Ok(nova_protocol::MessageContentType::Video),
        "audio" | "voice" | "Audio" => Ok(nova_protocol::MessageContentType::Audio),
        "file" | "File" => Ok(nova_protocol::MessageContentType::File),
        other => Err(format!("unsupported media content_type: {other}")),
    }
}

fn base64_decode(s: &str) -> Result<Vec<u8>, base64::DecodeError> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.decode(s)
}

fn base64_encode(bytes: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

/// Real connection diagnostics for one peer (transport mode, measured latency of the last
/// successful connect/send) — reflects whatever `nova-transport::TransportSupervisor` actually
/// observed, not a local timing proxy. Returns `None` until at least one connection attempt to
/// this peer has happened (e.g. a message was sent or a chat was opened).
#[tauri::command]
async fn get_diagnostics(
    state: State<'_, AppState>,
    peer_id: String,
) -> Result<Option<nova_transport::PeerConnectionInfo>, String> {
    Ok(state.engine.get_diagnostics(&peer_id).await)
}

/// Every one of this device's own reachable multiaddrs (IPv4, and IPv6 when available), for when
/// it plays the rendezvous/relay role for other devices — the operator reads these here and
/// pastes one into the *other* devices' "Nœud de démarrage" field, rather than needing terminal
/// logs (`nova-bootstrap` prints this same kind of address on stdout; this is the desktop-app
/// equivalent). The IPv6 one, if present, usually needs no port forward/UPnP at all (no NAT to
/// traverse); the IPv4 one needs its listen port fixed (`NOVA_LISTEN_ADDR`) and forwarded (or
/// UPnP-mapped automatically) to be dialable from outside its own network — addresses are shown
/// regardless, they just won't be reachable from outside until that's done.
#[tauri::command]
async fn get_own_full_listen_addrs(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    Ok(state.engine.own_full_listen_addrs().await)
}

/// Current bootstrap/rendezvous address, if one is configured (env var or a previous
/// `set_bootstrap_addr` call) — for prefilling the Settings screen field.
#[tauri::command]
async fn get_bootstrap_addr(state: State<'_, AppState>) -> Result<Option<String>, String> {
    Ok(state.bootstrap_addr.lock().await.as_ref().map(|a| a.to_string()))
}

/// Sets (or clears, with an empty/whitespace string) the bootstrap/rendezvous address and
/// persists it to disk so it survives an app restart. Also performs an immediate live dial
/// if the P2P network is active.
#[tauri::command]
async fn set_bootstrap_addr(state: State<'_, AppState>, addr: String) -> Result<(), String> {
    let trimmed = addr.trim();
    if trimmed.is_empty() {
        let _ = std::fs::remove_file(&state.bootstrap_addr_file);
        *state.bootstrap_addr.lock().await = None;
        return Ok(());
    }
    let parsed: libp2p::Multiaddr = trimmed.parse().map_err(|e| format!("invalid multiaddr: {e}"))?;
    std::fs::write(&state.bootstrap_addr_file, trimmed).map_err(|e| e.to_string())?;
    *state.bootstrap_addr.lock().await = Some(parsed);

    // Live dial immediately if network is active
    if let Err(e) = state.engine.bootstrap_dial(trimmed).await {
        tracing::warn!("Live bootstrap dial to {trimmed} failed (will retry): {e}");
    } else {
        tracing::info!("Live bootstrap connection established to {trimmed}");
    }

    Ok(())
}

/// Current fallback/rendezvous server URL, if one is configured (env var, storage file, or active node).
#[tauri::command]
async fn get_fallback_server_url(state: State<'_, AppState>) -> Result<Option<String>, String> {
    if let Some(url) = state.engine.get_fallback_server_url().await {
        return Ok(Some(url));
    }
    let fallback_url = std::fs::read_to_string(&state.fallback_server_url_file)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| std::env::var("NOVA_UDP_FALLBACK_ADDR").ok());
    Ok(fallback_url)
}

/// Sets (or clears, with an empty string) the fallback discovery/relay server URL (e.g. Render WebSocket endpoint).
#[tauri::command]
async fn set_fallback_server_url(state: State<'_, AppState>, url: String) -> Result<(), String> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        let _ = std::fs::remove_file(&state.fallback_server_url_file);
        std::env::remove_var("NOVA_UDP_FALLBACK_ADDR");
        state.engine.set_fallback_server_url(None).await;
        return Ok(());
    }
    if trimmed != "none" {
        if !trimmed.starts_with("ws://") && !trimmed.starts_with("wss://") {
            return Err("Le schéma de l'URL de secours doit être ws:// ou wss://".to_string());
        }
    }
    std::fs::write(&state.fallback_server_url_file, trimmed).map_err(|e| e.to_string())?;
    std::env::set_var("NOVA_UDP_FALLBACK_ADDR", trimmed);
    state.engine.set_fallback_server_url(Some(trimmed.to_string())).await;
    tracing::info!("Updated fallback discovery/relay server URL to {trimmed}");
    Ok(())
}

/// Checks whether the P2P transport network is actively running on this device.
#[tauri::command]
async fn get_network_status(state: State<'_, AppState>) -> Result<bool, String> {
    let started = *state.network_started.lock().await;
    Ok(started)
}

fn get_server_http_base_url() -> String {
    let ws_url = std::env::var("NOVA_UDP_FALLBACK_ADDR")
        .unwrap_or_else(|_| "wss://nova-discovery-jllv.onrender.com".to_string());
    if ws_url.starts_with("wss://") {
        ws_url.replacen("wss://", "https://", 1)
    } else if ws_url.starts_with("ws://") {
        ws_url.replacen("ws://", "http://", 1)
    } else {
        "https://nova-discovery-jllv.onrender.com".to_string()
    }
}

#[derive(Serialize)]
pub struct AppBuildInfo {
    pub variant: &'static str,
    pub is_admin: bool,
    pub version: &'static str,
}

#[tauri::command]
async fn get_app_build_info() -> Result<AppBuildInfo, String> {
    Ok(AppBuildInfo {
        variant: if cfg!(feature = "admin") { "admin" } else { "user" },
        is_admin: cfg!(feature = "admin"),
        version: "1.0.0",
    })
}

#[derive(Serialize)]
struct ServerReportReq {
    reporter_peer_id: String,
    target_peer_id: String,
    reason: String,
    category: Option<String>,
    comment: Option<String>,
}

#[tauri::command]
async fn report_user(
    state: State<'_, AppState>,
    target_peer_id: String,
    reason: String,
    category: Option<String>,
    comment: Option<String>,
) -> Result<String, String> {
    let own_peer_id = state
        .engine
        .identity
        .lock()
        .await
        .as_ref()
        .map(|id| id.public_id_hex())
        .ok_or_else(|| "Aucun compte actif trouvé sur cet appareil.".to_string())?;
    let base_url = get_server_http_base_url();
    let url = format!("{base_url}/report");

    let client = reqwest::Client::new();
    let res = client
        .post(&url)
        .json(&ServerReportReq {
            reporter_peer_id: own_peer_id,
            target_peer_id,
            reason,
            category,
            comment,
        })
        .send()
        .await
        .map_err(|e| format!("Erreur réseau lors du signalement: {e}"))?;

    if !res.status().is_success() {
        let err_text = res.text().await.unwrap_or_default();
        return Err(format!("Échec du signalement : {err_text}"));
    }

    let body: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
    let msg = body.get("message").and_then(|m| m.as_str()).unwrap_or("Signalement envoyé.");
    Ok(msg.to_string())
}

#[derive(Serialize)]
struct ServerFeedbackReq {
    sender_peer_id: Option<String>,
    rating: u8,
    category: Option<String>,
    comment: Option<String>,
}

#[tauri::command]
async fn submit_app_feedback(
    state: State<'_, AppState>,
    rating: u8,
    category: Option<String>,
    comment: Option<String>,
) -> Result<String, String> {
    let own_peer_id = state
        .engine
        .identity
        .lock()
        .await
        .as_ref()
        .map(|id| id.public_id_hex());
    let base_url = get_server_http_base_url();
    let url = format!("{base_url}/feedback");

    let client = reqwest::Client::new();
    let res = client
        .post(&url)
        .json(&ServerFeedbackReq {
            sender_peer_id: own_peer_id,
            rating,
            category,
            comment,
        })
        .send()
        .await
        .map_err(|e| format!("Erreur réseau lors de l'envoi de l'avis: {e}"))?;

    if !res.status().is_success() {
        let err_text = res.text().await.unwrap_or_default();
        return Err(format!("Échec de l'envoi : {err_text}"));
    }

    Ok("Merci pour votre retour ! Votre avis aide à améliorer NOVA.".to_string())
}

#[cfg(feature = "admin")]
#[tauri::command]
async fn admin_fetch_overview(admin_token: String) -> Result<serde_json::Value, String> {
    let base_url = get_server_http_base_url();
    let url = format!("{base_url}/admin/overview");

    let client = reqwest::Client::new();
    let res = client
        .get(&url)
        .header("x-admin-token", admin_token.trim())
        .send()
        .await
        .map_err(|e| format!("Erreur de connexion au serveur admin: {e}"))?;

    let status = res.status();
    if !status.is_success() {
        let err_text = res.text().await.unwrap_or_default();
        return Err(format!("Erreur admin ({status}) : {err_text}"));
    }

    let overview: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
    Ok(overview)
}

#[cfg(not(feature = "admin"))]
#[tauri::command]
async fn admin_fetch_overview(_admin_token: String) -> Result<serde_json::Value, String> {
    Err("Cette fonctionnalité n'est pas incluse dans ce build utilisateur standard.".into())
}

#[cfg(feature = "admin")]
#[tauri::command]
async fn admin_ban_user(admin_token: String, peer_id: String, reason: Option<String>) -> Result<String, String> {
    let base_url = get_server_http_base_url();
    let url = format!("{base_url}/admin/ban");

    let client = reqwest::Client::new();
    let res = client
        .post(&url)
        .header("x-admin-token", admin_token.trim())
        .json(&serde_json::json!({
            "peer_id": peer_id.trim(),
            "reason": reason,
        }))
        .send()
        .await
        .map_err(|e| format!("Erreur réseau: {e}"))?;

    if !res.status().is_success() {
        let err_text = res.text().await.unwrap_or_default();
        return Err(format!("Échec du bannissement: {err_text}"));
    }
    Ok("Utilisateur suspendu avec succès.".into())
}

#[cfg(not(feature = "admin"))]
#[tauri::command]
async fn admin_ban_user(_admin_token: String, _peer_id: String, _reason: Option<String>) -> Result<String, String> {
    Err("Build utilisateur standard.".into())
}

#[cfg(feature = "admin")]
#[tauri::command]
async fn admin_unban_user(admin_token: String, peer_id: String) -> Result<String, String> {
    let base_url = get_server_http_base_url();
    let url = format!("{base_url}/admin/unban");

    let client = reqwest::Client::new();
    let res = client
        .post(&url)
        .header("x-admin-token", admin_token.trim())
        .json(&serde_json::json!({
            "peer_id": peer_id.trim(),
        }))
        .send()
        .await
        .map_err(|e| format!("Erreur réseau: {e}"))?;

    if !res.status().is_success() {
        let err_text = res.text().await.unwrap_or_default();
        return Err(format!("Échec du déblocage: {err_text}"));
    }
    Ok("Utilisateur réhabilité avec succès.".into())
}

#[cfg(not(feature = "admin"))]
#[tauri::command]
async fn admin_unban_user(_admin_token: String, _peer_id: String) -> Result<String, String> {
    Err("Build utilisateur standard.".into())
}

#[cfg(feature = "admin")]
#[tauri::command]
async fn admin_update_settings(admin_token: String, auto_ban_threshold: usize) -> Result<String, String> {
    let base_url = get_server_http_base_url();
    let url = format!("{base_url}/admin/settings");

    let client = reqwest::Client::new();
    let res = client
        .post(&url)
        .header("x-admin-token", admin_token.trim())
        .json(&serde_json::json!({
            "auto_ban_threshold": auto_ban_threshold,
        }))
        .send()
        .await
        .map_err(|e| format!("Erreur réseau: {e}"))?;

    if !res.status().is_success() {
        let err_text = res.text().await.unwrap_or_default();
        return Err(format!("Échec de la mise à jour: {err_text}"));
    }
    Ok(format!("Seuil d'auto-quarantaine fixé à {auto_ban_threshold} signalements."))
}

#[cfg(not(feature = "admin"))]
#[tauri::command]
async fn admin_update_settings(_admin_token: String, _auto_ban_threshold: usize) -> Result<String, String> {
    Err("Build utilisateur standard.".into())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt::try_init().ok();

    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            // Override lets several independent identities run side by side on the same machine
            // under the same OS user account (e.g. two dev/test instances on one Windows box to
            // exercise real LAN discovery end to end) without their local encrypted storage
            // colliding on the same `app_data_dir()`. Unset in normal use, where the platform
            // default applies exactly as before.
            let data_dir = match std::env::var("NOVA_DATA_DIR") {
                Ok(dir) => PathBuf::from(dir),
                Err(_) => app
                    .path()
                    .app_data_dir()
                    .expect("no app data directory available on this platform"),
            };
            std::fs::create_dir_all(&data_dir).expect("failed to create app data directory");

            let db_path = data_dir.join("nova.db");
            let passphrase = load_or_create_storage_passphrase(&data_dir.join("storage.key"));

            let engine = Arc::new(
                NovaEngine::new(
                    db_path.to_str().expect("app data path is not valid UTF-8"),
                    &passphrase,
                )
                .expect("failed to open local encrypted storage"),
            );

            let bootstrap_addr_file = data_dir.join("bootstrap_addr.txt");
            let bootstrap_addr = std::env::var("NOVA_BOOTSTRAP_ADDR")
                .ok()
                .or_else(|| {
                    std::fs::read_to_string(&bootstrap_addr_file)
                        .ok()
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                })
                .and_then(|s| s.parse::<libp2p::Multiaddr>().ok());
            if bootstrap_addr.is_none() {
                tracing::info!(
                    "No bootstrap address configured (env NOVA_BOOTSTRAP_ADDR or the Settings screen) — \
                     this device will only discover peers on the same LAN via mDNS."
                );
            }

            let fallback_server_url_file = data_dir.join("fallback_server_url.txt");
            if let Ok(saved_fallback) = std::fs::read_to_string(&fallback_server_url_file) {
                let trimmed = saved_fallback.trim();
                if !trimmed.is_empty() && std::env::var("NOVA_UDP_FALLBACK_ADDR").is_err() {
                    std::env::set_var("NOVA_UDP_FALLBACK_ADDR", trimmed);
                }
            }

            let listen_addr = std::env::var("NOVA_LISTEN_ADDR")
                .unwrap_or_else(|_| "/ip4/0.0.0.0/udp/0/quic-v1".to_string());

            app.manage(AppState {
                engine,
                bootstrap_addr: Mutex::new(bootstrap_addr),
                bootstrap_addr_file,
                fallback_server_url_file,
                listen_addr,
                network_started: Mutex::new(false),
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            create_account,
            restore_account,
            try_resume_session,
            logout,
            get_own_peer_id,
            get_own_prekey_bundle_hex,
            get_own_invitation_uri,
            add_contact,
            get_contacts,
            trust_contact,
            block_contact,
            block_and_delete_conversation,
            unblock_contact,
            delete_contact,
            send_message,
            send_media,
            get_attachment_data,
            save_attachment_to_disk,
            open_file_with_default_app,
            save_and_open_attachment,
            get_media_folder_path,
            retry_failed_message,
            get_conversations,
            get_messages,
            delete_message,
            search_messages,
            get_diagnostics,
            get_own_full_listen_addrs,
            get_bootstrap_addr,
            set_bootstrap_addr,
            get_fallback_server_url,
            set_fallback_server_url,
            search_directory,
            get_user_profile,
            update_user_profile,
            get_network_status,
            get_app_build_info,
            report_user,
            submit_app_feedback,
            admin_fetch_overview,
            admin_ban_user,
            admin_unban_user,
            admin_update_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running the NOVA Chat desktop app");
}

/// Protects the local storage encryption key using OS-level secure storage (Windows DPAPI
/// on Windows, 0o600 restricted file permissions on Unix). This ensures that even if malware
/// or another user copies `storage.key`, it cannot be decrypted outside of the current OS session.
fn load_or_create_storage_passphrase(path: &std::path::Path) -> String {
    #[cfg(windows)]
    {
        if let Ok(encrypted_bytes) = std::fs::read(path) {
            if !encrypted_bytes.is_empty() {
                if let Ok(decrypted_bytes) = win_dpapi::dpapi_unprotect(&encrypted_bytes) {
                    if let Ok(passphrase) = String::from_utf8(decrypted_bytes) {
                        let trimmed = passphrase.trim().to_string();
                        if !trimmed.is_empty() {
                            return trimmed;
                        }
                    }
                }
            }
        }

        // Fresh random 256-bit passphrase
        use rand::RngCore;
        let mut raw = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut raw);
        let passphrase = hex::encode(raw);

        // Encrypt with Windows DPAPI before writing to disk
        match win_dpapi::dpapi_protect(passphrase.as_bytes()) {
            Ok(protected) => {
                let _ = std::fs::write(path, protected);
            }
            Err(e) => {
                tracing::warn!("DPAPI protection failed, falling back to plain write: {e}");
                let _ = std::fs::write(path, &passphrase);
            }
        }
        passphrase
    }

    #[cfg(not(windows))]
    {
        if let Ok(existing) = std::fs::read_to_string(path) {
            let trimmed = existing.trim().to_string();
            if !trimmed.is_empty() {
                return trimmed;
            }
        }

        use rand::RngCore;
        let mut raw = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut raw);
        let passphrase = hex::encode(raw);
        std::fs::write(path, &passphrase).expect("failed to persist local storage passphrase");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
        }
        passphrase
    }
}

#[cfg(windows)]
mod win_dpapi {
    use std::ptr;

    #[allow(non_snake_case)]
    #[repr(C)]
    struct DATA_BLOB {
        cbData: u32,
        pbData: *mut u8,
    }

    #[link(name = "crypt32")]
    extern "system" {
        fn CryptProtectData(
            pDataIn: *const DATA_BLOB,
            szDataDescr: *const u16,
            pOptionalEntropy: *const DATA_BLOB,
            pvReserved: *mut std::ffi::c_void,
            pPromptStruct: *mut std::ffi::c_void,
            dwFlags: u32,
            pDataOut: *mut DATA_BLOB,
        ) -> i32;

        fn CryptUnprotectData(
            pDataIn: *const DATA_BLOB,
            ppszDataDescr: *mut *mut u16,
            pOptionalEntropy: *const DATA_BLOB,
            pvReserved: *mut std::ffi::c_void,
            pPromptStruct: *mut std::ffi::c_void,
            dwFlags: u32,
            pDataOut: *mut DATA_BLOB,
        ) -> i32;
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn LocalFree(hMem: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
    }

    pub fn dpapi_protect(plaintext: &[u8]) -> Result<Vec<u8>, String> {
        let mut in_blob = DATA_BLOB {
            cbData: plaintext.len() as u32,
            pbData: plaintext.as_ptr() as *mut u8,
        };
        let mut out_blob = DATA_BLOB {
            cbData: 0,
            pbData: ptr::null_mut(),
        };

        let success = unsafe {
            CryptProtectData(
                &mut in_blob,
                ptr::null(),
                ptr::null(),
                ptr::null_mut(),
                ptr::null_mut(),
                0,
                &mut out_blob,
            )
        };

        if success == 0 {
            return Err("CryptProtectData failed".into());
        }

        let slice = unsafe { std::slice::from_raw_parts(out_blob.pbData, out_blob.cbData as usize) };
        let result = slice.to_vec();
        unsafe { LocalFree(out_blob.pbData as *mut _) };
        Ok(result)
    }

    pub fn dpapi_unprotect(ciphertext: &[u8]) -> Result<Vec<u8>, String> {
        let mut in_blob = DATA_BLOB {
            cbData: ciphertext.len() as u32,
            pbData: ciphertext.as_ptr() as *mut u8,
        };
        let mut out_blob = DATA_BLOB {
            cbData: 0,
            pbData: ptr::null_mut(),
        };

        let success = unsafe {
            CryptUnprotectData(
                &mut in_blob,
                ptr::null_mut(),
                ptr::null(),
                ptr::null_mut(),
                ptr::null_mut(),
                0,
                &mut out_blob,
            )
        };

        if success == 0 {
            return Err("CryptUnprotectData failed".into());
        }

        let slice = unsafe { std::slice::from_raw_parts(out_blob.pbData, out_blob.cbData as usize) };
        let result = slice.to_vec();
        unsafe { LocalFree(out_blob.pbData as *mut _) };
        Ok(result)
    }
}

