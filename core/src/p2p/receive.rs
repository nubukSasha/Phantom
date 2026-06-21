use tokio::sync::mpsc;
use tracing::{info, warn};

use super::error::P2pError;
use super::stream;
use crate::tor::hidden_service::HiddenService;
use crate::wire::packet::{MsgType, Packet};

/// An event received from a peer.
#[derive(Clone, Debug)]
pub enum PeerEvent {
    /// Received a data message (raw EncryptedMessage bytes).
    Data(Vec<u8>),
    /// Received a Ping — respond with Pong at the application layer.
    Ping,
    /// Peer sent an unexpected packet type.
    Unknown,
}

/// Start listening for incoming peer connections on the hidden service.
///
/// Each incoming connection is handled as a short-lived session:
///   1. Read one data packet
///   2. Send ACK
///   3. Close the connection
///
/// Events are sent to the returned receiver.
pub async fn start_listener(
    service: &HiddenService,
) -> mpsc::Receiver<PeerEvent> {
    let (tx, rx) = mpsc::channel::<PeerEvent>(64);

    // In a real implementation, the RunningOnionService would provide
    // a stream of incoming connections. For now, we simulate the event loop.
    //
    // The actual integration with tor-hsservice requires:
    //   let mut incoming = service.running().incoming();
    //   while let Some(stream) = incoming.next().await { ... }

    let tx_clone = tx.clone();
    tokio::spawn(async move {
        // This is a placeholder for the actual listener loop.
        // The real implementation will use tor-hsservice stream API.
        let _ = tx_clone;
        info!("phantom p2p listener started");
        // For now, just keep the task alive
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
        }
    });

    rx
}

/// Handle a single incoming connection.
///
/// Reads one packet, sends an ACK, and returns the parsed event.
pub async fn handle_connection(
    mut stream: tokio::net::TcpStream,
) -> Result<PeerEvent, P2pError> {
    let raw = stream::read_packet(&mut stream).await?;
    let (packet, _) = Packet::decode(&raw)
        .map_err(|e| P2pError::Protocol(e.to_string()))?;

    match packet.msg_type {
        MsgType::Data => {
            // Send ACK
            let ack = Packet::new_ack(0);
            stream::write_packet(&mut stream, &ack).await?;

            Ok(PeerEvent::Data(packet.payload))
        }
        MsgType::Ping => {
            let pong = Packet::new_ping(true);
            stream::write_packet(&mut stream, &pong).await?;

            Ok(PeerEvent::Ping)
        }
        MsgType::Handshake => {
            // Handshake is handled at a higher layer; treat as unknown here
            Err(P2pError::Protocol("unexpected handshake packet".into()))
        }
        _ => Ok(PeerEvent::Unknown),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wire::packet::Packet;

    #[test]
    fn test_peer_event_debug() {
        let event = PeerEvent::Data(vec![1, 2, 3]);
        assert!(format!("{event:?}").contains("Data"));
    }
}
