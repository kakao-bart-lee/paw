use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::{
    body::Body,
    extract::ConnectInfo,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use dashmap::DashMap;
use serde_json::json;

use crate::auth::middleware::UserId;
use crate::i18n::RequestLocale;
use crate::observability::RequestId;

// ── Rate limiter ──────────────────────────────────────────────────────

#[derive(Clone)]
pub struct RateLimiter {
    entries: Arc<DashMap<String, Entry>>,
    max_requests: u64,
    window: Duration,
}

struct Entry {
    count: u64,
    window_start: Instant,
}

impl RateLimiter {
    pub fn new(max_requests: u64, window: Duration) -> Self {
        Self {
            entries: Arc::new(DashMap::new()),
            max_requests,
            window,
        }
    }

    /// Returns `true` when the request is allowed, `false` when the limit is exceeded.
    pub fn check(&self, key: &str) -> bool {
        let now = Instant::now();
        let mut entry = self.entries.entry(key.to_owned()).or_insert_with(|| Entry {
            count: 0,
            window_start: now,
        });

        if now.duration_since(entry.window_start) >= self.window {
            entry.count = 1;
            entry.window_start = now;
            return true;
        }

        if entry.count < self.max_requests {
            entry.count += 1;
            true
        } else {
            false
        }
    }

    /// Remove entries whose window has fully elapsed.
    pub fn cleanup(&self) {
        let now = Instant::now();
        self.entries
            .retain(|_key, entry| now.duration_since(entry.window_start) < self.window);
    }
}

// ── Cleanup task ──────────────────────────────────────────────────────

pub fn spawn_cleanup_task(limiters: Vec<RateLimiter>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            for limiter in &limiters {
                limiter.cleanup();
            }
        }
    });
}

// ── Middleware helpers ─────────────────────────────────────────────────

fn extract_client_ip(request: &Request<Body>) -> String {
    // Prefer X-Forwarded-For, then X-Real-IP, then peer address.
    if let Some(forwarded) = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
    {
        if let Some(first) = forwarded.split(',').next() {
            let ip = first.trim();
            if !ip.is_empty() {
                return ip.to_owned();
            }
        }
    }

    if let Some(real_ip) = request
        .headers()
        .get("x-real-ip")
        .and_then(|v| v.to_str().ok())
    {
        let ip = real_ip.trim();
        if !ip.is_empty() {
            return ip.to_owned();
        }
    }

    request
        .extensions()
        .get::<ConnectInfo<std::net::SocketAddr>>()
        .map(|ci| ci.0.ip().to_string())
        .unwrap_or_else(|| "unknown".to_owned())
}

fn too_many_requests(request: &Request<Body>) -> Response {
    let request_id = request.extensions().get::<RequestId>().map(|r| r.0.clone());
    let locale = request
        .extensions()
        .get::<RequestLocale>()
        .map(|l| l.0.as_str())
        .unwrap_or("ko-KR");

    let message = match locale {
        l if l.starts_with("en") => "Too many requests. Please try again later.",
        _ => "요청이 너무 많습니다. 잠시 후 다시 시도해주세요.",
    };

    let mut body = serde_json::Map::new();
    body.insert("error".to_owned(), json!("rate_limit_exceeded"));
    body.insert("message".to_owned(), json!(message));
    if let Some(rid) = request_id {
        body.insert("request_id".to_owned(), json!(rid));
    }

    (
        StatusCode::TOO_MANY_REQUESTS,
        Json(serde_json::Value::Object(body)),
    )
        .into_response()
}

// ── Public-route middleware (keyed by IP) ──────────────────────────────

pub async fn public_rate_limit(request: Request<Body>, next: Next) -> Response {
    let limiter = request
        .extensions()
        .get::<PublicLimiter>()
        .expect("PublicLimiter extension missing – add it via Extension layer")
        .0
        .clone();

    let ip = extract_client_ip(&request);

    if !limiter.check(&ip) {
        tracing::warn!(ip = %ip, "public rate limit exceeded");
        return too_many_requests(&request);
    }

    next.run(request).await
}

/// Newtype so the extension lookup is unambiguous.
#[derive(Clone)]
pub struct PublicLimiter(pub RateLimiter);

// ── Protected-route middleware (keyed by user ID) ─────────────────────

pub async fn protected_rate_limit(request: Request<Body>, next: Next) -> Response {
    let limiter = request
        .extensions()
        .get::<ProtectedLimiter>()
        .expect("ProtectedLimiter extension missing – add it via Extension layer")
        .0
        .clone();

    // After the auth middleware has run, UserId is in extensions.
    let key = request
        .extensions()
        .get::<UserId>()
        .map(|uid| uid.0.to_string())
        .unwrap_or_else(|| extract_client_ip(&request));

    if !limiter.check(&key) {
        tracing::warn!(key = %key, "protected rate limit exceeded");
        return too_many_requests(&request);
    }

    next.run(request).await
}

/// Newtype so the extension lookup is unambiguous.
#[derive(Clone)]
pub struct ProtectedLimiter(pub RateLimiter);

// ── Configuration helper ──────────────────────────────────────────────

pub fn public_limiter_from_env() -> RateLimiter {
    let max: u64 = std::env::var("RATE_LIMIT_PUBLIC")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(10);
    RateLimiter::new(max, Duration::from_secs(60))
}

pub fn protected_limiter_from_env() -> RateLimiter {
    let max: u64 = std::env::var("RATE_LIMIT_PROTECTED")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(60);
    RateLimiter::new(max, Duration::from_secs(60))
}

// ── Tests ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{middleware, routing::get, Extension, Router};
    use tower::ServiceExt;

    async fn ok_handler() -> &'static str {
        "ok"
    }

    fn test_app(limiter: RateLimiter) -> Router {
        Router::new()
            .route("/test", get(ok_handler))
            .layer(middleware::from_fn(public_rate_limit))
            .layer(Extension(PublicLimiter(limiter)))
    }

    #[tokio::test]
    async fn allows_requests_within_limit() {
        let limiter = RateLimiter::new(3, Duration::from_secs(60));
        let app = test_app(limiter);

        for _ in 0..3 {
            let resp = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri("/test")
                        .header("x-forwarded-for", "1.2.3.4")
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
        }
    }

    #[tokio::test]
    async fn rejects_requests_exceeding_limit() {
        let limiter = RateLimiter::new(2, Duration::from_secs(60));
        let app = test_app(limiter);

        for _ in 0..2 {
            let resp = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri("/test")
                        .header("x-forwarded-for", "5.6.7.8")
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
        }

        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/test")
                    .header("x-forwarded-for", "5.6.7.8")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["error"], "rate_limit_exceeded");
    }

    #[tokio::test]
    async fn different_ips_have_separate_limits() {
        let limiter = RateLimiter::new(1, Duration::from_secs(60));
        let app = test_app(limiter);

        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/test")
                    .header("x-forwarded-for", "10.0.0.1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/test")
                    .header("x-forwarded-for", "10.0.0.2")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn window_resets_after_expiry() {
        let limiter = RateLimiter::new(1, Duration::from_millis(50));
        let app = test_app(limiter);

        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/test")
                    .header("x-forwarded-for", "9.9.9.9")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // Exceeds limit.
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/test")
                    .header("x-forwarded-for", "9.9.9.9")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);

        // Wait for window to expire.
        tokio::time::sleep(Duration::from_millis(60)).await;

        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/test")
                    .header("x-forwarded-for", "9.9.9.9")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn cleanup_removes_expired_entries() {
        let limiter = RateLimiter::new(10, Duration::from_millis(10));

        assert!(limiter.check("a"));
        assert!(limiter.check("b"));
        assert_eq!(limiter.entries.len(), 2);

        tokio::time::sleep(Duration::from_millis(20)).await;
        limiter.cleanup();

        assert_eq!(limiter.entries.len(), 0);
    }
}
