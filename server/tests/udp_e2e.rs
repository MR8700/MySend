//! End-to-end test of the discovery/relay server over a real UDP socket: this is the network
//! surface a hostile peer actually talks to, so it is exercised here rather than only through
//! the in-process unit tests in `registry.rs`/`relay.rs`.

use nova_crypto::{DeviceIdentity, MnemonicPhrase};
use nova_protocol::{PeerEndpoint, ServerRequest, ServerResponse, SignedDrainRequest, SignedPresenceRegistration};
use nova_server::run_server;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::net::UdpSocket;
use tokio::time::timeout;

fn identity(name: &str) -> DeviceIdentity {
    let mnemonic = MnemonicPhrase::generate().unwrap();
    DeviceIdentity::from_mnemonic(&mnemonic, name).unwrap()
}

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}

async fn start_test_server() -> String {
    // Port 0 lets the OS assign a free port; retry a couple of fixed high ports as a fallback
    // for environments where UdpSocket::bind(":0") behaves oddly is unnecessary here since we
    // bind directly and then read the resolved local address back out via the OS.
    let probe = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let addr = probe.local_addr().unwrap();
    drop(probe);
    let bind_addr = addr.to_string();

    let spawn_addr = bind_addr.clone();
    tokio::spawn(async move {
        let _ = run_server(&spawn_addr).await;
    });

    // Give the server a moment to bind before the test starts sending.
    tokio::time::sleep(Duration::from_millis(150)).await;
    bind_addr
}

async fn roundtrip(client: &UdpSocket, server_addr: &str, request: &ServerRequest) -> ServerResponse {
    let bytes = request.to_bytes().unwrap();
    client.send_to(&bytes, server_addr).await.unwrap();

    let mut buf = vec![0u8; 16 * 1024];
    let (len, _) = timeout(Duration::from_secs(2), client.recv_from(&mut buf))
        .await
        .expect("server did not respond in time")
        .unwrap();

    ServerResponse::from_bytes(&buf[..len]).unwrap()
}

#[tokio::test]
async fn test_register_then_lookup_over_real_udp_socket() {
    let server_addr = start_test_server().await;
    let client = UdpSocket::bind("127.0.0.1:0").await.unwrap();

    let alice = identity("alice");
    // Alice's device cannot know its own post-NAT public IP, so it claims a plausible but
    // fictitious one here — the server must ignore this and record the UDP source IP it actually
    // observed instead (a STUN-style reflexive-address correction, see
    // `PresenceRegistry::register`). The claimed port (51820, standing in for Alice's actual QUIC
    // endpoint port, which is a different socket than this signaling client) is kept as-is.
    let endpoint = PeerEndpoint {
        peer_id: String::new(),
        public_ip: "203.0.113.5".into(),
        public_port: 51820,
        local_ip: Some("10.1.1.2".into()),
        local_port: Some(51820),
    };
    let registration = SignedPresenceRegistration::sign(&alice, endpoint, now_secs());
    let client_local_addr = client.local_addr().unwrap();

    let register_resp = roundtrip(&client, &server_addr, &ServerRequest::Register(registration)).await;
    assert!(matches!(register_resp, ServerResponse::Registered));

    let lookup_resp = roundtrip(
        &client,
        &server_addr,
        &ServerRequest::Lookup {
            peer_id: alice.public_id_hex(),
        },
    )
    .await;

    match lookup_resp {
        ServerResponse::LookupResult(Some(found)) => {
            assert_eq!(found.public_ip, client_local_addr.ip().to_string());
            assert_ne!(found.public_ip, "203.0.113.5", "claimed IP must never be trusted verbatim");
            assert_eq!(found.public_port, 51820);
        }
        other => panic!("expected a lookup hit, got {other:?}"),
    }
}

#[tokio::test]
async fn test_spoofed_registration_is_rejected_over_real_udp_socket() {
    let server_addr = start_test_server().await;
    let client = UdpSocket::bind("127.0.0.1:0").await.unwrap();

    let alice = identity("alice");
    let bob = identity("bob");

    let endpoint = PeerEndpoint {
        peer_id: String::new(),
        public_ip: "198.51.100.9".into(),
        public_port: 4433,
        local_ip: None,
        local_port: None,
    };
    // Bob signs the registration honestly, then an attacker relabels it as Alice's — the
    // server must catch this via signature verification, not merely trust the peer_id field.
    let mut hijack = SignedPresenceRegistration::sign(&bob, endpoint, now_secs());
    hijack.endpoint.peer_id = alice.public_id_hex();

    let resp = roundtrip(&client, &server_addr, &ServerRequest::Register(hijack)).await;
    assert!(matches!(resp, ServerResponse::Error(_)));

    let lookup_resp = roundtrip(
        &client,
        &server_addr,
        &ServerRequest::Lookup {
            peer_id: alice.public_id_hex(),
        },
    )
    .await;
    assert!(matches!(lookup_resp, ServerResponse::LookupResult(None)));
}

#[tokio::test]
async fn test_relay_forward_and_authenticated_drain_over_real_udp_socket() {
    let server_addr = start_test_server().await;
    let client = UdpSocket::bind("127.0.0.1:0").await.unwrap();

    let bob = identity("bob");
    let bob_peer_id = bob.public_id_hex();

    let forward_resp = roundtrip(
        &client,
        &server_addr,
        &ServerRequest::RelayForward {
            target_peer_id: bob_peer_id.clone(),
            payload: b"opaque_ciphertext_blob".to_vec(),
        },
    )
    .await;
    assert!(matches!(forward_resp, ServerResponse::RelayForwarded));

    // An attacker who merely knows Bob's peer_id (public information) cannot drain his queue.
    let eve = identity("eve");
    let mut forged_drain = SignedDrainRequest::sign(&eve, now_secs());
    forged_drain.peer_id = bob_peer_id.clone();
    let forged_resp = roundtrip(&client, &server_addr, &ServerRequest::RelayDrain(forged_drain)).await;
    assert!(matches!(forged_resp, ServerResponse::Error(_)));

    // Bob, proving ownership of his own key, can drain it.
    let real_drain = SignedDrainRequest::sign(&bob, now_secs());
    let drain_resp = roundtrip(&client, &server_addr, &ServerRequest::RelayDrain(real_drain)).await;
    match drain_resp {
        ServerResponse::RelayDrained(items) => {
            assert_eq!(items.len(), 1);
            assert_eq!(items[0], b"opaque_ciphertext_blob");
        }
        other => panic!("expected drained packets, got {other:?}"),
    }
}
