-- =============================================================================
-- NOVA / MySend - Supabase & PostgreSQL Automated Schema & Indexing
-- =============================================================================
-- This file defines the persistent tables, indexes, and constraints for
-- high-speed E2EE directory search, presence registration, blind relay queue,
-- and moderation records.
--
-- Note: The Rust server (nova-server) also executes these statements automatically
-- on startup if they do not already exist.
-- =============================================================================

-- 1. Directory Profiles (Public prekey bundles, usernames, avatars)
CREATE TABLE IF NOT EXISTS directory_profiles (
    peer_id VARCHAR(64) PRIMARY KEY,
    username VARCHAR(64) NOT NULL,
    display_name VARCHAR(128) NOT NULL,
    avatar_data_url TEXT,
    prekey_bundle_hex TEXT NOT NULL,
    registered_at BIGINT NOT NULL,
    last_updated_at BIGINT NOT NULL
);

-- Fast lookup indexes for case-insensitive search
CREATE INDEX IF NOT EXISTS idx_dir_username ON directory_profiles (LOWER(username));
CREATE INDEX IF NOT EXISTS idx_dir_display_name ON directory_profiles (LOWER(display_name));

-- 2. Presence Entries (Ephemeral peer socket addresses & timestamps)
CREATE TABLE IF NOT EXISTS presence_entries (
    peer_id VARCHAR(64) PRIMARY KEY,
    public_ip VARCHAR(64) NOT NULL,
    public_port INTEGER NOT NULL,
    local_ip VARCHAR(64),
    local_port INTEGER,
    last_seen_at BIGINT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_presence_last_seen ON presence_entries (last_seen_at);

-- 3. Blind Relay Queue (Encrypted opaque blobs queued for offline peers)
CREATE TABLE IF NOT EXISTS relay_queue (
    id BIGSERIAL PRIMARY KEY,
    target_peer_id VARCHAR(64) NOT NULL,
    payload BYTEA NOT NULL,
    received_at BIGINT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_relay_target ON relay_queue (target_peer_id);
CREATE INDEX IF NOT EXISTS idx_relay_received ON relay_queue (received_at);

-- 4. User Reports (Abuse prevention & moderation)
CREATE TABLE IF NOT EXISTS user_reports (
    id VARCHAR(64) PRIMARY KEY,
    reporter_peer_id VARCHAR(64) NOT NULL,
    target_peer_id VARCHAR(64) NOT NULL,
    reason TEXT NOT NULL,
    category VARCHAR(32) NOT NULL,
    comment TEXT NOT NULL,
    created_at BIGINT NOT NULL
);

-- 5. User Feedbacks
CREATE TABLE IF NOT EXISTS user_feedbacks (
    id VARCHAR(64) PRIMARY KEY,
    sender_peer_id VARCHAR(64),
    rating SMALLINT NOT NULL,
    category VARCHAR(32) NOT NULL,
    comment TEXT NOT NULL,
    created_at BIGINT NOT NULL
);

-- 6. Banned Users
CREATE TABLE IF NOT EXISTS banned_users (
    peer_id VARCHAR(64) PRIMARY KEY,
    reason TEXT NOT NULL,
    banned_at BIGINT NOT NULL,
    report_count INTEGER NOT NULL DEFAULT 0,
    is_automatic BOOLEAN NOT NULL DEFAULT FALSE
);
