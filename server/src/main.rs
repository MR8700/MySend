use nova_server::run_server;
use tracing::{error, info};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    info!("Starting NOVA Sovereign Discovery & Ephemeral Relay Server...");

    let bind_addr = std::env::var("NOVA_SERVER_BIND").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    if let Err(e) = run_server(&bind_addr).await {
        error!("Server exited: {e}");
    }
}
