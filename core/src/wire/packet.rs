use serde::{Deserialize, Serialize};

use super::constants::{MAX_PAYLOAD_SIZE, PACKET_HEADER_SIZE, WIRE_VERSION};
use crate::crypto::types::EncryptedMessage;

/// Message type identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum MsgType {
    /// Encrypted data payload (an EncryptedMessage).
    Data = 0x01,
    /// Connection handshake.
    Handshake = 0x02,
    /// Delivery acknowledgement.
    Ack = 0x03,
    /// Keep-alive ping.
    Ping = 0x04,
    /// Keep-alive pong.
    Pong = 0x05,
}

impl TryFrom<u8> for MsgType {
    type Error = InvalidPacket;

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            0x01 => Ok(MsgType::Data),
            0x02 => Ok(MsgType::Handshake),
            0x03 => Ok(MsgType::Ack),
            0x04 => Ok(MsgType::Ping),
            0x05 => Ok(MsgType::Pong),
            _ => Err(InvalidPacket::UnknownMessageType(v)),
        }
    }
}

/// A parsed wire‑format packet.
#[derive(Clone, Debug)]
pub struct Packet {
    pub version: u8,
    pub msg_type: MsgType,
    pub payload: Vec<u8>,
}

/// Errors that can occur during packet parsing.
#[derive(Clone, Debug)]
pub enum InvalidPacket {
    BadVersion { expected: u8, got: u8 },
    UnknownMessageType(u8),
    PayloadTooLarge { size: usize, max: usize },
    Truncated,
}

impl Packet {
    /// Build a Data packet from an encrypted message.
    pub fn new_data(msg: &EncryptedMessage) -> Result<Vec<u8>, InvalidPacket> {
        let payload = msg.encode().map_err(|_| InvalidPacket::Truncated)?;
        if payload.len() > MAX_PAYLOAD_SIZE {
            return Err(InvalidPacket::PayloadTooLarge {
                size: payload.len(),
                max: MAX_PAYLOAD_SIZE,
            });
        }
        Self::encode(WIRE_VERSION, MsgType::Data, &payload)
    }

    /// Build a handshake packet containing the sender's public keys.
    pub fn new_handshake(sender_identity_pk: &[u8; 32], sender_static_pk: &[u8; 32]) -> Vec<u8> {
        let mut payload = Vec::with_capacity(64);
        payload.extend_from_slice(sender_identity_pk);
        payload.extend_from_slice(sender_static_pk);
        Self::encode(WIRE_VERSION, MsgType::Handshake, &payload)
            .expect("handshake payload is exactly 64 bytes")
    }

    /// Build an ACK packet for a given message id.
    pub fn new_ack(message_id: u64) -> Vec<u8> {
        let payload = message_id.to_be_bytes().to_vec();
        Self::encode(WIRE_VERSION, MsgType::Ack, &payload)
            .expect("ACK payload is exactly 8 bytes")
    }

    /// Build a Ping or Pong packet.
    pub fn new_ping(is_pong: bool) -> Vec<u8> {
        let msg_type = if is_pong { MsgType::Pong } else { MsgType::Ping };
        Self::encode(WIRE_VERSION, msg_type, &[])
            .expect("ping payload is empty")
    }

    // ── Wire encoding ───────────────────────────────────────

    fn encode(version: u8, msg_type: MsgType, payload: &[u8]) -> Result<Vec<u8>, InvalidPacket> {
        let payload_len =
            u16::try_from(payload.len()).map_err(|_| InvalidPacket::PayloadTooLarge {
                size: payload.len(),
                max: u16::MAX as usize,
            })?;

        let mut buf = Vec::with_capacity(PACKET_HEADER_SIZE + payload.len());
        buf.push(version);
        buf.push(msg_type as u8);
        buf.extend_from_slice(&0u16.to_be_bytes()); // padding_len = 0 for now
        buf.extend_from_slice(&payload_len.to_be_bytes());
        buf.extend_from_slice(payload);
        Ok(buf)
    }

    // ── Wire decoding ───────────────────────────────────────

    /// Parse a packet from raw bytes. Returns the packet and the number of bytes consumed.
    pub fn decode(bytes: &[u8]) -> Result<(Self, usize), InvalidPacket> {
        if bytes.len() < PACKET_HEADER_SIZE {
            return Err(InvalidPacket::Truncated);
        }

        let version = bytes[0];
        if version != WIRE_VERSION {
            return Err(InvalidPacket::BadVersion {
                expected: WIRE_VERSION,
                got: version,
            });
        }

        let msg_type = MsgType::try_from(bytes[1])?;

        let _padding_len = u16::from_be_bytes([bytes[2], bytes[3]]) as usize;
        let payload_len = u16::from_be_bytes([bytes[4], bytes[5]]) as usize;

        let total_len = PACKET_HEADER_SIZE + payload_len;
        if bytes.len() < total_len {
            return Err(InvalidPacket::Truncated);
        }

        let payload_start = PACKET_HEADER_SIZE;
        let payload_end = payload_start + payload_len;
        let payload = bytes[payload_start..payload_end].to_vec();

        Ok((Self { version, msg_type, payload }, total_len))
    }

    /// Extract the EncryptedMessage from a Data packet.
    pub fn into_encrypted_message(self) -> Result<EncryptedMessage, InvalidPacket> {
        if self.msg_type != MsgType::Data {
            // can't return InvalidPacket from here easily; use Truncated as stand-in
            return Err(InvalidPacket::Truncated);
        }
        EncryptedMessage::decode(&self.payload).map_err(|_| InvalidPacket::Truncated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::message;
    use crate::crypto::{identity, static_key};
    use crate::keystore::mock::MockKeystore;

    fn dummy_encrypted_msg() -> EncryptedMessage {
        let ks = MockKeystore::new();
        let alice_id = identity::generate(&ks, "alice_id").unwrap();
        let alice_st = static_key::generate(&ks, "alice_st").unwrap();
        let bob_id = identity::generate(&ks, "bob_id").unwrap();
        let bob_st = static_key::generate(&ks, "bob_st").unwrap();
        message::encrypt_message(
            &ks, "alice_st", &alice_id, &alice_st, &bob_id, &bob_st, b"hello",
        )
        .unwrap()
    }

    #[test]
    fn test_data_packet_roundtrip() {
        let msg = dummy_encrypted_msg();
        let encoded = Packet::new_data(&msg).unwrap();
        let (decoded, consumed) = Packet::decode(&encoded).unwrap();

        assert_eq!(consumed, encoded.len());
        assert_eq!(decoded.msg_type, MsgType::Data);

        let extracted = decoded.into_encrypted_message().unwrap();
        assert_eq!(extracted.ciphertext, msg.ciphertext);
    }

    #[test]
    fn test_handshake_packet() {
        let id_pk = [1u8; 32];
        let st_pk = [2u8; 32];
        let encoded = Packet::new_handshake(&id_pk, &st_pk);
        let (packet, _) = Packet::decode(&encoded).unwrap();

        assert_eq!(packet.msg_type, MsgType::Handshake);
        assert_eq!(&packet.payload[..32], &id_pk[..]);
        assert_eq!(&packet.payload[32..], &st_pk[..]);
    }

    #[test]
    fn test_ack_packet() {
        let encoded = Packet::new_ack(42);
        let (packet, _) = Packet::decode(&encoded).unwrap();

        assert_eq!(packet.msg_type, MsgType::Ack);
        assert_eq!(packet.payload, 42u64.to_be_bytes());
    }

    #[test]
    fn test_ping_pong() {
        let ping = Packet::new_ping(false);
        let (p, _) = Packet::decode(&ping).unwrap();
        assert_eq!(p.msg_type, MsgType::Ping);

        let pong = Packet::new_ping(true);
        let (p, _) = Packet::decode(&pong).unwrap();
        assert_eq!(p.msg_type, MsgType::Pong);
    }

    #[test]
    fn test_truncated_packet() {
        assert!(Packet::decode(&[0x01, 0x02]).is_err());
    }

    #[test]
    fn test_bad_version() {
        let mut buf = vec![0xFF, 0x01, 0x00, 0x00, 0x00, 0x00];
        let result = Packet::decode(&buf);
        assert!(matches!(result, Err(InvalidPacket::BadVersion { .. })));
    }

    #[test]
    fn test_unknown_msg_type() {
        let mut buf = vec![0x01, 0xFF, 0x00, 0x00, 0x00, 0x00];
        let result = Packet::decode(&buf);
        assert!(matches!(result, Err(InvalidPacket::UnknownMessageType(0xFF))));
    }
}
