use nova_crypto::{PreKeyBundle, RatchetHeader};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const NOVA_MAGIC_HEADER: [u8; 4] = [0x4E, 0x4F, 0x56, 0x41]; // "NOVA"
pub const PROTOCOL_VERSION: u8 = 0x01;

/// Hard ceiling on any single wire packet, enforced before CBOR decoding. Packets arrive
/// from an untrusted network; without this bound a hostile or corrupted peer could send a
/// header claiming an enormous payload and force unbounded allocation during deserialization.
///
/// This must stay comfortably under 1 MiB: every packet travels as one `libp2p-request-response`
/// *request*, and that crate's `cbor` codec hard-codes `REQUEST_SIZE_MAXIMUM = 1 MiB` with no
/// public knob to raise it (`libp2p_request_response::cbor::codec`) — the receiving side's
/// `io.take(REQUEST_SIZE_MAXIMUM).read_to_end(..)` silently truncates anything larger before our
/// own size check ever runs, so a packet at or above that transport ceiling never arrives at all
/// (it fails to CBOR-decode on the receiving end and the sender eventually times out waiting for
/// a response — this is what a naive 6 MiB `MEDIA_CHUNK_SIZE` hit before this was caught during
/// the 2026-08-22 remediation's final verification pass, appearing only as a "never arrives" test
/// failure with no error, since the failure happens below this crate).
pub const MAX_PACKET_SIZE: usize = 900 * 1024;

/// Target size for one media chunk's raw binary payload (see `MessagePayload::chunk_bytes`), set
/// safely below `MAX_PACKET_SIZE` (which is itself capped by the libp2p transport's 1 MiB request
/// ceiling — see its doc comment) to leave headroom for the Double Ratchet AEAD tag, the CBOR
/// envelope, and the outer `NovaPacket` wrapper — none of which are free, and all of which are
/// applied on top of this payload before it becomes one wire packet. A file larger than this is
/// split across multiple chunks (see `MediaMetadata::chunk_count`) rather than sent as one
/// oversized packet — the fix for the 2026-08-22 audit's "video is capped at 5 MB with no
/// chunking" finding.
pub const MEDIA_CHUNK_SIZE: usize = 512 * 1024;

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
    /// A recorded/attached video file. Distinct from `Image` and `File` so the UI can render it
    /// with a video player instead of an `<img>`/generic-file icon — see the 2026-08-22 audit's
    /// "video is capped at 5 MB with no compression" finding, fixed by chunking (this content
    /// type just needed to exist for a real video message to be represented at all).
    Video,
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
///
/// A media message (`content_type` other than `Text`/`SystemNotification`) travels as one
/// `MessagePayload` PER CHUNK, all sharing the same `message_id` and `content_type` — see
/// `MEDIA_CHUNK_SIZE`. `chunk_index` 0 carries `media_meta` (the full file's name/mime/size/
/// checksum/`chunk_count`); every chunk carries its own `chunk_bytes` slice. A `Text` message is
/// simply the degenerate case of one chunk (`chunk_index: None`, `chunk_bytes: None`).
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
    /// Present only on a media chunk: this chunk's 0-based index within `media_meta.chunk_count`
    /// (as seen on chunk 0's payload) — `None` for a plain text message.
    #[serde(default)]
    pub chunk_index: Option<u32>,
    /// Present only on a media chunk: this chunk's raw binary slice of the file — `None` for a
    /// plain text message. Never base64-encoded here; base64 is strictly a JS/IPC-boundary
    /// convenience in `ui/src-tauri`, not part of the wire format.
    ///
    /// `#[serde(with = "serde_bytes")]` is load-bearing, not cosmetic: plain `serde::Serialize`
    /// for `Vec<u8>` encodes it as a CBOR array of one integer item per byte (no built-in
    /// specialization for `T = u8`), not a compact CBOR byte string. For a `MEDIA_CHUNK_SIZE`
    /// chunk that alone roughly doubles the size — and compounds again at every later layer this
    /// plaintext gets wrapped in (`EncryptedFrame::ciphertext`, then `NovaPacket::payload` — see
    /// their own `serde_bytes` annotations), multiplying into several times the original size and
    /// blowing straight through `MAX_PACKET_SIZE`. Caught by
    /// `test_send_and_receive_media_message_multi_chunk_reassembly` during the 2026-08-22 audit's
    /// J4 implementation.
    #[serde(default, with = "serde_bytes")]
    pub chunk_bytes: Option<Vec<u8>>,
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
            chunk_index: None,
            chunk_bytes: None,
        }
    }

    /// Builds one chunk's payload for a media message. `media_meta` must be `Some` on
    /// `chunk_index == 0` and `None` on every other chunk (the header travels exactly once).
    #[allow(clippy::too_many_arguments)]
    pub fn new_media_chunk(
        message_id: String,
        conversation_id: String,
        sender_id: String,
        recipient_id: String,
        content_type: MessageContentType,
        media_meta: Option<MediaMetadata>,
        chunk_index: u32,
        chunk_bytes: Vec<u8>,
    ) -> Self {
        Self {
            message_id,
            conversation_id,
            sender_id,
            recipient_id,
            timestamp_utc: chrono::Utc::now().timestamp(),
            content_type,
            text_content: None,
            media_meta,
            reply_to_id: None,
            chunk_index: Some(chunk_index),
            chunk_bytes: Some(chunk_bytes),
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
///
/// See `MessagePayload::chunk_bytes`'s doc comment for why `#[serde(with = "serde_bytes")]` on
/// `ciphertext` is load-bearing, not cosmetic — this is the second of three compounding layers
/// that finding covers.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EncryptedFrame {
    pub header: RatchetHeader,
    #[serde(with = "serde_bytes")]
    pub ciphertext: Vec<u8>,
}

/// Universal top-level NOVA packet frame.
///
/// See `MessagePayload::chunk_bytes`'s doc comment for why `#[serde(with = "serde_bytes")]` on
/// `payload` is load-bearing, not cosmetic — this is the third of three compounding layers that
/// finding covers.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct NovaPacket {
    pub magic: [u8; 4],
    pub version: u8,
    pub frame_type: FrameType,
    pub session_id: String,
    #[serde(with = "serde_bytes")]
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
    fn test_media_chunk_payload_cbor_roundtrip() {
        let header = MessagePayload::new_media_chunk(
            "msg-media-1".into(),
            "conv-1".into(),
            "alice".into(),
            "bob".into(),
            MessageContentType::Image,
            Some(MediaMetadata {
                file_name: "photo.jpg".into(),
                mime_type: "image/jpeg".into(),
                size_bytes: 12_000_000,
                sha256_checksum: "deadbeef".into(),
                chunk_count: 2,
            }),
            0,
            vec![1, 2, 3, 4],
        );
        let bytes = header.to_bytes().unwrap();
        let decoded = MessagePayload::from_bytes(&bytes).unwrap();
        assert_eq!(header, decoded);
        assert_eq!(decoded.chunk_index, Some(0));
        assert_eq!(decoded.media_meta.unwrap().chunk_count, 2);

        let tail = MessagePayload::new_media_chunk(
            "msg-media-1".into(),
            "conv-1".into(),
            "alice".into(),
            "bob".into(),
            MessageContentType::Image,
            None,
            1,
            vec![5, 6, 7, 8],
        );
        let tail_decoded = MessagePayload::from_bytes(&tail.to_bytes().unwrap()).unwrap();
        assert_eq!(tail_decoded.chunk_index, Some(1));
        assert!(tail_decoded.media_meta.is_none());
    }

    /// A plain text `MessagePayload` (built before `chunk_index`/`chunk_bytes` existed) must
    /// still decode correctly — `#[serde(default)]` on the new fields is what makes that true.
    #[test]
    fn test_text_payload_without_chunk_fields_still_decodes() {
        let payload = MessagePayload::new_text(
            "msg-1".into(),
            "conv-1".into(),
            "alice".into(),
            "bob".into(),
            "hello".into(),
        );
        let decoded = MessagePayload::from_bytes(&payload.to_bytes().unwrap()).unwrap();
        assert_eq!(decoded.chunk_index, None);
        assert_eq!(decoded.chunk_bytes, None);
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
