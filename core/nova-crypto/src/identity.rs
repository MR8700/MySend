use crate::error::CryptoError;
use bip39::{Language, Mnemonic};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use hkdf::Hkdf;
use sha2::Sha256;
use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Represents a 12-word cryptographic seed phrase for user account recovery.
#[derive(Clone)]
pub struct MnemonicPhrase {
    phrase: String,
}

impl MnemonicPhrase {
    /// Generates a fresh 12-word BIP-39 mnemonic with 128 bits of entropy.
    pub fn generate() -> Result<Self, CryptoError> {
        let mut rng = rand::thread_rng();
        let mnemonic = Mnemonic::generate_in_with(&mut rng, Language::English, 12)
            .map_err(|e| CryptoError::InvalidMnemonic(e.to_string()))?;
        Ok(Self {
            phrase: mnemonic.to_string(),
        })
    }

    /// Parses and validates an existing mnemonic phrase.
    pub fn from_phrase(phrase: &str) -> Result<Self, CryptoError> {
        let clean = phrase.trim();
        let _mnemonic = Mnemonic::parse_in_normalized(Language::English, clean)
            .map_err(|e| CryptoError::InvalidMnemonic(e.to_string()))?;
        Ok(Self {
            phrase: clean.to_string(),
        })
    }

    /// Returns the readable string representation of the mnemonic (12 words).
    pub fn as_str(&self) -> &str {
        &self.phrase
    }

    /// Derives a 64-byte master seed from the mnemonic phrase and optional passphrase.
    pub fn to_seed(&self, passphrase: &str) -> [u8; 64] {
        let mnemonic = Mnemonic::parse_in_normalized(Language::English, &self.phrase)
            .expect("Mnemonic already validated");
        mnemonic.to_seed(passphrase)
    }
}

/// Holds device identity keys (Ed25519 for signing, X25519 for Diffie-Hellman).
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct DeviceIdentity {
    pub username: String,
    pub signing_key_bytes: [u8; 32],
    pub dh_secret_bytes: [u8; 32],
    #[zeroize(skip)]
    pub verifying_key_bytes: [u8; 32],
    #[zeroize(skip)]
    pub dh_public_bytes: [u8; 32],
}

impl DeviceIdentity {
    /// Generates a new device identity from a mnemonic phrase and username.
    pub fn from_mnemonic(mnemonic: &MnemonicPhrase, username: &str) -> Result<Self, CryptoError> {
        let seed = mnemonic.to_seed("NOVA_CHAT_MASTER_SALT");

        // Derive Ed25519 signing key using HKDF-SHA256
        let hk = Hkdf::<Sha256>::new(Some(b"NOVA_IDENTITY_HKDF_SALT"), &seed);
        
        let mut ed25519_seed = [0u8; 32];
        hk.expand(b"ed25519_identity_key", &mut ed25519_seed)
            .map_err(|e| CryptoError::KeyDerivationFailed(e.to_string()))?;
        
        let signing_key = SigningKey::from_bytes(&ed25519_seed);
        let verifying_key = signing_key.verifying_key();

        // Derive X25519 static DH key using HKDF-SHA256
        let mut x25519_seed = [0u8; 32];
        hk.expand(b"x25519_identity_key", &mut x25519_seed)
            .map_err(|e| CryptoError::KeyDerivationFailed(e.to_string()))?;
        
        let dh_secret = StaticSecret::from(x25519_seed);
        let dh_public = X25519PublicKey::from(&dh_secret);

        let identity = Self {
            username: username.to_string(),
            signing_key_bytes: ed25519_seed,
            dh_secret_bytes: x25519_seed,
            verifying_key_bytes: verifying_key.to_bytes(),
            dh_public_bytes: *dh_public.as_bytes(),
        };

        // Wipe temporary seeds
        ed25519_seed.zeroize();
        x25519_seed.zeroize();

        Ok(identity)
    }

    /// Sign data using Ed25519 identity key.
    pub fn sign(&self, message: &[u8]) -> [u8; 64] {
        let signing_key = SigningKey::from_bytes(&self.signing_key_bytes);
        let sig: Signature = signing_key.sign(message);
        sig.to_bytes()
    }

    /// Get public key as hex string.
    pub fn public_id_hex(&self) -> String {
        hex::encode(self.verifying_key_bytes)
    }

    /// Get readable truncated public identity format (e.g. "A1B2 C3D4 ... 9A0B")
    pub fn formatted_fingerprint(&self) -> String {
        let full = self.public_id_hex().to_uppercase();
        if full.len() >= 16 {
            format!("{} {} {} ... {}", &full[0..4], &full[4..8], &full[8..12], &full[full.len()-4..])
        } else {
            full
        }
    }

    /// Reconstruct X25519 StaticSecret for DH operations
    pub fn dh_secret(&self) -> StaticSecret {
        StaticSecret::from(self.dh_secret_bytes)
    }

    /// Get public signing key
    pub fn verifying_key(&self) -> Result<VerifyingKey, CryptoError> {
        VerifyingKey::from_bytes(&self.verifying_key_bytes)
            .map_err(|e| CryptoError::InvalidPublicKey(e.to_string()))
    }
}

/// Verify an Ed25519 signature from a peer's public key.
pub fn verify_signature(
    public_key_bytes: &[u8; 32],
    message: &[u8],
    signature_bytes: &[u8; 64],
) -> Result<(), CryptoError> {
    let verifying_key = VerifyingKey::from_bytes(public_key_bytes)
        .map_err(|e| CryptoError::InvalidPublicKey(e.to_string()))?;
    let signature = Signature::from_bytes(signature_bytes);
    verifying_key
        .verify(message, &signature)
        .map_err(|_| CryptoError::SignatureVerificationFailed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mnemonic_generation_and_deterministic_identity() {
        let mnemonic = MnemonicPhrase::generate().unwrap();
        let phrase_str = mnemonic.as_str().to_string();
        assert_eq!(phrase_str.split_whitespace().count(), 12);

        // Derive identity
        let identity1 = DeviceIdentity::from_mnemonic(&mnemonic, "alex").unwrap();
        
        // Re-import mnemonic
        let mnemonic_restored = MnemonicPhrase::from_phrase(&phrase_str).unwrap();
        let identity2 = DeviceIdentity::from_mnemonic(&mnemonic_restored, "alex").unwrap();

        // Must be 100% deterministic
        assert_eq!(identity1.verifying_key_bytes, identity2.verifying_key_bytes);
        assert_eq!(identity1.dh_public_bytes, identity2.dh_public_bytes);
        assert_eq!(identity1.public_id_hex(), identity2.public_id_hex());
    }

    #[test]
    fn test_sign_and_verify() {
        let mnemonic = MnemonicPhrase::generate().unwrap();
        let identity = DeviceIdentity::from_mnemonic(&mnemonic, "emma").unwrap();

        let message = b"Hello P2P world! Sovereign message.";
        let sig = identity.sign(message);

        assert!(verify_signature(&identity.verifying_key_bytes, message, &sig).is_ok());

        // Altered message should fail verification
        let altered = b"Hello P2P world! Tampered message.";
        assert!(verify_signature(&identity.verifying_key_bytes, altered, &sig).is_err());
    }
}
