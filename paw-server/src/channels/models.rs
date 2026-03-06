use crate::messages::models::Message;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateChannelRequest {
    pub name: String,
    pub is_public: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct CreateChannelResponse {
    pub id: Uuid,
    pub name: String,
    pub owner_id: Uuid,
    pub is_public: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct ListChannelsQuery {
    pub q: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ChannelSummary {
    pub id: Uuid,
    pub name: String,
    pub owner_id: Uuid,
    pub is_public: bool,
    pub created_at: DateTime<Utc>,
    pub subscribed: bool,
}

#[derive(Debug, Serialize)]
pub struct ListChannelsResponse {
    pub channels: Vec<ChannelSummary>,
}

#[derive(Debug, Serialize)]
pub struct SubscribeResponse {
    pub subscribed: bool,
}

#[derive(Debug, Serialize)]
pub struct UnsubscribeResponse {
    pub unsubscribed: bool,
}

#[derive(Debug, Deserialize)]
pub struct SendChannelMessageRequest {
    pub content: String,
    pub format: String,
    pub idempotency_key: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct GetChannelMessagesQuery {
    pub after_seq: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct GetChannelMessagesResponse {
    pub messages: Vec<Message>,
    pub has_more: bool,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ChannelRecord {
    pub id: Uuid,
    pub name: String,
    pub owner_id: Uuid,
    pub is_public: bool,
    pub created_at: DateTime<Utc>,
}
