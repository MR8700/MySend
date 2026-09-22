use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tracing::info;

pub async fn init_db(database_url: &str) -> Result<PgPool, sqlx::Error> {
    info!("Connecting to PostgreSQL / Supabase database...");
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(database_url)
        .await?;

    info!("Initializing automated database schema and indexes...");
    init_schema(&pool).await?;
    info!("PostgreSQL / Supabase schema and indexes verified successfully.");

    Ok(pool)
}

pub async fn init_schema(pool: &PgPool) -> Result<(), sqlx::Error> {
    let schema_sql = r#"
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
    "#;

    sqlx::raw_sql(schema_sql).execute(pool).await?;
    Ok(())
}
