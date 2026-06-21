use super::engine::Engine;
use super::error::StorageError;
use super::field;
use super::models::{Contact, ContactSummary};
use crate::crypto::types::{IdentityPublicKey, StaticPublicKey};

pub fn add_contact(
    engine: &Engine,
    alias: &str,
    identity_pk: &IdentityPublicKey,
    static_pk: &StaticPublicKey,
    onion_address: &str,
) -> Result<i64, StorageError> {
    let mk = engine.master_key().ok_or(StorageError::MasterKeyRequired)?;

    let (id_nonce, id_ct) = field::encrypt_field(mk, &identity_pk.0)?;
    let (st_nonce, st_ct) = field::encrypt_field(mk, &static_pk.0)?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;

    engine.conn().execute(
        "INSERT INTO contacts (alias, identity_pk, identity_nonce, static_pk, static_nonce, onion_address, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            alias,
            id_ct,
            id_nonce,
            st_ct,
            st_nonce,
            onion_address,
            now,
        ],
    )?;

    Ok(engine.conn().last_insert_rowid())
}

pub fn get_contact(engine: &Engine, id: i64) -> Result<Contact, StorageError> {
    let mk = engine.master_key().ok_or(StorageError::MasterKeyRequired)?;

    let mut stmt = engine.conn().prepare_cached(
        "SELECT id, alias, identity_pk, identity_nonce, static_pk, static_nonce, onion_address, created_at, last_seen
         FROM contacts WHERE id = ?1",
    )?;

    let mut rows = stmt.query_map([id], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, Vec<u8>>(2)?,
            row.get::<_, Vec<u8>>(3)?,
            row.get::<_, Vec<u8>>(4)?,
            row.get::<_, Vec<u8>>(5)?,
            row.get::<_, String>(6)?,
            row.get::<_, i64>(7)?,
            row.get::<_, Option<i64>>(8)?,
        ))
    })?;

    match rows.next() {
        Some(Ok((id, alias, id_ct, id_nonce, st_ct, st_nonce, onion, created_at, last_seen))) => {
            let identity_pk_bytes = field::decrypt_field(mk, &id_nonce, &id_ct)?;
            let static_pk_bytes = field::decrypt_field(mk, &st_nonce, &st_ct)?;

            let mut identity_arr = [0u8; 32];
            let mut static_arr = [0u8; 32];
            identity_arr.copy_from_slice(&identity_pk_bytes);
            static_arr.copy_from_slice(&static_pk_bytes);

            Ok(Contact {
                id,
                alias,
                identity_pk: identity_arr,
                static_pk: static_arr,
                onion_address: onion,
                created_at,
                last_seen,
            })
        }
        Some(Err(e)) => Err(StorageError::Database(e.to_string())),
        None => Err(StorageError::NotFound),
    }
}

pub fn get_contact_by_onion(engine: &Engine, onion: &str) -> Result<Contact, StorageError> {
    // Implementation delegates to get_contact after finding id
    let mut stmt = engine
        .conn()
        .prepare_cached("SELECT id FROM contacts WHERE onion_address = ?1")?;
    let id: i64 = stmt
        .query_row([onion], |row| row.get(0))
        .map_err(|_| StorageError::NotFound)?;
    get_contact(engine, id)
}

pub fn list_contacts(engine: &Engine) -> Result<Vec<ContactSummary>, StorageError> {
    let mk = engine.master_key().ok_or(StorageError::MasterKeyRequired)?;

    let mut stmt = engine.conn().prepare_cached(
        "SELECT c.id, c.alias, c.onion_address, c.last_seen,
                m.content_plain, m.content_nonce, m.created_at, m.direction,
                (SELECT COUNT(*) FROM messages WHERE contact_id = c.id AND seen_at IS NULL) as unread
         FROM contacts c
         LEFT JOIN messages m ON m.id = (
             SELECT id FROM messages WHERE contact_id = c.id ORDER BY created_at DESC LIMIT 1
         )
         ORDER BY COALESCE(m.created_at, 0) DESC",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(ContactSummary {
            id: row.get(0)?,
            alias: row.get(1)?,
            onion_address: row.get(2)?,
            last_seen: row.get(3)?,
            last_message_preview: None, // filled below
            last_message_at: row.get(6)?,
            unread_count: row.get(8)?,
        })
    })?;

    let mut summaries = Vec::new();
    for row in rows {
        let mut summary = row.map_err(|e| StorageError::Database(e.to_string()))?;
        // Decrypt last message preview
        let mut stmt2 = engine.conn().prepare_cached(
            "SELECT content_plain, content_nonce FROM messages
             WHERE contact_id = ?1 ORDER BY created_at DESC LIMIT 1",
        )?;
        if let Some(Ok((nonce, ct))) = stmt2
            .query_map([summary.id], |row| {
                Ok((row.get::<_, Vec<u8>>(0)?, row.get::<_, Vec<u8>>(1)?))
            })?
            .next()
        {
            if let Ok(plaintext) = field::decrypt_field(mk, &nonce, &ct) {
                if let Ok(text) = String::from_utf8(plaintext) {
                    summary.last_message_preview = Some(text);
                }
            }
        }
        summaries.push(summary);
    }

    Ok(summaries)
}

pub fn delete_contact(engine: &Engine, id: i64) -> Result<(), StorageError> {
    let affected = engine
        .conn()
        .execute("DELETE FROM contacts WHERE id = ?1", [id])?;
    if affected == 0 {
        return Err(StorageError::NotFound);
    }
    Ok(())
}

pub fn update_last_seen(engine: &Engine, id: i64) -> Result<(), StorageError> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;
    engine
        .conn()
        .execute("UPDATE contacts SET last_seen = ?1 WHERE id = ?2", rusqlite::params![now, id])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keystore::mock::MockKeystore;
    use crate::storage::models::Direction;
    use std::sync::Mutex;

    static DB_LOCK: Mutex<()> = Mutex::new(());

    fn setup() -> (MockKeystore, Engine) {
        let _lock = DB_LOCK.lock().unwrap();
        let ks = MockKeystore::new();
        let mut engine = Engine::open(":memory:").unwrap();
        engine.init_master_key(&ks, "test_mk").unwrap();
        (ks, engine)
    }

    fn test_contact(engine: &Engine) -> i64 {
        let id_pk = IdentityPublicKey([1u8; 32]);
        let st_pk = StaticPublicKey([2u8; 32]);
        add_contact(engine, "alice", &id_pk, &st_pk, "alice.onion").unwrap()
    }

    #[test]
    fn test_add_and_get_contact() {
        let (_, engine) = setup();
        let id = test_contact(&engine);
        let contact = get_contact(&engine, id).unwrap();
        assert_eq!(contact.alias, "alice");
        assert_eq!(contact.onion_address, "alice.onion");
        assert_eq!(contact.identity_pk, [1u8; 32]);
        assert_eq!(contact.static_pk, [2u8; 32]);
    }

    #[test]
    fn test_get_contact_not_found() {
        let (_, engine) = setup();
        let result = get_contact(&engine, 999);
        assert!(matches!(result, Err(StorageError::NotFound)));
    }

    #[test]
    fn test_get_contact_by_onion() {
        let (_, engine) = setup();
        let id = test_contact(&engine);
        let contact = get_contact_by_onion(&engine, "alice.onion").unwrap();
        assert_eq!(contact.id, id);
    }

    #[test]
    fn test_delete_contact() {
        let (_, engine) = setup();
        let id = test_contact(&engine);
        delete_contact(&engine, id).unwrap();
        assert!(get_contact(&engine, id).is_err());
    }

    #[test]
    fn test_list_contacts_empty() {
        let (_, engine) = setup();
        let list = list_contacts(&engine).unwrap();
        assert!(list.is_empty());
    }

    #[test]
    fn test_list_contacts_with_data() {
        let (_, engine) = setup();
        test_contact(&engine);
        let list = list_contacts(&engine).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].alias, "alice");
    }
}
