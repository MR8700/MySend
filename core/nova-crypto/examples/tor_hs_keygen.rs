//! Writes a real Tor v3 hidden-service key pair (`hs_ed25519_secret_key` /
//! `hs_ed25519_public_key`, in Tor's own on-disk format — see torspec's
//! `rend-spec-v3.txt` §2.1 and `torkeygen`) derived from a NOVA identity's mnemonic, so that a
//! real `tor` process serving that HiddenServiceDir presents *exactly* the `.onion` address this
//! app already derives and advertises via `nova_crypto::derive_onion_v3_address` — instead of
//! Tor generating (and serving) an unrelated, freshly-random hidden-service identity.
//!
//! Usage: tor_hs_keygen <username> <hidden-service-dir>
//!   (reads the 12-word mnemonic from stdin)
//!
//! Ed25519 "expansion" (RFC 8032 §5.1.5): the HS secret key file is not the 32-byte seed but the
//! post-SHA-512, clamped (scalar || prefix) form — the same computation `ed25519-dalek` does
//! internally when signing, replicated here by hand because the crate does not expose it as a
//! reusable public function on the version this project pins.
use nova_crypto::{DeviceIdentity, MnemonicPhrase};
use sha2::{Digest, Sha512};
use std::io::Read;

const SECRET_HEADER: &[u8; 32] = b"== ed25519v1-secret: type0 ==\0\0\0";
const PUBLIC_HEADER: &[u8; 32] = b"== ed25519v1-public: type0 ==\0\0\0";

fn expand_and_clamp(seed: &[u8; 32]) -> [u8; 64] {
    let hash = Sha512::digest(seed);
    let mut scalar = [0u8; 32];
    scalar.copy_from_slice(&hash[0..32]);
    scalar[0] &= 248;
    scalar[31] &= 127;
    scalar[31] |= 64;
    let mut expanded = [0u8; 64];
    expanded[0..32].copy_from_slice(&scalar);
    expanded[32..64].copy_from_slice(&hash[32..64]);
    expanded
}

fn main() {
    let username = std::env::args().nth(1).expect("usage: tor_hs_keygen <username> <hs-dir>");
    let hs_dir = std::env::args().nth(2).expect("usage: tor_hs_keygen <username> <hs-dir>");

    let mut mnemonic_str = String::new();
    std::io::stdin().read_to_string(&mut mnemonic_str).unwrap();
    let mnemonic_str = mnemonic_str.trim();

    let mnemonic = MnemonicPhrase::from_phrase(mnemonic_str).expect("invalid mnemonic");
    let identity = DeviceIdentity::from_mnemonic(&mnemonic, &username).expect("failed to derive identity");

    let expanded = expand_and_clamp(&identity.signing_key_bytes);

    let mut secret_file = Vec::with_capacity(96);
    secret_file.extend_from_slice(SECRET_HEADER);
    secret_file.extend_from_slice(&expanded);

    let mut public_file = Vec::with_capacity(64);
    public_file.extend_from_slice(PUBLIC_HEADER);
    public_file.extend_from_slice(&identity.verifying_key_bytes);

    std::fs::create_dir_all(&hs_dir).expect("failed to create HiddenServiceDir");
    std::fs::write(format!("{hs_dir}/hs_ed25519_secret_key"), &secret_file).unwrap();
    std::fs::write(format!("{hs_dir}/hs_ed25519_public_key"), &public_file).unwrap();

    let expected_onion = nova_crypto::derive_onion_v3_address(&identity.verifying_key_bytes);
    println!("Wrote HS keys to {hs_dir}");
    println!("Expected onion address (compare against Tor's own hostname file after it starts): {expected_onion}");
}
