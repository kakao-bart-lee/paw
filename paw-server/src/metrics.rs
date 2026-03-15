use axum::{
    body::Body,
    extract::State,
    http::Request,
    middleware::Next,
    response::{IntoResponse, Response},
};
use metrics::{counter, gauge, histogram};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use std::time::Instant;

/// Initializes the Prometheus metrics recorder and returns the handle
/// used to render metrics at the /metrics endpoint.
///
/// Must be called exactly once, before any metrics are recorded.
pub fn init_metrics() -> PrometheusHandle {
    PrometheusBuilder::new()
        .install_recorder()
        .expect("failed to install Prometheus metrics recorder")
}

/// Axum handler that renders all recorded metrics in Prometheus text format.
pub async fn metrics_handler(State(handle): State<PrometheusHandle>) -> impl IntoResponse {
    handle.render()
}

/// Middleware that records per-request HTTP metrics.
///
/// Tracked metrics:
/// - `http_requests_total` (counter) with labels: method, path_pattern, status_code
/// - `http_request_duration_seconds` (histogram) with labels: method, path_pattern
pub async fn metrics_middleware(request: Request<Body>, next: Next) -> Response {
    let method = request.method().to_string();
    let raw_path = request.uri().path().to_string();
    let path_pattern = normalize_path(&raw_path);

    let started_at = Instant::now();
    let response = next.run(request).await;
    let elapsed = started_at.elapsed().as_secs_f64();
    let status_code = response.status().as_u16().to_string();

    let labels = [
        ("method", method.clone()),
        ("path_pattern", path_pattern.clone()),
        ("status_code", status_code),
    ];
    counter!("http_requests_total", &labels).increment(1);

    let duration_labels = [("method", method), ("path_pattern", path_pattern)];
    histogram!("http_request_duration_seconds", &duration_labels).record(elapsed);

    response
}

/// Records a WebSocket connection being established.
pub fn ws_connection_opened() {
    gauge!("active_ws_connections").increment(1.0);
}

/// Records a WebSocket connection being closed.
pub fn ws_connection_closed() {
    gauge!("active_ws_connections").decrement(1.0);
}

/// Records an agent gateway call outcome.
pub fn record_agent_gateway_call(success: bool) {
    let status = if success { "success" } else { "error" };
    counter!("paw_agent_gateway_calls_total", "status" => status).increment(1);
}

/// Records a message being sent.
pub fn record_message_sent() {
    counter!("paw_messages_sent_total").increment(1);
}

/// Normalizes a request path to collapse UUID-like and numeric segments
/// into `{id}`, preventing high-cardinality label values.
///
/// Examples:
/// - `/conversations/550e8400-e29b-41d4-a716-446655440000/messages`
///   becomes `/conversations/{id}/messages`
/// - `/users/12345` becomes `/users/{id}`
fn normalize_path(path: &str) -> String {
    path.split('/')
        .map(|segment| {
            if looks_like_id(segment) {
                "{id}"
            } else {
                segment
            }
        })
        .collect::<Vec<_>>()
        .join("/")
}

/// Returns true if a path segment looks like a UUID or numeric ID.
fn looks_like_id(segment: &str) -> bool {
    if segment.is_empty() {
        return false;
    }

    // Numeric IDs
    if segment.chars().all(|c| c.is_ascii_digit()) && !segment.is_empty() {
        return true;
    }

    // UUID-like: 8-4-4-4-12 hex pattern, or hex string >= 16 chars
    if segment.len() >= 16 && segment.chars().all(|c| c.is_ascii_hexdigit() || c == '-') {
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{routing::get, Router};
    use tower::ServiceExt;

    #[test]
    fn normalize_path_replaces_uuids() {
        assert_eq!(
            normalize_path("/conversations/550e8400-e29b-41d4-a716-446655440000/messages"),
            "/conversations/{id}/messages"
        );
    }

    #[test]
    fn normalize_path_replaces_numeric_ids() {
        assert_eq!(normalize_path("/users/12345"), "/users/{id}");
    }

    #[test]
    fn normalize_path_preserves_named_segments() {
        assert_eq!(normalize_path("/api/v1/channels"), "/api/v1/channels");
    }

    #[test]
    fn normalize_path_preserves_short_hex() {
        // Short hex strings that are not IDs (e.g. "v1", "abc")
        assert_eq!(normalize_path("/api/v1/abc"), "/api/v1/abc");
    }

    #[test]
    fn normalize_path_handles_root() {
        assert_eq!(normalize_path("/"), "/");
    }

    #[test]
    fn looks_like_id_detects_uuid() {
        assert!(looks_like_id("550e8400-e29b-41d4-a716-446655440000"));
    }

    #[test]
    fn looks_like_id_detects_numeric() {
        assert!(looks_like_id("12345"));
    }

    #[test]
    fn looks_like_id_rejects_words() {
        assert!(!looks_like_id("conversations"));
        assert!(!looks_like_id("api"));
        assert!(!looks_like_id("v1"));
    }

    #[test]
    fn looks_like_id_rejects_empty() {
        assert!(!looks_like_id(""));
    }

    #[tokio::test]
    async fn metrics_endpoint_returns_200() {
        let handle = PrometheusBuilder::new().build_recorder().handle();

        let app = Router::new()
            .route("/metrics", get(metrics_handler))
            .with_state(handle);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/metrics")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), 200);
    }

    #[tokio::test]
    async fn metrics_endpoint_returns_prometheus_format() {
        let recorder = PrometheusBuilder::new().build_recorder();
        let handle = recorder.handle();

        // Record a metric directly via the handle's internal state
        // by using the metrics macros after installing the recorder
        metrics::with_local_recorder(&recorder, || {
            counter!("http_requests_total", "method" => "GET", "path_pattern" => "/health", "status_code" => "200").increment(1);
        });

        let app = Router::new()
            .route("/metrics", get(metrics_handler))
            .with_state(handle);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/metrics")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("read body");
        let text = String::from_utf8(body.to_vec()).expect("utf8");

        assert!(
            text.contains("http_requests_total"),
            "expected prometheus output to contain http_requests_total, got: {text}"
        );
    }
}
