use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Message {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub thread_id: Option<Uuid>,
    pub sender_id: Uuid,
    pub content: String,
    pub format: String,
    pub seq: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct MessageSendResult {
    pub id: Uuid,
    pub seq: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MessageAttachment {
    pub id: Uuid,
    pub message_id: Uuid,
    pub file_type: String,
    pub file_url: String,
    pub file_size: i64,
    pub mime_type: String,
    pub thumbnail_url: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MediaAttachment {
    pub id: Uuid,
    pub media_type: String,
    pub mime_type: String,
    pub file_size: i64,
    pub s3_key: String,
    pub thumbnail_s3_key: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ConversationListItem {
    pub id: Uuid,
    pub name: Option<String>,
    pub last_message: Option<String>,
    pub unread_count: i64,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ConversationCreateResult {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddMemberRequest {
    pub user_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RemoveMemberResponse {
    pub removed: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateGroupNameRequest {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateMemberRoleRequest {
    pub role: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateMemberRoleResponse {
    pub updated: bool,
}
