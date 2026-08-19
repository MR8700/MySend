use crate::error::CryptoError;
use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};

/// AEAD encryption using ChaCha20-Poly1305 (256-bit key, 96-bit nonce).
pub fn encrypt_aead(
    key: &[u8; 32],
    nonce: &[u8; 12],
    plaintext: &[u8],
    associated_data: &[u8],
) -> Result<Vec<u8>, CryptoError> {
    let cipher = ChaCha20Poly1305::new(Key::from_slice(key));
    let nonce_struct = Nonce::from_slice(nonce);

    let payload = Payload {
        msg: plaintext,
        aad: associated_data,
    };

    cipher
        .encrypt(nonce_struct, payload)
        .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))
}

/// AEAD decryption using ChaCha20-Poly1305. Rejects any altered or corrupted ciphertext.
pub fn decrypt_aead(
    key: &[u8; 32],
    nonce: &[u8; 12],
    ciphertext: &[u8],
    associated_data: &[u8],
) -> Result<Vec<u8>, CryptoError> {
    let cipher = ChaCha20Poly1305::new(Key::from_slice(key));
    let nonce_struct = Nonce::from_slice(nonce);

    let payload = Payload {
        msg: ciphertext,
        aad: associated_data,
    };

    cipher
        .decrypt(nonce_struct, payload)
        .map_err(|_| CryptoError::DecryptionFailed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::RngCore;

    #[test]
    fn test_aead_roundtrip() {
        let mut key = [0u8; 32];
        let mut nonce = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut key);
        rand::thread_rng().fill_bytes(&mut nonce);

        let plaintext = b"Top secret sovereign payload transmitted over P2P QUIC";
        let aad = b"header_metadata_v1";

        let ciphertext = encrypt_aead(&key, &nonce, plaintext, aad).unwrap();
        assert_ne!(&ciphertext[..], plaintext);

        let decrypted = decrypt_aead(&key, &nonce, &ciphertext, aad).unwrap();
        assert_eq!(&decrypted[..], plaintext);
    }

    #[test]
    fn test_aead_tamper_detection() {
        let mut key = [0u8; 32];
        let mut nonce = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut key);
        rand::thread_rng().fill_bytes(&mut nonce);

        let plaintext = b"Original message";
        let aad = b"auth_tag_data";

        let mut ciphertext = encrypt_aead(&key, &nonce, plaintext, aad).unwrap();

        // Tamper with one bit
        ciphertext[0] ^= 0x01;

        let result = decrypt_aead(&key, &nonce, &ciphertext, aad);
        assert!(result.is_err());
    }
}
