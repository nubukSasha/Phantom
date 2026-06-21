use std::sync::Arc;

use tokio::sync::mpsc;
use tracing::info;

use super::error::TorError;

/// Placeholder for a managed onion service.
///
/// In a full implementation, this would use `tor-hsservice::OnionService`
/// to create a real .onion service that listens for incoming connections.
///
/// The hidden service private key is the same as the Ed25519 identity key,
/// and the `.onion` address is derived from the corresponding public key.
pub struct HiddenService {
    onion_address: String,
}

impl HiddenService {
    /// Create a new hidden service placeholder.
    ///
    /// The caller provides the `.onion` address (derived from the
    /// Ed25519 identity key by the `crypto::identity` module).
    ///
    /// In a real implementation, this would launch the Arti
    /// `tor-hsservice::OnionService` with the appropriate key.
    pub async fn create(onion_address: String) -> Result<Self, TorError> {
        info!(onion = %onion_address, "starting hidden service");

        // TODO: integrate with tor-hsservice
        //   let config = OnionServiceConfig::builder()
        //       .nickname("phantom")
        //       .build()?;
        //   let (service, running) = OnionService::new(config, ...).await?;
        //   tokio::spawn(service.launch());

        Ok(Self { onion_address })
    }

    /// Return the `.onion` address of this service.
    pub fn onion_address(&self) -> &str {
        &self.onion_address
    }
}

impl std::fmt::Debug for HiddenService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HiddenService")
            .field("onion", &self.onion_address)
            .finish()
    }
}
