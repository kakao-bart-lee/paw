use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

#[cfg(test)]
mod tests {
    use super::{ArchiveThreadResponse, UpdateThreadTitleRequest};

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
}
