use thiserror::Error;

#[derive(Error, Debug)]
pub enum TorError {
    #[error("bootstrap failed: {0}")]
    Bootstrap(String),

    #[error("connection failed: {0}")]
    Connection(String),

    #[error("hidden service error: {0}")]
    HiddenService(String),

    #[error("timeout")]
    Timeout,

    #[error("stream closed")]
    StreamClosed,

    #[error("{0}")]
    Other(String),
}
