//! A standalone, headless bootstrap/relay node for the NOVA Chat DHT.
//!
//! This is not a server that owns anyone's data or conversations — it is a well-known, publicly
//! reachable peer that new devices can dial once to join the Kademlia DHT (the same role a
//! BitTorrent/IPFS bootstrap node plays), and that other peers may use as a circuit-relay-v2
//! relay when they cannot establish a direct connection to each other (e.g. both behind
//! carrier-grade NAT). It never sees plaintext: relayed traffic is already Double Ratchet
//! ciphertext, and DHT records are self-authenticating (see `nova_protocol::SignedDhtPeerRecord`)
//! so this node does not need to be trusted with anything beyond being reachable.
//!
//! Anyone can run one of these — it does not need to be operated by whoever built the app.
//!
//! ## Running
//!
//! ```text
//! NOVA_BOOTSTRAP_LISTEN=/ip4/0.0.0.0/udp/4001/quic-v1 cargo run -p nova-bootstrap --release
//! ```
//!
//! The listen address defaults to `/ip4/0.0.0.0/udp/4001/quic-v1` if unset. The node's identity
//! (a 12-word mnemonic, exactly like a user's — a bootstrap node is just a device with no
//! conversations) is generated on first run and persisted to `NOVA_BOOTSTRAP_IDENTITY_FILE`
//! (default `bootstrap_identity.txt`) so its address stays stable across restarts — a bootstrap
//! peer whose identity changes every restart is useless, since nobody could keep pointing at it.

use nova_crypto::{DeviceIdentity, MnemonicPhrase};
use nova_transport::P2PNode;
use std::path::PathBuf;

const DEFAULT_LISTEN_ADDR: &str = "/ip4/0.0.0.0/udp/4001/quic-v1";
const DEFAULT_IDENTITY_FILE: &str = "bootstrap_identity.txt";

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let listen_addr =
        std::env::var("NOVA_BOOTSTRAP_LISTEN").unwrap_or_else(|_| DEFAULT_LISTEN_ADDR.to_string());
    let identity_file: PathBuf = std::env::var("NOVA_BOOTSTRAP_IDENTITY_FILE")
        .unwrap_or_else(|_| DEFAULT_IDENTITY_FILE.to_string())
        .into();
    // A device cannot reliably learn its own public-facing address by enumerating its own
    // network interfaces — the same reason NAT-bound clients need the DHT's reflexive-address
    // record correction (see nova-transport/dht_record docs). On a multi-interface host (Docker
    // bridges, WSL/Hyper-V virtual switches, VPN adapters) the first "listen address" libp2p
    // happens to report is not necessarily the public one at all. A bootstrap node's whole job
    // is being a known-good address others can dial, so on a real deployment set this explicitly
    // to the VPS's actual public IP/port rather than trusting auto-detection.
    let public_addr_override = std::env::var("NOVA_BOOTSTRAP_PUBLIC_ADDR").ok();

    let identity = load_or_create_identity(&identity_file);
    let nova_peer_id = identity.public_id_hex();

    let node = P2PNode::start(identity, &listen_addr)
        .await
        .expect("failed to start bootstrap node — is the listen address already in use?");

    if let Err(e) = node.announce_presence().await {
        tracing::warn!("Initial presence announcement failed (will retry on the usual heartbeat): {e}");
    }

    println!("NOVA bootstrap/relay node is running.");
    println!("  nova peer_id      : {nova_peer_id}");
    for addr in node.listen_addrs() {
        println!("  local listen addr : {addr}");
    }
    let ipv6_full_addr = node
        .full_listen_addrs()
        .into_iter()
        .find(|a| a.to_string().starts_with("/ip6/"));
    match &public_addr_override {
        Some(addr) => {
            println!("  give this to a client for first contact (NOVA_BOOTSTRAP_PUBLIC_ADDR):");
            println!("  {addr}/p2p/{}", node.local_libp2p_peer_id());
            if let Some(ipv6) = ipv6_full_addr {
                println!("  IPv6 also works directly, no override needed — this works too:");
                println!("  {ipv6}");
            }
        }
        None => {
            println!("  give this to a client for first contact:");
            for addr in node.full_listen_addrs() {
                println!("  {addr}");
            }
            println!();
            println!(
                "  WARNING: no NOVA_BOOTSTRAP_PUBLIC_ADDR was set for the IPv4 address above. On a \
                 host with more than one network interface (common on VPS providers with a private \
                 + public NIC, or any machine running Docker/WSL/a VPN) that address may not be the \
                 one other peers can actually reach. Set \
                 NOVA_BOOTSTRAP_PUBLIC_ADDR=/ip4/<public-ip>/udp/<port>/quic-v1 explicitly for a \
                 real deployment — unless the IPv6 address above (if listed) already works for your \
                 clients, which needs no such override at all."
            );
        }
    }
    println!();
    println!("Press Ctrl+C to stop.");

    tokio::signal::ctrl_c().await.expect("failed to listen for Ctrl+C");
    println!("Shutting down.");
}

fn load_or_create_identity(path: &PathBuf) -> DeviceIdentity {
    if let Ok(phrase) = std::fs::read_to_string(path) {
        let mnemonic = MnemonicPhrase::from_phrase(phrase.trim())
            .expect("identity file exists but does not contain a valid 12-word mnemonic");
        return DeviceIdentity::from_mnemonic(&mnemonic, "bootstrap")
            .expect("failed to derive identity from saved mnemonic");
    }

    let mnemonic = MnemonicPhrase::generate().expect("failed to generate a fresh mnemonic");
    std::fs::write(path, mnemonic.as_str())
        .unwrap_or_else(|e| panic!("failed to persist bootstrap identity to {path:?}: {e}"));
    println!("Generated a new bootstrap identity and saved it to {path:?}.");
    println!("Keep this file if you want this node's address to stay the same across restarts.");
    DeviceIdentity::from_mnemonic(&mnemonic, "bootstrap").expect("failed to derive identity from fresh mnemonic")
}
