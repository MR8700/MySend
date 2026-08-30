//! Fuzzes `NovaPacket::from_cbor` — the very first thing done with any byte blob a peer sends
//! over the wire (libp2p request/response, or the WebSocket discovery/relay fallback), before
//! any signature or session state is checked. Must never panic or hang on adversarial input.

#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = nova_protocol::NovaPacket::from_cbor(data);
});
