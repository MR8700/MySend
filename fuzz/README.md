# Fuzzing NOVA Chat's untrusted-input decoders

Part of the 2026-08-22 audit's J6 hardening pass. These targets exercise the four decoders that
parse bytes arriving from a network peer *before* any signature check protects them — the class
of bug a manual code review cannot cover exhaustively (a malformed length prefix, an integer
overflow in a size calculation, an unexpected CBOR structure) but a fuzzer finds routinely.

| Target                         | What it fuzzes                              | Who can reach it                                  |
|---------------------------------|----------------------------------------------|----------------------------------------------------|
| `nova_packet_from_cbor`         | `NovaPacket::from_cbor`                       | Any connected libp2p/WebSocket-relay-fallback peer  |
| `message_payload_from_bytes`    | `MessagePayload::from_bytes`                  | Any peer with an established (or attacker-controlled) Double Ratchet session |
| `dht_peer_record_from_bytes`    | `SignedDhtPeerRecord::from_bytes`             | Any node storing/serving a Kademlia DHT record      |
| `server_request_from_bytes`     | `ServerRequest::from_bytes` (`nova-server`)   | Anyone who can open a WebSocket connection to the fallback server — no prior relationship needed at all |

## Running locally

Requires the nightly toolchain and `cargo-fuzz` (not installed in the main dev environment this
was written in — CI installs both fresh, see `.github/workflows/ci.yml`'s `fuzz-smoke` job):

```sh
rustup install nightly
cargo install cargo-fuzz --locked
cd fuzz
cargo +nightly fuzz run nova_packet_from_cbor
```

Each target runs until stopped (Ctrl+C) or a crash is found. CI runs each for 60 seconds per
push/PR as a smoke test, not a full campaign — for real coverage, run one target for several
hours locally (`-max_total_time=<seconds>`), ideally seeded with real captured wire packets in
`fuzz/corpus/<target-name>/`.

A crash produces a minimized reproducer under `fuzz/artifacts/<target-name>/` — turn it into a
regression test in the relevant crate's `#[cfg(test)] mod tests` once fixed, the same way every
other finding in this codebase's history has been tracked.
