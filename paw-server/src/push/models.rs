use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PushPlatform {
    Fcm,
    Apns,
}

impl PushPlatform {
    pub fn as_str(&self) -> &'static str {
        match self {
            PushPlatform::Fcm => "fcm",
            PushPlatform::Apns => "apns",
        }
    }
}

impl std::fmt::Display for PushPlatform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegisterPushTokenRequest {
    pub platform: PushPlatform,
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterPushTokenResponse {
    pub registered: bool,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct PushTokenRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub device_id: Uuid,
    pub platform: String,
    pub token: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MuteConversationRequest {
    #[serde(default)]
    pub duration_minutes: Option<i64>,
    #[serde(default)]
    pub forever: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MuteConversationResponse {
    pub muted: bool,
    pub muted_until: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UnmuteConversationResponse {
    pub unmuted: bool,
}

/// SECURITY: E2EE push payload MUST NOT contain message content.
/// Only metadata for the client to fetch & decrypt locally.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PushPayload {
    #[serde(rename = "type")]
    pub payload_type: String,
    pub conversation_id: Uuid,
    pub sender_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UnregisterPushTokenResponse {
    pub unregistered: bool,
}
