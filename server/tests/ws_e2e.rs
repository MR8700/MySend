//! End-to-end test of the discovery/relay server over a real WebSocket connection (what it
//! actually serves once deployed, since a host like Render exposes no raw UDP ingress — see
//! `service.rs`'s module doc comment): this is the network surface a hostile peer actually talks
//! to, so it is exercised here rather than only through the in-process unit tests in
//! `registry.rs`/`relay.rs`.

use futures_util::{SinkExt, StreamExt};
use nova_crypto::{DeviceIdentity, MnemonicPhrase};
use nova_protocol::{PeerEndpoint, ServerRequest, ServerResponse, SignedDrainRequest, SignedPresenceRegistration};
use nova_server::bind;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::timeout;
use tokio_tungstenite::tungstenite::Message;

fn identity(name: &str) -> DeviceIdentity {
    let mnemonic = MnemonicPhrase::generate().unwrap();
    DeviceIdentity::from_mnemonic(&mnemonic, name).unwrap()
}

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}

async fn start_test_server() -> String {
    let (listener, registry, relay) = bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    tokio::spawn(nova_server::serve_forever(listener, registry, relay));
    // Give the server a moment to start accepting connections before the test starts dialing.
    tokio::time::sleep(Duration::from_millis(150)).await;
    addr
}

/// One request, one response, one fresh WebSocket connection — mirrors exactly how
/// `nova-transport`'s real fallback client uses this server (see `udp_fallback.rs`).
async fn roundtrip(server_addr: &str, request: &ServerRequest) -> ServerResponse {
    let url = format!("ws://{server_addr}/");
    let (mut ws, _) = timeout(Duration::from_secs(2), tokio_tungstenite::connect_async(&url))
        .await
        .expect("server did not accept the connection in time")
        .unwrap();

    ws.send(Message::Binary(request.to_bytes().unwrap())).await.unwrap();

    let msg = timeout(Duration::from_secs(2), ws.next())
        .await
        .expect("server did not respond in time")
        .expect("connection closed without a response")
        .unwrap();

    let Message::Binary(bytes) = msg else {
        panic!("expected a binary WebSocket message, got {msg:?}");
    };
    ServerResponse::from_bytes(&bytes).unwrap()
}

#[tokio::test]
async fn test_register_then_lookup_over_real_websocket() {
    let server_addr = start_test_server().await;

    let alice = identity("alice");
    // Alice's device cannot know its own post-NAT public IP, so it claims a plausible but
    // fictitious one here — the server must ignore this and record the address it actually
    // observed the connection arrive from instead (a STUN-style reflexive-address correction, see
    // `PresenceRegistry::register`). The claimed port (51820, standing in for Alice's actual QUIC
    // endpoint port, which is a different socket than this signaling connection) is kept as-is.
    let endpoint = PeerEndpoint {
        peer_id: String::new(),
        public_ip: "203.0.113.5".into(),
        public_port: 51820,
        local_ip: Some("10.1.1.2".into()),
        local_port: Some(51820),
    };
    let registration = SignedPresenceRegistration::sign(&alice, endpoint, now_secs());

    let register_resp = roundtrip(&server_addr, &ServerRequest::Register(registration)).await;
    assert!(matches!(register_resp, ServerResponse::Registered));

    let lookup_resp = roundtrip(
        &server_addr,
        &ServerRequest::Lookup {
            peer_id: alice.public_id_hex(),
        },
    )
    .await;

    match lookup_resp {
        ServerResponse::LookupResult(Some(found)) => {
            // No `X-Forwarded-For` header on a direct connection like this test's, so the
            // observed address falls back to the real TCP peer — loopback, since both ends are
            // 127.0.0.1 here.
            assert_eq!(found.public_ip, "127.0.0.1");
            assert_ne!(found.public_ip, "203.0.113.5", "claimed IP must never be trusted verbatim");
            assert_eq!(found.public_port, 51820);
        }
        other => panic!("expected a lookup hit, got {other:?}"),
    }
}

#[tokio::test]
async fn test_spoofed_registration_is_rejected_over_real_websocket() {
    let server_addr = start_test_server().await;

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

    let resp = roundtrip(&server_addr, &ServerRequest::Register(hijack)).await;
    assert!(matches!(resp, ServerResponse::Error(_)));

    let lookup_resp = roundtrip(
        &server_addr,
        &ServerRequest::Lookup {
            peer_id: alice.public_id_hex(),
        },
    )
    .await;
    assert!(matches!(lookup_resp, ServerResponse::LookupResult(None)));
}

#[tokio::test]
async fn test_relay_forward_and_authenticated_drain_over_real_websocket() {
    let server_addr = start_test_server().await;

    let bob = identity("bob");
    let bob_peer_id = bob.public_id_hex();

    let forward_resp = roundtrip(
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
    let forged_resp = roundtrip(&server_addr, &ServerRequest::RelayDrain(forged_drain)).await;
    assert!(matches!(forged_resp, ServerResponse::Error(_)));

    // Bob, proving ownership of his own key, can drain it.
    let real_drain = SignedDrainRequest::sign(&bob, now_secs());
    let drain_resp = roundtrip(&server_addr, &ServerRequest::RelayDrain(real_drain)).await;
    match drain_resp {
        ServerResponse::RelayDrained(items) => {
            assert_eq!(items.len(), 1);
            assert_eq!(items[0], b"opaque_ciphertext_blob");
        }
        other => panic!("expected drained packets, got {other:?}"),
    }
}
