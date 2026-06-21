use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

pub const PROTOCOL_VERSION: u8 = 1;
pub const EPHEMERAL_KEY_SIZE: usize = 32;
pub const STATIC_KEY_SIZE: usize = 32;
pub const IDENTITY_KEY_SIZE: usize = 32;
pub const NONCE_SIZE: usize = 12;
pub const TAG_SIZE: usize = 16;

// ── Public key types ───────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityPublicKey(pub [u8; IDENTITY_KEY_SIZE]);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StaticPublicKey(pub [u8; STATIC_KEY_SIZE]);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EphemeralPublicKey(pub [u8; EPHEMERAL_KEY_SIZE]);

// ── Secret types (RAM-only, zeroized on drop) ───────────────

pub struct SharedSecret([u8; 32]);

impl SharedSecret {
    pub(crate) fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub(crate) fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl Drop for SharedSecret {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

pub struct SessionKey([u8; 32]);

impl SessionKey {
    pub(crate) fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub(crate) fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl Drop for SessionKey {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

// ── AEAD wire types ────────────────────────────────────────

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Nonce(pub [u8; NONCE_SIZE]);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tag(pub [u8; TAG_SIZE]);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SealedBox {
    pub nonce: Nonce,
    pub ciphertext: Vec<u8>,
}

// ── Encrypted message (wire format) ────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EncryptedMessage {
    pub version: u8,
    pub sender_identity_pk: [u8; IDENTITY_KEY_SIZE],
    pub sender_static_pk: [u8; STATIC_KEY_SIZE],
    pub eph_pk: [u8; EPHEMERAL_KEY_SIZE],
    pub nonce: [u8; NONCE_SIZE],
    pub ciphertext: Vec<u8>,
    pub tag: [u8; TAG_SIZE],
}

impl EncryptedMessage {
    pub fn encode(&self) -> Result<Vec<u8>, postcard::Error> {
        postcard::to_allocvec(self)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, postcard::Error> {
        postcard::from_bytes(bytes)
    }
}

// ── Session context for HKDF ───────────────────────────────

pub struct SessionContext {
    pub sender_identity_pk: IdentityPublicKey,
    pub recipient_identity_pk: IdentityPublicKey,
    pub sender_static_pk: StaticPublicKey,
    pub recipient_static_pk: StaticPublicKey,
    pub eph_pk: EphemeralPublicKey,
}
