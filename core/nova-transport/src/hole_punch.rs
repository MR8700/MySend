use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::UdpSocket;
use tracing::{debug, info};

pub struct HolePuncher;

impl HolePuncher {
    /// Attempts UDP Hole Punching by emitting coordinated datagrams toward the peer's public address.
    pub async fn attempt_punch(
        local_socket: &UdpSocket,
        target_addr: SocketAddr,
        timeout_duration: Duration,
    ) -> bool {
        info!("Starting UDP Hole Punching toward {}", target_addr);
        let ping_packet = b"NOVA_HOLE_PUNCH_SYN";
        let start = tokio::time::Instant::now();

        // Send a burst of 5 packets spaced by 30ms
        for _ in 0..5 {
            if let Err(e) = local_socket.send_to(ping_packet, target_addr).await {
                debug!("Failed to send hole punch packet: {}", e);
            }
            tokio::time::sleep(Duration::from_millis(30)).await;
        }

        let mut buf = [0u8; 64];
        while start.elapsed() < timeout_duration {
            tokio::select! {
                res = local_socket.recv_from(&mut buf) => {
                    if let Ok((len, src)) = res {
                        if src == target_addr && &buf[..len] == b"NOVA_HOLE_PUNCH_SYN" {
                            info!("Hole punching successful from {}", src);
                            // Acknowledge
                            let _ = local_socket.send_to(b"NOVA_HOLE_PUNCH_ACK", target_addr).await;
                            return true;
                        }
                    }
                }
                _ = tokio::time::sleep(Duration::from_millis(50)) => {}
            }
        }

        debug!("Hole punching timed out against {}", target_addr);
        false
    }
}
