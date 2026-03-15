use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub const LINK_PREVIEW_PENDING: &str = "pending";
pub const LINK_PREVIEW_READY: &str = "ready";
pub const LINK_PREVIEW_FAILED: &str = "failed";

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LinkPreview {
    pub url: String,
    pub canonical_url: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub site_name: Option<String>,
    pub status: String,
    pub fetched_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct RequestLinkPreviewsPayload {
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct RequestLinkPreviewsResponse {
    pub requested_urls: Vec<String>,
    pub previews: Vec<LinkPreview>,
}

#[derive(Debug, Deserialize)]
pub struct GetLinkPreviewQuery {
    pub url: String,
}
