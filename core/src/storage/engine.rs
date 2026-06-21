use super::error::StorageError;
use super::master_key::MasterKey;
use crate::keystore::KeystoreOps;

/// Storage engine: manages SQLite database, master key lifecycle, and schema migrations.
pub struct Engine {
    conn: rusqlite::Connection,
    master_key: Option<MasterKey>,
}

impl Engine {
    /// Open or create the SQLite database at the given path.
    pub fn open(path: &str) -> Result<Self, StorageError> {
        let conn = rusqlite::Connection::open(path)?;

        let engine = Self {
            conn,
            master_key: None,
        };

        engine.enable_pragmas()?;
        engine.migrate()?;

        Ok(engine)
    }

    fn enable_pragmas(&self) -> Result<(), StorageError> {
        self.conn.execute_batch(
            "
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA foreign_keys = ON;
            PRAGMA trusted_schema = OFF;
            ",
        )?;
        Ok(())
    }

    // ── Master key lifecycle ─────────────────────────────────

    /// Check if the storage has been initialized (master key exists).
    pub fn is_initialized(&self) -> Result<bool, StorageError> {
        Ok(self.config_get("dek_wrapped")?.is_some())
    }

    /// Initialize the master key on first boot.
    pub fn init_master_key(
        &mut self,
        keystore: &dyn KeystoreOps,
        alias: &str,
    ) -> Result<(), StorageError> {
        if self.master_key.is_some() {
            return Err(StorageError::AlreadyInitialized);
        }

        let mk = MasterKey::generate();
        let wrapped = mk.wrap(keystore, alias).map_err(StorageError::InvalidData)?;
        self.config_set("dek_wrapped", &wrapped)?;

        // Keep the DEK in RAM
        self.master_key = Some(mk);
        Ok(())
    }

    /// Unlock the storage by unwrapping the stored master key.
    pub fn unlock(
        &mut self,
        keystore: &dyn KeystoreOps,
        alias: &str,
    ) -> Result<(), StorageError> {
        let wrapped = self
            .config_get("dek_wrapped")?
            .ok_or(StorageError::MasterKeyRequired)?;

        let mk =
            MasterKey::unwrap(keystore, alias, &wrapped).map_err(StorageError::InvalidData)?;
        self.master_key = Some(mk);
        Ok(())
    }

    /// Lock the storage by dropping the master key from RAM.
    pub fn lock(&mut self) {
        self.master_key = None;
    }

    /// Returns `true` if the master key is currently loaded.
    pub fn is_unlocked(&self) -> bool {
        self.master_key.is_some()
    }

    /// Access the master key for field encryption.
    pub fn master_key(&self) -> Option<&MasterKey> {
        self.master_key.as_ref()
    }

    // ── Schema migrations ────────────────────────────────────

    fn migrate(&self) -> Result<(), StorageError> {
        let version: i32 = self
            .config_get("schema_version")?
            .and_then(|v| {
                let s = std::str::from_utf8(&v).ok()?;
                s.parse().ok()
            })
            .unwrap_or(0);

        if version < 1 {
            self.migrate_v1()?;
            self.config_set("schema_version", b"1")?;
        }

        Ok(())
    }

    fn migrate_v1(&self) -> Result<(), StorageError> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS contacts (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                alias           TEXT NOT NULL,
                identity_pk     BLOB NOT NULL,
                identity_nonce  BLOB NOT NULL,
                static_pk       BLOB NOT NULL,
                static_nonce    BLOB NOT NULL,
                onion_address   TEXT NOT NULL UNIQUE,
                created_at      INTEGER NOT NULL,
                last_seen       INTEGER
            );

            CREATE TABLE IF NOT EXISTS messages (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                contact_id      INTEGER NOT NULL REFERENCES contacts(id) ON DELETE CASCADE,
                direction       INTEGER NOT NULL,
                wire_packet     BLOB NOT NULL,
                content_plain   BLOB,
                content_nonce   BLOB,
                created_at      INTEGER NOT NULL,
                delivered_at    INTEGER,
                seen_at         INTEGER
            );

            CREATE INDEX IF NOT EXISTS idx_messages_contact_time
                ON messages(contact_id, created_at DESC);

            CREATE INDEX IF NOT EXISTS idx_contacts_onion
                ON contacts(onion_address);
            ",
        )?;
        Ok(())
    }

    // ── Config key-value store ────────────────────────────────

    /// The `config` table is created lazily on first access.
    fn ensure_config_table(&self) -> Result<(), StorageError> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS config (key TEXT PRIMARY KEY, value BLOB NOT NULL);",
        )?;
        Ok(())
    }

    pub fn config_get(&self, key: &str) -> Result<Option<Vec<u8>>, StorageError> {
        self.ensure_config_table()?;
        let mut stmt = self
            .conn
            .prepare_cached("SELECT value FROM config WHERE key = ?1")?;
        let mut rows = stmt.query_map([key], |row| row.get::<_, Vec<u8>>(0))?;
        match rows.next() {
            Some(Ok(val)) => Ok(Some(val)),
            _ => Ok(None),
        }
    }

    pub fn config_set(&self, key: &str, value: &[u8]) -> Result<(), StorageError> {
        self.ensure_config_table()?;
        self.conn.execute(
            "INSERT OR REPLACE INTO config (key, value) VALUES (?1, ?2)",
            rusqlite::params![key, value],
        )?;
        Ok(())
    }

    // ── Low-level access ──────────────────────────────────────

    pub fn conn(&self) -> &rusqlite::Connection {
        &self.conn
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        self.lock();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keystore::mock::MockKeystore;
    use std::sync::Mutex;

    // Use a mutex to avoid concurrent SQLite access in tests
    static DB_LOCK: Mutex<()> = Mutex::new(());

    fn test_path() -> String {
        format!(":memory:")
    }

    fn setup() -> (MockKeystore, Engine) {
        let _lock = DB_LOCK.lock().unwrap();
        let ks = MockKeystore::new();
        let mut engine = Engine::open(&test_path()).unwrap();

        // Init master key
        engine.init_master_key(&ks, "test_mk").unwrap();
        (ks, engine)
    }

    #[test]
    fn test_open_and_init() {
        let _lock = DB_LOCK.lock().unwrap();
        let ks = MockKeystore::new();
        let mut engine = Engine::open(&test_path()).unwrap();

        assert!(!engine.is_unlocked());
        assert!(!engine.is_initialized().unwrap());

        engine.init_master_key(&ks, "test_mk").unwrap();
        assert!(engine.is_unlocked());
        assert!(engine.is_initialized().unwrap());
    }

    #[test]
    fn test_lock_unlock() {
        let _lock = DB_LOCK.lock().unwrap();
        let ks = MockKeystore::new();
        let mut engine = Engine::open(&test_path()).unwrap();

        engine.init_master_key(&ks, "test_mk").unwrap();
        assert!(engine.is_unlocked());

        engine.lock();
        assert!(!engine.is_unlocked());

        engine.unlock(&ks, "test_mk").unwrap();
        assert!(engine.is_unlocked());
    }

    #[test]
    fn test_init_twice_fails() {
        let _lock = DB_LOCK.lock().unwrap();
        let ks = MockKeystore::new();
        let mut engine = Engine::open(&test_path()).unwrap();

        engine.init_master_key(&ks, "test_mk").unwrap();
        let result = engine.init_master_key(&ks, "test_mk");
        assert!(matches!(result, Err(StorageError::AlreadyInitialized)));
    }

    #[test]
    fn test_unlock_without_init_fails() {
        let _lock = DB_LOCK.lock().unwrap();
        let ks = MockKeystore::new();
        let mut engine = Engine::open(&test_path()).unwrap();

        let result = engine.unlock(&ks, "test_mk");
        assert!(matches!(result, Err(StorageError::MasterKeyRequired)));
    }

    #[test]
    fn test_config_roundtrip() {
        let _lock = DB_LOCK.lock().unwrap();
        let ks = MockKeystore::new();
        let mut engine = Engine::open(&test_path()).unwrap();
        engine.init_master_key(&ks, "test_mk").unwrap();

        engine.config_set("foo", b"bar").unwrap();
        let val = engine.config_get("foo").unwrap().unwrap();
        assert_eq!(val, b"bar");
    }

    #[test]
    fn test_config_missing_key() {
        let _lock = DB_LOCK.lock().unwrap();
        let ks = MockKeystore::new();
        let mut engine = Engine::open(&test_path()).unwrap();
        engine.init_master_key(&ks, "test_mk").unwrap();

        let val = engine.config_get("nonexistent").unwrap();
        assert!(val.is_none());
    }

    #[test]
    fn test_engine_drop_locks() {
        let _lock = DB_LOCK.lock().unwrap();
        let ks = MockKeystore::new();
        let mut engine = Engine::open(&test_path()).unwrap();
        engine.init_master_key(&ks, "test_mk").unwrap();
        assert!(engine.is_unlocked());
        drop(engine);
        // Master key zeroized on drop
    }
}
