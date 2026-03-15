use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::{
    body::Body,
    extract::ConnectInfo,
    http::{header, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use dashmap::DashMap;
use serde::Serialize;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RateLimitDecision {
    pub allowed: bool,
    pub retry_after_secs: u64,
}

impl RateLimiter {
    pub fn new(max_requests: u64, window: Duration) -> Self {
        Self {
            entries: Arc::new(DashMap::new()),
            max_requests,
            window,
        }
    }

    pub fn check(&self, key: &str) -> RateLimitDecision {
        let now = Instant::now();
        let mut entry = self.entries.entry(key.to_owned()).or_insert_with(|| Entry {
            count: 0,
            window_start: now,
        });

        let elapsed = now.duration_since(entry.window_start);
        if elapsed >= self.window {
            entry.count = 1;
            entry.window_start = now;
            return RateLimitDecision {
                allowed: true,
                retry_after_secs: 0,
            };
        }

        if entry.count < self.max_requests {
            entry.count += 1;
            return RateLimitDecision {
                allowed: true,
                retry_after_secs: 0,
            };
        }

        let remaining = self.window.saturating_sub(elapsed);
        let retry_after_secs = remaining.as_secs() + u64::from(remaining.subsec_nanos() > 0);

        RateLimitDecision {
            allowed: false,
            retry_after_secs,
        }
    }

    pub fn cleanup(&self) {
        let now = Instant::now();
        self.entries
            .retain(|_, entry| now.duration_since(entry.window_start) < self.window);
    }
}

#[derive(Clone)]
pub struct RateLimitLayer(pub RateLimiter);

pub fn spawn_cleanup_task(limiter: RateLimiter) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            limiter.cleanup();
        }
    });
}

fn extract_client_ip(request: &Request<Body>) -> String {
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

#[derive(Serialize)]
struct RateLimitedBody {
    code: &'static str,
    message: &'static str,
    retry_after: u64,
}

fn too_many_requests(retry_after: u64) -> Response {
    let mut response = (
        StatusCode::TOO_MANY_REQUESTS,
        Json(RateLimitedBody {
            code: "rate_limited",
            message: "Too many requests",
            retry_after,
        }),
    )
        .into_response();

    if let Ok(value) = retry_after.to_string().parse() {
        response.headers_mut().insert(header::RETRY_AFTER, value);
    }

    response
}

pub async fn rate_limit_middleware(request: Request<Body>, next: Next) -> Response {
    let limiter = request
        .extensions()
        .get::<RateLimitLayer>()
        .expect("RateLimitLayer extension missing")
        .0
        .clone();

    let ip = extract_client_ip(&request);
    let decision = limiter.check(&ip);

    if !decision.allowed {
        tracing::warn!(ip = %ip, retry_after = decision.retry_after_secs, "rate limit exceeded");
        return too_many_requests(decision.retry_after_secs);
    }

    next.run(request).await
}

pub fn limiter_from_env() -> RateLimiter {
    let max = std::env::var("RATE_LIMIT_RPM")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(100);
    RateLimiter::new(max, Duration::from_secs(60))
}

pub fn agent_limiter_from_env() -> RateLimiter {
    let max: u64 = std::env::var("RATE_LIMIT_AGENT")
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
            .layer(middleware::from_fn(rate_limit_middleware))
            .layer(Extension(RateLimitLayer(limiter)))
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
    async fn rejects_requests_exceeding_limit_with_expected_payload() {
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
        assert_eq!(
            resp.headers()
                .get(header::RETRY_AFTER)
                .and_then(|v| v.to_str().ok()),
            Some("60")
        );

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["code"], "rate_limited");
        assert_eq!(json["message"], "Too many requests");
        assert_eq!(json["retry_after"], 60);
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
    async fn cleanup_removes_expired_entries() {
        let limiter = RateLimiter::new(10, Duration::from_millis(10));

        assert!(limiter.check("a").allowed);
        assert!(limiter.check("b").allowed);
        assert_eq!(limiter.entries.len(), 2);

        tokio::time::sleep(Duration::from_millis(20)).await;
        limiter.cleanup();

        assert_eq!(limiter.entries.len(), 0);
    }

    #[test]
    fn per_agent_limits_are_isolated() {
        let limiter = RateLimiter::new(1, Duration::from_secs(60));
        assert!(limiter.check("agent:one"));
        assert!(!limiter.check("agent:one"));

        assert!(limiter.check("agent:two"));
        assert!(!limiter.check("agent:two"));
    }

    #[test]
    fn per_agent_default_limit_is_sixty_per_minute() {
        let limiter = RateLimiter::new(60, Duration::from_secs(60));
        for _ in 0..60 {
            assert!(limiter.check("agent:test"));
        }
        assert!(!limiter.check("agent:test"));
    }
}
