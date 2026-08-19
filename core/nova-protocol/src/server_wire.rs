use crate::packet::{ProtocolError, MAX_PACKET_SIZE};
use crate::presence::{PeerEndpoint, SignedDrainRequest, SignedPresenceRegistration};
use serde::{Deserialize, Serialize};

/// Requests understood by the discovery/signaling server's UDP endpoint.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ServerRequest {
    /// Announce (or refresh) this peer's reachability. Must be signed by the peer's own
    /// identity key — see [`SignedPresenceRegistration`].
    Register(SignedPresenceRegistration),
    /// Look up a peer's last-announced reachability, for NAT hole punching.
    Lookup { peer_id: String },
    /// Deposit an opaque, already E2E-encrypted blob for a peer who is currently unreachable
    /// directly. The server cannot decrypt or otherwise interpret `payload`.
    RelayForward { target_peer_id: String, payload: Vec<u8> },
    /// Fetch and purge all packets queued for the caller. Must be signed to prove ownership of
    /// the peer_id being drained — see [`SignedDrainRequest`].
    RelayDrain(SignedDrainRequest),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ServerResponse {
    Registered,
    LookupResult(Option<PeerEndpoint>),
    RelayForwarded,
    RelayDrained(Vec<Vec<u8>>),
    Error(String),
}

impl ServerRequest {
    pub fn to_bytes(&self) -> Result<Vec<u8>, ProtocolError> {
        let mut buf = Vec::new();
        ciborium::into_writer(self, &mut buf).map_err(|e| ProtocolError::SerializationFailed(e.to_string()))?;
        Ok(buf)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ProtocolError> {
        if bytes.len() > MAX_PACKET_SIZE {
            return Err(ProtocolError::PacketTooLarge(bytes.len()));
        }
        ciborium::from_reader(bytes).map_err(|e| ProtocolError::DeserializationFailed(e.to_string()))
    }
}

impl ServerResponse {
    pub fn to_bytes(&self) -> Result<Vec<u8>, ProtocolError> {
        let mut buf = Vec::new();
        ciborium::into_writer(self, &mut buf).map_err(|e| ProtocolError::SerializationFailed(e.to_string()))?;
        Ok(buf)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ProtocolError> {
        if bytes.len() > MAX_PACKET_SIZE {
            return Err(ProtocolError::PacketTooLarge(bytes.len()));
        }
        ciborium::from_reader(bytes).map_err(|e| ProtocolError::DeserializationFailed(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_request_roundtrip() {
        let req = ServerRequest::Lookup {
            peer_id: "aabb".to_string(),
        };
        let bytes = req.to_bytes().unwrap();
        let decoded = ServerRequest::from_bytes(&bytes).unwrap();
        assert!(matches!(decoded, ServerRequest::Lookup { peer_id } if peer_id == "aabb"));
    }

    #[test]
    fn test_response_roundtrip() {
        let resp = ServerResponse::RelayDrained(vec![b"a".to_vec(), b"b".to_vec()]);
        let bytes = resp.to_bytes().unwrap();
        let decoded = ServerResponse::from_bytes(&bytes).unwrap();
        match decoded {
            ServerResponse::RelayDrained(items) => assert_eq!(items.len(), 2),
            _ => panic!("unexpected variant"),
        }
    }
}
