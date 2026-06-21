use super::engine::Engine;
use super::error::StorageError;
use super::field;
use super::models::{ConversationSummary, Direction, Message};

/// Insert a new message into storage.
///
/// If `plaintext` is provided, it will be AEAD-encrypted before storing.
pub fn insert_message(
    engine: &Engine,
    contact_id: i64,
    direction: Direction,
    wire_packet: &[u8],
    plaintext: Option<&str>,
    created_at: i64,
) -> Result<i64, StorageError> {
    let mk = engine.master_key().ok_or(StorageError::MasterKeyRequired)?;

    let (content_plain, content_nonce) = match plaintext {
        Some(text) => {
            let (nonce, ct) = field::encrypt_field(mk, text.as_bytes())?;
            (Some(ct), Some(nonce))
        }
        None => (None, None),
    };

    engine.conn().execute(
        "INSERT INTO messages (contact_id, direction, wire_packet, content_plain, content_nonce, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            contact_id,
            direction as i32,
            wire_packet,
            content_plain,
            content_nonce,
            created_at,
        ],
    )?;

    Ok(engine.conn().last_insert_rowid())
}

/// Retrieve messages for a contact, ordered by creation time descending.
pub fn get_messages(
    engine: &Engine,
    contact_id: i64,
    limit: u32,
    offset: u32,
) -> Result<Vec<Message>, StorageError> {
    let mk = engine.master_key().ok_or(StorageError::MasterKeyRequired)?;

    let mut stmt = engine.conn().prepare_cached(
        "SELECT id, contact_id, direction, wire_packet, content_plain, content_nonce,
                created_at, delivered_at, seen_at
         FROM messages
         WHERE contact_id = ?1
         ORDER BY created_at DESC
         LIMIT ?2 OFFSET ?3",
    )?;

    let rows = stmt.query_map(
        rusqlite::params![contact_id, limit, offset],
        |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i32>(2)?,
                row.get::<_, Vec<u8>>(3)?,
                row.get::<_, Option<Vec<u8>>>(4)?,
                row.get::<_, Option<Vec<u8>>>(5)?,
                row.get::<_, i64>(6)?,
                row.get::<_, Option<i64>>(7)?,
                row.get::<_, Option<i64>>(8)?,
            ))
        },
    )?;

    let mut messages = Vec::new();
    for row in rows {
        let (
            id,
            cid,
            dir,
            wp,
            content_ct_opt,
            content_nonce_opt,
            created_at,
            delivered_at,
            seen_at,
        ) = row.map_err(|e| StorageError::Database(e.to_string()))?;

        let direction = Direction::from_i32(dir).unwrap_or(Direction::Received);

        let plaintext = match (content_nonce_opt, content_ct_opt) {
            (Some(nonce), Some(ct)) => field::decrypt_field(mk, &nonce, &ct)
                .ok()
                .and_then(|b| String::from_utf8(b).ok()),
            _ => None,
        };

        messages.push(Message {
            id,
            contact_id: cid,
            direction,
            wire_packet: wp,
            plaintext,
            created_at,
            delivered_at,
            seen_at,
        });
    }

    Ok(messages)
}

/// Get conversation list for the main screen.
pub fn get_conversations(engine: &Engine) -> Result<Vec<ConversationSummary>, StorageError> {
    let mk = engine.master_key().ok_or(StorageError::MasterKeyRequired)?;

    let mut stmt = engine.conn().prepare_cached(
        "SELECT c.id, c.alias, c.onion_address, c.last_seen,
                m.content_plain, m.content_nonce, m.created_at, m.direction,
                (SELECT COUNT(*) FROM messages WHERE contact_id = c.id AND seen_at IS NULL) as unread
         FROM contacts c
         INNER JOIN messages m ON m.id = (
             SELECT id FROM messages WHERE contact_id = c.id ORDER BY created_at DESC LIMIT 1
         )
         ORDER BY m.created_at DESC",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, Option<i64>>(3)?,
            row.get::<_, Option<Vec<u8>>>(4)?,
            row.get::<_, Option<Vec<u8>>>(5)?,
            row.get::<_, i64>(6)?,
            row.get::<_, i32>(7)?,
            row.get::<_, i32>(8)?,
        ))
    })?;

    let mut conversations = Vec::new();
    for row in rows {
        let (
            contact_id,
            alias,
            onion_address,
            _last_seen,
            ct_opt,
            nonce_opt,
            created_at,
            dir_int,
            unread,
        ) = row.map_err(|e| StorageError::Database(e.to_string()))?;

        let direction = Direction::from_i32(dir_int).unwrap_or(Direction::Received);

        let preview = match (nonce_opt, ct_opt) {
            (Some(nonce), Some(ct)) => field::decrypt_field(mk, &nonce, &ct)
                .ok()
                .and_then(|b| String::from_utf8(b).ok()),
            _ => None,
        };

        conversations.push(ConversationSummary {
            contact_id,
            alias,
            onion_address,
            last_message_preview: preview,
            last_message_at: created_at,
            last_direction: direction,
            unread_count: unread,
        });
    }

    Ok(conversations)
}

pub fn mark_delivered(engine: &Engine, message_id: i64) -> Result<(), StorageError> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;
    engine.conn().execute(
        "UPDATE messages SET delivered_at = ?1 WHERE id = ?2",
        rusqlite::params![now, message_id],
    )?;
    Ok(())
}

pub fn mark_seen(engine: &Engine, contact_id: i64) -> Result<(), StorageError> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;
    engine.conn().execute(
        "UPDATE messages SET seen_at = ?1 WHERE contact_id = ?2 AND seen_at IS NULL",
        rusqlite::params![now, contact_id],
    )?;
    Ok(())
}

pub fn get_unread_count(engine: &Engine, contact_id: i64) -> Result<i32, StorageError> {
    let count: i32 = engine.conn().query_row(
        "SELECT COUNT(*) FROM messages WHERE contact_id = ?1 AND seen_at IS NULL",
        [contact_id],
        |row| row.get(0),
    )?;
    Ok(count)
}

pub fn delete_conversation(engine: &Engine, contact_id: i64) -> Result<(), StorageError> {
    engine
        .conn()
        .execute("DELETE FROM messages WHERE contact_id = ?1", [contact_id])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keystore::mock::MockKeystore;
    use crate::storage::contact;
    use crate::crypto::types::{IdentityPublicKey, StaticPublicKey};
    use std::sync::Mutex;

    static DB_LOCK: Mutex<()> = Mutex::new(());

    fn setup() -> (MockKeystore, Engine, i64) {
        let _lock = DB_LOCK.lock().unwrap();
        let ks = MockKeystore::new();
        let mut engine = Engine::open(":memory:").unwrap();
        engine.init_master_key(&ks, "test_mk").unwrap();

        let contact_id = contact::add_contact(
            &engine,
            "bob",
            &IdentityPublicKey([1u8; 32]),
            &StaticPublicKey([2u8; 32]),
            "bob.onion",
        )
        .unwrap();

        (ks, engine, contact_id)
    }

    #[test]
    fn test_insert_and_get_messages() {
        let (_, engine, contact_id) = setup();

        let id = insert_message(
            &engine,
            contact_id,
            Direction::Sent,
            b"wire_bytes",
            Some("hello bob"),
            1000,
        )
        .unwrap();

        let msgs = get_messages(&engine, contact_id, 10, 0).unwrap();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].id, id);
        assert_eq!(msgs[0].plaintext.as_deref(), Some("hello bob"));
        assert_eq!(msgs[0].direction, Direction::Sent);
    }

    #[test]
    fn test_insert_message_no_plaintext() {
        let (_, engine, contact_id) = setup();

        insert_message(
            &engine,
            contact_id,
            Direction::Received,
            b"wire",
            None,
            2000,
        )
        .unwrap();

        let msgs = get_messages(&engine, contact_id, 10, 0).unwrap();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].plaintext, None);
    }

    #[test]
    fn test_get_conversations() {
        let (_, engine, contact_id) = setup();

        insert_message(
            &engine,
            contact_id,
            Direction::Received,
            b"wire1",
            Some("first"),
            1000,
        )
        .unwrap();

        insert_message(
            &engine,
            contact_id,
            Direction::Sent,
            b"wire2",
            Some("second"),
            2000,
        )
        .unwrap();

        let convos = get_conversations(&engine).unwrap();
        assert_eq!(convos.len(), 1);
        assert_eq!(convos[0].last_message_preview.as_deref(), Some("second"));
    }

    #[test]
    fn test_mark_delivered_and_seen() {
        let (_, engine, contact_id) = setup();

        let id = insert_message(
            &engine,
            contact_id,
            Direction::Received,
            b"wire",
            Some("test"),
            1000,
        )
        .unwrap();

        mark_delivered(&engine, id).unwrap();
        mark_seen(&engine, contact_id).unwrap();

        let msgs = get_messages(&engine, contact_id, 10, 0).unwrap();
        assert!(msgs[0].delivered_at.is_some());
        assert!(msgs[0].seen_at.is_some());
    }

    #[test]
    fn test_unread_count() {
        let (_, engine, contact_id) = setup();

        insert_message(
            &engine,
            contact_id,
            Direction::Received,
            b"wire1",
            Some("msg1"),
            1000,
        )
        .unwrap();

        insert_message(
            &engine,
            contact_id,
            Direction::Received,
            b"wire2",
            Some("msg2"),
            2000,
        )
        .unwrap();

        assert_eq!(get_unread_count(&engine, contact_id).unwrap(), 2);

        mark_seen(&engine, contact_id).unwrap();
        assert_eq!(get_unread_count(&engine, contact_id).unwrap(), 0);
    }

    #[test]
    fn test_delete_conversation() {
        let (_, engine, contact_id) = setup();

        insert_message(
            &engine,
            contact_id,
            Direction::Sent,
            b"wire",
            Some("delete me"),
            1000,
        )
        .unwrap();

        delete_conversation(&engine, contact_id).unwrap();
        let msgs = get_messages(&engine, contact_id, 10, 0).unwrap();
        assert!(msgs.is_empty());
    }
}
