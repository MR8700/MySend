//! Standalone, disposable probe: does the embedded Tor client (`arti-client`) actually manage to
//! bootstrap on this machine, and how long does it really take? Deliberately NOT a `#[test]` —
//! a plain `fn main()` process is trivial to kill from outside (`taskkill`/`Stop-Process`) without
//! any of the "a tokio test runtime's Drop blocks forever on a still-running spawned task" hazard
//! that made this exact question hang the whole `nova-transport` test binary earlier.
//!
//! Run with: `cargo run -p nova-transport --example tor_bootstrap_probe`

use nova_transport::TorManager;
use nova_transport::tor::TorConfig;
use std::time::{Duration, Instant};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_env_filter("info,arti_client=debug,tor_dirmgr=debug,tor_guardmgr=debug").init();

    let state_dir = std::env::temp_dir().join("nova_tor_bootstrap_probe");
    println!("state_dir = {state_dir:?}");

    let manager = TorManager::new(TorConfig { enabled: true, ..Default::default() }, [7u8; 32], state_dir);

    let start = Instant::now();
    println!("Dialing example.com:443 through a fresh Tor circuit (this forces a bootstrap)...");

    // 8-minute ceiling on the probe itself — generous, but this whole process is disposable and
    // externally killable, unlike the earlier test-binary hang.
    match tokio::time::timeout(Duration::from_secs(480), manager.connect_onion_stream("example.com", 443)).await {
        Ok(Ok(_stream)) => println!("SUCCESS: Tor circuit established and connection made in {:?}", start.elapsed()),
        Ok(Err(e)) => println!("FAILED after {:?}: {e}", start.elapsed()),
        Err(_) => println!("TIMED OUT after {:?} (480s ceiling)", start.elapsed()),
    }
}
