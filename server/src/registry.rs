use nova_protocol::{
    DirectoryProfile, DirectorySearchResult, PeerEndpoint, SignedDirectoryEntry,
    SignedPresenceRegistration,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;
use tokio::sync::RwLock;

const DEFAULT_TTL_SECONDS: u64 = 60;
const DIRECTORY_TTL_SECONDS: u64 = 7 * 24 * 3600; // 7 days retention
const REGISTRATION_FRESHNESS_WINDOW_SECS: u64 = 86400;
const MAX_TRACKED_PEERS: usize = 50_000;
pub const DEFAULT_AUTO_BAN_THRESHOLD: usize = 3;

#[derive(Error, Debug)]
pub enum RegistryError {
    #[error("registration failed cryptographic verification: {0}")]
    InvalidRegistration(#[from] nova_protocol::PresenceError),
    #[error("registration timestamp is stale or too far in the future")]
    StaleTimestamp,
    #[error("presence registry is at capacity")]
    AtCapacity,
    #[error("this user account has been banned")]
    UserBanned,
    #[error("invalid input: {0}")]
    InvalidInput(String),
}

struct TrackedEndpoint {
    endpoint: PeerEndpoint,
    last_seen_utc: u64,
}

struct TrackedDirectoryEntry {
    profile: DirectoryProfile,
    last_updated_utc: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportRecord {
    pub id: String,
    pub reporter_peer_id: String,
    pub target_peer_id: String,
    pub reason: String,
    pub category: String, // "spam", "harassment", "inappropriate", "scam", "impersonation", "other"
    pub comment: String,
    pub timestamp_utc: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackRecord {
    pub id: String,
    pub sender_peer_id: Option<String>,
    pub rating: u8, // 1 to 5
    pub category: String, // "general", "call_quality", "ui", "suggestion", "bug"
    pub comment: String,
    pub timestamp_utc: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BannedUserRecord {
    pub peer_id: String,
    pub reason: String,
    pub banned_at_utc: u64,
    pub report_count: usize,
    pub is_automatic: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserModerationItem {
    pub peer_id: String,
    pub username: String,
    pub display_name: String,
    pub avatar_data_url: Option<String>,
    pub is_online: bool,
    pub status: String, // "healthy", "reported", "banned"
    pub report_count: usize,
    pub last_seen_utc: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminOverview {
    pub total_users: usize,
    pub healthy_users: usize,
    pub reported_users: usize,
    pub banned_users: usize,
    pub average_rating: f32,
    pub total_feedbacks: usize,
    pub auto_ban_threshold: usize,
    pub users: Vec<UserModerationItem>,
    pub reports: Vec<ReportRecord>,
    pub feedbacks: Vec<FeedbackRecord>,
    pub banned_records: Vec<BannedUserRecord>,
}

#[derive(Clone)]
pub struct PresenceRegistry {
    entries: Arc<RwLock<HashMap<String, TrackedEndpoint>>>,
    directory: Arc<RwLock<HashMap<String, TrackedDirectoryEntry>>>,
    reports: Arc<RwLock<Vec<ReportRecord>>>,
    feedbacks: Arc<RwLock<Vec<FeedbackRecord>>>,
    banned_users: Arc<RwLock<HashMap<String, BannedUserRecord>>>,
    auto_ban_threshold: Arc<RwLock<usize>>,
    pool: Option<sqlx::PgPool>,
}

impl Default for PresenceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl PresenceRegistry {
    pub fn new() -> Self {
        Self::with_pool(None)
    }

    pub fn with_pool(pool: Option<sqlx::PgPool>) -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            directory: Arc::new(RwLock::new(HashMap::new())),
            reports: Arc::new(RwLock::new(Vec::new())),
            feedbacks: Arc::new(RwLock::new(Vec::new())),
            banned_users: Arc::new(RwLock::new(HashMap::new())),
            auto_ban_threshold: Arc::new(RwLock::new(DEFAULT_AUTO_BAN_THRESHOLD)),
            pool,
        }
    }

    pub async fn is_banned(&self, peer_id: &str) -> bool {
        {
            let lock = self.banned_users.read().await;
            if lock.contains_key(peer_id) {
                return true;
            }
        }
        if let Some(ref pool) = self.pool {
            let res: Result<Option<String>, _> = sqlx::query_scalar("SELECT peer_id FROM banned_users WHERE peer_id = $1")
                .bind(peer_id)
                .fetch_optional(pool)
                .await;
            if let Ok(Some(_)) = res {
                return true;
            }
        }
        false
    }

    /// Registers (or refreshes) a peer's presence. The registration MUST be signed by the
    /// Ed25519 identity key matching its own `peer_id`.
    pub async fn register(
        &self,
        registration: SignedPresenceRegistration,
        observed_addr: SocketAddr,
    ) -> Result<(), RegistryError> {
        let now = now_secs();
        let ts = registration.timestamp_utc;
        let within_window = now.saturating_sub(ts) <= REGISTRATION_FRESHNESS_WINDOW_SECS
            && ts.saturating_sub(now) <= REGISTRATION_FRESHNESS_WINDOW_SECS;
        if !within_window {
            return Err(RegistryError::StaleTimestamp);
        }

        registration.verify()?;

        if self.is_banned(&registration.endpoint.peer_id).await {
            return Err(RegistryError::UserBanned);
        }

        let mut endpoint = registration.endpoint;
        endpoint.public_ip = observed_addr.ip().to_string();

        if let Some(ref pool) = self.pool {
            let _ = sqlx::query(r#"
                INSERT INTO presence_entries (peer_id, public_ip, public_port, local_ip, local_port, last_seen_at)
                VALUES ($1, $2, $3, $4, $5, $6)
                ON CONFLICT (peer_id) DO UPDATE SET
                    public_ip = EXCLUDED.public_ip,
                    public_port = EXCLUDED.public_port,
                    local_ip = EXCLUDED.local_ip,
                    local_port = EXCLUDED.local_port,
                    last_seen_at = EXCLUDED.last_seen_at
            "#)
            .bind(&endpoint.peer_id)
            .bind(&endpoint.public_ip)
            .bind(endpoint.public_port as i32)
            .bind(endpoint.local_ip.as_deref())
            .bind(endpoint.local_port.map(|p| p as i32))
            .bind(now as i64)
            .execute(pool)
            .await;
        }

        let mut lock = self.entries.write().await;
        if !lock.contains_key(&endpoint.peer_id) && lock.len() >= MAX_TRACKED_PEERS {
            return Err(RegistryError::AtCapacity);
        }

        lock.insert(
            endpoint.peer_id.clone(),
            TrackedEndpoint {
                endpoint,
                last_seen_utc: now,
            },
        );
        Ok(())
    }

    /// Registers or updates a peer's public directory profile and PreKey bundle.
    /// Cryptographically verified against the peer's Ed25519 identity key.
    pub async fn register_directory(
        &self,
        entry: SignedDirectoryEntry,
    ) -> Result<(), RegistryError> {
        let now = now_secs();
        let ts = entry.timestamp_utc;
        let within_window = now.saturating_sub(ts) <= REGISTRATION_FRESHNESS_WINDOW_SECS
            && ts.saturating_sub(now) <= REGISTRATION_FRESHNESS_WINDOW_SECS;
        if !within_window {
            return Err(RegistryError::StaleTimestamp);
        }

        entry.verify()?;

        if self.is_banned(&entry.profile.peer_id).await {
            return Err(RegistryError::UserBanned);
        }

        if let Some(ref pool) = self.pool {
            let _ = sqlx::query(r#"
                INSERT INTO directory_profiles (peer_id, username, display_name, avatar_data_url, prekey_bundle_hex, registered_at, last_updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                ON CONFLICT (peer_id) DO UPDATE SET
                    username = EXCLUDED.username,
                    display_name = EXCLUDED.display_name,
                    avatar_data_url = EXCLUDED.avatar_data_url,
                    prekey_bundle_hex = EXCLUDED.prekey_bundle_hex,
                    last_updated_at = EXCLUDED.last_updated_at
            "#)
            .bind(&entry.profile.peer_id)
            .bind(&entry.profile.username)
            .bind(&entry.profile.display_name)
            .bind(entry.profile.avatar_data_url.as_deref())
            .bind(&entry.profile.prekey_bundle_hex)
            .bind(entry.timestamp_utc as i64)
            .bind(now as i64)
            .execute(pool)
            .await;
        }

        let mut lock = self.directory.write().await;
        if !lock.contains_key(&entry.profile.peer_id) && lock.len() >= MAX_TRACKED_PEERS {
            return Err(RegistryError::AtCapacity);
        }

        lock.insert(
            entry.profile.peer_id.clone(),
            TrackedDirectoryEntry {
                profile: entry.profile,
                last_updated_utc: now,
            },
        );
        Ok(())
    }

    /// Searches the public directory by peer_id (full or prefix), @username, or display name.
    pub async fn search_directory(&self, query: &str) -> Vec<DirectorySearchResult> {
        let now = now_secs();
        let q = query.trim().trim_start_matches('@').to_lowercase();
        if q.is_empty() {
            return Vec::new();
        }

        let dir_lock = self.directory.read().await;
        let presence_lock = self.entries.read().await;
        let banned_lock = self.banned_users.read().await;

        let mut results = Vec::new();

        for (peer_id, entry) in dir_lock.iter() {
            if banned_lock.contains_key(peer_id) {
                continue;
            }

            let pid_lower = peer_id.to_lowercase();
            let uname_lower = entry.profile.username.to_lowercase();
            let dname_lower = entry.profile.display_name.to_lowercase();

            let matches = pid_lower.starts_with(&q)
                || pid_lower == q
                || uname_lower == q
                || uname_lower.starts_with(&q)
                || uname_lower.contains(&q)
                || dname_lower.contains(&q);

            if matches {
                let is_online = presence_lock
                    .get(peer_id)
                    .map(|p| now.saturating_sub(p.last_seen_utc) <= DEFAULT_TTL_SECONDS)
                    .unwrap_or(false);

                results.push(DirectorySearchResult {
                    peer_id: entry.profile.peer_id.clone(),
                    username: entry.profile.username.clone(),
                    display_name: entry.profile.display_name.clone(),
                    avatar_data_url: entry.profile.avatar_data_url.clone(),
                    prekey_bundle_hex: entry.profile.prekey_bundle_hex.clone(),
                    is_online,
                    last_seen_utc: entry.last_updated_utc,
                });

                if results.len() >= 100 {
                    break;
                }
            }
        }

        if let Some(ref pool) = self.pool {
            use sqlx::Row;
            let pattern = format!("%{q}%");
            if let Ok(rows) = sqlx::query(r#"
                SELECT peer_id, username, display_name, avatar_data_url, prekey_bundle_hex, last_updated_at
                FROM directory_profiles
                WHERE LOWER(peer_id) LIKE $1 OR LOWER(username) LIKE $1 OR LOWER(display_name) LIKE $1
                ORDER BY last_updated_at DESC LIMIT 50
            "#)
            .bind(&pattern)
            .fetch_all(pool)
            .await {
                for r in rows {
                    let pid: String = r.get("peer_id");
                    if !banned_lock.contains_key(&pid) && !results.iter().any(|res| res.peer_id == pid) {
                        let is_online = presence_lock
                            .get(&pid)
                            .map(|p| now.saturating_sub(p.last_seen_utc) <= DEFAULT_TTL_SECONDS)
                            .unwrap_or(false);
                        results.push(DirectorySearchResult {
                            peer_id: pid,
                            username: r.get("username"),
                            display_name: r.get("display_name"),
                            avatar_data_url: r.get("avatar_data_url"),
                            prekey_bundle_hex: r.get("prekey_bundle_hex"),
                            is_online,
                            last_seen_utc: r.get::<i64, _>("last_updated_at") as u64,
                        });
                    }
                }
            }
        }

        // Sort: exact matches first, then online peers, then alphabetical
        results.sort_by(|a, b| {
            let a_exact = a.peer_id.to_lowercase() == q || a.username.to_lowercase() == q;
            let b_exact = b.peer_id.to_lowercase() == q || b.username.to_lowercase() == q;
            b_exact.cmp(&a_exact)
                .then_with(|| b.is_online.cmp(&a_online(a)))
                .then_with(|| a.display_name.to_lowercase().cmp(&b.display_name.to_lowercase()))
        });

        fn a_online(a: &DirectorySearchResult) -> bool {
            a.is_online
        }

        results.truncate(20);
        results
    }

    pub async fn get_peer(&self, peer_id: &str) -> Option<PeerEndpoint> {
        if self.is_banned(peer_id).await {
            return None;
        }
        let now = now_secs();
        {
            let lock = self.entries.read().await;
            if let Some(tracked) = lock.get(peer_id) {
                if now.saturating_sub(tracked.last_seen_utc) <= DEFAULT_TTL_SECONDS {
                    return Some(tracked.endpoint.clone());
                }
            }
        }
        if let Some(ref pool) = self.pool {
            use sqlx::Row;
            if let Ok(Some(row)) = sqlx::query(
                "SELECT peer_id, public_ip, public_port, local_ip, local_port, last_seen_at FROM presence_entries WHERE peer_id = $1"
            )
            .bind(peer_id)
            .fetch_optional(pool)
            .await
            {
                let last_seen: i64 = row.get("last_seen_at");
                if now.saturating_sub(last_seen as u64) <= DEFAULT_TTL_SECONDS {
                    return Some(PeerEndpoint {
                        peer_id: row.get("peer_id"),
                        public_ip: row.get("public_ip"),
                        public_port: row.get::<i32, _>("public_port") as u16,
                        local_ip: row.get("local_ip"),
                        local_port: row.get::<Option<i32>, _>("local_port").map(|p| p as u16),
                    });
                }
            }
        }
        None
    }

    // ==========================================
    // Moderation, Reporting & Feedback Engine
    // ==========================================

    /// Submits a report against a target user.
    /// If distinct reporters reach or exceed `auto_ban_threshold`, the user is automatically quarantined.
    pub async fn submit_report(
        &self,
        reporter_peer_id: String,
        target_peer_id: String,
        reason: String,
        category: String,
        comment: String,
    ) -> Result<bool, RegistryError> {
        if reporter_peer_id.trim().is_empty() || target_peer_id.trim().is_empty() {
            return Err(RegistryError::InvalidInput("reporter and target must be specified".into()));
        }
        if reporter_peer_id == target_peer_id {
            return Err(RegistryError::InvalidInput("cannot report self".into()));
        }

        let now = now_secs();
        let report_id = format!("rep_{}_{}", now, &target_peer_id[..target_peer_id.len().min(8)]);
        let record = ReportRecord {
            id: report_id,
            reporter_peer_id: reporter_peer_id.clone(),
            target_peer_id: target_peer_id.clone(),
            reason: reason.clone(),
            category,
            comment,
            timestamp_utc: now,
        };

        let mut reports_lock = self.reports.write().await;
        reports_lock.push(record);

        // Count unique reporters for this target
        let unique_reporters: HashSet<&str> = reports_lock
            .iter()
            .filter(|r| r.target_peer_id == target_peer_id)
            .map(|r| r.reporter_peer_id.as_str())
            .collect();
        let count = unique_reporters.len();

        let threshold = *self.auto_ban_threshold.read().await;
        let mut auto_banned = false;

        if count >= threshold && !self.is_banned(&target_peer_id).await {
            let mut banned_lock = self.banned_users.write().await;
            banned_lock.insert(
                target_peer_id.clone(),
                BannedUserRecord {
                    peer_id: target_peer_id.clone(),
                    reason: format!("Mise en quarantaine automatique ({count} signalements distincts : {reason})"),
                    banned_at_utc: now,
                    report_count: count,
                    is_automatic: true,
                },
            );
            auto_banned = true;

            // Remove from active presence
            let mut entries_lock = self.entries.write().await;
            entries_lock.remove(&target_peer_id);
        }

        Ok(auto_banned)
    }

    /// Submits user rating (1-5 stars) and feedback comment.
    pub async fn submit_feedback(
        &self,
        sender_peer_id: Option<String>,
        rating: u8,
        category: String,
        comment: String,
    ) -> Result<FeedbackRecord, RegistryError> {
        let valid_rating = rating.clamp(1, 5);
        let now = now_secs();
        let id = format!("fb_{}_{}", now, sender_peer_id.as_deref().unwrap_or("anon"));
        let record = FeedbackRecord {
            id,
            sender_peer_id,
            rating: valid_rating,
            category,
            comment,
            timestamp_utc: now,
        };

        let mut lock = self.feedbacks.write().await;
        lock.push(record.clone());
        Ok(record)
    }

    /// Manually bans a user.
    pub async fn ban_user(&self, peer_id: String, reason: String) -> Result<(), RegistryError> {
        let now = now_secs();
        let reports_lock = self.reports.read().await;
        let report_count = reports_lock.iter().filter(|r| r.target_peer_id == peer_id).count();

        let mut banned_lock = self.banned_users.write().await;
        banned_lock.insert(
            peer_id.clone(),
            BannedUserRecord {
                peer_id: peer_id.clone(),
                reason,
                banned_at_utc: now,
                report_count,
                is_automatic: false,
            },
        );

        // Remove from active presence
        let mut entries_lock = self.entries.write().await;
        entries_lock.remove(&peer_id);

        Ok(())
    }

    /// Manually unbans a user.
    pub async fn unban_user(&self, peer_id: &str) -> Result<bool, RegistryError> {
        let mut banned_lock = self.banned_users.write().await;
        let removed = banned_lock.remove(peer_id).is_some();
        Ok(removed)
    }

    pub async fn set_auto_ban_threshold(&self, threshold: usize) {
        let mut lock = self.auto_ban_threshold.write().await;
        *lock = threshold.max(1);
    }

    pub async fn get_auto_ban_threshold(&self) -> usize {
        *self.auto_ban_threshold.read().await
    }

    /// Compiles a comprehensive administration overview for the platform moderator.
    pub async fn get_admin_overview(&self) -> AdminOverview {
        let now = now_secs();
        let dir_lock = self.directory.read().await;
        let presence_lock = self.entries.read().await;
        let reports_lock = self.reports.read().await;
        let banned_lock = self.banned_users.read().await;
        let feedbacks_lock = self.feedbacks.read().await;
        let threshold = *self.auto_ban_threshold.read().await;

        // Group reports by target_peer_id
        let mut target_report_counts: HashMap<String, usize> = HashMap::new();
        for r in reports_lock.iter() {
            *target_report_counts.entry(r.target_peer_id.clone()).or_insert(0) += 1;
        }

        let mut users = Vec::new();
        let mut healthy_count = 0;
        let mut reported_count = 0;
        let banned_count = banned_lock.len();

        for (peer_id, entry) in dir_lock.iter() {
            let is_banned = banned_lock.contains_key(peer_id);
            let report_cnt = target_report_counts.get(peer_id).copied().unwrap_or(0);
            let is_online = presence_lock
                .get(peer_id)
                .map(|p| now.saturating_sub(p.last_seen_utc) <= DEFAULT_TTL_SECONDS)
                .unwrap_or(false);

            let status = if is_banned {
                "banned".to_string()
            } else if report_cnt > 0 {
                reported_count += 1;
                "reported".to_string()
            } else {
                healthy_count += 1;
                "healthy".to_string()
            };

            users.push(UserModerationItem {
                peer_id: peer_id.clone(),
                username: entry.profile.username.clone(),
                display_name: entry.profile.display_name.clone(),
                avatar_data_url: entry.profile.avatar_data_url.clone(),
                is_online,
                status,
                report_count: report_cnt,
                last_seen_utc: entry.last_updated_utc,
            });
        }

        let total_feedbacks = feedbacks_lock.len();
        let average_rating = if total_feedbacks > 0 {
            let sum: u32 = feedbacks_lock.iter().map(|f| f.rating as u32).sum();
            (sum as f32) / (total_feedbacks as f32)
        } else {
            5.0
        };

        AdminOverview {
            total_users: users.len(),
            healthy_users: healthy_count,
            reported_users: reported_count,
            banned_users: banned_count,
            average_rating,
            total_feedbacks,
            auto_ban_threshold: threshold,
            users,
            reports: reports_lock.clone(),
            feedbacks: feedbacks_lock.clone(),
            banned_records: banned_lock.values().cloned().collect(),
        }
    }

    pub async fn cleanup_expired(&self) {
        let now = now_secs();
        {
            let mut lock = self.entries.write().await;
            lock.retain(|_, tracked| now.saturating_sub(tracked.last_seen_utc) <= DEFAULT_TTL_SECONDS);
        }
        {
            let mut lock = self.directory.write().await;
            lock.retain(|_, tracked| now.saturating_sub(tracked.last_updated_utc) <= DIRECTORY_TTL_SECONDS);
        }
    }

    pub async fn count(&self) -> usize {
        let lock = self.entries.read().await;
        lock.len()
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use nova_crypto::{DeviceIdentity, MnemonicPhrase};

    fn identity(name: &str) -> DeviceIdentity {
        let mnemonic = MnemonicPhrase::generate().unwrap();
        DeviceIdentity::from_mnemonic(&mnemonic, name).unwrap()
    }

    fn endpoint() -> PeerEndpoint {
        PeerEndpoint {
            peer_id: String::new(),
            public_ip: "192.0.2.1".into(),
            public_port: 9000,
            local_ip: Some("10.0.0.5".into()),
            local_port: Some(9000),
        }
    }

    fn loopback_addr() -> SocketAddr {
        "127.0.0.1:51820".parse().unwrap()
    }

    #[tokio::test]
    async fn test_presence_registry_lifecycle() {
        let registry = PresenceRegistry::new();
        let alice = identity("alice");
        let now = now_secs();

        let reg = SignedPresenceRegistration::sign(&alice, endpoint(), now);
        registry.register(reg, loopback_addr()).await.unwrap();
        assert_eq!(registry.count().await, 1);

        let retrieved = registry.get_peer(&alice.public_id_hex()).await.unwrap();
        assert_eq!(retrieved.public_ip, "127.0.0.1");
        assert_eq!(retrieved.public_port, 9000);

        assert!(registry.get_peer("non_existent").await.is_none());
    }

    #[tokio::test]
    async fn test_unsigned_or_spoofed_registration_is_rejected() {
        let registry = PresenceRegistry::new();
        let alice = identity("alice");
        let bob = identity("bob");
        let now = now_secs();

        let mut reg = SignedPresenceRegistration::sign(&bob, endpoint(), now);
        reg.endpoint.peer_id = alice.public_id_hex();

        let result = registry.register(reg, loopback_addr()).await;
        assert!(matches!(result, Err(RegistryError::InvalidRegistration(_))));
        assert_eq!(registry.count().await, 0);
    }

    #[tokio::test]
    async fn test_stale_timestamp_is_rejected() {
        let registry = PresenceRegistry::new();
        let alice = identity("alice");
        let ancient = now_secs().saturating_sub(REGISTRATION_FRESHNESS_WINDOW_SECS + 3600);

        let reg = SignedPresenceRegistration::sign(&alice, endpoint(), ancient);
        let result = registry.register(reg, loopback_addr()).await;
        assert!(matches!(result, Err(RegistryError::StaleTimestamp)));
    }

    #[tokio::test]
    async fn test_client_claimed_public_ip_is_never_trusted_but_claimed_port_is_kept() {
        let registry = PresenceRegistry::new();
        let alice = identity("alice");
        let now = now_secs();

        let mut lying_endpoint = endpoint();
        lying_endpoint.public_ip = "203.0.113.99".into();
        lying_endpoint.public_port = 4433;
        let reg = SignedPresenceRegistration::sign(&alice, lying_endpoint, now);

        let real_addr: SocketAddr = "198.51.100.42:33221".parse().unwrap();
        registry.register(reg, real_addr).await.unwrap();

        let retrieved = registry.get_peer(&alice.public_id_hex()).await.unwrap();
        assert_eq!(retrieved.public_ip, "198.51.100.42");
        assert_eq!(retrieved.public_port, 4433);
    }

    #[tokio::test]
    async fn test_directory_registration_and_search() {
        let registry = PresenceRegistry::new();
        let bob = identity("bob");
        let now = now_secs();

        let profile = DirectoryProfile {
            peer_id: bob.public_id_hex(),
            username: "bob".into(),
            display_name: "Bob Martin".into(),
            avatar_data_url: None,
            prekey_bundle_hex: "deadbeef0102".into(),
        };

        let entry = SignedDirectoryEntry::sign(&bob, profile, now);
        registry.register_directory(entry).await.unwrap();

        // Search by username
        let res1 = registry.search_directory("bob").await;
        assert_eq!(res1.len(), 1);
        assert_eq!(res1[0].display_name, "Bob Martin");
        assert_eq!(res1[0].peer_id, bob.public_id_hex());

        // Search by peer_id prefix
        let prefix = &bob.public_id_hex()[..8];
        let res2 = registry.search_directory(prefix).await;
        assert_eq!(res2.len(), 1);
        assert_eq!(res2[0].username, "bob");

        // Search by display name
        let res3 = registry.search_directory("Martin").await;
        assert_eq!(res3.len(), 1);

        // Search non-existent
        let res4 = registry.search_directory("alice").await;
        assert_eq!(res4.len(), 0);
    }

    #[tokio::test]
    async fn test_user_reporting_and_auto_quarantine() {
        let registry = PresenceRegistry::new();
        registry.set_auto_ban_threshold(3).await;

        let bad_user = identity("bad_user");
        let bad_pid = bad_user.public_id_hex();

        let rep1 = identity("reporter_1").public_id_hex();
        let rep2 = identity("reporter_2").public_id_hex();
        let rep3 = identity("reporter_3").public_id_hex();

        // Register bad user
        let profile = DirectoryProfile {
            peer_id: bad_pid.clone(),
            username: "baduser".into(),
            display_name: "Bad Actor".into(),
            avatar_data_url: None,
            prekey_bundle_hex: "deadbeef".into(),
        };
        registry.register_directory(SignedDirectoryEntry::sign(&bad_user, profile, now_secs())).await.unwrap();

        // Report 1
        let auto1 = registry.submit_report(rep1, bad_pid.clone(), "Spam".into(), "spam".into(), "Spam messages".into()).await.unwrap();
        assert!(!auto1);
        assert!(!registry.is_banned(&bad_pid).await);

        // Report 2
        let auto2 = registry.submit_report(rep2, bad_pid.clone(), "Harcèlement".into(), "harassment".into(), "".into()).await.unwrap();
        assert!(!auto2);
        assert!(!registry.is_banned(&bad_pid).await);

        // Report 3 -> Threshold (3) reached -> Auto Quarantine!
        let auto3 = registry.submit_report(rep3, bad_pid.clone(), "Arnaque".into(), "scam".into(), "Asked for money".into()).await.unwrap();
        assert!(auto3);
        assert!(registry.is_banned(&bad_pid).await);

        // Banned user is hidden from search
        let search_res = registry.search_directory("baduser").await;
        assert_eq!(search_res.len(), 0);

        // Overview reflects banned count
        let overview = registry.get_admin_overview().await;
        assert_eq!(overview.banned_users, 1);
        assert_eq!(overview.reports.len(), 3);
    }

    #[tokio::test]
    async fn test_feedback_and_average_rating() {
        let registry = PresenceRegistry::new();
        registry.submit_feedback(Some("user1".into()), 5, "general".into(), "Super appli !".into()).await.unwrap();
        registry.submit_feedback(Some("user2".into()), 4, "ui".into(), "Très fluide".into()).await.unwrap();

        let overview = registry.get_admin_overview().await;
        assert_eq!(overview.total_feedbacks, 2);
        assert!((overview.average_rating - 4.5).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_manual_ban_and_unban() {
        let registry = PresenceRegistry::new();
        let target = "target_peer_123".to_string();

        assert!(!registry.is_banned(&target).await);
        registry.ban_user(target.clone(), "Manuel violation".into()).await.unwrap();
        assert!(registry.is_banned(&target).await);

        registry.unban_user(&target).await.unwrap();
        assert!(!registry.is_banned(&target).await);
    }
}
