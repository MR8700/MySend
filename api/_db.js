const { Pool } = require('pg');

let pool = null;
let schemaInitialized = false;

function getConnectionString() {
  return (
    process.env.DATABASE_URL ||
    process.env.POSTGRES_URL ||
    process.env.POSTGRES_PRISMA_URL ||
    process.env.POSTGRES_URL_NON_POOLING
  );
}

function getPool() {
  if (!pool) {
    const connectionString = getConnectionString();
    if (!connectionString) {
      throw new Error(
        'Neither DATABASE_URL nor POSTGRES_URL is configured. Please enable the Supabase integration on Vercel.'
      );
    }
    pool = new Pool({
      connectionString,
      ssl: {
        rejectUnauthorized: false,
      },
      max: 5,
      idleTimeoutMillis: 30000,
      connectionTimeoutMillis: 5000,
    });
  }
  return pool;
}

async function ensureSchema() {
  if (schemaInitialized) return;
  const p = getPool();
  try {
    await p.query(`
      CREATE TABLE IF NOT EXISTS directory_profiles (
          peer_id VARCHAR(64) PRIMARY KEY,
          username VARCHAR(64) NOT NULL,
          display_name VARCHAR(128) NOT NULL,
          avatar_data_url TEXT,
          prekey_bundle_hex TEXT NOT NULL,
          registered_at BIGINT NOT NULL,
          last_updated_at BIGINT NOT NULL
      );
      CREATE INDEX IF NOT EXISTS idx_dir_username ON directory_profiles (LOWER(username));
      CREATE INDEX IF NOT EXISTS idx_dir_display_name ON directory_profiles (LOWER(display_name));

      CREATE TABLE IF NOT EXISTS presence_entries (
          peer_id VARCHAR(64) PRIMARY KEY,
          public_ip VARCHAR(64) NOT NULL,
          public_port INTEGER NOT NULL,
          local_ip VARCHAR(64),
          local_port INTEGER,
          last_seen_at BIGINT NOT NULL
      );
      CREATE INDEX IF NOT EXISTS idx_presence_last_seen ON presence_entries (last_seen_at);

      CREATE TABLE IF NOT EXISTS relay_queue (
          id BIGSERIAL PRIMARY KEY,
          target_peer_id VARCHAR(64) NOT NULL,
          payload BYTEA NOT NULL,
          received_at BIGINT NOT NULL
      );
      CREATE INDEX IF NOT EXISTS idx_relay_target ON relay_queue (target_peer_id);
      CREATE INDEX IF NOT EXISTS idx_relay_received ON relay_queue (received_at);

      CREATE TABLE IF NOT EXISTS user_reports (
          id VARCHAR(64) PRIMARY KEY,
          reporter_peer_id VARCHAR(64) NOT NULL,
          target_peer_id VARCHAR(64) NOT NULL,
          reason TEXT NOT NULL,
          category VARCHAR(32) NOT NULL,
          comment TEXT NOT NULL,
          created_at BIGINT NOT NULL
      );

      CREATE TABLE IF NOT EXISTS user_feedbacks (
          id VARCHAR(64) PRIMARY KEY,
          sender_peer_id VARCHAR(64),
          rating SMALLINT NOT NULL,
          category VARCHAR(32) NOT NULL,
          comment TEXT NOT NULL,
          created_at BIGINT NOT NULL
      );

      CREATE TABLE IF NOT EXISTS banned_users (
          peer_id VARCHAR(64) PRIMARY KEY,
          reason TEXT NOT NULL,
          banned_at BIGINT NOT NULL,
          report_count INTEGER NOT NULL DEFAULT 0,
          is_automatic BOOLEAN NOT NULL DEFAULT FALSE
      );
    `);
    schemaInitialized = true;
  } catch (err) {
    console.warn('Auto-schema creation notice:', err.message);
  }
}

module.exports = { getPool, ensureSchema, getConnectionString };
