use thiserror::Error;

#[derive(Error, Debug)]
pub enum P2pError {
    #[error("tor error: {0}")]
    Tor(String),

    #[error("connection refused: {0}")]
    ConnectionRefused(String),

    #[error("timeout waiting for peer")]
    Timeout,

    #[error("invalid handshake: {0}")]
    InvalidHandshake(String),

    #[error("protocol error: {0}")]
    Protocol(String),

    #[error("crypto error: {0}")]
    Crypto(String),

    #[error("storage error: {0}")]
    Storage(String),

    #[error("peer disconnected")]
    Disconnected,

    #[error("{0}")]
    Other(String),
}
