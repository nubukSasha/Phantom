use tokio::io::{AsyncReadExt, AsyncWriteExt};

use super::error::P2pError;
use crate::wire::constants::MAX_PACKET_SIZE;

/// Read a complete packet from a Tor stream.
///
/// First reads a 2-byte length prefix (big-endian), then reads that many bytes.
pub async fn read_packet(
    stream: &mut tokio::net::TcpStream,
) -> Result<Vec<u8>, P2pError> {
    // Read 2-byte length prefix
    let mut len_buf = [0u8; 2];
    stream
        .read_exact(&mut len_buf)
        .await
        .map_err(|_| P2pError::Disconnected)?;

    let payload_len = u16::from_be_bytes(len_buf) as usize;

    if payload_len > MAX_PACKET_SIZE {
        return Err(P2pError::Protocol(format!(
            "packet too large: {payload_len} > {MAX_PACKET_SIZE}"
        )));
    }

    // Read payload
    let mut buf = vec![0u8; payload_len];
    stream
        .read_exact(&mut buf)
        .await
        .map_err(|_| P2pError::Disconnected)?;

    Ok(buf)
}

/// Write a complete packet to a Tor stream.
///
/// Prefixes the data with a 2-byte big-endian length.
pub async fn write_packet(
    stream: &mut tokio::net::TcpStream,
    data: &[u8],
) -> Result<(), P2pError> {
    if data.len() > MAX_PACKET_SIZE {
        return Err(P2pError::Protocol(format!(
            "packet too large: {} > {MAX_PACKET_SIZE}",
            data.len()
        )));
    }

    let len = u16::try_from(data.len())
        .map_err(|_| P2pError::Protocol("payload exceeds u16::MAX".into()))?;

    // Write length prefix
    stream
        .write_all(&len.to_be_bytes())
        .await
        .map_err(|_| P2pError::Disconnected)?;

    // Write payload
    stream
        .write_all(data)
        .await
        .map_err(|_| P2pError::Disconnected)?;

    stream
        .flush()
        .await
        .map_err(|_| P2pError::Disconnected)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Basic encoding test without a real socket.
    #[test]
    fn test_packet_size_limits() {
        let large = vec![0u8; MAX_PACKET_SIZE + 1];
        // write_packet would fail with Protocol error
        let result = write_packet_len_check(&large);
        assert!(result.is_err());
    }

    fn write_packet_len_check(data: &[u8]) -> Result<(), P2pError> {
        if data.len() > MAX_PACKET_SIZE {
            return Err(P2pError::Protocol(format!(
                "packet too large: {} > {MAX_PACKET_SIZE}",
                data.len()
            )));
        }
        let _len = u16::try_from(data.len())
            .map_err(|_| P2pError::Protocol("payload exceeds u16::MAX".into()))?;
        Ok(())
    }
}
