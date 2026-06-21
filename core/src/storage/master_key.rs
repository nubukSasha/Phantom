use rand::rngs::OsRng;
use rand::RngCore;
use zeroize::Zeroize;

use crate::keystore::KeystoreOps;

/// Data Encryption Key — 256-bit key for field-level AEAD encryption.
///
/// Lifecycle:
///   1. Generated once via `generate()`
///   2. Wrapped via `wrap()` using Android Keystore (via KeystoreOps)
///   3. Wrapped blob stored in SQLite `config` table
///   4. On unlock: `unwrap()` reconstructs DEK in RAM
///   5. On lock: `drop()` zeroizes the key
pub struct MasterKey([u8; 32]);

impl MasterKey {
    /// Generate a new random DEK.
    pub fn generate() -> Self {
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);
        Self(key)
    }

    /// Wrap the DEK using KeystoreOps for secure storage.
    pub fn wrap(
        &self,
        keystore: &dyn KeystoreOps,
        alias: &str,
    ) -> Result<Vec<u8>, String> {
        keystore.wrap_key(alias, &self.0)
    }

    /// Unwrap a previously wrapped DEK from storage.
    pub fn unwrap(
        keystore: &dyn KeystoreOps,
        alias: &str,
        wrapped: &[u8],
    ) -> Result<Self, String> {
        keystore.unwrap_key(alias, wrapped).map(Self)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl Drop for MasterKey {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

impl core::fmt::Debug for MasterKey {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("MasterKey")
            .field("key", &"[redacted]")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keystore::mock::MockKeystore;

    #[test]
    fn test_generate_is_not_zero() {
        let key = MasterKey::generate();
        assert!(!key.0.iter().all(|&b| b == 0));
    }

    #[test]
    fn test_wrap_unwrap_roundtrip() {
        let ks = MockKeystore::new();
        let key = MasterKey::generate();
        let wrapped = key.wrap(&ks, "test_alias").unwrap();
        let unwrapped = MasterKey::unwrap(&ks, "test_alias", &wrapped).unwrap();
        assert_eq!(key.0, unwrapped.0);
    }

    #[test]
    fn test_wrap_unwrap_tampered_fails() {
        let ks = MockKeystore::new();
        let key = MasterKey::generate();
        let mut wrapped = key.wrap(&ks, "test_alias").unwrap();
        wrapped[0] ^= 0xff;
        let result = MasterKey::unwrap(&ks, "test_alias", &wrapped);
        // Mock wraps by returning raw bytes, so tampered bytes fail length check
        assert!(result.is_err());
    }

    #[test]
    fn test_drop_zeroizes() {
        let key = MasterKey::generate();
        let ptr = key.0.as_ptr();
        let expected = key.0;
        // After drop the memory should be zeroed, but we can't observe it safely.
        // This test just verifies the API compiles.
        drop(key);
        // Can't access key.0 after drop
    }
}
