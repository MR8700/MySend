//! Fuzzes `ServerRequest::from_bytes` — the first thing `nova-server` (the UDP discovery/blind-
//! relay fallback, see the 2026-08-22 audit's J5 finding) does with any UDP datagram arriving at
//! its public port, before any signature check. The single most exposed decoder in the whole
//! system: reachable by anyone on the internet who can send it a UDP packet, with no prior
//! handshake, session, or contact relationship required at all.

#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = nova_protocol::ServerRequest::from_bytes(data);
});
