use crate::db::DbPool;
use crate::link_preview::models::{
    LinkPreview, LINK_PREVIEW_FAILED, LINK_PREVIEW_PENDING, LINK_PREVIEW_READY,
};
use anyhow::{anyhow, Context};
use regex::Regex;
use reqwest::header::CONTENT_TYPE;
use reqwest::{Client, Url};
use scraper::{Html, Selector};
use std::collections::HashSet;
use std::time::Duration;

const MAX_LINK_PREVIEW_URLS: usize = 3;
const LINK_PREVIEW_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Clone)]
pub struct LinkPreviewService {
    client: Client,
}

#[derive(Debug, Default)]
struct ParsedPreview {
    canonical_url: Option<String>,
    title: Option<String>,
    description: Option<String>,
    image_url: Option<String>,
    site_name: Option<String>,
}

impl ParsedPreview {
    fn has_renderable_content(&self) -> bool {
        self.title.is_some() || self.description.is_some() || self.image_url.is_some()
    }
}

impl LinkPreviewService {
    pub fn new() -> Self {
        let client = Client::builder()
            .redirect(reqwest::redirect::Policy::limited(5))
            .user_agent("paw-server-link-preview/2")
            .build()
            .expect("link preview client initialization must succeed");

        Self { client }
    }

    pub fn extract_urls(content: &str) -> Vec<String> {
        let pattern = Regex::new(r"https?://[^\s<>()]+")
            .expect("URL extraction regex must be valid at compile-time");
        let mut dedup = HashSet::new();
        let mut urls = Vec::new();

        for candidate in pattern.find_iter(content) {
            if urls.len() >= MAX_LINK_PREVIEW_URLS {
                break;
            }

            let normalized = match normalize_url(candidate.as_str()) {
                Some(url) => url,
                None => continue,
            };

            if dedup.insert(normalized.clone()) {
                urls.push(normalized);
            }
        }

        urls
    }

    pub async fn request_previews(
        &self,
        pool: &DbPool,
        content: &str,
    ) -> anyhow::Result<Vec<String>> {
        let urls = Self::extract_urls(content);
        for url in &urls {
            if self.enqueue_url(pool, url).await? {
                self.spawn_fetch(pool.clone(), url.clone());
            }
        }

        Ok(urls)
    }

    pub async fn get_preview_by_url(
        &self,
        pool: &DbPool,
        url: &str,
    ) -> anyhow::Result<Option<LinkPreview>> {
        sqlx::query_as::<_, LinkPreview>(
            "SELECT url, canonical_url, title, description, image_url, site_name, status, fetched_at, created_at, updated_at
             FROM link_previews
             WHERE url = $1",
        )
        .bind(url)
        .fetch_optional(pool.as_ref())
        .await
        .context("fetch link preview by url")
    }

    pub async fn get_previews_by_urls(
        &self,
        pool: &DbPool,
        urls: &[String],
    ) -> anyhow::Result<Vec<LinkPreview>> {
        if urls.is_empty() {
            return Ok(Vec::new());
        }

        sqlx::query_as::<_, LinkPreview>(
            "SELECT url, canonical_url, title, description, image_url, site_name, status, fetched_at, created_at, updated_at
             FROM link_previews
             WHERE url = ANY($1::text[])
             ORDER BY updated_at DESC",
        )
        .bind(urls)
        .fetch_all(pool.as_ref())
        .await
        .context("fetch link previews by urls")
    }

    async fn enqueue_url(&self, pool: &DbPool, url: &str) -> anyhow::Result<bool> {
        let inserted = sqlx::query(
            "INSERT INTO link_previews (url, status)
             VALUES ($1, $2)
             ON CONFLICT (url) DO NOTHING",
        )
        .bind(url)
        .bind(LINK_PREVIEW_PENDING)
        .execute(pool.as_ref())
        .await
        .context("insert pending link preview")?
        .rows_affected();

        if inserted > 0 {
            return Ok(true);
        }

        let reset_failed = sqlx::query(
            "UPDATE link_previews
             SET status = $2,
                 error_text = NULL,
                 updated_at = NOW()
             WHERE url = $1 AND status = $3",
        )
        .bind(url)
        .bind(LINK_PREVIEW_PENDING)
        .bind(LINK_PREVIEW_FAILED)
        .execute(pool.as_ref())
        .await
        .context("reset failed link preview state")?
        .rows_affected();

        Ok(reset_failed > 0)
    }

    fn spawn_fetch(&self, pool: DbPool, url: String) {
        let client = self.client.clone();

        tokio::spawn(async move {
            let result =
                tokio::time::timeout(LINK_PREVIEW_TIMEOUT, fetch_and_parse(&client, &url)).await;

            match result {
                Ok(Ok(parsed)) => {
                    if let Err(err) = sqlx::query(
                        "UPDATE link_previews
                         SET canonical_url = $2,
                             title = $3,
                             description = $4,
                             image_url = $5,
                             site_name = $6,
                             status = $7,
                             fetched_at = NOW(),
                             error_text = NULL,
                             updated_at = NOW()
                         WHERE url = $1",
                    )
                    .bind(&url)
                    .bind(parsed.canonical_url)
                    .bind(parsed.title)
                    .bind(parsed.description)
                    .bind(parsed.image_url)
                    .bind(parsed.site_name)
                    .bind(LINK_PREVIEW_READY)
                    .execute(pool.as_ref())
                    .await
                    {
                        tracing::error!(%err, target_url = %url, "failed to persist ready link preview");
                    }
                }
                Ok(Err(err)) => {
                    mark_failed(&pool, &url, &err.to_string()).await;
                }
                Err(_) => {
                    mark_failed(&pool, &url, "link preview fetch timed out after 5s").await;
                }
            }
        });
    }
}

pub fn normalize_url(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    let stripped = trimmed
        .trim_start_matches(['(', '[', '{', '"', '\''])
        .trim_end_matches(|ch: char| {
            matches!(
                ch,
                ')' | ']' | '}' | '.' | ',' | '!' | '?' | ':' | ';' | '"' | '\''
            )
        });

    let mut parsed = Url::parse(stripped).ok()?;
    if parsed.scheme() != "http" && parsed.scheme() != "https" {
        return None;
    }

    parsed.set_fragment(None);
    Some(parsed.to_string())
}

async fn fetch_and_parse(client: &Client, url: &str) -> anyhow::Result<ParsedPreview> {
    let response = client
        .get(url)
        .send()
        .await
        .context("request link preview URL")?
        .error_for_status()
        .context("link preview source returned non-success status")?;

    if let Some(content_type) = response.headers().get(CONTENT_TYPE) {
        let as_str = content_type
            .to_str()
            .unwrap_or_default()
            .to_ascii_lowercase();
        if !as_str.contains("text/html") {
            return Err(anyhow!("link preview source is not HTML"));
        }
    }

    let final_url = response.url().to_string();
    let body = response.text().await.context("read link preview html")?;
    let mut parsed = parse_open_graph(&body);
    if parsed.canonical_url.is_none() {
        parsed.canonical_url = Some(final_url);
    }

    if parsed.has_renderable_content() {
        Ok(parsed)
    } else {
        Err(anyhow!("link preview metadata not found"))
    }
}

fn parse_open_graph(html: &str) -> ParsedPreview {
    let document = Html::parse_document(html);
    let mut parsed = ParsedPreview::default();

    if let Ok(meta_selector) = Selector::parse("meta") {
        for node in document.select(&meta_selector) {
            let content = match node.value().attr("content") {
                Some(value) if !value.trim().is_empty() => value.trim().to_owned(),
                _ => continue,
            };

            let property = node
                .value()
                .attr("property")
                .map(|value| value.trim().to_ascii_lowercase());
            let name = node
                .value()
                .attr("name")
                .map(|value| value.trim().to_ascii_lowercase());

            match property.as_deref().or(name.as_deref()) {
                Some("og:title") => parsed.title = Some(content),
                Some("og:description") => parsed.description = Some(content),
                Some("og:image") => parsed.image_url = Some(content),
                Some("og:site_name") => parsed.site_name = Some(content),
                Some("og:url") => parsed.canonical_url = Some(content),
                Some("description") if parsed.description.is_none() => {
                    parsed.description = Some(content)
                }
                _ => {}
            }
        }
    }

    if parsed.title.is_none() {
        if let Ok(title_selector) = Selector::parse("title") {
            if let Some(title_node) = document.select(&title_selector).next() {
                let title = title_node.text().collect::<String>().trim().to_owned();
                if !title.is_empty() {
                    parsed.title = Some(title);
                }
            }
        }
    }

    parsed
}

async fn mark_failed(pool: &DbPool, url: &str, reason: &str) {
    if let Err(err) = sqlx::query(
        "UPDATE link_previews
         SET status = $2,
             error_text = LEFT($3, 500),
             fetched_at = NOW(),
             updated_at = NOW()
         WHERE url = $1",
    )
    .bind(url)
    .bind(LINK_PREVIEW_FAILED)
    .bind(reason)
    .execute(pool.as_ref())
    .await
    {
        tracing::error!(%err, target_url = %url, "failed to persist link preview failure");
    }
}

#[cfg(test)]
mod tests {
    use super::{normalize_url, parse_open_graph, LinkPreviewService};

    #[test]
    fn extract_urls_normalizes_deduplicates_and_limits_to_three() {
        let content = "Check https://example.com/a. duplicate https://example.com/a and https://example.com/b?x=1#frag then https://example.com/c and https://example.com/d";
        let urls = LinkPreviewService::extract_urls(content);

        assert_eq!(urls.len(), 3);
        assert_eq!(urls[0], "https://example.com/a");
        assert_eq!(urls[1], "https://example.com/b?x=1");
        assert_eq!(urls[2], "https://example.com/c");
    }

    #[test]
    fn normalize_url_rejects_non_http_and_strips_wrappers() {
        assert_eq!(
            normalize_url("(https://example.com/path?x=1#section)."),
            Some("https://example.com/path?x=1".to_owned())
        );
        assert_eq!(normalize_url("ftp://example.com"), None);
    }

    #[test]
    fn parse_open_graph_prefers_og_and_falls_back_to_title() {
        let html = r#"
            <html>
              <head>
                <title>Fallback Title</title>
                <meta property='og:title' content='OG Title' />
                <meta property='og:description' content='OG Description' />
                <meta property='og:image' content='https://cdn.example.com/image.png' />
                <meta property='og:site_name' content='Paw Site' />
              </head>
            </html>
        "#;

        let parsed = parse_open_graph(html);
        assert_eq!(parsed.title.as_deref(), Some("OG Title"));
        assert_eq!(parsed.description.as_deref(), Some("OG Description"));
        assert_eq!(
            parsed.image_url.as_deref(),
            Some("https://cdn.example.com/image.png")
        );
        assert_eq!(parsed.site_name.as_deref(), Some("Paw Site"));
    }

    #[test]
    fn parse_open_graph_uses_title_when_og_missing() {
        let html = "<html><head><title>Only Title</title></head><body></body></html>";
        let parsed = parse_open_graph(html);

        assert_eq!(parsed.title.as_deref(), Some("Only Title"));
        assert_eq!(parsed.description, None);
    }
}
