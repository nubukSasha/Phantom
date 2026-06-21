/// Maximum plaintext size per message (64 KiB).
pub const MAX_MESSAGE_SIZE: usize = 65536;

/// Maximum padding length in bytes (1 KiB).
pub const MAX_PADDING_SIZE: usize = 1024;

/// Wire protocol version identifier.
pub const WIRE_VERSION: u8 = 1;

/// Fixed header size for a Data packet:
/// version(1) + msg_type(1) + padding_len(2) + payload_len(2) = 6 bytes
pub const PACKET_HEADER_SIZE: usize = 6;

/// Maximum payload per packet (message + padding).
pub const MAX_PAYLOAD_SIZE: usize = MAX_MESSAGE_SIZE + MAX_PADDING_SIZE;

/// Maximum total packet size.
pub const MAX_PACKET_SIZE: usize = PACKET_HEADER_SIZE + MAX_PAYLOAD_SIZE;
