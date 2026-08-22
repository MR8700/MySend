//! Fuzzes `SignedDhtPeerRecord::from_bytes` — decodes whatever value ANY peer in the Kademlia
//! DHT happens to be storing/serving for a given key, before the signature is even checked (the
//! decode has to succeed first). Storing nodes are untrusted by design (that's the whole point
//! of the record being self-authenticating), so this is directly attacker-controlled input from
//! any participant in the DHT, not just this device's own contacts.

#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = nova_protocol::SignedDhtPeerRecord::from_bytes(data);
});
