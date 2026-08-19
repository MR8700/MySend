pub mod aead;
pub mod error;
pub mod fingerprint;
pub mod identity;
pub mod kdf;
pub mod ratchet;
pub mod storage_kdf;
pub mod x3dh;

// Convenient re-exports
pub use aead::{decrypt_aead, encrypt_aead};
pub use error::CryptoError;
pub use fingerprint::{compute_numeric_safety_number, compute_safety_number};
pub use identity::{verify_signature, DeviceIdentity, MnemonicPhrase};
pub use ratchet::{DoubleRatchetSession, RatchetHeader};
pub use storage_kdf::{derive_storage_key, generate_storage_salt, STORAGE_SALT_LEN};
pub use x3dh::{
    generate_one_time_prekey, generate_signed_prekey, verify_prekey_bundle, x3dh_initiate,
    x3dh_respond, OneTimePreKeyPublic, OneTimePreKeySecret, PreKeyBundle, SignedPreKeyPublic,
    SignedPreKeySecret, X3dhInitResult,
};
