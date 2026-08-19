use crate::error::CryptoError;
use argon2::{Algorithm, Argon2, Params, Version};
use rand::RngCore;

pub const STORAGE_SALT_LEN: usize = 16;

/// Argon2id parameters tuned for an interactive local unlock (single derivation per app
/// start, not a hot path): 64 MiB memory, 3 iterations, 1 lane. Deliberately memory-hard
/// to resist GPU/ASIC brute-forcing of the local unlock passphrase.
fn argon2id() -> Argon2<'static> {
    let params = Params::new(64 * 1024, 3, 1, Some(32)).expect("valid static Argon2id params");
    Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
}

/// Generates a fresh random salt for a new local storage encryption key.
pub fn generate_storage_salt() -> [u8; STORAGE_SALT_LEN] {
    let mut salt = [0u8; STORAGE_SALT_LEN];
    rand::thread_rng().fill_bytes(&mut salt);
    salt
}

/// Derives a 256-bit storage encryption key from a local unlock passphrase and salt via
/// Argon2id. This key never touches disk; only the salt (and a verifier ciphertext) does.
pub fn derive_storage_key(passphrase: &str, salt: &[u8]) -> Result<[u8; 32], CryptoError> {
    let mut out = [0u8; 32];
    argon2id()
        .hash_password_into(passphrase.as_bytes(), salt, &mut out)
        .map_err(|e| CryptoError::KeyDerivationFailed(format!("Argon2id: {e}")))?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_same_passphrase_and_salt_are_deterministic() {
        let salt = generate_storage_salt();
        let k1 = derive_storage_key("correct horse battery staple", &salt).unwrap();
        let k2 = derive_storage_key("correct horse battery staple", &salt).unwrap();
        assert_eq!(k1, k2);
    }

    #[test]
    fn test_different_passphrase_yields_different_key() {
        let salt = generate_storage_salt();
        let k1 = derive_storage_key("passphrase A", &salt).unwrap();
        let k2 = derive_storage_key("passphrase B", &salt).unwrap();
        assert_ne!(k1, k2);
    }

    #[test]
    fn test_different_salt_yields_different_key() {
        let k1 = derive_storage_key("same passphrase", &generate_storage_salt()).unwrap();
        let k2 = derive_storage_key("same passphrase", &generate_storage_salt()).unwrap();
        assert_ne!(k1, k2);
    }
}
