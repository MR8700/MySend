use crate::error::CryptoError;
use crate::identity::{verify_signature, DeviceIdentity};
use hkdf::Hkdf;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret};
use zeroize::Zeroize;

/// Domain-separation prefix prepended to X3DH signatures over a signed prekey.
const SIGNED_PREKEY_SIG_DOMAIN: &[u8] = b"NOVA_SIGNED_PREKEY_V1";
/// Domain-separation prefix prepended to the X3DH KDF input (see X3DH spec section 2.2:
/// a fixed byte sequence to make the DH outputs contributory even against legacy low-order points).
const X3DH_KDF_PREFIX: [u8; 32] = [0xFFu8; 32];
const X3DH_KDF_SALT: &[u8] = b"NOVA_X3DH_SALT_V1";
const X3DH_KDF_INFO: &[u8] = b"NOVA_X3DH_SHARED_SECRET";

/// A locally held X25519 signed prekey (medium-term, rotated periodically).
/// The secret half never leaves the device and must be persisted encrypted at rest.
pub struct SignedPreKeySecret {
    pub key_id: u32,
    secret: StaticSecret,
}

impl SignedPreKeySecret {
    pub fn from_parts(key_id: u32, secret_bytes: [u8; 32]) -> Self {
        Self {
            key_id,
            secret: StaticSecret::from(secret_bytes),
        }
    }

    pub fn to_secret_bytes(&self) -> [u8; 32] {
        self.secret.to_bytes()
    }

    pub fn public_bytes(&self) -> [u8; 32] {
        *X25519PublicKey::from(&self.secret).as_bytes()
    }

    /// Consumes this secret, returning the raw `StaticSecret` — used to seed a
    /// `DoubleRatchetSession::init_bob` with the exact keypair Alice targeted via X3DH.
    pub fn into_static_secret(self) -> StaticSecret {
        self.secret
    }
}

/// The public half of a signed prekey, safe to publish/exchange out-of-band.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SignedPreKeyPublic {
    pub key_id: u32,
    pub public: [u8; 32],
    /// Ed25519 signature (64 bytes). Stored as a `Vec<u8>` because `serde`'s derive does not
    /// implement (De)Serialize for fixed-size arrays larger than 32 elements.
    pub signature: Vec<u8>,
}

/// A locally held one-time X25519 prekey. Consumed after a single use and then deleted.
pub struct OneTimePreKeySecret {
    pub key_id: u32,
    secret: StaticSecret,
}

impl OneTimePreKeySecret {
    pub fn from_parts(key_id: u32, secret_bytes: [u8; 32]) -> Self {
        Self {
            key_id,
            secret: StaticSecret::from(secret_bytes),
        }
    }

    pub fn to_secret_bytes(&self) -> [u8; 32] {
        self.secret.to_bytes()
    }

    pub fn public_bytes(&self) -> [u8; 32] {
        *X25519PublicKey::from(&self.secret).as_bytes()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct OneTimePreKeyPublic {
    pub key_id: u32,
    pub public: [u8; 32],
}

/// Publishable identity + prekey material for a peer, exchanged out-of-band
/// (QR code, direct handshake, or a future signaling server) before a first
/// message can be sent to them.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PreKeyBundle {
    pub identity_ed25519_pub: [u8; 32],
    pub identity_x25519_pub: [u8; 32],
    pub signed_prekey: SignedPreKeyPublic,
    pub one_time_prekey: Option<OneTimePreKeyPublic>,
}

/// Generates a fresh signed prekey, signed by the device's long-term Ed25519 identity key.
pub fn generate_signed_prekey(
    identity: &DeviceIdentity,
    key_id: u32,
) -> (SignedPreKeySecret, SignedPreKeyPublic) {
    let mut rng = rand::thread_rng();
    let secret = StaticSecret::random_from_rng(&mut rng);
    let public = *X25519PublicKey::from(&secret).as_bytes();

    let signature = identity.sign(&signed_prekey_signing_input(key_id, &public));

    (
        SignedPreKeySecret { key_id, secret },
        SignedPreKeyPublic {
            key_id,
            public,
            signature: signature.to_vec(),
        },
    )
}

/// Generates a fresh one-time prekey (no signature required by the X3DH spec: it is
/// only ever distributed alongside an already-authenticated signed prekey bundle).
pub fn generate_one_time_prekey(key_id: u32) -> (OneTimePreKeySecret, OneTimePreKeyPublic) {
    let mut rng = rand::thread_rng();
    let secret = StaticSecret::random_from_rng(&mut rng);
    let public = *X25519PublicKey::from(&secret).as_bytes();
    (
        OneTimePreKeySecret { key_id, secret },
        OneTimePreKeyPublic { key_id, public },
    )
}

fn signed_prekey_signing_input(key_id: u32, public: &[u8; 32]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(SIGNED_PREKEY_SIG_DOMAIN.len() + 4 + 32);
    buf.extend_from_slice(SIGNED_PREKEY_SIG_DOMAIN);
    buf.extend_from_slice(&key_id.to_be_bytes());
    buf.extend_from_slice(public);
    buf
}

/// Verifies that a prekey bundle's signed prekey was genuinely signed by the claimed
/// Ed25519 identity key. MUST be called before a bundle is trusted for X3DH.
pub fn verify_prekey_bundle(bundle: &PreKeyBundle) -> Result<(), CryptoError> {
    let msg = signed_prekey_signing_input(bundle.signed_prekey.key_id, &bundle.signed_prekey.public);
    let sig_bytes: [u8; 64] = bundle
        .signed_prekey
        .signature
        .as_slice()
        .try_into()
        .map_err(|_| CryptoError::InvalidPublicKey("Signature must be 64 bytes".into()))?;
    verify_signature(&bundle.identity_ed25519_pub, &msg, &sig_bytes)
}

pub struct X3dhInitResult {
    pub shared_secret: [u8; 32],
    pub ephemeral_pub: [u8; 32],
    pub used_signed_prekey_id: u32,
    pub used_one_time_prekey_id: Option<u32>,
}

/// Alice's side of X3DH: derives a shared secret with Bob from his published prekey bundle.
///
/// The bundle's signature MUST already have been checked with [`verify_prekey_bundle`] by
/// the caller (kept as a separate, explicit step so callers cannot silently skip it).
pub fn x3dh_initiate(
    my_identity: &DeviceIdentity,
    their_bundle: &PreKeyBundle,
) -> Result<X3dhInitResult, CryptoError> {
    let mut rng = rand::thread_rng();
    let ephemeral_secret = StaticSecret::random_from_rng(&mut rng);
    let ephemeral_pub = *X25519PublicKey::from(&ephemeral_secret).as_bytes();

    let their_identity_x25519 = X25519PublicKey::from(their_bundle.identity_x25519_pub);
    let their_signed_prekey = X25519PublicKey::from(their_bundle.signed_prekey.public);

    let ik_a = my_identity.dh_secret();

    let dh1 = ik_a.diffie_hellman(&their_signed_prekey);
    let dh2 = ephemeral_secret.diffie_hellman(&their_identity_x25519);
    let dh3 = ephemeral_secret.diffie_hellman(&their_signed_prekey);

    let dh4 = their_bundle
        .one_time_prekey
        .as_ref()
        .map(|opk| ephemeral_secret.diffie_hellman(&X25519PublicKey::from(opk.public)));

    reject_low_order(dh1.as_bytes())?;
    reject_low_order(dh2.as_bytes())?;
    reject_low_order(dh3.as_bytes())?;
    if let Some(ref dh4) = dh4 {
        reject_low_order(dh4.as_bytes())?;
    }

    let shared_secret = x3dh_kdf(dh1.as_bytes(), dh2.as_bytes(), dh3.as_bytes(), dh4.as_ref().map(|d| d.as_bytes()));

    Ok(X3dhInitResult {
        shared_secret,
        ephemeral_pub,
        used_signed_prekey_id: their_bundle.signed_prekey.key_id,
        used_one_time_prekey_id: their_bundle.one_time_prekey.as_ref().map(|k| k.key_id),
    })
}

/// Bob's side of X3DH: reconstructs the same shared secret from Alice's identity key and
/// ephemeral key, using the local secret halves of the signed (and optional one-time) prekey
/// that Alice claims to have used.
pub fn x3dh_respond(
    my_identity: &DeviceIdentity,
    my_signed_prekey_secret: &SignedPreKeySecret,
    my_one_time_prekey_secret: Option<&OneTimePreKeySecret>,
    their_identity_x25519_pub: &[u8; 32],
    their_ephemeral_pub: &[u8; 32],
) -> Result<[u8; 32], CryptoError> {
    let their_identity_x25519 = X25519PublicKey::from(*their_identity_x25519_pub);
    let their_ephemeral = X25519PublicKey::from(*their_ephemeral_pub);

    let ik_b = my_identity.dh_secret();

    let dh1 = my_signed_prekey_secret.secret.diffie_hellman(&their_identity_x25519);
    let dh2 = ik_b.diffie_hellman(&their_ephemeral);
    let dh3 = my_signed_prekey_secret.secret.diffie_hellman(&their_ephemeral);

    let dh4 = my_one_time_prekey_secret.map(|opk| opk.secret.diffie_hellman(&their_ephemeral));

    reject_low_order(dh1.as_bytes())?;
    reject_low_order(dh2.as_bytes())?;
    reject_low_order(dh3.as_bytes())?;
    if let Some(ref dh4) = dh4 {
        reject_low_order(dh4.as_bytes())?;
    }

    Ok(x3dh_kdf(dh1.as_bytes(), dh2.as_bytes(), dh3.as_bytes(), dh4.as_ref().map(|d| d.as_bytes())))
}

/// Rejects an X25519 DH output that collapsed to the all-zero point, which would indicate
/// a low-order / invalid public key was used (contributory behavior check).
fn reject_low_order(dh_output: &[u8; 32]) -> Result<(), CryptoError> {
    if dh_output.iter().all(|b| *b == 0) {
        return Err(CryptoError::InvalidPublicKey(
            "X25519 DH output is the identity element (low-order key rejected)".into(),
        ));
    }
    Ok(())
}

fn x3dh_kdf(dh1: &[u8; 32], dh2: &[u8; 32], dh3: &[u8; 32], dh4: Option<&[u8; 32]>) -> [u8; 32] {
    let mut ikm = Vec::with_capacity(32 + 32 * 4);
    ikm.extend_from_slice(&X3DH_KDF_PREFIX);
    ikm.extend_from_slice(dh1);
    ikm.extend_from_slice(dh2);
    ikm.extend_from_slice(dh3);
    if let Some(dh4) = dh4 {
        ikm.extend_from_slice(dh4);
    }

    let hk = Hkdf::<Sha256>::new(Some(X3DH_KDF_SALT), &ikm);
    let mut okm = [0u8; 32];
    hk.expand(X3DH_KDF_INFO, &mut okm)
        .expect("HKDF-SHA256 expand of 32 bytes cannot fail");

    ikm.zeroize();
    okm
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::MnemonicPhrase;

    fn make_identity(name: &str) -> DeviceIdentity {
        let mnemonic = MnemonicPhrase::generate().unwrap();
        DeviceIdentity::from_mnemonic(&mnemonic, name).unwrap()
    }

    #[test]
    fn test_x3dh_with_one_time_prekey_matches_both_sides() {
        let alice = make_identity("alice");
        let bob = make_identity("bob");

        let (spk_secret, spk_public) = generate_signed_prekey(&bob, 1);
        let (opk_secret, opk_public) = generate_one_time_prekey(7);

        let bundle = PreKeyBundle {
            identity_ed25519_pub: bob.verifying_key_bytes,
            identity_x25519_pub: bob.dh_public_bytes,
            signed_prekey: spk_public,
            one_time_prekey: Some(opk_public),
        };

        verify_prekey_bundle(&bundle).expect("bundle signature must verify");

        let init = x3dh_initiate(&alice, &bundle).unwrap();

        let responded_secret = x3dh_respond(
            &bob,
            &spk_secret,
            Some(&opk_secret),
            &alice.dh_public_bytes,
            &init.ephemeral_pub,
        )
        .unwrap();

        assert_eq!(init.shared_secret, responded_secret);
        assert_eq!(init.used_signed_prekey_id, 1);
        assert_eq!(init.used_one_time_prekey_id, Some(7));
    }

    #[test]
    fn test_x3dh_without_one_time_prekey_matches_both_sides() {
        let alice = make_identity("alice");
        let bob = make_identity("bob");

        let (spk_secret, spk_public) = generate_signed_prekey(&bob, 42);

        let bundle = PreKeyBundle {
            identity_ed25519_pub: bob.verifying_key_bytes,
            identity_x25519_pub: bob.dh_public_bytes,
            signed_prekey: spk_public,
            one_time_prekey: None,
        };

        verify_prekey_bundle(&bundle).unwrap();
        let init = x3dh_initiate(&alice, &bundle).unwrap();
        let responded = x3dh_respond(&bob, &spk_secret, None, &alice.dh_public_bytes, &init.ephemeral_pub).unwrap();

        assert_eq!(init.shared_secret, responded);
        assert_eq!(init.used_one_time_prekey_id, None);
    }

    #[test]
    fn test_tampered_bundle_signature_is_rejected() {
        let bob = make_identity("bob");
        let (_secret, mut spk_public) = generate_signed_prekey(&bob, 1);
        spk_public.public[0] ^= 0x01; // tamper the published prekey after signing

        let bundle = PreKeyBundle {
            identity_ed25519_pub: bob.verifying_key_bytes,
            identity_x25519_pub: bob.dh_public_bytes,
            signed_prekey: spk_public,
            one_time_prekey: None,
        };

        assert!(verify_prekey_bundle(&bundle).is_err());
    }

    #[test]
    fn test_two_different_conversations_yield_different_secrets() {
        let alice = make_identity("alice");
        let bob = make_identity("bob");

        let (_secret, spk_public) = generate_signed_prekey(&bob, 1);
        let bundle = PreKeyBundle {
            identity_ed25519_pub: bob.verifying_key_bytes,
            identity_x25519_pub: bob.dh_public_bytes,
            signed_prekey: spk_public,
            one_time_prekey: None,
        };

        let init1 = x3dh_initiate(&alice, &bundle).unwrap();
        let init2 = x3dh_initiate(&alice, &bundle).unwrap();

        // Fresh ephemeral each time => different shared secrets even with the same bundle.
        assert_ne!(init1.shared_secret, init2.shared_secret);
        assert_ne!(init1.ephemeral_pub, init2.ephemeral_pub);
    }
}
