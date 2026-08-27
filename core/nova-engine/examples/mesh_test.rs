//! Standalone THREE-process LAN mesh connectivity check — extends `lan_test.rs`'s two-process
//! design to a full 3-way mesh, exercising the exact production code path (`P2PNode::start` on a
//! `0.0.0.0` wildcard listen address, mDNS discovery, DHT presence announce/lookup, invitation
//! rendezvous-address auto-dial, real X3DH-encrypted message delivery) across three independent
//! OS processes on the same machine/LAN — i.e. what three separate "NOVA Chat" desktop instances
//! actually do, without needing to drive three Tauri/WebView windows by hand.
//!
//! Run as three separate OS processes on the same machine (any order — each one waits for the
//! other two's invitation files before it can add them as contacts):
//!
//!   cargo run --example mesh_test -- node-a
//!   cargo run --example mesh_test -- node-b
//!   cargo run --example mesh_test -- node-c
//!
//! Each process: creates a fresh account, starts its P2P node, writes its own signed invitation
//! to a per-role temp file, waits for the other two roles' invitation files, adds each as a
//! contact, sends one unique message to each, then waits until it has received one message from
//! each of the other two AND both of its own outgoing messages show `Delivered`. Prints
//! `MESH_TEST_RESULT: <role> OK` and exits 0 on full success, or `MESH_TEST_RESULT: <role> FAIL
//! <reason>` and exits 1 on timeout/error — the overall mesh test passed only if all three
//! processes print OK.
use nova_crypto::{DeviceIdentity, MnemonicPhrase};
use nova_engine::NovaEngine;
use std::sync::Arc;
use std::time::Duration;

const ROLES: [&str; 3] = ["node-a", "node-b", "node-c"];
const LISTEN_ADDR: &str = "/ip4/0.0.0.0/udp/0/quic-v1";
const DISCOVERY_TIMEOUT: Duration = Duration::from_secs(60);
const DELIVERY_TIMEOUT: Duration = Duration::from_secs(60);

fn invitation_file(role: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("nova_mesh_test_{role}_invitation.txt"))
}

fn db_file(role: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("nova_mesh_test_{role}.db"))
}

fn fail(role: &str, reason: &str) -> ! {
    println!("MESH_TEST_RESULT: {role} FAIL {reason}");
    std::process::exit(1);
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::new(
            "info,nova_engine=debug,nova_transport=debug,libp2p_swarm=info,libp2p_quic=info,quinn=warn",
        ))
        .init();

    let role = std::env::args().nth(1).unwrap_or_default();
    if !ROLES.contains(&role.as_str()) {
        eprintln!("usage: mesh_test <{}>", ROLES.join("|"));
        std::process::exit(2);
    }
    let peers: Vec<&str> = ROLES.iter().copied().filter(|r| *r != role).collect();

    // Always a fresh identity/db and a fresh invitation file: this harness orchestrates all three
    // processes itself in one run, so there is no "resume across a human tester's pacing" need
    // (unlike lan_test.rs) — starting clean every run avoids stale contacts/conversations from a
    // previous run confusing the pass/fail check.
    let _ = std::fs::remove_file(db_file(&role));
    let _ = std::fs::remove_file(invitation_file(&role));

    let engine = Arc::new(NovaEngine::new(db_file(&role).to_str().unwrap(), "mesh-test-pass").unwrap());
    let (peer_id, mnemonic) = engine.create_account(&role).await.expect("create_account failed");
    eprintln!("[{role}] account ready, peer_id={peer_id}");

    let identity = DeviceIdentity::from_mnemonic(&MnemonicPhrase::from_phrase(&mnemonic).unwrap(), &role).unwrap();
    let node = nova_transport::P2PNode::start(identity, LISTEN_ADDR)
        .await
        .expect("P2PNode::start failed");
    eprintln!("[{role}] network started, listen_addrs={:?}", node.full_listen_addrs());

    if let Err(e) = node.announce_presence().await {
        eprintln!("[{role}] initial announce_presence failed (expected on a cold start): {e}");
    }
    engine.attach_network(node.clone()).await;

    let uri = engine
        .get_own_invitation_uri(Some(3600))
        .await
        .unwrap_or_else(|e| fail(&role, &format!("get_own_invitation_uri failed: {e}")));
    std::fs::write(invitation_file(&role), &uri).unwrap();
    eprintln!("[{role}] wrote invitation to {}", invitation_file(&role).display());

    // Wait for, then add, both other peers as contacts.
    let mut contact_peer_ids = Vec::new();
    for peer_role in &peers {
        let deadline = tokio::time::Instant::now() + DISCOVERY_TIMEOUT;
        let peer_uri = loop {
            if let Ok(s) = std::fs::read_to_string(invitation_file(peer_role)) {
                if !s.trim().is_empty() {
                    break s;
                }
            }
            if tokio::time::Instant::now() >= deadline {
                fail(&role, &format!("timeout waiting for {peer_role}'s invitation file"));
            }
            tokio::time::sleep(Duration::from_millis(300)).await;
        };
        let contact = engine
            .add_contact(peer_role, peer_role, peer_uri.trim().as_bytes())
            .await
            .unwrap_or_else(|e| fail(&role, &format!("add_contact({peer_role}) failed: {e}")));
        eprintln!("[{role}] added {peer_role} as contact: peer_id={}", contact.peer_id);
        contact_peer_ids.push((peer_role.to_string(), contact.peer_id));
    }

    // Send one unique message to each peer.
    for (peer_role, peer_id) in &contact_peer_ids {
        let conv_id = format!("conv_{peer_id}");
        let text = format!("hello from {role} to {peer_role}");
        engine
            .send_message(&conv_id, peer_id, &text)
            .await
            .unwrap_or_else(|e| fail(&role, &format!("send_message to {peer_role} failed: {e}")));
        eprintln!("[{role}] message queued for {peer_role}");
    }

    // Wait until: (a) an incoming message has arrived from each peer, and (b) each outgoing
    // message to each peer shows Delivered — i.e. full round-trip confirmation both ways.
    let deadline = tokio::time::Instant::now() + DELIVERY_TIMEOUT;
    loop {
        let mut all_received = true;
        let mut all_delivered = true;
        for (peer_role, peer_id) in &contact_peer_ids {
            let conv_id = format!("conv_{peer_id}");
            let msgs = engine.get_messages(&conv_id).unwrap_or_default();
            let received = msgs.iter().any(|m| {
                !m.is_outgoing && m.text_content == format!("hello from {peer_role} to {role}")
            });
            let delivered = msgs
                .iter()
                .any(|m| m.is_outgoing && matches!(m.status, nova_storage::DbMessageStatus::Delivered));
            all_received &= received;
            all_delivered &= delivered;
        }
        if all_received && all_delivered {
            println!("MESH_TEST_RESULT: {role} OK");
            std::process::exit(0);
        }
        if tokio::time::Instant::now() >= deadline {
            fail(
                &role,
                &format!("timeout waiting for full round-trip (received={all_received}, delivered={all_delivered})"),
            );
        }
        tokio::time::sleep(Duration::from_millis(300)).await;
    }
}
