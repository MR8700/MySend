//! Tor Onion Routing & SOCKS5 Anonymization Module
//!
//! Provides SOCKS5 proxy negotiation, Tor Onion v3 routing controls,
//! circuit validation, and strict privacy boundary gating to prevent IP address leakage.

use std::net::SocketAddr;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::debug;
use crate::error::TransportError;

/// Operating transport mode for privacy and onion routing.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TorMode {
    /// Standard P2P: direct QUIC/UDP, DCUtR hole punching, direct relay.
    DirectOnly,
    /// Hybrid: direct connections for local LAN / bootstrap, Tor SOCKS5 for `.onion` peers.
    Hybrid,
    /// Tor Strict: all outbound connections are strictly gated through Tor circuits.
    /// Direct IPv4/IPv6 connections to unknown peers are dropped to guarantee zero IP leaks.
    TorStrict,
}

impl Default for TorMode {
    fn default() -> Self {
        TorMode::DirectOnly
    }
}

/// Configuration parameters for Tor anonymization.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TorConfig {
    pub enabled: bool,
    pub mode: TorMode,
    pub socks_proxy: String,
    pub onion_address: Option<String>,
    pub bridge_type: Option<String>, // "snowflake", "obfs4", or None (direct)
}

impl Default for TorConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            mode: TorMode::DirectOnly,
            socks_proxy: "127.0.0.1:9050".to_string(),
            onion_address: None,
            bridge_type: None,
        }
    }
}

/// Live status report of the Tor subsystem.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TorStatus {
    pub enabled: bool,
    pub connected: bool,
    pub bootstrap_percent: u8,
    pub onion_address: String,
    pub socks_proxy: String,
    pub mode: TorMode,
    pub bridge_type: Option<String>,
}

/// Manages Tor circuit status and SOCKS5 proxy stream establishment.
pub struct TorManager {
    config: TorConfig,
}

impl TorManager {
    pub fn new(config: TorConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &TorConfig {
        &self.config
    }

    pub fn update_config(&mut self, config: TorConfig) {
        self.config = config;
    }

    /// Checks whether an address string references a Tor `.onion` hidden service.
    pub fn is_onion_address(addr: &str) -> bool {
        let clean = addr.trim().to_ascii_lowercase();
        clean.contains(".onion")
    }

    /// Verifies if an outbound dial is permitted under the current Tor privacy policy.
    /// Returns Ok(()) if permitted, or Err(TransportError) if it violates TorStrict mode.
    pub fn validate_outbound_dial(&self, target_addr: &str) -> Result<(), TransportError> {
        match self.config.mode {
            TorMode::DirectOnly => Ok(()),
            TorMode::Hybrid => Ok(()),
            TorMode::TorStrict => {
                if Self::is_onion_address(target_addr)
                    || target_addr.contains("127.0.0.1")
                    || target_addr.contains("localhost")
                    || target_addr.contains("::1")
                {
                    Ok(())
                } else {
                    Err(TransportError::Setup(format!(
                        "TorStrict mode: direct non-onion dial to '{target_addr}' blocked to prevent IP leak"
                    )))
                }
            }
        }
    }

    /// Probes the local SOCKS5 proxy to verify whether the Tor daemon / Arti is running and ready.
    pub async fn check_proxy_liveness(&self) -> bool {
        let proxy_addr = match self.config.socks_proxy.parse::<SocketAddr>() {
            Ok(addr) => addr,
            Err(_) => return false,
        };

        match tokio::time::timeout(Duration::from_millis(1500), TcpStream::connect(proxy_addr)).await {
            Ok(Ok(mut stream)) => {
                // Send SOCKS5 greeting [VER=5, NMETHODS=1, NO_AUTH=0]
                if stream.write_all(&[0x05, 0x01, 0x00]).await.is_err() {
                    return false;
                }
                let mut response = [0u8; 2];
                if stream.read_exact(&mut response).await.is_err() {
                    return false;
                }
                // Expect [VER=5, METHOD=0 (No authentication required)]
                response == [0x05, 0x00]
            }
            _ => false,
        }
    }

    /// Establishes an anonymous TCP stream to a remote `.onion` hidden service via SOCKS5.
    /// SOCKS5 command 0x01 (CONNECT) with address type 0x03 (DOMAINNAME) prevents any local DNS leak.
    pub async fn connect_onion_stream(
        &self,
        target_onion: &str,
        target_port: u16,
    ) -> Result<TcpStream, TransportError> {
        let proxy_addr = self.config.socks_proxy.parse::<SocketAddr>().map_err(|e| {
            TransportError::Setup(format!("invalid SOCKS5 proxy address '{}': {e}", self.config.socks_proxy))
        })?;

        let clean_onion = target_onion.trim().to_ascii_lowercase();
        let domain = clean_onion.strip_prefix("http://")
            .unwrap_or(&clean_onion)
            .strip_prefix("https://")
            .unwrap_or(&clean_onion);
        let domain = domain.split(':').next().unwrap_or(domain);

        if domain.len() > 255 {
            return Err(TransportError::Setup("onion hostname exceeds maximum 255 bytes".into()));
        }

        let mut stream = TcpStream::connect(proxy_addr).await.map_err(|e| {
            TransportError::Setup(format!("failed to connect to Tor SOCKS5 proxy at {proxy_addr}: {e}"))
        })?;

        // 1. Send SOCKS5 Greeting: [VER=5, NMETHODS=1, METHOD=0x00]
        stream.write_all(&[0x05, 0x01, 0x00]).await.map_err(|e| {
            TransportError::Setup(format!("failed to write SOCKS5 greeting: {e}"))
        })?;

        let mut greeting_resp = [0u8; 2];
        stream.read_exact(&mut greeting_resp).await.map_err(|e| {
            TransportError::Setup(format!("failed to read SOCKS5 greeting response: {e}"))
        })?;

        if greeting_resp[0] != 0x05 || greeting_resp[1] != 0x00 {
            return Err(TransportError::Setup(format!(
                "SOCKS5 proxy rejected no-auth method: [0x{:02x}, 0x{:02x}]",
                greeting_resp[0], greeting_resp[1]
            )));
        }

        // 2. Send SOCKS5 CONNECT Request with DOMAINNAME (0x03)
        let domain_bytes = domain.as_bytes();
        let mut request = Vec::with_capacity(7 + domain_bytes.len());
        request.push(0x05); // Version 5
        request.push(0x01); // Command: CONNECT
        request.push(0x00); // Reserved
        request.push(0x03); // Address Type: DOMAINNAME (Tor resolves remotely)
        request.push(domain_bytes.len() as u8);
        request.extend_from_slice(domain_bytes);
        request.extend_from_slice(&target_port.to_be_bytes());

        stream.write_all(&request).await.map_err(|e| {
            TransportError::Setup(format!("failed to write SOCKS5 connect request: {e}"))
        })?;

        // 3. Read SOCKS5 Response: [VER, REP, RSV, ATYP, BND.ADDR, BND.PORT]
        let mut head = [0u8; 4];
        stream.read_exact(&mut head).await.map_err(|e| {
            TransportError::Setup(format!("failed to read SOCKS5 connect response: {e}"))
        })?;

        if head[0] != 0x05 {
            return Err(TransportError::Setup("invalid SOCKS5 response version".into()));
        }

        if head[1] != 0x00 {
            let error_msg = match head[1] {
                0x01 => "general SOCKS server failure",
                0x02 => "connection not allowed by ruleset",
                0x03 => "network unreachable",
                0x04 => "host unreachable (onion service may be offline)",
                0x05 => "connection refused by onion service",
                0x06 => "TTL expired",
                0x07 => "command not supported",
                0x08 => "address type not supported",
                _ => "unknown SOCKS5 error",
            };
            return Err(TransportError::Setup(format!("Tor SOCKS5 connect failed: {error_msg} (0x{:02x})", head[1])));
        }

        // Drain bound address bytes
        let mut bound_drain = match head[3] {
            0x01 => vec![0u8; 4 + 2], // IPv4 + Port
            0x04 => vec![0u8; 16 + 2], // IPv6 + Port
            0x03 => {
                let mut len_buf = [0u8; 1];
                stream.read_exact(&mut len_buf).await.map_err(|e| TransportError::Setup(e.to_string()))?;
                vec![0u8; len_buf[0] as usize + 2]
            }
            _ => return Err(TransportError::Setup("invalid SOCKS5 bound address type".into())),
        };
        stream.read_exact(&mut bound_drain).await.map_err(|e| TransportError::Setup(e.to_string()))?;

        debug!("Established Tor SOCKS5 circuit to {domain}:{target_port}");
        Ok(stream)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_onion_address() {
        assert!(TorManager::is_onion_address("expyuzz5wqqfdgah56trgahhqhnkrybe5vdggqnxzjptlkpq5ldnfiyd.onion"));
        assert!(TorManager::is_onion_address("/dns4/mytestaddress.onion/tcp/8443"));
        assert!(!TorManager::is_onion_address("192.168.1.1"));
        assert!(!TorManager::is_onion_address("/ip4/127.0.0.1/udp/4001/quic-v1"));
    }

    #[test]
    fn test_tor_strict_mode_blocks_clearnet_dials() {
        let mut config = TorConfig::default();
        config.mode = TorMode::TorStrict;
        let manager = TorManager::new(config);

        // Allowed: Onion addresses and localhost
        assert!(manager.validate_outbound_dial("testservice.onion").is_ok());
        assert!(manager.validate_outbound_dial("127.0.0.1:9050").is_ok());

        // Blocked: Public IP / clearnet addresses
        assert!(manager.validate_outbound_dial("/ip4/82.64.12.34/udp/4001/quic-v1").is_err());
        assert!(manager.validate_outbound_dial("example.com:4001").is_err());
    }

    #[test]
    fn test_hybrid_and_direct_modes_permit_all() {
        let mut config = TorConfig::default();
        config.mode = TorMode::Hybrid;
        let manager = TorManager::new(config);

        assert!(manager.validate_outbound_dial("testservice.onion").is_ok());
        assert!(manager.validate_outbound_dial("/ip4/1.2.3.4/udp/4001").is_ok());
    }
}
