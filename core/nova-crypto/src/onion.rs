//! Tor Onion v3 Address Derivation and Verification
//!
//! Implements the Tor v3 hidden service specification (rend-spec-v3.txt § 6):
//! - checksum = SHA512(".onion checksum" || pubkey || version_byte)[0..2]
//! - address = base32(pubkey || checksum || version_byte) || ".onion"
//! - Total encoded length: 56 characters in lowercase RFC 4648 base32 + ".onion".

use sha2::{Digest, Sha512};
use crate::error::CryptoError;

const ONION_CHECKSUM_PREFIX: &[u8] = b".onion checksum";
const ONION_V3_VERSION: u8 = 0x03;
const BASE32_ALPHABET: &[u8; 32] = b"abcdefghijklmnopqrstuvwxyz234567";

/// Derives a 56-character `.onion` v3 address string from an Ed25519 public key.
pub fn derive_onion_v3_address(ed25519_pub: &[u8; 32]) -> String {
    // 1. Calculate 2-byte checksum
    let mut hasher = Sha512::new();
    hasher.update(ONION_CHECKSUM_PREFIX);
    hasher.update(ed25519_pub);
    hasher.update([ONION_V3_VERSION]);
    let digest = hasher.finalize();
    let checksum = &digest[0..2];

    // 2. Concatenate: pubkey (32) + checksum (2) + version (1) = 35 bytes
    let mut payload = [0u8; 35];
    payload[0..32].copy_from_slice(ed25519_pub);
    payload[32..34].copy_from_slice(checksum);
    payload[34] = ONION_V3_VERSION;

    // 3. Encode into 56 Base32 characters
    let mut result = encode_base32_no_padding(&payload);
    result.push_str(".onion");
    result
}

/// Parses and validates a Tor Onion v3 address, returning the underlying Ed25519 public key.
pub fn parse_onion_v3_address(onion_addr: &str) -> Result<[u8; 32], CryptoError> {
    let clean = onion_addr.trim().to_ascii_lowercase();
    let body = clean.strip_suffix(".onion").unwrap_or(&clean);

    if body.len() != 56 {
        return Err(CryptoError::InvalidPublicKey(format!(
            "invalid onion v3 address length: {} (expected 56 chars)",
            body.len()
        )));
    }

    let decoded = decode_base32_no_padding(body)?;
    if decoded.len() != 35 {
        return Err(CryptoError::InvalidPublicKey(format!(
            "invalid onion v3 decoded byte length: {}",
            decoded.len()
        )));
    }

    let mut pubkey = [0u8; 32];
    pubkey.copy_from_slice(&decoded[0..32]);
    let checksum = &decoded[32..34];
    let version = decoded[34];

    if version != ONION_V3_VERSION {
        return Err(CryptoError::InvalidPublicKey(format!(
            "unsupported onion version byte: 0x{:02x}",
            version
        )));
    }

    // Verify checksum
    let mut hasher = Sha512::new();
    hasher.update(ONION_CHECKSUM_PREFIX);
    hasher.update(&pubkey);
    hasher.update([ONION_V3_VERSION]);
    let digest = hasher.finalize();

    if checksum != &digest[0..2] {
        return Err(CryptoError::InvalidPublicKey("onion v3 checksum mismatch".into()));
    }

    Ok(pubkey)
}

fn encode_base32_no_padding(input: &[u8]) -> String {
    let mut result = String::with_capacity((input.len() * 8 + 4) / 5);
    let mut buffer: u64 = 0;
    let mut bits_left = 0;

    for &byte in input {
        buffer = (buffer << 8) | (byte as u64);
        bits_left += 8;
        while bits_left >= 5 {
            bits_left -= 5;
            let index = ((buffer >> bits_left) & 0x1F) as usize;
            result.push(BASE32_ALPHABET[index] as char);
        }
    }

    if bits_left > 0 {
        let index = ((buffer << (5 - bits_left)) & 0x1F) as usize;
        result.push(BASE32_ALPHABET[index] as char);
    }

    result
}

fn decode_base32_no_padding(input: &str) -> Result<Vec<u8>, CryptoError> {
    let mut result = Vec::with_capacity((input.len() * 5) / 8);
    let mut buffer: u64 = 0;
    let mut bits_left = 0;

    for c in input.chars() {
        let val = match c {
            'a'..='z' => (c as u8 - b'a') as u64,
            '2'..='7' => (c as u8 - b'2' + 26) as u64,
            _ => return Err(CryptoError::InvalidPublicKey(format!("invalid base32 character: '{c}'"))),
        };

        buffer = (buffer << 5) | val;
        bits_left += 5;

        if bits_left >= 8 {
            bits_left -= 8;
            let byte = ((buffer >> bits_left) & 0xFF) as u8;
            result.push(byte);
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_onion_v3_address_derivation_and_roundtrip() {
        let pubkey = [0x42u8; 32];
        let onion = derive_onion_v3_address(&pubkey);

        assert!(onion.ends_with(".onion"));
        assert_eq!(onion.len(), 56 + 6); // 56 chars + ".onion"

        let recovered_pubkey = parse_onion_v3_address(&onion).expect("failed to parse valid onion address");
        assert_eq!(pubkey, recovered_pubkey);
    }

    #[test]
    fn test_tampered_onion_address_is_rejected() {
        let pubkey = [0x99u8; 32];
        let mut onion = derive_onion_v3_address(&pubkey);

        // Tamper with a single character in the body
        let mut chars: Vec<char> = onion.chars().collect();
        chars[10] = if chars[10] == 'a' { 'b' } else { 'a' };
        onion = chars.into_iter().collect();

        assert!(parse_onion_v3_address(&onion).is_err());
    }

    #[test]
    fn test_onion_address_deterministic_output() {
        let pubkey1 = [0x01u8; 32];
        let pubkey2 = [0x02u8; 32];

        let addr1 = derive_onion_v3_address(&pubkey1);
        let addr1_repeat = derive_onion_v3_address(&pubkey1);
        let addr2 = derive_onion_v3_address(&pubkey2);

        assert_eq!(addr1, addr1_repeat);
        assert_ne!(addr1, addr2);
    }
}
