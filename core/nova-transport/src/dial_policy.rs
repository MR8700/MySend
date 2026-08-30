//! This device's IP-hiding-from-contact security property (formerly provided by Tor's
//! `TorStrict` mode — see the 2026-08-27 removal of embedded Tor support) without needing an
//! embedded Tor client, a slow bootstrap, or a third-party network this app's own reachability
//! depended on.
//!
//! The rule is simple and unconditional (not a Settings toggle — there is no way to turn it
//! off): a direct dial is only ever permitted to an address on the same LAN (private, loopback,
//! or link-local) or to a relay-circuit address (`/p2p-circuit` in the multiaddr, which never
//! reveals this device's real IP to whoever it's dialing — the relay forwards opaque bytes, it
//! never sees plaintext, exactly like the discovery DHT itself). A direct dial to any other
//! (i.e. public WAN) address is refused outright. Combined with `dcutr::Behaviour` being
//! deliberately absent from `NovaBehaviour` (see `dht_node`'s module doc comment — DCUtR would
//! otherwise silently upgrade a relayed connection to a direct one via hole-punching, defeating
//! this rule the moment it succeeded), this means: LAN peers connect directly (nothing to hide
//! from someone already on the same network), and every WAN peer is reached exclusively through
//! a relay circuit.

use crate::error::TransportError;
use libp2p::core::multiaddr::Protocol;
use libp2p::Multiaddr;

/// Returns `Ok(())` if `target_addr` is safe to dial directly (LAN/loopback/link-local, or
/// already a relayed `/p2p-circuit` address), `Err` otherwise. See this module's doc comment.
pub fn validate_outbound_dial(target_addr: &Multiaddr) -> Result<(), TransportError> {
    let is_relayed = target_addr.iter().any(|p| matches!(p, Protocol::P2pCircuit));
    if is_relayed {
        // Relayed — the relay forwards opaque (already end-to-end-encrypted) bytes and never
        // learns this device's real IP is behind it any more than it learns the other party's.
        return Ok(());
    }

    let is_lan = target_addr.iter().any(|p| match p {
        Protocol::Ip4(ip) => ip.is_private() || ip.is_loopback() || ip.is_link_local(),
        Protocol::Ip6(ip) => ip.is_loopback() || ip.is_unique_local() || ip.is_unicast_link_local(),
        _ => false,
    });
    if is_lan {
        return Ok(());
    }

    Err(TransportError::Setup(format!(
        "direct dial to '{target_addr}' blocked — only LAN or relayed connections are permitted, \
         to avoid exposing this device's real IP address to a contact"
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn addr(s: &str) -> Multiaddr {
        s.parse().unwrap()
    }

    #[test]
    fn test_lan_ipv4_direct_dial_is_permitted() {
        assert!(validate_outbound_dial(&addr("/ip4/192.168.1.42/udp/4001/quic-v1")).is_ok());
        assert!(validate_outbound_dial(&addr("/ip4/10.0.0.5/udp/4001/quic-v1")).is_ok());
        assert!(validate_outbound_dial(&addr("/ip4/127.0.0.1/udp/4001/quic-v1")).is_ok());
    }

    #[test]
    fn test_wan_ipv4_direct_dial_is_blocked() {
        assert!(validate_outbound_dial(&addr("/ip4/203.0.113.9/udp/4001/quic-v1")).is_err());
    }

    #[test]
    fn test_wan_ipv6_direct_dial_is_blocked() {
        assert!(validate_outbound_dial(&addr("/ip6/2001:db8::1/udp/4001/quic-v1")).is_err());
    }

    #[test]
    fn test_relay_circuit_address_is_always_permitted_even_for_a_wan_relay() {
        let relayed = addr("/ip4/203.0.113.9/udp/4001/quic-v1/p2p/12D3KooWGjMGZjJZgQjJZgQjJZgQjJZgQjJZgQjJZgQjJZgQjJZg/p2p-circuit");
        assert!(validate_outbound_dial(&relayed).is_ok());
    }
}
