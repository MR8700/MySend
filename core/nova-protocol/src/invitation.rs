use nova_crypto::{verify_prekey_bundle, verify_signature, DeviceIdentity, PreKeyBundle};
use serde::{Deserialize, Serialize};
use thiserror::Error;

const INVITATION_SIG_DOMAIN: &[u8] = b"NOVA_INVITATION_V1:";

#[derive(Error, Debug, PartialEq, Eq)]
pub enum InvitationError {
    #[error("L'invitation a expiré le {expires_at_utc} (heure actuelle: {now_utc}). Demandez un nouveau code à votre contact.")]
    Expired { expires_at_utc: i64, now_utc: i64 },
    #[error("Signature cryptographique de l'invitation invalide ou falsifiée: {0}")]
    InvalidSignature(String),
    #[error("Erreur de sérialisation/désérialisation de l'invitation: {0}")]
    Serialization(String),
    #[error("Format d'URI ou de code d'invitation non reconnu: {0}")]
    MalformedUri(String),
    #[error("Erreur cryptographique du trousseau X3DH: {0}")]
    Crypto(String),
}

/// Unencrypted payload of a contact invitation containing identity, prekeys, validity deadline,
/// and optional rendezvous addresses for automatic P2P connection.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContactInvitationPayload {
    pub version: u32,
    pub created_at_utc: i64,
    pub expires_at_utc: i64,
    pub bundle: PreKeyBundle,
    pub rendezvous_addrs: Vec<String>,
}

/// A cryptographically signed contact invitation ticket with an unforgeable expiry deadline.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SignedContactInvitation {
    pub payload_cbor: Vec<u8>,
    pub signature: Vec<u8>,
}

impl SignedContactInvitation {
    /// Creates a new signed invitation valid for `ttl_seconds` from now.
    pub fn create(
        identity: &DeviceIdentity,
        bundle: PreKeyBundle,
        ttl_seconds: i64,
        rendezvous_addrs: Vec<String>,
    ) -> Result<Self, InvitationError> {
        let now = chrono::Utc::now().timestamp();
        let expires_at_utc = now.saturating_add(ttl_seconds);

        let payload = ContactInvitationPayload {
            version: 1,
            created_at_utc: now,
            expires_at_utc,
            bundle,
            rendezvous_addrs,
        };

        let mut payload_cbor = Vec::new();
        ciborium::into_writer(&payload, &mut payload_cbor)
            .map_err(|e| InvitationError::Serialization(e.to_string()))?;

        let mut to_sign = Vec::with_capacity(INVITATION_SIG_DOMAIN.len() + payload_cbor.len());
        to_sign.extend_from_slice(INVITATION_SIG_DOMAIN);
        to_sign.extend_from_slice(&payload_cbor);

        let signature = identity.sign(&to_sign).to_vec();

        Ok(Self {
            payload_cbor,
            signature,
        })
    }

    /// Verifies the deadline timestamp, the Ed25519 signature, and the embedded X3DH bundle signature.
    pub fn verify(&self, now_utc: i64) -> Result<ContactInvitationPayload, InvitationError> {
        let payload: ContactInvitationPayload = ciborium::from_reader(self.payload_cbor.as_slice())
            .map_err(|e| InvitationError::Serialization(format!("CBOR decode: {e}")))?;

        // 1. Strict deadline verification
        if now_utc > payload.expires_at_utc {
            return Err(InvitationError::Expired {
                expires_at_utc: payload.expires_at_utc,
                now_utc,
            });
        }

        // 2. Cryptographic signature check against the claimer's identity key
        let mut to_verify = Vec::with_capacity(INVITATION_SIG_DOMAIN.len() + self.payload_cbor.len());
        to_verify.extend_from_slice(INVITATION_SIG_DOMAIN);
        to_verify.extend_from_slice(&self.payload_cbor);

        let sig_64: &[u8; 64] = self
            .signature
            .as_slice()
            .try_into()
            .map_err(|_| InvitationError::InvalidSignature("Signature length must be 64 bytes".into()))?;

        verify_signature(
            &payload.bundle.identity_ed25519_pub,
            &to_verify,
            sig_64,
        )
        .map_err(|e| InvitationError::InvalidSignature(e.to_string()))?;

        // 3. Underlying prekey bundle signature check
        verify_prekey_bundle(&payload.bundle)
            .map_err(|e| InvitationError::Crypto(e.to_string()))?;

        Ok(payload)
    }

    /// Serializes to the standard NOVA URI format (`nova://invite?d=<hex>&onion=<address>`).
    pub fn to_uri(&self) -> Result<String, InvitationError> {
        let mut ticket_cbor = Vec::new();
        ciborium::into_writer(self, &mut ticket_cbor)
            .map_err(|e| InvitationError::Serialization(e.to_string()))?;

        let onion_suffix = if let Ok(payload) = ciborium::from_reader::<ContactInvitationPayload, _>(self.payload_cbor.as_slice()) {
            format!("&onion={}", payload.bundle.onion_address())
        } else {
            String::new()
        };

        Ok(format!("nova://invite?d={}{}", hex::encode(ticket_cbor), onion_suffix))
    }

    /// Parses a URI, link, or hex string into a `SignedContactInvitation`.
    pub fn from_uri_or_code(input: &str) -> Result<Self, InvitationError> {
        let trimmed = input.trim();
        let hex_data = if let Some(idx) = trimmed.find("?d=") {
            let after = &trimmed[idx + 3..];
            after.split('&').next().unwrap_or(after)
        } else if let Some(stripped) = trimmed.strip_prefix("nova://invite/") {
            stripped
        } else if let Some(stripped) = trimmed.strip_prefix("nova://") {
            stripped
        } else if let Some(stripped) = trimmed.strip_prefix("NOVA:") {
            stripped
        } else if let Some(stripped) = trimmed.strip_prefix("0x") {
            stripped
        } else {
            trimmed
        };

        let clean_hex: String = hex_data.chars().filter(|c| !c.is_whitespace()).collect();
        let bytes = hex::decode(&clean_hex)
            .map_err(|e| InvitationError::MalformedUri(format!("Hex decode error: {e}")))?;

        // Try decoding as SignedContactInvitation
        if let Ok(ticket) = ciborium::from_reader::<SignedContactInvitation, _>(bytes.as_slice()) {
            return Ok(ticket);
        }

        // Fallback: if it was raw PreKeyBundle bytes (legacy support), wrap it in a SignedContactInvitation
        if let Ok(bundle) = ciborium::from_reader::<PreKeyBundle, _>(bytes.as_slice()) {
            let payload = ContactInvitationPayload {
                version: 0,
                created_at_utc: chrono::Utc::now().timestamp(),
                expires_at_utc: i64::MAX, // legacy format has no deadline
                bundle,
                rendezvous_addrs: Vec::new(),
            };
            let mut payload_cbor = Vec::new();
            let _ = ciborium::into_writer(&payload, &mut payload_cbor);
            return Ok(Self {
                payload_cbor,
                signature: Vec::new(),
            });
        }

        Err(InvitationError::MalformedUri("Données d'invitation non valides".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nova_crypto::{generate_signed_prekey, MnemonicPhrase};

    fn test_identity_and_bundle(name: &str) -> (DeviceIdentity, PreKeyBundle) {
        let mnemonic = MnemonicPhrase::generate().unwrap();
        let identity = DeviceIdentity::from_mnemonic(&mnemonic, name).unwrap();
        let (_spk_sec, spk_pub) = generate_signed_prekey(&identity, 1);
        let bundle = PreKeyBundle {
            identity_ed25519_pub: identity.verifying_key_bytes,
            identity_x25519_pub: identity.dh_public_bytes,
            signed_prekey: spk_pub,
            one_time_prekey: None,
            onion_address: None,
        };
        (identity, bundle)
    }

    #[test]
    fn test_signed_contact_invitation_lifecycle_and_uri_roundtrip() {
        let (alice_id, alice_bundle) = test_identity_and_bundle("alice");
        let ttl = 86400; // 24 hours
        let addrs = vec!["/ip4/1.2.3.4/udp/4001/quic-v1".to_string()];

        let invitation = SignedContactInvitation::create(&alice_id, alice_bundle.clone(), ttl, addrs.clone()).unwrap();
        let now = chrono::Utc::now().timestamp();
        let verified = invitation.verify(now).unwrap();

        assert_eq!(verified.bundle, alice_bundle);
        assert_eq!(verified.rendezvous_addrs, addrs);
        assert!(verified.expires_at_utc >= now + ttl);

        let uri = invitation.to_uri().unwrap();
        assert!(uri.starts_with("nova://invite?d="));

        let parsed = SignedContactInvitation::from_uri_or_code(&uri).unwrap();
        let verified_parsed = parsed.verify(now).unwrap();
        assert_eq!(verified_parsed, verified);
    }

    #[test]
    fn test_expired_invitation_is_rejected() {
        let (alice_id, alice_bundle) = test_identity_and_bundle("alice");
        let ttl = 60; // 60s
        let invitation = SignedContactInvitation::create(&alice_id, alice_bundle, ttl, vec![]).unwrap();

        let future_time = chrono::Utc::now().timestamp() + 3600; // 1 hour later
        let err = invitation.verify(future_time).unwrap_err();
        match err {
            InvitationError::Expired { expires_at_utc, now_utc } => {
                assert_eq!(now_utc, future_time);
                assert!(expires_at_utc < now_utc);
            }
            other => panic!("Expected Expired error, got {other:?}"),
        }
    }

    #[test]
    fn test_tampered_invitation_is_rejected() {
        let (alice_id, alice_bundle) = test_identity_and_bundle("alice");
        let mut invitation = SignedContactInvitation::create(&alice_id, alice_bundle, 86400, vec![]).unwrap();

        // Tamper with payload byte
        if let Some(byte) = invitation.payload_cbor.get_mut(10) {
            *byte ^= 0xFF;
        }

        let now = chrono::Utc::now().timestamp();
        assert!(invitation.verify(now).is_err());
    }
}
