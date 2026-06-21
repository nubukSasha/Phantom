use thiserror::Error;

use crate::storage::error::StorageError;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Crypto: {0}")]
    Crypto(#[from] CryptoError),

    #[error("Keystore: {0}")]
    Keystore(String),

    #[error("Storage: {0}")]
    Storage(String),

    #[error("Network: {0}")]
    Network(String),

    #[cfg(feature = "tor")]
    #[error("Tor: {0}")]
    Tor(#[from] crate::tor::TorError),

    #[cfg(feature = "tor")]
    #[error("P2P: {0}")]
    P2p(#[from] crate::p2p::P2pError),

    #[error("{0}")]
    Other(String),
}

impl From<StorageError> for Error {
    fn from(e: StorageError) -> Self {
        Error::Storage(e.to_string())
    }
}

impl From<String> for Error {
    fn from(e: String) -> Self {
        Error::Other(e)
    }
}

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("authentication failed (tag mismatch)")]
    AuthFailed,

    #[error("protocol version mismatch: expected {expected}, got {got}")]
    ProtocolVersionMismatch { expected: u8, got: u8 },

    #[error("invalid key length: expected {expected}, got {got}")]
    InvalidKeyLength { expected: usize, got: usize },

    #[error("invalid format: {0}")]
    InvalidFormat(String),

    #[error("key not found: {0}")]
    KeyNotFound(String),

    #[error("signature verification failed")]
    SignatureVerificationFailed,

    #[error("HKDF error")]
    HkdfError,
}
