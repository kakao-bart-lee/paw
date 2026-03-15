pub mod handlers;

use crate::db::DbPool;
use anyhow::Context;
use chrono::{DateTime, Utc};
use regex::Regex;
use reqwest::{redirect::Policy, Client, Url};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::OnceLock;
use std::time::Duration;
use uuid::Uuid;

pub const FETCH_TIMEOUT_SECS: u64 = 5;
pub const MAX_URLS_PER_MESSAGE: usize = 3;
pub const MAX_REDIRECTS: usize = 3;

#[derive(Debug, Clone)]
pub struct LinkPreviewService {
    client: Client,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MessagePreviewSource {
    pub message_id: Uuid,
    pub conversation_id: Uuid,
    pub content: String,
}

#[derive(Debug, Clone)]
struct LinkPreviewUpsert {
    url: String,
    title: Option<String>,
    description: Option<String>,
    image_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct LinkPreviewRecord {
    pub id: Uuid,
    pub message_id: Uuid,
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub fetched_at: DateTime<Utc>,
}

#[derive(Debug, Default)]
struct ParsedOgTags {
    title: Option<String>,
    description: Option<String>,
    image: Option<String>,
    url: Option<String>,
}

impl LinkPreviewService {
    pub fn new() -> anyhow::Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(FETCH_TIMEOUT_SECS))
            .redirect(Policy::limited(MAX_REDIRECTS))
            .build()
            .context("build link preview HTTP client")?;

        Ok(Self { client })
    }

    pub fn extract_urls(&self, content: &str) -> Vec<String> {
        extract_urls(content)
    }

    pub async fn generate_and_store_for_message(
        &self,
        db: &DbPool,
        message_id: Uuid,
        content: &str,
    ) -> anyhow::Result<usize> {
        let mut stored_count = 0usize;
        for url in self.extract_urls(content) {
            match self.fetch_preview(&url).await {
                Ok(Some(preview)) => {
                    upsert_preview(db, message_id, &preview).await?;
                    stored_count += 1;
                }
                Ok(None) => {}
                Err(err) => {
                    tracing::warn!(%err, message_id = %message_id, url = %url, "link preview fetch failed");
                }
            }
        }
        Ok(stored_count)
    }

    async fn fetch_preview(&self, url: &str) -> anyhow::Result<Option<LinkPreviewUpsert>> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .with_context(|| format!("fetch link preview for {url}"))?;

        if !response.status().is_success() {
            return Ok(None);
        }

        let final_url = response.url().clone();
        let body = response
            .text()
            .await
            .with_context(|| format!("read link preview response body for {url}"))?;
        let og = parse_og_tags(&body);

        let canonical_url = og
            .url
            .as_deref()
            .and_then(|value| normalize_http_url(value, Some(&final_url)))
            .unwrap_or_else(|| final_url.as_str().to_owned());

        let image_url = og
            .image
            .as_deref()
            .and_then(|value| sanitize_https_image(value, Some(&final_url)));

        if og.title.is_none() && og.description.is_none() && image_url.is_none() {
            return Ok(None);
        }

        Ok(Some(LinkPreviewUpsert {
            url: canonical_url,
            title: og.title,
            description: og.description,
            image_url,
        }))
    }
}

pub async fn find_message_for_user(
    db: &DbPool,
    message_id: Uuid,
    user_id: Uuid,
) -> anyhow::Result<Option<MessagePreviewSource>> {
    sqlx::query_as::<_, MessagePreviewSource>(
        "SELECT m.id AS message_id, m.conversation_id, m.content
         FROM messages m
         JOIN conversation_members cm ON cm.conversation_id = m.conversation_id
         WHERE m.id = $1
           AND m.is_deleted = FALSE
           AND cm.user_id = $2
         LIMIT 1",
    )
    .bind(message_id)
    .bind(user_id)
    .fetch_optional(db.as_ref())
    .await
    .context("load message for link preview access")
}

pub async fn list_cached_previews(
    db: &DbPool,
    message_id: Uuid,
) -> anyhow::Result<Vec<LinkPreviewRecord>> {
    sqlx::query_as::<_, LinkPreviewRecord>(
        "SELECT id, message_id, url, title, description, image_url, fetched_at
         FROM link_previews
         WHERE message_id = $1
         ORDER BY fetched_at DESC, id ASC",
    )
    .bind(message_id)
    .fetch_all(db.as_ref())
    .await
    .context("list cached link previews")
}

async fn upsert_preview(
    db: &DbPool,
    message_id: Uuid,
    preview: &LinkPreviewUpsert,
) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO link_previews (message_id, url, title, description, image_url)
         VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT (message_id, url)
         DO UPDATE SET title = EXCLUDED.title,
                       description = EXCLUDED.description,
                       image_url = EXCLUDED.image_url,
                       fetched_at = NOW()",
    )
    .bind(message_id)
    .bind(&preview.url)
    .bind(&preview.title)
    .bind(&preview.description)
    .bind(&preview.image_url)
    .execute(db.as_ref())
    .await
    .context("upsert link preview")?;

    Ok(())
}

fn extract_urls(content: &str) -> Vec<String> {
    static URL_RE: OnceLock<Regex> = OnceLock::new();
    let url_re = URL_RE
        .get_or_init(|| Regex::new(r#"https?://[^\s<>()\[\]{}\"']+"#).expect("valid url regex"));

    let mut urls = Vec::new();
    for mat in url_re.find_iter(content) {
        let trimmed = trim_trailing_punctuation(mat.as_str());
        if let Some(normalized) = normalize_http_url(trimmed, None) {
            urls.push(normalized);
        }

        if urls.len() >= MAX_URLS_PER_MESSAGE {
            break;
        }
    }

    urls
}

fn parse_og_tags(html: &str) -> ParsedOgTags {
    static META_RE: OnceLock<Regex> = OnceLock::new();
    static ATTR_RE: OnceLock<Regex> = OnceLock::new();

    let meta_re =
        META_RE.get_or_init(|| Regex::new(r#"(?is)<meta\s+[^>]*>"#).expect("valid meta tag regex"));
    let attr_re = ATTR_RE.get_or_init(|| {
        Regex::new(r#"(?is)([a-zA-Z_:.-]+)\s*=\s*(?:\"([^\"]*)\"|'([^']*)'|([^\s\"'>]+))"#)
            .expect("valid html attr regex")
    });

    let mut tags = ParsedOgTags::default();

    for meta in meta_re.find_iter(html) {
        let attrs = parse_attrs(meta.as_str(), attr_re);
        let property = attrs
            .get("property")
            .or_else(|| attrs.get("name"))
            .map(|value| value.to_ascii_lowercase());
        let Some(property) = property else {
            continue;
        };

        let Some(content) = attrs.get("content") else {
            continue;
        };

        let value = content.trim();
        if value.is_empty() {
            continue;
        }

        match property.as_str() {
            "og:title" if tags.title.is_none() => tags.title = Some(value.to_owned()),
            "og:description" if tags.description.is_none() => {
                tags.description = Some(value.to_owned())
            }
            "og:image" if tags.image.is_none() => tags.image = Some(value.to_owned()),
            "og:url" if tags.url.is_none() => tags.url = Some(value.to_owned()),
            _ => {}
        }
    }

    tags
}

fn parse_attrs(tag: &str, attr_re: &Regex) -> HashMap<String, String> {
    let mut attrs = HashMap::new();
    for caps in attr_re.captures_iter(tag) {
        let Some(key_match) = caps.get(1) else {
            continue;
        };

        let value = caps
            .get(2)
            .or_else(|| caps.get(3))
            .or_else(|| caps.get(4))
            .map(|m| m.as_str().to_owned())
            .unwrap_or_default();

        attrs.insert(key_match.as_str().to_ascii_lowercase(), value);
    }
    attrs
}

fn trim_trailing_punctuation(raw_url: &str) -> &str {
    raw_url.trim_end_matches(['.', ',', ';', ':', '!', '?', ')', ']', '}', '\'', '"'])
}

fn normalize_http_url(raw_url: &str, base: Option<&Url>) -> Option<String> {
    let parsed = match Url::parse(raw_url) {
        Ok(url) => url,
        Err(_) => {
            let base = base?;
            base.join(raw_url).ok()?
        }
    };

    if parsed.scheme() != "http" && parsed.scheme() != "https" {
        return None;
    }

    Some(parsed.to_string())
}

fn sanitize_https_image(raw_url: &str, base: Option<&Url>) -> Option<String> {
    let normalized = normalize_http_url(raw_url, base)?;
    let parsed = Url::parse(&normalized).ok()?;

    if parsed.scheme() != "https" {
        return None;
    }

    Some(parsed.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        http::{StatusCode, Uri},
        response::{Html, IntoResponse},
        routing::get,
        Router,
    };
    use tokio::net::TcpListener;

    fn service() -> LinkPreviewService {
        LinkPreviewService::new().expect("service construction")
    }

    #[test]
    fn extract_urls_respects_limit_and_trims_punctuation() {
        let content = "https://a.example/path, https://b.example/path! https://c.example/path? https://d.example/path";
        let urls = extract_urls(content);

        assert_eq!(urls.len(), MAX_URLS_PER_MESSAGE);
        assert_eq!(urls[0], "https://a.example/path");
        assert_eq!(urls[1], "https://b.example/path");
        assert_eq!(urls[2], "https://c.example/path");
    }

    #[test]
    fn parse_og_tags_extracts_expected_fields() {
        let html = r#"
            <html><head>
                <meta property="og:title" content="Paw Title" />
                <meta property='og:description' content='Paw Description' />
                <meta property='og:image' content='https://cdn.example.com/image.png' />
                <meta property='og:url' content='https://example.com/canonical' />
            </head></html>
        "#;

        let parsed = parse_og_tags(html);
        assert_eq!(parsed.title.as_deref(), Some("Paw Title"));
        assert_eq!(parsed.description.as_deref(), Some("Paw Description"));
        assert_eq!(
            parsed.image.as_deref(),
            Some("https://cdn.example.com/image.png")
        );
        assert_eq!(parsed.url.as_deref(), Some("https://example.com/canonical"));
    }

    #[test]
    fn sanitize_https_image_rejects_non_https() {
        assert!(sanitize_https_image("http://example.com/image.png", None).is_none());
        assert!(sanitize_https_image("ftp://example.com/image.png", None).is_none());
        assert_eq!(
            sanitize_https_image("https://example.com/image.png", None).as_deref(),
            Some("https://example.com/image.png")
        );
    }

    #[tokio::test]
    async fn fetch_preview_parses_response_and_sanitizes_image() {
        let app = Router::new().route(
            "/ok",
            get(|| async {
                Html(
                    r#"<html><head>
                        <meta property="og:title" content="Title" />
                        <meta property="og:description" content="Description" />
                        <meta property="og:image" content="http://example.com/not-https.png" />
                        <meta property="og:url" content="https://example.com/final" />
                    </head></html>"#,
                )
            }),
        );

        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let addr = listener.local_addr().expect("local addr");
        tokio::spawn(async move {
            axum::serve(listener, app.into_make_service())
                .await
                .expect("serve app");
        });

        let preview = service()
            .fetch_preview(&format!("http://{addr}/ok"))
            .await
            .expect("fetch preview")
            .expect("preview should exist");

        assert_eq!(preview.title.as_deref(), Some("Title"));
        assert_eq!(preview.description.as_deref(), Some("Description"));
        assert_eq!(preview.url, "https://example.com/final");
        assert!(preview.image_url.is_none());
    }

    #[tokio::test]
    async fn fetch_preview_respects_redirect_limit() {
        async fn redirect(uri: &'static str) -> impl IntoResponse {
            (
                StatusCode::FOUND,
                [(axum::http::header::LOCATION, uri)],
                "redirect",
            )
        }

        async fn redirect_handler(uri: Uri) -> impl IntoResponse {
            match uri.path() {
                "/r1" => redirect("/r2").await.into_response(),
                "/r2" => redirect("/r3").await.into_response(),
                "/r3" => redirect("/r4").await.into_response(),
                "/r4" => redirect("/ok").await.into_response(),
                "/ok" => Html(r#"<meta property="og:title" content="redirect reached"/>"#)
                    .into_response(),
                _ => StatusCode::NOT_FOUND.into_response(),
            }
        }

        let app = Router::new().route("/{*path}", get(redirect_handler));

        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let addr = listener.local_addr().expect("local addr");
        tokio::spawn(async move {
            axum::serve(listener, app.into_make_service())
                .await
                .expect("serve app");
        });

        let result = service().fetch_preview(&format!("http://{addr}/r1")).await;
        assert!(result.is_err(), "expected redirect limit error");
    }
}
