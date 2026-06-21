use std::collections::HashMap;
use std::sync::Mutex;

use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use x25519_dalek::{PublicKey as XPublicKey, StaticSecret};

use super::KeystoreOps;

#[derive(Debug)]
pub(crate) struct MockKeystore {
    ed_keys: Mutex<HashMap<String, SigningKey>>,
    x_keys: Mutex<HashMap<String, StaticSecret>>,
}

impl MockKeystore {
    pub(crate) fn new() -> Self {
        Self {
            ed_keys: Mutex::new(HashMap::new()),
            x_keys: Mutex::new(HashMap::new()),
        }
    }

    pub(crate) fn preload_ed25519(&self, alias: &str, key: SigningKey) {
        let mut keys = self.ed_keys.lock().unwrap();
        keys.insert(alias.to_string(), key);
    }

    pub(crate) fn preload_x25519(&self, alias: &str, key: StaticSecret) {
        let mut keys = self.x_keys.lock().unwrap();
        keys.insert(alias.to_string(), key);
    }
}

impl KeystoreOps for MockKeystore {
    fn ed25519_generate(&self, alias: &str) -> Result<[u8; 32], String> {
        let mut keys = self.ed_keys.lock().map_err(|e| e.to_string())?;
        let signing_key = SigningKey::generate(&mut OsRng);
        let pub_bytes = signing_key.verifying_key().to_bytes();
        keys.insert(alias.to_string(), signing_key);
        Ok(pub_bytes)
    }

    fn ed25519_public(&self, alias: &str) -> Result<[u8; 32], String> {
        let keys = self.ed_keys.lock().map_err(|e| e.to_string())?;
        keys.get(alias)
            .map(|k| k.verifying_key().to_bytes())
            .ok_or_else(|| format!("key not found: {alias}"))
    }

    fn ed25519_sign(&self, alias: &str, msg: &[u8]) -> Result<[u8; 64], String> {
        let keys = self.ed_keys.lock().map_err(|e| e.to_string())?;
        keys.get(alias)
            .map(|k| k.sign(msg).to_bytes())
            .ok_or_else(|| format!("key not found: {alias}"))
    }

    fn ed25519_delete(&self, alias: &str) -> Result<(), String> {
        let mut keys = self.ed_keys.lock().map_err(|e| e.to_string())?;
        keys.remove(alias).ok_or_else(|| format!("key not found: {alias}"))?;
        Ok(())
    }

    fn x25519_generate(&self, alias: &str) -> Result<[u8; 32], String> {
        let mut keys = self.x_keys.lock().map_err(|e| e.to_string())?;
        let secret = StaticSecret::random_from_rng(&mut OsRng);
        let public = XPublicKey::from(&secret);
        keys.insert(alias.to_string(), secret);
        Ok(public.to_bytes())
    }

    fn x25519_public(&self, alias: &str) -> Result<[u8; 32], String> {
        let keys = self.x_keys.lock().map_err(|e| e.to_string())?;
        keys.get(alias)
            .map(|k| XPublicKey::from(k).to_bytes())
            .ok_or_else(|| format!("key not found: {alias}"))
    }

    fn x25519_agree(
        &self,
        alias: &str,
        peer_public: &[u8; 32],
    ) -> Result<[u8; 32], String> {
        let keys = self.x_keys.lock().map_err(|e| e.to_string())?;
        let secret = keys.get(alias).ok_or_else(|| format!("key not found: {alias}"))?;
        let pub_key = XPublicKey::from(*peer_public);
        let shared = secret.diffie_hellman(&pub_key);
        Ok(shared.to_bytes())
    }

    fn x25519_delete(&self, alias: &str) -> Result<(), String> {
        let mut keys = self.x_keys.lock().map_err(|e| e.to_string())?;
        keys.remove(alias).ok_or_else(|| format!("key not found: {alias}"))?;
        Ok(())
    }

    fn wrap_key(&self, _alias: &str, key: &[u8; 32]) -> Result<Vec<u8>, String> {
        Ok(key.to_vec())
    }

    fn unwrap_key(&self, _alias: &str, wrapped: &[u8]) -> Result<[u8; 32], String> {
        let mut arr = [0u8; 32];
        if wrapped.len() != 32 {
            return Err("invalid wrapped key length".into());
        }
        arr.copy_from_slice(wrapped);
        Ok(arr)
    }
}
