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
///
/// `#[serde(with = "serde_bytes")]` on both fields below is load-bearing, not cosmetic: without
/// it, plain `serde::Serialize` for `Vec<u8>` encodes it as a CBOR array of one integer item per
/// byte (no built-in specialization for `T = u8`) instead of a compact CBOR byte string — see the
/// identical footgun documented on `MessagePayload::chunk_bytes` in nova-protocol/src/packet.rs.
/// `payload_cbor` is a doubly costly instance of this: it already holds an inner CBOR encoding
/// (itself inflated ~2x by `SignedPreKeyPublic::signature`'s own missing annotation — now fixed
/// in nova-crypto), so re-encoding *that* as an array-of-ints multiplies the bloat again, several
/// times over — which previously blew the resulting `nova://invite?d=...` URI straight through
/// the QR code format's max capacity ("The amount of data is too big to be stored in a QR Code").
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SignedContactInvitation {
    #[serde(with = "serde_bytes")]
    pub payload_cbor: Vec<u8>,
    #[serde(with = "serde_bytes")]
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

    /// Serializes to the standard NOVA URI format (`nova://invite?d=<hex>`).
    pub fn to_uri(&self) -> Result<String, InvitationError> {
        let mut ticket_cbor = Vec::new();
        ciborium::into_writer(self, &mut ticket_cbor)
            .map_err(|e| InvitationError::Serialization(e.to_string()))?;

        Ok(format!("nova://invite?d={}", hex::encode(ticket_cbor)))
    }
}

/// Result of parsing a pasted invitation code/URI/link: either a fully signed, time-limited
/// ticket (the normal path — see [`SignedContactInvitation::create`]), or a bare legacy prekey
/// bundle with no signature or deadline of its own (the fallback the UI shows when it cannot
/// mint a signed ticket — see `get_own_prekey_bundle_hex` on the Tauri side).
///
/// An earlier version of this parser collapsed both cases into a `SignedContactInvitation` by
/// wrapping the legacy bundle in one with an empty `signature`. That made the two
/// indistinguishable once returned, so every caller had to remember to special-case an empty
/// signature *before* calling `.verify()` — and `NovaEngine::add_contact` didn't, so a perfectly
/// valid legacy code was rejected every single time with a signature-length error. Two explicit
/// variants make that mistake impossible to repeat: there is no `.verify()` to forget to skip.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParsedInvitation {
    Signed(SignedContactInvitation),
    LegacyBundle(PreKeyBundle),
}

impl ParsedInvitation {
    /// Parses a URI, link, or bare hex string into either a signed invitation ticket or a
    /// legacy prekey bundle. Performs no verification — callers MUST call
    /// [`SignedContactInvitation::verify`] or [`verify_prekey_bundle`] respectively before
    /// trusting the result.
    pub fn parse(input: &str) -> Result<Self, InvitationError> {
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

        if let Ok(ticket) = ciborium::from_reader::<SignedContactInvitation, _>(bytes.as_slice()) {
            return Ok(ParsedInvitation::Signed(ticket));
        }

        if let Ok(bundle) = ciborium::from_reader::<PreKeyBundle, _>(bytes.as_slice()) {
            return Ok(ParsedInvitation::LegacyBundle(bundle));
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

        let parsed = ParsedInvitation::parse(&uri).unwrap();
        let ticket = match parsed {
            ParsedInvitation::Signed(t) => t,
            ParsedInvitation::LegacyBundle(_) => panic!("expected a signed ticket, got a legacy bundle"),
        };
        let verified_parsed = ticket.verify(now).unwrap();
        assert_eq!(verified_parsed, verified);
    }

    /// Regression test for the missing `#[serde(with = "serde_bytes")]` bug: without it, every
    /// `Vec<u8>` in `PreKeyBundle`/`SignedContactInvitation` was CBOR-encoded as an array of one
    /// integer per byte instead of a compact byte string, and — because `payload_cbor` is itself
    /// already-encoded CBOR bytes re-embedded as a `Vec<u8>` field — that inflation compounded on
    /// re-encoding, several times over. In production this pushed the invitation URI's length
    /// past a QR code's absolute max capacity, so the Identity screen's QR code silently failed
    /// to render ("The amount of data is too big to be stored in a QR Code").
    ///
    /// Builds the same realistic, worst-case bundle `get_own_prekey_bundle`
    /// (nova-storage/src/db.rs) actually produces — unlike `test_identity_and_bundle` above, this
    /// fills in `one_time_prekey`, which every real invitation has — plus a dual-stack (IPv4 +
    /// IPv6) rendezvous address list with the trailing `/p2p/<peer-id>` suffix
    /// `own_full_listen_addrs` actually appends, then asserts the resulting URI stays well under
    /// a QR code's max byte-mode capacity (2331 bytes at error-correction level 'M', version 40 —
    /// see vendor/qrcode.js's `Version.getBestVersionForData`), with generous headroom for the
    /// `errorCorrectionLevel: 'M'` the UI actually requests.
    #[test]
    fn test_realistic_invitation_uri_fits_in_a_qr_code() {
        let mnemonic = MnemonicPhrase::generate().unwrap();
        let identity = DeviceIdentity::from_mnemonic(&mnemonic, "alice").unwrap();
        let (_spk_sec, spk_pub) = generate_signed_prekey(&identity, 1);
        let (_opk_sec, opk_pub) = nova_crypto::generate_one_time_prekey(2);
        let bundle = PreKeyBundle {
            identity_ed25519_pub: identity.verifying_key_bytes,
            identity_x25519_pub: identity.dh_public_bytes,
            signed_prekey: spk_pub,
            one_time_prekey: Some(opk_pub),
        };
        let libp2p_peer_id = "12D3KooWGjMGZjJZgQjJZgQjJZgQjJZgQjJZgQjJZgQjJZgQjJZg"; // realistic length
        let addrs = vec![
            format!("/ip4/203.0.113.42/udp/4001/quic-v1/p2p/{libp2p_peer_id}"),
            format!("/ip6/2001:db8::1/udp/4001/quic-v1/p2p/{libp2p_peer_id}"),
        ];

        let invitation = SignedContactInvitation::create(&identity, bundle, 86400, addrs).unwrap();
        let uri = invitation.to_uri().unwrap();

        // 2200 leaves ~130 bytes of headroom below the hard 2331-byte cap (room for e.g. a
        // slightly longer multiaddr) while still catching a regression of the serde_bytes bug:
        // without it, this same realistic bundle measured ~4x larger, several thousand bytes
        // over the cap.
        assert!(
            uri.len() < 2200,
            "invitation URI is {} bytes — too close to (or over) a QR code's ~2331-byte max \
             capacity at error-correction level 'M'; check for a missing #[serde(with = \"serde_bytes\")] \
             on a Vec<u8> field somewhere in PreKeyBundle/SignedContactInvitation's encoding path",
            uri.len()
        );
    }

    /// The exact bug from the 2026-08-22 audit: the UI's fallback "share code" (a bare hex
    /// prekey bundle, no `nova://` wrapper) used to be parsed into an unsigned ticket that could
    /// never pass `.verify()`, so `add_contact` rejected it unconditionally. `ParsedInvitation`
    /// must surface it as `LegacyBundle` instead, verifiable via `verify_prekey_bundle` directly.
    #[test]
    fn test_legacy_bare_hex_bundle_parses_as_legacy_and_verifies() {
        let (_bob_id, bob_bundle) = test_identity_and_bundle("bob");

        let mut bundle_cbor = Vec::new();
        ciborium::into_writer(&bob_bundle, &mut bundle_cbor).unwrap();
        let bare_hex = hex::encode(bundle_cbor);

        let parsed = ParsedInvitation::parse(&bare_hex).unwrap();
        match parsed {
            ParsedInvitation::LegacyBundle(bundle) => {
                assert_eq!(bundle, bob_bundle);
                verify_prekey_bundle(&bundle).expect("legacy bundle signature must still verify");
            }
            ParsedInvitation::Signed(_) => panic!("expected a legacy bundle, got a signed ticket"),
        }
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
