use crate::error::CryptoError;

/// Computes a deterministic Safety Number / Fingerprint for two peers.
///
/// Both peers sort their public keys lexicographically, ensuring that
/// Alice checking Bob and Bob checking Alice produce the EXACT same fingerprint.
pub fn compute_safety_number(
    own_pubkey: &[u8; 32],
    peer_pubkey: &[u8; 32],
) -> Result<String, CryptoError> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"NOVA_SAFETY_NUMBER_V1");

    // Lexicographical ordering for symmetric verification
    if own_pubkey < peer_pubkey {
        hasher.update(own_pubkey);
        hasher.update(peer_pubkey);
    } else {
        hasher.update(peer_pubkey);
        hasher.update(own_pubkey);
    }

    let hash_bytes = hasher.finalize();
    let hex_full = hex::encode(hash_bytes.as_bytes()).to_uppercase();

    // Format into standard readable grouped segments (e.g. "4A9F-2B1C-88E0-9142")
    let formatted = format!(
        "{}-{}-{}-{}",
        &hex_full[0..4],
        &hex_full[4..8],
        &hex_full[8..12],
        &hex_full[12..16]
    );

    Ok(formatted)
}

/// Computes a numerical format (6 blocks of 5 digits) similar to Signal safety numbers.
pub fn compute_numeric_safety_number(
    own_pubkey: &[u8; 32],
    peer_pubkey: &[u8; 32],
) -> Result<String, CryptoError> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"NOVA_NUMERIC_SAFETY_NUMBER_V1");

    if own_pubkey < peer_pubkey {
        hasher.update(own_pubkey);
        hasher.update(peer_pubkey);
    } else {
        hasher.update(peer_pubkey);
        hasher.update(own_pubkey);
    }

    let hash = hasher.finalize();
    let bytes = hash.as_bytes();

    let mut blocks = Vec::new();
    for chunk in bytes.chunks_exact(4).take(6) {
        let val = u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]) % 100_000;
        blocks.push(format!("{:05}", val));
    }

    Ok(blocks.join(" "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safety_number_symmetry() {
        let alice_pub = [10u8; 32];
        let bob_pub = [20u8; 32];

        let sn_alice = compute_safety_number(&alice_pub, &bob_pub).unwrap();
        let sn_bob = compute_safety_number(&bob_pub, &alice_pub).unwrap();

        assert_eq!(sn_alice, sn_bob);

        let num_alice = compute_numeric_safety_number(&alice_pub, &bob_pub).unwrap();
        let num_bob = compute_numeric_safety_number(&bob_pub, &alice_pub).unwrap();

        assert_eq!(num_alice, num_bob);
        assert_eq!(num_alice.split_whitespace().count(), 6);
    }
}
