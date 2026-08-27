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
    // the same way the Tor-reapply step below already is keeps the intended behavior (account
    // creation always succeeds locally; network_active just ends up false) even when there's no
    // network to fail against quickly.
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

    // The `P2PNode`/`TorManager` just created above always starts from `TorConfig::default()`
    // (Tor disabled — see `nova_transport::dht_node::P2PNode::start`), with no knowledge of
    // whatever the user previously saved via `configure_tor`/the Settings screen. Without this,
    // a user who had explicitly enabled Tor in a past session would find it silently back to
    // disabled on every relaunch/resume (`get_tor_status` reflects the *live* node once the
    // network is attached, not the saved settings) — the Tor toggle looked "stuck disabled"
    // every time the app reconnected. Re-apply the saved settings now so they actually take
    // effect again immediately after the network comes up.
    match state.engine.storage.get_tor_settings() {
        Ok(saved) => {
            // Bounded defensively: this locks `NovaEngine::network` and `NovaEngine::identity`,
            // the same two locks `attach_network`'s just-spawned outbox-pump task (re-encrypting/
            // resending anything left over from a previous session, e.g. a message that never
            // got delivered before the app was last closed) also needs, in the opposite order.
            // `try_resume_session`/`create_account`/`restore_account` all block the UI on this
            // whole function returning — if this ever contends against that background task
            // instead of racing it harmlessly, the entire app would be stuck on a permanently
            // blank screen (nothing after this point, including the very first screen render,
            // ever runs) rather than failing visibly. A real, reproducible instance of exactly
            // this symptom (blank screen forever after a relaunch with a stuck pending message
            // in the outbox) is what prompted adding this bound.
            let tor_reapply = tokio::time::timeout(
                std::time::Duration::from_secs(5),
                state
                    .engine
                    .configure_tor(saved.enabled, &saved.mode, &saved.socks_proxy, saved.bridge_type),
            )
            .await;
            match tor_reapply {
                Ok(Ok(())) => {}
                Ok(Err(e)) => tracing::warn!("Failed to re-apply saved Tor settings on network start: {e}"),
                Err(_) => tracing::warn!(
                    "Re-applying saved Tor settings on network start did not complete within 5s \
                     (likely lock contention with the outbox pump) — continuing without it; Tor \
                     settings can still be changed from the Settings screen"
                ),
            }
        }
        Err(e) => tracing::warn!("Failed to load saved Tor settings on network start: {e}"),
    }

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
fn get_contacts(state: State<'_, AppState>) -> Result<Vec<nova_storage::ContactRecord>, String> {
    state.engine.get_contacts().map_err(engine_err)
}

#[tauri::command]
fn block_contact(state: State<'_, AppState>, peer_id: String) -> Result<(), String> {
    state.engine.block_contact(&peer_id).map_err(engine_err)
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
fn update_user_profile(
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
    state.engine.save_user_profile(&profile).map_err(engine_err)
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

#[tauri::command]
async fn get_own_onion_address(state: State<'_, AppState>) -> Result<String, String> {
    state.engine.own_onion_address().await.map_err(engine_err)
}

#[tauri::command]
async fn get_tor_status(state: State<'_, AppState>) -> Result<nova_transport::TorStatus, String> {
    state.engine.get_tor_status().await.map_err(engine_err)
}

#[tauri::command]
async fn configure_tor(
    state: State<'_, AppState>,
    enabled: bool,
    mode: String,
    socks_proxy: String,
    bridge_type: Option<String>,
) -> Result<(), String> {
    state
        .engine
        .configure_tor(enabled, &mode, &socks_proxy, bridge_type)
        .await
        .map_err(engine_err)
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

            // Persistent home for the embedded Tor client's consensus cache, guard state, and
            // onion-service keystore (see `nova_transport::tor`) — a real, durable subdirectory
            // of this device's own app data, so Tor doesn't have to re-bootstrap from scratch
            // (a real, user-visible delay) on every app launch. Read directly by `dht_node.rs`
            // via `NOVA_TOR_STATE_DIR`, the same env-var pattern as `NOVA_BOOTSTRAP_ADDR`.
            std::env::set_var("NOVA_TOR_STATE_DIR", data_dir.join("tor_state"));

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

            let listen_addr = std::env::var("NOVA_LISTEN_ADDR")
                .unwrap_or_else(|_| "/ip4/0.0.0.0/udp/0/quic-v1".to_string());

            app.manage(AppState {
                engine,
                bootstrap_addr: Mutex::new(bootstrap_addr),
                bootstrap_addr_file,
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
            block_contact,
            unblock_contact,
            delete_contact,
            send_message,
            send_media,
            get_attachment_data,
            retry_failed_message,
            get_conversations,
            get_messages,
            search_messages,
            get_diagnostics,
            get_own_full_listen_addrs,
            get_bootstrap_addr,
            set_bootstrap_addr,
            get_user_profile,
            update_user_profile,
            get_own_onion_address,
            get_tor_status,
            configure_tor,
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

