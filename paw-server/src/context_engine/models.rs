use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub enum ContextEventType {
    MessageCreated,
    MessageEdited,
    MessageDeleted,
    MemberJoined,
    MemberLeft,
    ThreadCreated,
    ConversationSettingsChanged,
}

impl ContextEventType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MessageCreated => "message_created",
            Self::MessageEdited => "message_edited",
            Self::MessageDeleted => "message_deleted",
            Self::MemberJoined => "member_joined",
            Self::MemberLeft => "member_left",
            Self::ThreadCreated => "thread_created",
            Self::ConversationSettingsChanged => "conversation_settings_changed",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct MessageCreatedData {
    pub message_id: Uuid,
    pub thread_id: Option<Uuid>,
    pub sender_id: Uuid,
    pub content: String,
    pub format: String,
    pub seq: i64,
}

#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
pub struct MessageEditedData {
    pub message_id: Uuid,
    pub thread_id: Option<Uuid>,
    pub edited_by: Uuid,
    pub content: String,
    pub format: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MessageDeletedData {
    pub message_id: Uuid,
    pub thread_id: Option<Uuid>,
    pub deleted_by: Uuid,
}

#[derive(Debug, Clone, Serialize)]
pub struct MemberJoinedData {
    pub member_id: Uuid,
    pub joined_by: Uuid,
}

#[derive(Debug, Clone, Serialize)]
pub struct MemberLeftData {
    pub member_id: Uuid,
    pub left_by: Uuid,
}

#[derive(Debug, Clone, Serialize)]
pub struct ThreadCreatedData {
    pub thread_id: Uuid,
    pub root_message_id: Uuid,
    pub created_by: Uuid,
    pub title: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MessageCreatedHook {
    pub conversation_id: Uuid,
    pub message_id: Uuid,
    pub thread_id: Option<Uuid>,
    pub sender_id: Uuid,
    pub content: String,
    pub format: String,
    pub seq: i64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct MessageEditedHook {
    pub conversation_id: Uuid,
    pub message_id: Uuid,
    pub thread_id: Option<Uuid>,
    pub edited_by: Uuid,
    pub content: String,
    pub format: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct MessageDeletedHook {
    pub conversation_id: Uuid,
    pub message_id: Uuid,
    pub thread_id: Option<Uuid>,
    pub deleted_by: Uuid,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct MemberJoinedHook {
    pub conversation_id: Uuid,
    pub member_id: Uuid,
    pub joined_by: Uuid,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct MemberLeftHook {
    pub conversation_id: Uuid,
    pub member_id: Uuid,
    pub left_by: Uuid,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct ThreadCreatedHook {
    pub conversation_id: Uuid,
    pub thread_id: Uuid,
    pub root_message_id: Uuid,
    pub created_by: Uuid,
    pub title: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct ConversationSettingsChangedHook {
    pub conversation_id: Uuid,
    pub changed_by: Uuid,
    pub changes: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
