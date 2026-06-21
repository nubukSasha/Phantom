use arti_client::{StreamPrefs, TorClient as ArtiClient, TorClientConfig};
use tor_rtcompat::tokio::TokioRuntimeHandle;

use super::error::TorError;

/// Wrapper around the Arti Tor client.
pub struct TorClient {
    inner: ArtiClient<TokioRuntimeHandle>,
}

impl TorClient {
    /// Bootstrap an Arti Tor client with default configuration.
    pub async fn bootstrap() -> Result<Self, TorError> {
        let config = TorClientConfig::default();
        Self::bootstrap_with_config(config).await
    }

    /// Bootstrap with a custom configuration.
    pub async fn bootstrap_with_config(
        config: TorClientConfig,
    ) -> Result<Self, TorError> {
        let client = ArtiClient::create_bootstrapped(config)
            .await
            .map_err(|e| TorError::Bootstrap(e.to_string()))?;
        Ok(Self { inner: client })
    }

    /// Connect to a hidden service at the given `.onion` address and port.
    ///
    /// Returns a `tokio::net::TcpStream` (or whatever the configured runtime provides).
    pub async fn connect(
        &self,
        onion: &str,
        port: u16,
    ) -> Result<tokio::net::TcpStream, TorError> {
        let prefs = StreamPrefs::new();
        let stream = self
            .inner
            .connect((onion, port), Some(prefs))
            .await
            .map_err(|e| TorError::Connection(e.to_string()))?;
        Ok(stream)
    }

    /// Return a reference to the underlying Arti client.
    pub fn inner(&self) -> &ArtiClient<TokioRuntimeHandle> {
        &self.inner
    }
}
