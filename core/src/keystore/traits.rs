use std::fmt::Debug;

pub trait KeystoreOps: Send + Sync + Debug {
    fn ed25519_generate(&self, alias: &str) -> Result<[u8; 32], String>;
    fn ed25519_public(&self, alias: &str) -> Result<[u8; 32], String>;
    fn ed25519_sign(&self, alias: &str, msg: &[u8]) -> Result<[u8; 64], String>;
    fn ed25519_delete(&self, alias: &str) -> Result<(), String>;

    fn x25519_generate(&self, alias: &str) -> Result<[u8; 32], String>;
    fn x25519_public(&self, alias: &str) -> Result<[u8; 32], String>;
    fn x25519_agree(
        &self,
        alias: &str,
        peer_public: &[u8; 32],
    ) -> Result<[u8; 32], String>;
    fn x25519_delete(&self, alias: &str) -> Result<(), String>;

    fn wrap_key(&self, alias: &str, key: &[u8; 32]) -> Result<Vec<u8>, String>;
    fn unwrap_key(&self, alias: &str, wrapped: &[u8]) -> Result<[u8; 32], String>;
}
