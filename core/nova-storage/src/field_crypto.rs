use crate::StorageError;
use nova_crypto::{decrypt_aead, encrypt_aead};
use rand::RngCore;
use zeroize::Zeroize;

const NONCE_LEN: usize = 12;

/// Wraps the local storage encryption key (derived once via Argon2id at unlock time,
/// see `StorageEngine::open`) and encrypts/decrypts individual sensitive columns with
/// ChaCha20-Poly1305. Every ciphertext is bound to `aad` (typically the row's primary
/// key / column name) so a ciphertext copied into a different row fails to decrypt.
pub struct FieldCipher {
    key: [u8; 32],
}

impl FieldCipher {
    pub fn new(key: [u8; 32]) -> Self {
        Self { key }
    }

    /// Encrypts `plaintext`, returning `nonce (12 bytes) || ciphertext` ready to store as a BLOB.
    pub fn encrypt(&self, plaintext: &[u8], aad: &[u8]) -> Vec<u8> {
        let mut nonce = [0u8; NONCE_LEN];
        rand::thread_rng().fill_bytes(&mut nonce);

        let ciphertext = encrypt_aead(&self.key, &nonce, plaintext, aad)
            .expect("ChaCha20-Poly1305 encryption with a valid 32-byte key cannot fail");

        let mut out = Vec::with_capacity(NONCE_LEN + ciphertext.len());
        out.extend_from_slice(&nonce);
        out.extend_from_slice(&ciphertext);
        out
    }

    pub fn encrypt_str(&self, plaintext: &str, aad: &[u8]) -> Vec<u8> {
        self.encrypt(plaintext.as_bytes(), aad)
    }

    /// Decrypts a `nonce || ciphertext` blob previously produced by [`encrypt`]. Fails if the
    /// blob is malformed or the AEAD tag does not verify (tampering, wrong key, wrong `aad`).
    pub fn decrypt(&self, blob: &[u8], aad: &[u8]) -> Result<Vec<u8>, StorageError> {
        if blob.len() < NONCE_LEN {
            return Err(StorageError::CorruptedField);
        }
        let (nonce, ciphertext) = blob.split_at(NONCE_LEN);
        let nonce_arr: [u8; NONCE_LEN] = nonce.try_into().expect("checked length above");
        decrypt_aead(&self.key, &nonce_arr, ciphertext, aad).map_err(|_| StorageError::CorruptedField)
    }

    pub fn decrypt_string(&self, blob: &[u8], aad: &[u8]) -> Result<String, StorageError> {
        let bytes = self.decrypt(blob, aad)?;
        String::from_utf8(bytes).map_err(|_| StorageError::CorruptedField)
    }
}

impl Drop for FieldCipher {
    fn drop(&mut self) {
        self.key.zeroize();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip() {
        let cipher = FieldCipher::new([7u8; 32]);
        let blob = cipher.encrypt(b"top secret mnemonic", b"identities:mnemonic");
        let plain = cipher.decrypt(&blob, b"identities:mnemonic").unwrap();
        assert_eq!(plain, b"top secret mnemonic");
    }

    #[test]
    fn test_wrong_aad_fails() {
        let cipher = FieldCipher::new([7u8; 32]);
        let blob = cipher.encrypt(b"data", b"row:1");
        assert!(cipher.decrypt(&blob, b"row:2").is_err());
    }

    #[test]
    fn test_tampered_blob_fails() {
        let cipher = FieldCipher::new([7u8; 32]);
        let mut blob = cipher.encrypt(b"data", b"aad");
        let last = blob.len() - 1;
        blob[last] ^= 0x01;
        assert!(cipher.decrypt(&blob, b"aad").is_err());
    }
}
