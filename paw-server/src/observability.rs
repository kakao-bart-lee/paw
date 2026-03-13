use axum::{
    body::Body,
    http::{HeaderValue, Request},
    middleware::Next,
    response::Response,
};
use std::time::Instant;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct RequestId(pub String);

pub async fn request_id_middleware(mut request: Request<Body>, next: Next) -> Response {
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let request_id = request
        .headers()
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    request
        .extensions_mut()
        .insert(RequestId(request_id.clone()));

    let started_at = Instant::now();
    let mut response = next.run(request).await;
    let elapsed_ms = started_at.elapsed().as_millis() as u64;
    let status = response.status().as_u16();

    if let Ok(header_value) = HeaderValue::from_str(&request_id) {
        response.headers_mut().insert("x-request-id", header_value);
    }

    tracing::info!(
        request_id = %request_id,
        method = %method,
        path = %path,
        status = status,
        latency_ms = elapsed_ms,
        "request completed"
    );

    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{routing::get, Router};
    use tower::ServiceExt;

    async fn ok() -> &'static str {
        "ok"
    }

    #[tokio::test]
    async fn adds_request_id_header_when_missing() {
        let app = Router::new()
            .route("/ok", get(ok))
            .layer(axum::middleware::from_fn(request_id_middleware));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/ok")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        let request_id = response
            .headers()
            .get("x-request-id")
            .and_then(|value| value.to_str().ok())
            .unwrap_or("");
        assert!(!request_id.is_empty());
    }

    #[tokio::test]
    async fn preserves_request_id_header_when_provided() {
        let app = Router::new()
            .route("/ok", get(ok))
            .layer(axum::middleware::from_fn(request_id_middleware));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/ok")
                    .header("x-request-id", "req-test-123")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        let request_id = response
            .headers()
            .get("x-request-id")
            .and_then(|value| value.to_str().ok());
        assert_eq!(request_id, Some("req-test-123"));
    }
}
