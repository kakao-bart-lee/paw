use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct MediaAttachment {
    pub id: Uuid,
    pub message_id: Option<Uuid>,
    pub uploader_id: Uuid,
    pub media_type: String,
    pub mime_type: String,
    pub file_name: String,
    pub file_size: i64,
    pub s3_key: String,
    pub thumbnail_s3_key: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub duration_ms: Option<i32>,
    pub created_at: DateTime<Utc>,
}
