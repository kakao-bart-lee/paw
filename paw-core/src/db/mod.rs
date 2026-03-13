use std::path::Path;
use std::sync::{Mutex, MutexGuard};

use chrono::{DateTime, TimeZone, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use thiserror::Error;

const SCHEMA_VERSION: i64 = 2;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("mutex poisoned")]
    LockPoisoned,
    #[error("database encryption is not available on this platform yet")]
    EncryptionUnavailable,
}

pub type DbResult<T> = Result<T, DbError>;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MessageRecord {
    pub id: String,
    pub conversation_id: String,
    pub sender_id: String,
    pub content: String,
    pub format: String,
    pub seq: i64,
    pub created_at: DateTime<Utc>,
    pub is_me: bool,
    pub is_agent: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConversationRecord {
    pub id: String,
    pub name: String,
    pub avatar_url: Option<String>,
    pub last_seq: i64,
    pub unread_count: i64,
    pub updated_at: DateTime<Utc>,
}

pub struct AppDatabase {
    connection: Mutex<Connection>,
}

impl AppDatabase {
    pub fn open_in_memory() -> DbResult<Self> {
        let connection = Connection::open_in_memory()?;
        let db = Self {
            connection: Mutex::new(connection),
        };
        db.initialize()?;
        Ok(db)
    }

    pub fn open<P: AsRef<Path>>(path: P) -> DbResult<Self> {
        Self::open_with_key(path, None)
    }

    pub fn open_encrypted<P: AsRef<Path>>(path: P, key: &str) -> DbResult<Self> {
        Self::open_with_key(path, Some(key))
    }

    fn open_with_key<P: AsRef<Path>>(path: P, key: Option<&str>) -> DbResult<Self> {
        let connection = Connection::open(path)?;
        configure_connection(&connection, key)?;
        let db = Self {
            connection: Mutex::new(connection),
        };
        db.initialize()?;
        Ok(db)
    }

    pub(crate) fn conn(&self) -> DbResult<MutexGuard<'_, Connection>> {
        self.connection.lock().map_err(|_| DbError::LockPoisoned)
    }

    pub fn schema_version(&self) -> i64 {
        SCHEMA_VERSION
    }

    fn initialize(&self) -> DbResult<()> {
        let conn = self.conn()?;
        conn.execute_batch(&format!(
            r#"
            PRAGMA foreign_keys = ON;
            PRAGMA user_version = {SCHEMA_VERSION};

            CREATE TABLE IF NOT EXISTS messages_table (
                id TEXT PRIMARY KEY,
                conversation_id TEXT NOT NULL,
                sender_id TEXT NOT NULL,
                content TEXT NOT NULL,
                format TEXT NOT NULL DEFAULT 'markdown',
                seq INTEGER NOT NULL,
                created_at INTEGER NOT NULL,
                is_me INTEGER NOT NULL DEFAULT 0,
                is_agent INTEGER NOT NULL DEFAULT 0
            );

            CREATE INDEX IF NOT EXISTS messages_conv_seq
            ON messages_table (conversation_id, seq);

            CREATE TABLE IF NOT EXISTS conversations_table (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                avatar_url TEXT,
                last_seq INTEGER NOT NULL DEFAULT 0,
                unread_count INTEGER NOT NULL DEFAULT 0,
                updated_at INTEGER NOT NULL
            );

            CREATE VIRTUAL TABLE IF NOT EXISTS messages_fts USING fts5(
                content,
                content='messages_table',
                content_rowid='rowid',
                tokenize='unicode61'
            );

            CREATE TRIGGER IF NOT EXISTS messages_fts_ai
            AFTER INSERT ON messages_table BEGIN
                INSERT INTO messages_fts(rowid, content)
                VALUES (new.rowid, new.content);
            END;

            CREATE TRIGGER IF NOT EXISTS messages_fts_ad
            AFTER DELETE ON messages_table BEGIN
                INSERT INTO messages_fts(messages_fts, rowid, content)
                VALUES ('delete', old.rowid, old.content);
            END;

            CREATE TRIGGER IF NOT EXISTS messages_fts_au
            AFTER UPDATE ON messages_table BEGIN
                INSERT INTO messages_fts(messages_fts, rowid, content)
                VALUES ('delete', old.rowid, old.content);
                INSERT INTO messages_fts(rowid, content)
                VALUES (new.rowid, new.content);
            END;
            "#,
        ))?;
        Ok(())
    }

    pub fn rebuild_fts_index(&self) -> DbResult<()> {
        let conn = self.conn()?;
        conn.execute(
            "INSERT INTO messages_fts(messages_fts) VALUES ('rebuild')",
            [],
        )?;
        Ok(())
    }

    pub fn upsert_message(&self, message: &MessageRecord) -> DbResult<()> {
        let conn = self.conn()?;
        conn.execute(
            r#"
            INSERT INTO messages_table (
                id, conversation_id, sender_id, content, format, seq, created_at, is_me, is_agent
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            ON CONFLICT(id) DO UPDATE SET
                conversation_id = excluded.conversation_id,
                sender_id = excluded.sender_id,
                content = excluded.content,
                format = excluded.format,
                seq = excluded.seq,
                created_at = excluded.created_at,
                is_me = excluded.is_me,
                is_agent = excluded.is_agent
            "#,
            params![
                message.id.as_str(),
                message.conversation_id.as_str(),
                message.sender_id.as_str(),
                message.content.as_str(),
                message.format.as_str(),
                message.seq,
                message.created_at.timestamp(),
                message.is_me,
                message.is_agent,
            ],
        )?;
        Ok(())
    }

    pub fn get_messages(
        &self,
        conversation_id: &str,
        after_seq: i64,
    ) -> DbResult<Vec<MessageRecord>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare(
            r#"
            SELECT id, conversation_id, sender_id, content, format, seq, created_at, is_me, is_agent
            FROM messages_table
            WHERE conversation_id = ?1 AND seq > ?2
            ORDER BY seq ASC
            "#,
        )?;
        let rows = stmt.query_map(params![conversation_id, after_seq], read_message)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn get_last_seq(&self, conversation_id: &str) -> DbResult<i64> {
        let conn = self.conn()?;
        let seq = conn.query_row(
            "SELECT COALESCE(MAX(seq), 0) FROM messages_table WHERE conversation_id = ?1",
            [conversation_id],
            |row| row.get(0),
        )?;
        Ok(seq)
    }

    pub fn upsert_conversation(&self, conversation: &ConversationRecord) -> DbResult<()> {
        let conn = self.conn()?;
        conn.execute(
            r#"
            INSERT INTO conversations_table (id, name, avatar_url, last_seq, unread_count, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                avatar_url = excluded.avatar_url,
                last_seq = excluded.last_seq,
                unread_count = excluded.unread_count,
                updated_at = excluded.updated_at
            "#,
            params![
                conversation.id.as_str(),
                conversation.name.as_str(),
                conversation.avatar_url.as_deref(),
                conversation.last_seq,
                conversation.unread_count,
                conversation.updated_at.timestamp(),
            ],
        )?;
        Ok(())
    }

    pub fn get_all_conversations(&self) -> DbResult<Vec<ConversationRecord>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare(
            r#"
            SELECT id, name, avatar_url, last_seq, unread_count, updated_at
            FROM conversations_table
            ORDER BY updated_at DESC
            "#,
        )?;
        let rows = stmt.query_map([], read_conversation)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn update_last_seq(&self, conversation_id: &str, seq: i64) -> DbResult<()> {
        let conn = self.conn()?;
        conn.execute(
            "UPDATE conversations_table SET last_seq = ?2 WHERE id = ?1",
            params![conversation_id, seq],
        )?;
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn clear_fts_rows_for_test(&self) -> DbResult<()> {
        let conn = self.conn()?;
        conn.execute("DELETE FROM messages_fts", [])?;
        Ok(())
    }
}

fn configure_connection(connection: &Connection, key: Option<&str>) -> DbResult<()> {
    if let Some(key) = key {
        if !supports_database_encryption() {
            return Err(DbError::EncryptionUnavailable);
        }
        let escaped = key.replace('\'', "''");
        connection.execute_batch(&format!(
            "PRAGMA key = '{escaped}'; PRAGMA cipher_page_size = 4096; PRAGMA kdf_iter = 64000;"
        ))?;
    }
    Ok(())
}

fn supports_database_encryption() -> bool {
    cfg!(not(any(target_os = "android", target_os = "ios")))
}

fn read_message(row: &rusqlite::Row<'_>) -> rusqlite::Result<MessageRecord> {
    Ok(MessageRecord {
        id: row.get(0)?,
        conversation_id: row.get(1)?,
        sender_id: row.get(2)?,
        content: row.get(3)?,
        format: row.get(4)?,
        seq: row.get(5)?,
        created_at: Utc
            .timestamp_opt(row.get::<_, i64>(6)?, 0)
            .single()
            .unwrap(),
        is_me: row.get(7)?,
        is_agent: row.get(8)?,
    })
}

fn read_conversation(row: &rusqlite::Row<'_>) -> rusqlite::Result<ConversationRecord> {
    Ok(ConversationRecord {
        id: row.get(0)?,
        name: row.get(1)?,
        avatar_url: row.get(2)?,
        last_seq: row.get(3)?,
        unread_count: row.get(4)?,
        updated_at: Utc
            .timestamp_opt(row.get::<_, i64>(5)?, 0)
            .single()
            .unwrap(),
    })
}

#[cfg(test)]
mod tests {
    use super::{AppDatabase, ConversationRecord, MessageRecord};
    use crate::search::SearchService;
    use chrono::{TimeZone, Utc};
    use tempfile::tempdir;

    fn message(id: &str, conversation_id: &str, seq: i64, content: &str) -> MessageRecord {
        MessageRecord {
            id: id.to_string(),
            conversation_id: conversation_id.to_string(),
            sender_id: "user-1".to_string(),
            content: content.to_string(),
            format: "markdown".to_string(),
            seq,
            created_at: Utc.timestamp_opt(1_735_689_600 + seq, 0).single().unwrap(),
            is_me: false,
            is_agent: false,
        }
    }

    fn conversation(id: &str, updated_at: i64) -> ConversationRecord {
        ConversationRecord {
            id: id.to_string(),
            name: format!("Conversation {id}"),
            avatar_url: None,
            last_seq: 0,
            unread_count: 0,
            updated_at: Utc.timestamp_opt(updated_at, 0).single().unwrap(),
        }
    }

    #[test]
    fn message_queries_match_flutter_dao_behavior() {
        let db = AppDatabase::open_in_memory().unwrap();
        db.upsert_message(&message("msg-2", "conv-1", 2, "second"))
            .unwrap();
        db.upsert_message(&message("msg-1", "conv-1", 1, "first"))
            .unwrap();
        db.upsert_message(&message("msg-3", "conv-2", 3, "other"))
            .unwrap();

        let messages = db.get_messages("conv-1", 1).unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].id, "msg-2");
        assert_eq!(db.get_last_seq("conv-1").unwrap(), 2);
        assert_eq!(db.get_last_seq("missing").unwrap(), 0);
    }

    #[test]
    fn upsert_message_updates_existing_rows_and_fts_index() {
        let db = AppDatabase::open_in_memory().unwrap();
        let mut original = message("msg-1", "conv-1", 1, "hello world");
        db.upsert_message(&original).unwrap();

        original.content = "updated phrase".to_string();
        original.seq = 4;
        db.upsert_message(&original).unwrap();

        let messages = db.get_messages("conv-1", 0).unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].content, "updated phrase");
        assert_eq!(messages[0].seq, 4);

        let results = SearchService::new(&db).search("updated", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].message_id, "msg-1");
    }

    #[test]
    fn conversation_queries_match_flutter_dao_behavior() {
        let db = AppDatabase::open_in_memory().unwrap();
        db.upsert_conversation(&conversation("conv-1", 100))
            .unwrap();
        db.upsert_conversation(&conversation("conv-2", 200))
            .unwrap();
        db.update_last_seq("conv-1", 9).unwrap();

        let conversations = db.get_all_conversations().unwrap();
        assert_eq!(
            conversations
                .iter()
                .map(|c| c.id.as_str())
                .collect::<Vec<_>>(),
            vec!["conv-2", "conv-1"]
        );
        assert_eq!(conversations[1].last_seq, 9);
    }

    #[test]
    fn rebuild_fts_restores_search_index_after_drift() {
        let db = AppDatabase::open_in_memory().unwrap();
        db.upsert_message(&message("msg-1", "conv-1", 1, "rebuild target"))
            .unwrap();
        assert_eq!(
            SearchService::new(&db).search("rebuild", 10).unwrap().len(),
            1
        );

        db.clear_fts_rows_for_test().unwrap();
        assert!(SearchService::new(&db)
            .search("rebuild", 10)
            .unwrap()
            .is_empty());

        db.rebuild_fts_index().unwrap();
        let results = SearchService::new(&db).search("rebuild", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].message_id, "msg-1");
    }

    #[test]
    fn encrypted_file_database_reopens_with_same_key() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("paw-encrypted.db");
        let key = "paw-phase2-test-key";

        {
            let db = AppDatabase::open_encrypted(&path, key).unwrap();
            db.upsert_conversation(&conversation("conv-1", 100))
                .unwrap();
            db.upsert_message(&message("msg-1", "conv-1", 1, "persisted secret"))
                .unwrap();
        }

        let reopened = AppDatabase::open_encrypted(&path, key).unwrap();
        let conversations = reopened.get_all_conversations().unwrap();
        let messages = reopened.get_messages("conv-1", 0).unwrap();

        assert_eq!(conversations.len(), 1);
        assert_eq!(conversations[0].id, "conv-1");
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].content, "persisted secret");
    }
}
