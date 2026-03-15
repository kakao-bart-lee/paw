use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::messages::models::Message;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Thread {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub root_message_id: Uuid,
    pub title: Option<String>,
    pub created_by: Uuid,
    pub message_count: i32,
    pub last_seq: Option<i64>,
    pub last_message_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateThreadRequest {
    pub root_message_id: Uuid,
    pub title: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct UpdateThreadTitleRequest {
    pub title: Option<String>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct ArchiveThreadResponse {
    pub archived: bool,
}

#[derive(Debug, Deserialize, Default, PartialEq, Eq)]
pub struct GetThreadMessagesQuery {
    pub since_seq: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct GetThreadMessagesResponse {
    pub messages: Vec<Message>,
    pub has_more: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ThreadStateSnapshot {
    pub thread_id: Uuid,
    pub message_count: i32,
    pub last_seq: i64,
    pub participants: Vec<Uuid>,
    pub last_message_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ThreadMembershipResponse {
    pub ok: bool,
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::{
        ArchiveThreadResponse, GetThreadMessagesQuery, ThreadMembershipResponse,
        ThreadStateSnapshot, UpdateThreadTitleRequest,
    };
    use uuid::Uuid;

    #[test]
    fn update_thread_title_request_roundtrip() {
        let request = UpdateThreadTitleRequest {
            title: Some("  renamed thread  ".to_string()),
        };

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["title"], "  renamed thread  ");

        let parsed: UpdateThreadTitleRequest = serde_json::from_value(json).unwrap();
        assert_eq!(parsed, request);
    }

    #[test]
    fn archive_thread_response_serializes_archived_flag() {
        let response = ArchiveThreadResponse { archived: true };
        let json = serde_json::to_value(&response).unwrap();

        assert_eq!(json["archived"], true);
    }

    #[test]
    fn get_thread_messages_query_defaults_to_empty_values() {
        let query = GetThreadMessagesQuery::default();

        assert_eq!(query.since_seq, None);
        assert_eq!(query.limit, None);
    }

    #[test]
    fn thread_state_snapshot_roundtrip_preserves_participants() {
        let snapshot = ThreadStateSnapshot {
            thread_id: Uuid::new_v4(),
            message_count: 4,
            last_seq: 9,
            participants: vec![Uuid::new_v4(), Uuid::new_v4()],
            last_message_at: Some(Utc.with_ymd_and_hms(2026, 3, 19, 9, 30, 0).unwrap()),
        };

        let json = serde_json::to_value(&snapshot).unwrap();
        assert_eq!(json["message_count"], 4);
        assert_eq!(json["last_seq"], 9);
        assert_eq!(json["participants"].as_array().unwrap().len(), 2);

        let parsed: ThreadStateSnapshot = serde_json::from_value(json).unwrap();
        assert_eq!(parsed, snapshot);
    }

    #[test]
    fn thread_membership_response_serializes_ok_flag() {
        let response = ThreadMembershipResponse { ok: true };
        let json = serde_json::to_value(&response).unwrap();

        assert_eq!(json["ok"], true);
    }
}
