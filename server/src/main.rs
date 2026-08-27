use nova_server::run_server;
use tracing::{error, info};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    info!("Starting NOVA Sovereign Discovery & Ephemeral Relay Server...");

    // Render (and most PaaS hosts) assign the public port at deploy time via `$PORT` and expect
    // the app to bind to it — `NOVA_SERVER_BIND` is checked first only so a full explicit
    // "ip:port" override still works for local/VPS runs that set it directly.
    let bind_addr = std::env::var("NOVA_SERVER_BIND").unwrap_or_else(|_| {
        let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
        format!("0.0.0.0:{port}")
    });
    if let Err(e) = run_server(&bind_addr).await {
        error!("Server exited: {e}");
    }
}
