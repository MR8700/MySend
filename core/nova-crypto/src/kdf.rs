use crate::error::CryptoError;
use hkdf::Hkdf;
use sha2::Sha256;
use zeroize::Zeroize;

pub type Key32 = [u8; 32];

/// KDF for the Root Chain (DH Ratchet step).
/// Computes new (RootKey, ChainKey) from current RootKey and DH shared secret.
pub fn kdf_rk(rk: &Key32, dh_out: &Key32) -> Result<(Key32, Key32), CryptoError> {
    let hk = Hkdf::<Sha256>::new(Some(rk), dh_out);
    let mut okm = [0u8; 64];
    hk.expand(b"NOVA_DOUBLE_RATCHET_RK_KDF", &mut okm)
        .map_err(|e| CryptoError::KeyDerivationFailed(e.to_string()))?;

    let mut next_rk = [0u8; 32];
    let mut next_ck = [0u8; 32];
    next_rk.copy_from_slice(&okm[0..32]);
    next_ck.copy_from_slice(&okm[32..64]);

    okm.zeroize();
    Ok((next_rk, next_ck))
}

/// KDF for the Symmetric Chain (Message key step).
/// Computes (NextChainKey, MessageKey) from current ChainKey.
pub fn kdf_ck(ck: &Key32) -> Result<(Key32, Key32), CryptoError> {
    let hk = Hkdf::<Sha256>::new(Some(b"NOVA_CHAIN_SALT"), ck);

    let mut next_ck = [0u8; 32];
    let mut msg_key = [0u8; 32];

    hk.expand(b"NOVA_CHAIN_STEP_NEXT_CK", &mut next_ck)
        .map_err(|e| CryptoError::KeyDerivationFailed(e.to_string()))?;
    hk.expand(b"NOVA_CHAIN_STEP_MESSAGE_KEY", &mut msg_key)
        .map_err(|e| CryptoError::KeyDerivationFailed(e.to_string()))?;

    Ok((next_ck, msg_key))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kdf_rk_evolution() {
        let rk = [1u8; 32];
        let dh_out = [2u8; 32];
        let (next_rk, ck) = kdf_rk(&rk, &dh_out).unwrap();

        assert_ne!(rk, next_rk);
        assert_ne!(next_rk, ck);
    }

    #[test]
    fn test_kdf_ck_step() {
        let ck = [3u8; 32];
        let (next_ck, msg_key) = kdf_ck(&ck).unwrap();

        assert_ne!(ck, next_ck);
        assert_ne!(next_ck, msg_key);
    }
}
