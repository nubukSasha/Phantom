uniffi::setup_scaffolding!();
use std::sync::Mutex;

use phantom_core::crypto::identity;
use phantom_core::crypto::message;
use phantom_core::crypto::static_key;
use phantom_core::crypto::types::{IdentityPublicKey, StaticPublicKey};
use phantom_core::keystore::KeystoreOps;
use phantom_core::storage::engine::Engine;
use phantom_core::storage::error::StorageError;
use phantom_core::storage::models::Direction;
use sha3::{Digest, Sha3_256};

// ────────────────────────────────────────────────────────────
// Keystore callback (Kotlin → Rust)
// ────────────────────────────────────────────────────────────

#[derive(Debug, uniffi::Error)]
pub enum KeystoreError {
    Error { msg: String },
}

impl KeystoreError {
    fn msg(self) -> String {
        let KeystoreError::Error { msg } = self;
        msg
    }
}

#[uniffi::export(callback_interface)]
pub trait FfiKeystoreOps: Send + Sync + std::fmt::Debug {
    fn ed25519_generate(&self, alias: String) -> Result<Vec<u8>, KeystoreError>;
    fn ed25519_public(&self, alias: String) -> Result<Vec<u8>, KeystoreError>;
    fn ed25519_sign(&self, alias: String, msg: Vec<u8>) -> Result<Vec<u8>, KeystoreError>;
    fn ed25519_delete(&self, alias: String) -> Result<(), KeystoreError>;
    fn x25519_generate(&self, alias: String) -> Result<Vec<u8>, KeystoreError>;
    fn x25519_public(&self, alias: String) -> Result<Vec<u8>, KeystoreError>;
    fn x25519_agree(&self, alias: String, peer_public: Vec<u8>) -> Result<Vec<u8>, KeystoreError>;
    fn x25519_delete(&self, alias: String) -> Result<(), KeystoreError>;
    fn wrap_key(&self, alias: String, key: Vec<u8>) -> Result<Vec<u8>, KeystoreError>;
    fn unwrap_key(&self, alias: String, wrapped: Vec<u8>) -> Result<Vec<u8>, KeystoreError>;
}

// Temporary adapter: borrows a &dyn FfiKeystoreOps to implement phantom_core::KeystoreOps

struct FfiKeystoreAdapter<'a> {
    inner: &'a dyn FfiKeystoreOps,
}

impl std::fmt::Debug for FfiKeystoreAdapter<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FfiKeystoreAdapter").finish()
    }
}

impl KeystoreOps for FfiKeystoreAdapter<'_> {
    fn ed25519_generate(&self, alias: &str) -> Result<[u8; 32], String> {
        self.inner.ed25519_generate(alias.to_string())
            .map_err(|e| e.msg())
            .and_then(|v| <[u8; 32]>::try_from(v.as_slice()).map_err(|_| "ed25519 pk len".into()))
    }
    fn ed25519_public(&self, alias: &str) -> Result<[u8; 32], String> {
        self.inner.ed25519_public(alias.to_string())
            .map_err(|e| e.msg())
            .and_then(|v| <[u8; 32]>::try_from(v.as_slice()).map_err(|_| "ed25519 pk len".into()))
    }
    fn ed25519_sign(&self, alias: &str, msg: &[u8]) -> Result<[u8; 64], String> {
        self.inner.ed25519_sign(alias.to_string(), msg.to_vec())
            .map_err(|e| e.msg())
            .and_then(|v| <[u8; 64]>::try_from(v.as_slice()).map_err(|_| "ed25519 sig len".into()))
    }
    fn ed25519_delete(&self, alias: &str) -> Result<(), String> {
        self.inner.ed25519_delete(alias.to_string()).map_err(|e| e.msg())
    }
    fn x25519_generate(&self, alias: &str) -> Result<[u8; 32], String> {
        self.inner.x25519_generate(alias.to_string())
            .map_err(|e| e.msg())
            .and_then(|v| <[u8; 32]>::try_from(v.as_slice()).map_err(|_| "x25519 pk len".into()))
    }
    fn x25519_public(&self, alias: &str) -> Result<[u8; 32], String> {
        self.inner.x25519_public(alias.to_string())
            .map_err(|e| e.msg())
            .and_then(|v| <[u8; 32]>::try_from(v.as_slice()).map_err(|_| "x25519 pk len".into()))
    }
    fn x25519_agree(&self, alias: &str, peer_public: &[u8; 32]) -> Result<[u8; 32], String> {
        self.inner.x25519_agree(alias.to_string(), peer_public.to_vec())
            .map_err(|e| e.msg())
            .and_then(|v| <[u8; 32]>::try_from(v.as_slice()).map_err(|_| "x25519 shared len".into()))
    }
    fn x25519_delete(&self, alias: &str) -> Result<(), String> {
        self.inner.x25519_delete(alias.to_string()).map_err(|e| e.msg())
    }
    fn wrap_key(&self, alias: &str, key: &[u8; 32]) -> Result<Vec<u8>, String> {
        self.inner.wrap_key(alias.to_string(), key.to_vec()).map_err(|e| e.msg())
    }
    fn unwrap_key(&self, alias: &str, wrapped: &[u8]) -> Result<[u8; 32], String> {
        self.inner.unwrap_key(alias.to_string(), wrapped.to_vec())
            .map_err(|e| e.msg())
            .and_then(|v| <[u8; 32]>::try_from(v.as_slice()).map_err(|_| "unwrap key len".into()))
    }
}

// ────────────────────────────────────────────────────────────
// Message callback (Rust → Kotlin)
// ────────────────────────────────────────────────────────────

#[uniffi::export(callback_interface)]
pub trait MessageCallback: Send + Sync {
    fn on_message_received(&self, contact_id: i64, text: String, message_id: i64);
    fn on_contact_online(&self, contact_id: i64);
    fn on_contact_offline(&self, contact_id: i64);
}

// ────────────────────────────────────────────────────────────
// Error
// ────────────────────────────────────────────────────────────

#[derive(Debug, uniffi::Error)]
pub enum PhantomError {
    Crypto { msg: String },
    Keystore { msg: String },
    Storage { msg: String },
    Network { msg: String },
    InvalidInput { msg: String },
    NotInitialized,
    AlreadyInitialized,
}

impl std::fmt::Display for PhantomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PhantomError::Crypto { msg } => write!(f, "Crypto error: {msg}"),
            PhantomError::Keystore { msg } => write!(f, "Keystore error: {msg}"),
            PhantomError::Storage { msg } => write!(f, "Storage error: {msg}"),
            PhantomError::Network { msg } => write!(f, "Network error: {msg}"),
            PhantomError::InvalidInput { msg } => write!(f, "Invalid input: {msg}"),
            PhantomError::NotInitialized => write!(f, "Not initialized"),
            PhantomError::AlreadyInitialized => write!(f, "Already initialized"),
        }
    }
}

impl From<StorageError> for PhantomError {
    fn from(e: StorageError) -> Self {
        PhantomError::Storage { msg: e.to_string() }
    }
}

// ────────────────────────────────────────────────────────────
// FFI records
// ────────────────────────────────────────────────────────────

#[derive(Debug, uniffi::Record)]
pub struct FfiContact {
    pub id: i64,
    pub alias: String,
    pub identity_pk: Vec<u8>,
    pub static_pk: Vec<u8>,
    pub onion_address: String,
    pub created_at: i64,
    pub last_seen: Option<i64>,
}

#[derive(Debug, uniffi::Record)]
pub struct FfiContactSummary {
    pub id: i64,
    pub alias: String,
    pub onion_address: String,
    pub last_seen: Option<i64>,
    pub last_message_preview: Option<String>,
    pub last_message_at: Option<i64>,
    pub unread_count: i32,
}

#[derive(Debug, uniffi::Record)]
pub struct FfiMessage {
    pub id: i64,
    pub contact_id: i64,
    pub direction: FfiDirection,
    pub text: Option<String>,
    pub created_at: i64,
    pub delivered_at: Option<i64>,
    pub seen_at: Option<i64>,
}

#[derive(Debug, uniffi::Record)]
pub struct FfiConversationSummary {
    pub contact_id: i64,
    pub alias: String,
    pub onion_address: String,
    pub last_message_preview: Option<String>,
    pub last_message_at: i64,
    pub last_direction: FfiDirection,
    pub unread_count: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, uniffi::Enum)]
pub enum FfiDirection {
    Sent,
    Received,
}

impl From<Direction> for FfiDirection {
    fn from(d: Direction) -> Self {
        match d {
            Direction::Sent => FfiDirection::Sent,
            Direction::Received => FfiDirection::Received,
        }
    }
}

// ────────────────────────────────────────────────────────────
// Core — main object
// ────────────────────────────────────────────────────────────

const IDENTITY_ALIAS: &str = "phantom_identity";
const STATIC_ALIAS: &str = "phantom_static";
const MASTER_KEY_ALIAS: &str = "phantom_master";

#[derive(uniffi::Object)]
pub struct PhantomCore {
    keystore: Box<dyn FfiKeystoreOps>,
    callback: Box<dyn MessageCallback>,
    engine: Mutex<Engine>,
    identity_pk: [u8; 32],
    static_pk: [u8; 32],
    onion_address: String,
}

#[uniffi::export(async_runtime = "tokio")]
impl PhantomCore {
    #[uniffi::constructor]
    pub async fn init(
        storage_path: String,
        keystore: Box<dyn FfiKeystoreOps>,
        callback: Box<dyn MessageCallback>,
    ) -> Result<std::sync::Arc<Self>, PhantomError> {
        let adapter = FfiKeystoreAdapter { inner: &*keystore };

        let mut engine = Engine::open(&storage_path)?;

        let (identity_pk, static_pk) = if engine.is_initialized()? {
            engine.unlock(&adapter, MASTER_KEY_ALIAS)?;
            let id_pk = identity::load_public(&adapter, IDENTITY_ALIAS)
                .map_err(|e| PhantomError::Keystore { msg: e })?;
            let st_pk = static_key::load_public(&adapter, STATIC_ALIAS)
                .map_err(|e| PhantomError::Keystore { msg: e })?;
            (id_pk.0, st_pk.0)
        } else {
            let id_pk = identity::generate(&adapter, IDENTITY_ALIAS)
                .map_err(|e| PhantomError::Keystore { msg: e })?;
            let st_pk = static_key::generate(&adapter, STATIC_ALIAS)
                .map_err(|e| PhantomError::Keystore { msg: e })?;
            engine.init_master_key(&adapter, MASTER_KEY_ALIAS)?;
            engine.config_set("identity_pk", &id_pk.0)?;
            engine.config_set("static_pk", &st_pk.0)?;
            (id_pk.0, st_pk.0)
        };

        let onion_address = derive_onion_address(&identity_pk);

        Ok(std::sync::Arc::new(Self {
            keystore,
            callback,
            engine: Mutex::new(engine),
            identity_pk,
            static_pk,
            onion_address,
        }))
    }

    pub fn identity_public(&self) -> Vec<u8> {
        self.identity_pk.to_vec()
    }

    pub fn onion_address(&self) -> String {
        self.onion_address.clone()
    }

    // ── Contacts ────────────────────────────────────────────

    pub fn add_contact(
        &self,
        alias: String,
        identity_pk: Vec<u8>,
        static_pk: Vec<u8>,
        onion_address: String,
    ) -> Result<i64, PhantomError> {
        let id_arr = vec_to_32(&identity_pk)
            .ok_or_else(|| PhantomError::InvalidInput { msg: "identity_pk: 32 bytes".into() })?;
        let st_arr = vec_to_32(&static_pk)
            .ok_or_else(|| PhantomError::InvalidInput { msg: "static_pk: 32 bytes".into() })?;

        let engine = self.engine.lock().unwrap();
        let id = phantom_core::storage::contact::add_contact(
            &engine, &alias,
            &IdentityPublicKey(id_arr),
            &StaticPublicKey(st_arr),
            &onion_address,
        )?;
        Ok(id)
    }

    pub fn get_contact(&self, id: i64) -> Result<FfiContact, PhantomError> {
        let engine = self.engine.lock().unwrap();
        let c = phantom_core::storage::contact::get_contact(&engine, id)?;
        Ok(FfiContact {
            id: c.id,
            alias: c.alias,
            identity_pk: c.identity_pk.to_vec(),
            static_pk: c.static_pk.to_vec(),
            onion_address: c.onion_address,
            created_at: c.created_at,
            last_seen: c.last_seen,
        })
    }

    pub fn list_contacts(&self) -> Vec<FfiContactSummary> {
        let engine = self.engine.lock().unwrap();
        phantom_core::storage::contact::list_contacts(&engine)
            .unwrap_or_default()
            .into_iter()
            .map(|c| FfiContactSummary {
                id: c.id,
                alias: c.alias,
                onion_address: c.onion_address,
                last_seen: c.last_seen,
                last_message_preview: c.last_message_preview,
                last_message_at: c.last_message_at,
                unread_count: c.unread_count,
            })
            .collect()
    }

    pub fn delete_contact(&self, id: i64) -> Result<(), PhantomError> {
        let engine = self.engine.lock().unwrap();
        phantom_core::storage::contact::delete_contact(&engine, id)?;
        Ok(())
    }

    // ── Messages ────────────────────────────────────────────

    pub async fn send_message(
        &self,
        contact_id: i64,
        text: String,
    ) -> Result<i64, PhantomError> {
        let adapter = FfiKeystoreAdapter { inner: &*self.keystore };
        let engine = self.engine.lock().unwrap();

        let contact = phantom_core::storage::contact::get_contact(&engine, contact_id)?;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;

        let encrypted = message::encrypt_message(
            &adapter,
            STATIC_ALIAS,
            &IdentityPublicKey(self.identity_pk),
            &StaticPublicKey(self.static_pk),
            &IdentityPublicKey(contact.identity_pk),
            &StaticPublicKey(contact.static_pk),
            text.as_bytes(),
        )
        .map_err(|e| PhantomError::Crypto { msg: e.to_string() })?;

        let wire_packet = encrypted
            .encode()
            .map_err(|e| PhantomError::Crypto { msg: e.to_string() })?;

        let message_id = phantom_core::storage::message::insert_message(
            &engine,
            contact_id,
            Direction::Sent,
            &wire_packet,
            Some(&text),
            now,
        )?;

        Ok(message_id)
    }

    pub fn get_messages(
        &self,
        contact_id: i64,
        limit: u32,
        offset: u32,
    ) -> Vec<FfiMessage> {
        let engine = self.engine.lock().unwrap();
        phantom_core::storage::message::get_messages(&engine, contact_id, limit, offset)
            .unwrap_or_default()
            .into_iter()
            .map(|m| FfiMessage {
                id: m.id,
                contact_id: m.contact_id,
                direction: m.direction.into(),
                text: m.plaintext,
                created_at: m.created_at,
                delivered_at: m.delivered_at,
                seen_at: m.seen_at,
            })
            .collect()
    }

    pub fn get_conversations(&self) -> Vec<FfiConversationSummary> {
        let engine = self.engine.lock().unwrap();
        phantom_core::storage::message::get_conversations(&engine)
            .unwrap_or_default()
            .into_iter()
            .map(|c| FfiConversationSummary {
                contact_id: c.contact_id,
                alias: c.alias,
                onion_address: c.onion_address,
                last_message_preview: c.last_message_preview,
                last_message_at: c.last_message_at,
                last_direction: c.last_direction.into(),
                unread_count: c.unread_count,
            })
            .collect()
    }

    pub fn mark_seen(&self, contact_id: i64) -> Result<(), PhantomError> {
        let engine = self.engine.lock().unwrap();
        phantom_core::storage::message::mark_seen(&engine, contact_id)?;
        Ok(())
    }

    // ── Lifecycle ───────────────────────────────────────────

    pub fn lock(&self) {
        self.engine.lock().unwrap().lock();
    }

    pub fn unlock(
        &self,
        keystore: Box<dyn FfiKeystoreOps>,
    ) -> Result<(), PhantomError> {
        let adapter = FfiKeystoreAdapter { inner: &*keystore };
        self.engine.lock().unwrap().unlock(&adapter, MASTER_KEY_ALIAS)?;
        Ok(())
    }

    pub fn destroy(&self) -> Result<(), PhantomError> {
        let engine = self.engine.lock().unwrap();
        let contacts = phantom_core::storage::contact::list_contacts(&engine).unwrap_or_default();
        for c in &contacts {
            let _ = phantom_core::storage::contact::delete_contact(&engine, c.id);
        }
        for c in &contacts {
            let _ = phantom_core::storage::message::delete_conversation(&engine, c.id);
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────
// Helpers
// ────────────────────────────────────────────────────────────

fn vec_to_32(v: &[u8]) -> Option<[u8; 32]> {
    if v.len() != 32 {
        return None;
    }
    let mut arr = [0u8; 32];
    arr.copy_from_slice(v);
    Some(arr)
}

fn derive_onion_address(public_key: &[u8; 32]) -> String {
    let hash = Sha3_256::digest(public_key);
    let truncated = &hash[..32];
    let encoded = data_encoding::BASE32_NOPAD.encode(truncated);
    format!("{}.onion", encoded.to_lowercase())
}
