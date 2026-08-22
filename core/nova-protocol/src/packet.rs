use nova_crypto::{PreKeyBundle, RatchetHeader};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const NOVA_MAGIC_HEADER: [u8; 4] = [0x4E, 0x4F, 0x56, 0x41]; // "NOVA"
pub const PROTOCOL_VERSION: u8 = 0x01;

/// Hard ceiling on any single wire packet, enforced before CBOR decoding. Packets arrive
/// from an untrusted network; without this bound a hostile or corrupted peer could send a
/// header claiming an enormous payload and force unbounded allocation during deserialization.
/// 8 MiB comfortably covers a rich chat message, audio note or file-transfer chunk.
pub const MAX_PACKET_SIZE: usize = 8 * 1024 * 1024;

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("Invalid magic header")]
    InvalidMagic,
    #[error("Unsupported protocol version: {0}")]
    UnsupportedVersion(u8),
    #[error("Packet of {0} bytes exceeds the maximum allowed size of {MAX_PACKET_SIZE} bytes")]
    PacketTooLarge(usize),
    #[error("CBOR serialization error: {0}")]
    SerializationFailed(String),
    #[error("CBOR deserialization error: {0}")]
    DeserializationFailed(String),
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum FrameType {
    HandshakeInit = 0x01,
    HandshakeResponse = 0x02,
    EncryptedMessage = 0x10,
    DeliveryAck = 0x20,
    ReadAck = 0x21,
    PresencePing = 0x30,
    PresencePong = 0x31,
    RelayForward = 0x40,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum MessageContentType {
    Text,
    Image,
    Audio,
    File,
    SystemNotification,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MediaMetadata {
    pub file_name: String,
    pub mime_type: String,
    pub size_bytes: u64,
    pub sha256_checksum: String,
    pub chunk_count: u32,
}

/// Plaintext inner message content before Double Ratchet encryption.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MessagePayload {
    pub message_id: String,
    pub conversation_id: String,
    pub sender_id: String,
    pub recipient_id: String,
    pub timestamp_utc: i64,
    pub content_type: MessageContentType,
    pub text_content: Option<String>,
    pub media_meta: Option<MediaMetadata>,
    pub reply_to_id: Option<String>,
}

impl MessagePayload {
    pub fn new_text(
        message_id: String,
        conversation_id: String,
        sender_id: String,
        recipient_id: String,
        text: String,
    ) -> Self {
        Self {
            message_id,
            conversation_id,
            sender_id,
            recipient_id,
            timestamp_utc: chrono::Utc::now().timestamp(),
            content_type: MessageContentType::Text,
            text_content: Some(text),
            media_meta: None,
            reply_to_id: None,
        }
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, ProtocolError> {
        let mut buf = Vec::new();
        ciborium::into_writer(self, &mut buf)
            .map_err(|e| ProtocolError::SerializationFailed(e.to_string()))?;
        Ok(buf)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ProtocolError> {
        if bytes.len() > MAX_PACKET_SIZE {
            return Err(ProtocolError::PacketTooLarge(bytes.len()));
        }
        ciborium::from_reader(bytes)
            .map_err(|e| ProtocolError::DeserializationFailed(e.to_string()))
    }
}

/// Acknowledgment of message delivery or read state.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AckPayload {
    pub message_id: String,
    pub conversation_id: String,
    pub timestamp_utc: i64,
    pub is_read_ack: bool,
}

/// Outer encrypted envelope transmitted across QUIC streams or through the Relay.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EncryptedFrame {
    pub header: RatchetHeader,
    pub ciphertext: Vec<u8>,
}

/// Universal top-level NOVA packet frame.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct NovaPacket {
    pub magic: [u8; 4],
    pub version: u8,
    pub frame_type: FrameType,
    pub session_id: String,
    pub payload: Vec<u8>,
}

impl NovaPacket {
    pub fn new(frame_type: FrameType, session_id: String, payload: Vec<u8>) -> Self {
        Self {
            magic: NOVA_MAGIC_HEADER,
            version: PROTOCOL_VERSION,
            frame_type,
            session_id,
            payload,
        }
    }

    pub fn to_cbor(&self) -> Result<Vec<u8>, ProtocolError> {
        let mut buf = Vec::new();
        ciborium::into_writer(self, &mut buf)
            .map_err(|e| ProtocolError::SerializationFailed(e.to_string()))?;
        Ok(buf)
    }

    pub fn from_cbor(bytes: &[u8]) -> Result<Self, ProtocolError> {
        if bytes.len() > MAX_PACKET_SIZE {
            return Err(ProtocolError::PacketTooLarge(bytes.len()));
        }
        let packet: NovaPacket = ciborium::from_reader(bytes)
            .map_err(|e| ProtocolError::DeserializationFailed(e.to_string()))?;
        if packet.magic != NOVA_MAGIC_HEADER {
            return Err(ProtocolError::InvalidMagic);
        }
        if packet.version != PROTOCOL_VERSION {
            return Err(ProtocolError::UnsupportedVersion(packet.version));
        }
        Ok(packet)
    }
}

/// X3DH handshake material carried alongside the first Double Ratchet message sent to a peer
/// (mirrors Signal's "prekey message": the handshake and the first ciphertext travel together
/// so the recipient can derive the shared secret and decrypt in a single round-trip).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct HandshakeInitPayload {
    pub sender_identity_ed25519_pub: [u8; 32],
    pub sender_identity_x25519_pub: [u8; 32],
    pub sender_ephemeral_pub: [u8; 32],
    pub used_signed_prekey_id: u32,
    pub used_one_time_prekey_id: Option<u32>,
    pub first_message: EncryptedFrame,
}

impl HandshakeInitPayload {
    pub fn to_bytes(&self) -> Result<Vec<u8>, ProtocolError> {
        let mut buf = Vec::new();
        ciborium::into_writer(self, &mut buf)
            .map_err(|e| ProtocolError::SerializationFailed(e.to_string()))?;
        Ok(buf)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ProtocolError> {
        if bytes.len() > MAX_PACKET_SIZE {
            return Err(ProtocolError::PacketTooLarge(bytes.len()));
        }
        ciborium::from_reader(bytes)
            .map_err(|e| ProtocolError::DeserializationFailed(e.to_string()))
    }
}

/// Wire helpers for publishing/parsing a peer's [`PreKeyBundle`] (e.g. embedded in a QR code
/// or exchanged directly), independent of the `HandshakeInit` frame used once a session starts.
pub fn prekey_bundle_to_bytes(bundle: &PreKeyBundle) -> Result<Vec<u8>, ProtocolError> {
    let mut buf = Vec::new();
    ciborium::into_writer(bundle, &mut buf)
        .map_err(|e| ProtocolError::SerializationFailed(e.to_string()))?;
    Ok(buf)
}

pub fn prekey_bundle_from_bytes(bytes: &[u8]) -> Result<PreKeyBundle, ProtocolError> {
    if bytes.len() > MAX_PACKET_SIZE {
        return Err(ProtocolError::PacketTooLarge(bytes.len()));
    }
    ciborium::from_reader(bytes).map_err(|e| ProtocolError::DeserializationFailed(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_payload_cbor_roundtrip() {
        let payload = MessagePayload::new_text(
            "msg-123".into(),
            "conv-456".into(),
            "alice_pubkey".into(),
            "bob_pubkey".into(),
            "Bonjour P2P!".into(),
        );

        let bytes = payload.to_bytes().unwrap();
        let decoded = MessagePayload::from_bytes(&bytes).unwrap();
        assert_eq!(payload, decoded);
    }

    #[test]
    fn test_nova_packet_serialization() {
        let packet = NovaPacket::new(
            FrameType::EncryptedMessage,
            "session_xyz".into(),
            vec![1, 2, 3, 4, 5],
        );

        let cbor = packet.to_cbor().unwrap();
        let decoded = NovaPacket::from_cbor(&cbor).unwrap();
        assert_eq!(packet, decoded);
    }

    #[test]
    fn test_oversized_packet_is_rejected_before_decoding() {
        let oversized = vec![0u8; MAX_PACKET_SIZE + 1];
        let err = NovaPacket::from_cbor(&oversized).unwrap_err();
        assert!(matches!(err, ProtocolError::PacketTooLarge(_)));

        let err2 = MessagePayload::from_bytes(&oversized).unwrap_err();
        assert!(matches!(err2, ProtocolError::PacketTooLarge(_)));
    }
}
