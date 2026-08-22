use nova_crypto::PreKeyBundle;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum DbMessageStatus {
    Created = 0,
    Encrypted = 1,
    Queued = 2,
    Sent = 3,
    Delivered = 4,
    Read = 5,
    Failed = 6,
}

impl DbMessageStatus {
    pub fn to_i32(self) -> i32 {
        self as i32
    }

    pub fn from_i32(val: i32) -> Self {
        match val {
            0 => DbMessageStatus::Created,
            1 => DbMessageStatus::Encrypted,
            2 => DbMessageStatus::Queued,
            3 => DbMessageStatus::Sent,
            4 => DbMessageStatus::Delivered,
            5 => DbMessageStatus::Read,
            _ => DbMessageStatus::Failed,
        }
    }
}

/// A contact known locally, identified by their Ed25519 identity public key (hex).
/// `prekey_bundle` is the peer's X3DH prekey material, verified (signature checked)
/// before being accepted by the engine — see `NovaEngine::add_contact`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContactRecord {
    pub peer_id: String,
    pub username: String,
    pub display_name: String,
    pub prekey_bundle: PreKeyBundle,
    pub safety_number: String,
    pub is_online: bool,
    pub is_blocked: bool,
    pub last_seen_utc: i64,
}

/// Represents a direct 1-to-1 sovereign conversation with a single peer.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConversationRecord {
    pub id: String,
    pub peer_id: String,
    pub title: String,
    pub last_message_text: String,
    pub last_message_time_utc: i64,
    pub unread_count: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MessageRecord {
    pub id: String,
    pub conversation_id: String,
    pub sender_id: String,
    pub recipient_id: String,
    pub text_content: String,
    pub timestamp_utc: i64,
    pub status: DbMessageStatus,
    pub is_outgoing: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OutboxItem {
    pub id: i64,
    pub message_id: String,
    pub conversation_id: String,
    pub recipient_id: String,
    pub payload: Vec<u8>,
    pub attempt_count: i32,
    pub next_retry_utc: i64,
}

/// Personal profile metadata for the local device identity, encrypted at rest.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserProfileRecord {
    pub display_name: String,
    pub bio: String,
    pub avatar_data_url: Option<String>,
}

/// Configuration for Tor anonymous routing and SOCKS5 proxying stored in the local database.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TorSettingsRecord {
    pub enabled: bool,
    pub mode: String, // "direct_only", "hybrid", "tor_strict"
    pub socks_proxy: String,
    pub bridge_type: Option<String>,
}

impl Default for TorSettingsRecord {
    fn default() -> Self {
        Self {
            enabled: false,
            mode: "direct_only".to_string(),
            socks_proxy: "127.0.0.1:9050".to_string(),
            bridge_type: None,
        }
    }
}
