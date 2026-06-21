use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Contact {
    pub id: i64,
    pub alias: String,
    pub identity_pk: [u8; 32],
    pub static_pk: [u8; 32],
    pub onion_address: String,
    pub created_at: i64,
    pub last_seen: Option<i64>,
}

#[derive(Clone, Debug)]
pub struct ContactSummary {
    pub id: i64,
    pub alias: String,
    pub onion_address: String,
    pub last_seen: Option<i64>,
    pub last_message_preview: Option<String>,
    pub last_message_at: Option<i64>,
    pub unread_count: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub id: i64,
    pub contact_id: i64,
    pub direction: Direction,
    pub wire_packet: Vec<u8>,
    pub plaintext: Option<String>,
    pub created_at: i64,
    pub delivered_at: Option<i64>,
    pub seen_at: Option<i64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum Direction {
    Sent = 0,
    Received = 1,
}

impl Direction {
    pub fn from_i32(v: i32) -> Option<Self> {
        match v {
            0 => Some(Direction::Sent),
            1 => Some(Direction::Received),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ConversationSummary {
    pub contact_id: i64,
    pub alias: String,
    pub onion_address: String,
    pub last_message_preview: Option<String>,
    pub last_message_at: i64,
    pub last_direction: Direction,
    pub unread_count: i32,
}
