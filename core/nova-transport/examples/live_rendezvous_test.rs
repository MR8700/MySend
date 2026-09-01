use nova_crypto::{DeviceIdentity, MnemonicPhrase};
use nova_protocol::{
    DirectoryProfile, PeerEndpoint, SignedDirectoryEntry, SignedDrainRequest,
    SignedPresenceRegistration,
};
use nova_transport::seed_nodes::{fetch_remote_nodes_config, MultiFallbackPool};
use nova_transport::udp_fallback::UdpFallbackClient;
use std::time::{SystemTime, UNIX_EPOCH};

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[tokio::main]
async fn main() {
    println!("============================================================");
    println!("  NOVA LIVE RENDEZVOUS & MULTI-DEVICE COMMUNICATION TEST");
    println!("============================================================");

    // 1. Fetch remote config from GitHub Raw
    println!("\n[1] Fetching dynamic node config from GitHub...");
    let config = fetch_remote_nodes_config().await;
    println!("    Found {} fallback servers:", config.fallback_servers.len());
    for s in &config.fallback_servers {
        println!("    - {} ({}) enabled={}", s.url, s.name, s.enabled);
    }

    let pool = MultiFallbackPool::from_config(&config);
    println!("    Active Primary Relay URL: {}", pool.primary_url());

    let server_url = pool.primary_url();
    let client_a = UdpFallbackClient::new(&server_url);
    let client_b = UdpFallbackClient::new(&server_url);

    // 2. Generate Device Identities
    println!("\n[2] Generating identities for Device A (Samsung) & Device B (PC)...");
    let mn_a = MnemonicPhrase::generate().expect("mn_a");
    let mn_b = MnemonicPhrase::generate().expect("mn_b");
    let dev_a = DeviceIdentity::from_mnemonic(&mn_a, "samsung_user").expect("dev_a");
    let dev_b = DeviceIdentity::from_mnemonic(&mn_b, "pc_user").expect("dev_b");

    let id_a = dev_a.public_id_hex();
    let id_b = dev_b.public_id_hex();

    println!("    Device A (Samsung) PeerID: {}", id_a);
    println!("    Device B (PC)      PeerID: {}", id_b);

    // 3. Device A Registers Presence & Directory Profile
    println!("\n[3] Device A (Samsung) registering presence & directory on relay server...");
    let endpoint_a = PeerEndpoint {
        peer_id: id_a.clone(),
        public_ip: "127.0.0.1".to_string(),
        public_port: 45000,
        local_ip: Some("192.168.1.50".to_string()),
        local_port: Some(45000),
    };
    let presence_a = SignedPresenceRegistration::sign(&dev_a, endpoint_a, now_secs());
    match client_a.register_presence_signed(presence_a).await {
        Ok(()) => println!("    -> [OK] Device A presence registered successfully!"),
        Err(e) => println!("    -> [ERR] Device A presence registration failed: {e}"),
    }

    let profile_a = DirectoryProfile {
        peer_id: id_a.clone(),
        username: "samsung_test".to_string(),
        display_name: "Samsung Galaxy Test".to_string(),
        avatar_data_url: None,
        prekey_bundle_hex: "0102030405060708090a0b0c0d0e0f10".to_string(),
    };
    let signed_entry_a = SignedDirectoryEntry::sign(&dev_a, profile_a, now_secs());
    match client_a.register_directory_signed(signed_entry_a).await {
        Ok(()) => println!("    -> [OK] Device A directory profile registered!"),
        Err(e) => println!("    -> [ERR] Device A directory registration failed: {e}"),
    }

    // 4. Device B (PC) searches for Device A by username prefix
    println!("\n[4] Device B (PC) searching for Device A on the rendezvous server...");
    match client_b.search_directory("samsung").await {
        Ok(results) => {
            println!("    Search results for 'samsung': {} result(s) found", results.len());
            for r in &results {
                println!("    - Peer: {} | Display: {} | User: @{}", r.peer_id, r.display_name, r.username);
            }
        }
        Err(e) => println!("    -> [ERR] Search directory failed: {e}"),
    }

    match client_b.lookup(&id_a).await {
        Ok(Some(ep)) => println!("    -> [OK] Lookup for Device A succeeded: IP={}:{} Local={}:{:?}", ep.public_ip, ep.public_port, ep.local_ip.unwrap_or_default(), ep.local_port),
        Ok(None) => println!("    -> [WARN] Lookup for Device A returned None"),
        Err(e) => println!("    -> [ERR] Lookup failed: {e}"),
    }

    // 5. Device B (PC) sends relay message packets to Device A (Samsung) via RelayForward
    println!("\n[5] Device B (PC) forwarding encrypted data payload to Device A via Rendezvous Relay...");
    let test_payload_1 = b"NOVA_TEST_MESSAGE_PACKET_#1_FROM_PC".to_vec();
    let test_payload_2 = b"NOVA_TEST_MESSAGE_PACKET_#2_FROM_PC".to_vec();

    match client_b.relay_forward(&id_a, test_payload_1.clone()).await {
        Ok(()) => println!("    -> [OK] Packet #1 relayed to Device A"),
        Err(e) => println!("    -> [ERR] Failed to relay Packet #1: {e}"),
    }
    match client_b.relay_forward(&id_a, test_payload_2.clone()).await {
        Ok(()) => println!("    -> [OK] Packet #2 relayed to Device A"),
        Err(e) => println!("    -> [ERR] Failed to relay Packet #2: {e}"),
    }

    // 6. Device A (Samsung) drains messages from Relay
    println!("\n[6] Device A (Samsung) polling & draining queued packets from Rendezvous Relay...");
    let drain_req = SignedDrainRequest::sign(&dev_a, now_secs());
    match client_a.drain_incoming_signed(drain_req).await {
        Ok(packets) => {
            println!("    -> [OK] Drained {} packet(s) successfully:", packets.len());
            for (i, p) in packets.iter().enumerate() {
                let text = String::from_utf8_lossy(p);
                println!("       Packet {}: \"{}\" ({} bytes)", i + 1, text, p.len());
            }
            if packets.len() == 2 && packets[0] == test_payload_1 && packets[1] == test_payload_2 {
                println!("\n>>> ALL TESTS PASSED: Device-to-Device Rendezvous Communication is 100% OPERATIONAL! <<<");
            } else {
                println!("\n>>> WARNING: Received packets count or content mismatch! <<<");
            }
        }
        Err(e) => println!("    -> [ERR] Drain request failed: {e}"),
    }
    println!("============================================================\n");
}
