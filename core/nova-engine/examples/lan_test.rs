//! Standalone two-process LAN connectivity check, exercising the exact production code path
//! (`NovaEngine::attach_network`, `P2PNode::start` on a `0.0.0.0` wildcard listen address, mDNS
//! discovery, DHT presence announce/lookup, real message delivery) without needing the Tauri UI
//! or a second physical device. Run as two separate OS processes on the same machine/LAN:
//!
//!   cargo run --example lan_test -- bob
//!   cargo run --example lan_test -- alice
//!
//! `bob` writes its invitation URI to a temp file as soon as its network starts; `alice` waits
//! for that file, adds bob as a contact from it (exactly what pasting an invitation link into
//! "Lien sécurisé ou code" does), and sends one message. `bob` polls its own conversations for
//! that message and prints `LAN_TEST_RESULT: RECEIVED "<text>"` on success, or times out.
use nova_crypto::{DeviceIdentity, MnemonicPhrase};
use nova_engine::NovaEngine;
use std::sync::Arc;
use std::time::Duration;

const LISTEN_ADDR: &str = "/ip4/0.0.0.0/udp/0/quic-v1";
const TEST_MESSAGE: &str = "hello from lan_test";

fn invitation_file() -> std::path::PathBuf {
    std::env::temp_dir().join("nova_lan_test_bob_invitation.txt")
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::new(
            "debug,libp2p_swarm=info,libp2p_quic=info,quinn=warn",
        ))
        .init();
    let role = std::env::args().nth(1).unwrap_or_default();
    // bob's identity must survive a restart (e.g. its receive window timing out while the human
    // tester is still navigating a real device's UI) so a contact added from an earlier run's
    // invitation still matches — only wipe on first-ever run (no existing account to keep).
    let db_path = std::env::temp_dir().join(format!("nova_lan_test_{role}.db"));
    let fresh_account = std::env::args().any(|a| a == "--fresh");
    if fresh_account {
        let _ = std::fs::remove_file(&db_path);
    }
    let inv_path = invitation_file();
    if role == "bob" {
        let _ = std::fs::remove_file(&inv_path);
    }

    let engine = Arc::new(NovaEngine::new(db_path.to_str().unwrap(), "lan-test-pass").unwrap());
    let (peer_id, mnemonic) = match engine.create_account(&role).await {
        Ok(v) => v,
        Err(_) => {
            // Identity already exists on disk from an earlier run — resume it instead, so a
            // contact added elsewhere from that earlier run's invitation still matches this peer_id.
            let resumed = engine.try_resume_session().await.unwrap().expect("no identity to resume");
            (resumed.0, resumed.1)
        }
    };
    eprintln!("[{role}] account ready, peer_id={peer_id}");
    // Only for standing up a matching Tor hidden service (see tor_hs_keygen) — never do this for
    // a real account, the mnemonic is the entire key to the identity.
    std::fs::write(std::env::temp_dir().join(format!("nova_lan_test_{role}_mnemonic.txt")), &mnemonic).ok();

    let identity = DeviceIdentity::from_mnemonic(&MnemonicPhrase::from_phrase(&mnemonic).unwrap(), &role).unwrap();
    let node = nova_transport::P2PNode::start(identity, LISTEN_ADDR)
        .await
        .expect("P2PNode::start failed");
    eprintln!("[{role}] network started, listen_addrs={:?}", node.full_listen_addrs());

    // Mirrors ui/src-tauri's start_network_once: one immediate best-effort announce, then
    // attach_network's own periodic re-announce task (added this session) takes over from there.
    if let Err(e) = node.announce_presence().await {
        eprintln!("[{role}] initial announce_presence failed (expected on a cold start): {e}");
    }
    engine.attach_network(node.clone()).await;

    if role == "bob" {
        let uri = engine.get_own_invitation_uri(Some(3600)).await.unwrap();
        std::fs::write(&inv_path, &uri).unwrap();
        eprintln!("[bob] wrote invitation to {}", inv_path.display());

        let deadline = tokio::time::Instant::now() + Duration::from_secs(1800);
        loop {
            if tokio::time::Instant::now() >= deadline {
                println!("LAN_TEST_RESULT: TIMEOUT waiting for incoming message");
                std::process::exit(1);
            }
            let convs = engine.get_conversations().unwrap_or_default();
            if let Some(conv) = convs.first() {
                let msgs = engine.get_messages(&conv.id).unwrap_or_default();
                if let Some(m) = msgs.iter().find(|m| !m.is_outgoing) {
                    println!("LAN_TEST_RESULT: RECEIVED \"{}\"", m.text_content);
                    std::process::exit(0);
                }
            }
            tokio::time::sleep(Duration::from_millis(300)).await;
        }
    } else {
        eprintln!("[alice] waiting for bob's invitation file...");
        let deadline = tokio::time::Instant::now() + Duration::from_secs(20);
        let uri = loop {
            if let Ok(s) = std::fs::read_to_string(&inv_path) {
                if !s.trim().is_empty() {
                    break s;
                }
            }
            if tokio::time::Instant::now() >= deadline {
                println!("LAN_TEST_RESULT: TIMEOUT waiting for bob's invitation file");
                std::process::exit(1);
            }
            tokio::time::sleep(Duration::from_millis(300)).await;
        };

        let contact = engine.add_contact("bob", "Bob", uri.trim().as_bytes()).await.unwrap();
        eprintln!("[alice] added bob as contact: peer_id={}", contact.peer_id);

        let conv_id = format!("conv_{}", contact.peer_id);
        engine.send_message(&conv_id, &contact.peer_id, TEST_MESSAGE).await.unwrap();
        eprintln!("[alice] message queued, waiting for delivery confirmation...");

        let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
        loop {
            if tokio::time::Instant::now() >= deadline {
                println!("LAN_TEST_RESULT: TIMEOUT waiting for delivery");
                std::process::exit(1);
            }
            let msgs = engine.get_messages(&conv_id).unwrap_or_default();
            if let Some(m) = msgs.iter().find(|m| m.is_outgoing) {
                if matches!(m.status, nova_storage::DbMessageStatus::Delivered) {
                    println!("LAN_TEST_RESULT: DELIVERED");
                    std::process::exit(0);
                }
            }
            tokio::time::sleep(Duration::from_millis(300)).await;
        }
    }
}
