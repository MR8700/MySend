//! QUIC/TLS setup for direct peer-to-peer connections.
//!
//! The TLS certificate used here is a throwaway, self-signed one generated fresh on every
//! process start. That is intentional, not a shortcut: QUIC requires *some* TLS handshake to
//! exist below it, but this application does not rely on that TLS layer for peer authentication
//! — real authentication happens one layer up, via the already-verified X3DH/Ed25519 identity
//! handshake carried inside the connection (see `nova-crypto::x3dh` and `nova-engine`). The QUIC
//! transport's job is only to give us an encrypted, multiplexed, congestion-controlled pipe
//! between two already-cryptographically-authenticated devices — so the client deliberately
//! skips certificate validation (`SkipServerVerification`) rather than standing up a
//! certificate authority no sovereign P2P app could realistically operate anyway.

use rcgen::generate_simple_self_signed;
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::crypto::CryptoProvider;
use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer, ServerName, UnixTime};
use rustls::{DigitallySignedStruct, SignatureScheme};
use std::sync::Arc;
use thiserror::Error;

/// Application-layer protocol identifier negotiated via TLS ALPN. Both sides must offer the
/// same value or the QUIC handshake fails.
pub const NOVA_ALPN: &[u8] = b"nova-p2p/1";

#[derive(Debug, Error)]
pub enum TlsSetupError {
    #[error("failed to generate self-signed certificate: {0}")]
    CertGeneration(String),
    #[error("failed to build TLS configuration: {0}")]
    RustlsConfig(String),
}

fn crypto_provider() -> Arc<CryptoProvider> {
    Arc::new(rustls::crypto::ring::default_provider())
}

/// Builds a fresh, throwaway self-signed identity for the local QUIC endpoint's server role.
fn generate_ephemeral_cert() -> Result<(CertificateDer<'static>, PrivatePkcs8KeyDer<'static>), TlsSetupError> {
    let certified_key = generate_simple_self_signed(vec!["nova-p2p.local".to_string()])
        .map_err(|e| TlsSetupError::CertGeneration(e.to_string()))?;
    let cert_der = certified_key.cert.der().clone();
    let key_der = PrivatePkcs8KeyDer::from(certified_key.key_pair.serialize_der());
    Ok((cert_der, key_der))
}

/// Server-side QUIC config: accepts incoming direct connections from peers.
pub fn build_server_config() -> Result<quinn::ServerConfig, TlsSetupError> {
    let (cert_der, key_der) = generate_ephemeral_cert()?;
    let provider = crypto_provider();

    let mut rustls_config = rustls::ServerConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(|e| TlsSetupError::RustlsConfig(e.to_string()))?
        .with_no_client_auth()
        .with_single_cert(vec![cert_der], key_der.into())
        .map_err(|e| TlsSetupError::RustlsConfig(e.to_string()))?;
    rustls_config.alpn_protocols = vec![NOVA_ALPN.to_vec()];
    rustls_config.max_early_data_size = u32::MAX;

    let quic_crypto = quinn::crypto::rustls::QuicServerConfig::try_from(rustls_config)
        .map_err(|e| TlsSetupError::RustlsConfig(e.to_string()))?;
    let mut server_config = quinn::ServerConfig::with_crypto(Arc::new(quic_crypto));
    // NAT mappings on real mobile/carrier networks (the Ouagadougou/Bobo-Dioulasso scenario this
    // is built for) can be short-lived; keep idle connections alive with frequent keep-alives
    // rather than letting the NAT binding silently expire mid-conversation.
    let mut transport = quinn::TransportConfig::default();
    transport.keep_alive_interval(Some(std::time::Duration::from_secs(10)));
    server_config.transport_config(Arc::new(transport));

    Ok(server_config)
}

/// Client-side QUIC config: dials out to a peer's observed address. Certificate validation is
/// intentionally disabled — see module docs.
pub fn build_client_config() -> Result<quinn::ClientConfig, TlsSetupError> {
    let provider = crypto_provider();

    let mut rustls_config = rustls::ClientConfig::builder_with_provider(provider.clone())
        .with_safe_default_protocol_versions()
        .map_err(|e| TlsSetupError::RustlsConfig(e.to_string()))?
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(SkipServerVerification(provider)))
        .with_no_client_auth();
    rustls_config.alpn_protocols = vec![NOVA_ALPN.to_vec()];

    let quic_crypto = quinn::crypto::rustls::QuicClientConfig::try_from(rustls_config)
        .map_err(|e| TlsSetupError::RustlsConfig(e.to_string()))?;
    let mut client_config = quinn::ClientConfig::new(Arc::new(quic_crypto));
    let mut transport = quinn::TransportConfig::default();
    transport.keep_alive_interval(Some(std::time::Duration::from_secs(10)));
    client_config.transport_config(Arc::new(transport));

    Ok(client_config)
}

#[derive(Debug)]
struct SkipServerVerification(Arc<CryptoProvider>);

impl ServerCertVerifier for SkipServerVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls12_signature(
            message,
            cert,
            dss,
            &self.0.signature_verification_algorithms,
        )
        .map(|_| HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls13_signature(
            message,
            cert,
            dss,
            &self.0.signature_verification_algorithms,
        )
        .map(|_| HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.0.signature_verification_algorithms.supported_schemes()
    }
}
