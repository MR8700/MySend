//! Fuzzes `MessagePayload::from_bytes` — the decrypted plaintext of every incoming message
//! (text or media chunk) is parsed with this before `nova-engine` does anything else with it.
//! Reachable by any peer who can complete a session with this device (no signature protects this
//! layer specifically — the Double Ratchet's AEAD tag already authenticated the bytes, but a
//! genuine peer with a compromised or buggy client could still send a structurally malformed
//! payload inside a validly-encrypted envelope).

#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = nova_protocol::MessagePayload::from_bytes(data);
});
