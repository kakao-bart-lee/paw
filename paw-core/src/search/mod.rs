use chrono::{DateTime, TimeZone, Utc};
use rusqlite::params;
use serde::{Deserialize, Serialize};

use crate::db::{AppDatabase, DbResult};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub message_id: String,
    pub conversation_id: String,
    pub snippet: String,
    pub highlighted_content: String,
    pub created_at: DateTime<Utc>,
}

impl PartialEq for SearchResult {
    fn eq(&self, other: &Self) -> bool {
        self.message_id == other.message_id
    }
}

impl Eq for SearchResult {}

pub struct SearchService<'a> {
    db: &'a AppDatabase,
}

impl<'a> SearchService<'a> {
    pub fn new(db: &'a AppDatabase) -> Self {
        Self { db }
    }

    pub fn build_fts5_query(raw: &str) -> String {
        raw.split_whitespace()
            .filter(|token| !token.is_empty())
            .map(|token| format!("\"{}\"", token.replace('"', "\"\"")))
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub fn search(&self, query: &str, limit: usize) -> DbResult<Vec<SearchResult>> {
        let fts_query = Self::build_fts5_query(query);
        if fts_query.is_empty() {
            return Ok(Vec::new());
        }

        let conn = self.db.conn()?;
        let mut stmt = conn.prepare(
            r#"
            SELECT
                m.id,
                m.conversation_id,
                snippet(messages_fts, 0, '[', ']', '...', 32) AS snippet,
                highlight(messages_fts, 0, '[', ']') AS highlighted_content,
                m.created_at
            FROM messages_fts
            JOIN messages_table m ON m.rowid = messages_fts.rowid
            WHERE messages_fts MATCH ?1
            ORDER BY rank
            LIMIT ?2
            "#,
        )?;

        let rows = stmt.query_map(params![fts_query, limit as i64], |row| {
            Ok(SearchResult {
                message_id: row.get(0)?,
                conversation_id: row.get(1)?,
                snippet: row.get(2)?,
                highlighted_content: row.get(3)?,
                created_at: Utc
                    .timestamp_opt(row.get::<_, i64>(4)?, 0)
                    .single()
                    .unwrap(),
            })
        })?;

        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::{SearchResult, SearchService};
    use crate::db::{AppDatabase, MessageRecord};
    use chrono::{TimeZone, Utc};

    #[test]
    fn build_fts5_query_matches_flutter_behavior() {
        assert_eq!(SearchService::build_fts5_query(""), "");
        assert_eq!(SearchService::build_fts5_query("   "), "");
        assert_eq!(SearchService::build_fts5_query("hello"), "\"hello\"");
        assert_eq!(
            SearchService::build_fts5_query("hello world"),
            "\"hello\" \"world\""
        );
        assert_eq!(
            SearchService::build_fts5_query("say \"hi\""),
            "\"say\" \"\"\"hi\"\"\""
        );
        assert_eq!(
            SearchService::build_fts5_query("안녕 세계"),
            "\"안녕\" \"세계\""
        );
    }

    #[test]
    fn search_result_equality_is_based_on_message_id() {
        let a = SearchResult {
            message_id: "msg-1".to_string(),
            conversation_id: "conv-1".to_string(),
            snippet: "hello".to_string(),
            highlighted_content: "[hello]".to_string(),
            created_at: Utc.timestamp_opt(1_735_689_600, 0).single().unwrap(),
        };
        let b = SearchResult {
            message_id: "msg-1".to_string(),
            conversation_id: "conv-2".to_string(),
            snippet: "different".to_string(),
            highlighted_content: "[different]".to_string(),
            created_at: Utc.timestamp_opt(1_735_700_000, 0).single().unwrap(),
        };

        assert_eq!(a, b);
    }

    #[test]
    fn search_respects_limit_and_highlights_matches() {
        let db = AppDatabase::open_in_memory().unwrap();
        for index in 0..5 {
            db.upsert_message(&MessageRecord {
                id: format!("msg-{index}"),
                conversation_id: "conv-1".to_string(),
                thread_id: None,
                sender_id: "user-1".to_string(),
                content: format!("Searchable content item {index}"),
                format: "markdown".to_string(),
                seq: index,
                created_at: Utc
                    .timestamp_opt(1_735_689_600 + index, 0)
                    .single()
                    .unwrap(),
                is_me: false,
                is_agent: false,
            })
            .unwrap();
        }

        let results = SearchService::new(&db).search("searchable", 3).unwrap();
        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|result| {
            result.highlighted_content.contains("[Searchable]")
                || result.highlighted_content.contains("[searchable]")
        }));
    }

    #[test]
    fn blank_queries_short_circuit_without_touching_sqlite_match() {
        let db = AppDatabase::open_in_memory().unwrap();
        let service = SearchService::new(&db);

        assert!(service.search("", 50).unwrap().is_empty());
        assert!(service.search("   ", 50).unwrap().is_empty());
    }
}
