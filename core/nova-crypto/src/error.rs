use thiserror::Error;

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Invalid mnemonic phrase: {0}")]
    InvalidMnemonic(String),

    #[error("Key derivation error: {0}")]
    KeyDerivationFailed(String),

    #[error("AEAD encryption error: {0}")]
    EncryptionFailed(String),

    #[error("AEAD decryption error (MAC verification failed or ciphertext corrupted)")]
    DecryptionFailed,

    #[error("Signature verification failed")]
    SignatureVerificationFailed,

    #[error("Invalid public key format: {0}")]
    InvalidPublicKey(String),

    #[error("Double Ratchet error: {0}")]
    RatchetError(String),

    #[error("Message out of bounds or duplicate: index {0}")]
    DuplicateMessage(u32),

    #[error("Maximum skipped message keys exceeded")]
    MaxSkippedKeysExceeded,

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Invalid passphrase or corrupted storage encryption key")]
    InvalidPassphrase,
}
