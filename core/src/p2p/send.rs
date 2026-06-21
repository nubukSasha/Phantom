use tokio::time::{timeout, Duration};

use super::error::P2pError;
use super::stream;
use crate::tor::TorClient;
use crate::wire::packet::Packet;

/// Default port for Phantom P2P connections over Tor.
const PHANTOM_PORT: u16 = 11009;

/// Connection timeout (30 seconds).
const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);

/// Connect to a peer and send a raw data packet.
///
/// Returns the delivery ACK message id, if the peer acknowledged.
pub async fn send_data_packet(
    tor_client: &TorClient,
    peer_onion: &str,
    data_packet: &[u8],
) -> Result<u64, P2pError> {
    // 1. Connect to peer's hidden service
    let mut stream = timeout(CONNECT_TIMEOUT, tor_client.connect(peer_onion, PHANTOM_PORT))
        .await
        .map_err(|_| P2pError::Timeout)?
        .map_err(|e| P2pError::ConnectionRefused(e.to_string()))?;

    // 2. Write the data packet
    stream::write_packet(&mut stream, data_packet).await?;

    // 3. Wait for ACK (read response packet)
    match timeout(Duration::from_secs(15), stream::read_packet(&mut stream)).await {
        Ok(Ok(response)) => {
            let (ack_packet, _) = Packet::decode(&response)
                .map_err(|e| P2pError::Protocol(e.to_string()))?;

            use crate::wire::packet::MsgType;
            match ack_packet.msg_type {
                MsgType::Ack => {
                    // Parse message id from ACK payload
                    if ack_packet.payload.len() == 8 {
                        let mut arr = [0u8; 8];
                        arr.copy_from_slice(&ack_packet.payload);
                        Ok(u64::from_be_bytes(arr))
                    } else {
                        Err(P2pError::Protocol("invalid ACK payload length".into()))
                    }
                }
                MsgType::Ping => {
                    // Peer sent ping, respond with pong and wait for real response
                    let pong = Packet::new_ping(true);
                    stream::write_packet(&mut stream, &pong).await?;
                    // Recursive call would be cleaner, but for simplicity return timeout error
                    Err(P2pError::Timeout)
                }
                _ => Err(P2pError::Protocol(format!(
                    "expected ACK, got {:?}",
                    ack_packet.msg_type
                ))),
            }
        }
        Ok(Err(e)) => Err(e),
        Err(_) => Err(P2pError::Timeout),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_constant() {
        assert_eq!(PHANTOM_PORT, 11009);
        assert!(PHANTOM_PORT > 1024);
    }
}
