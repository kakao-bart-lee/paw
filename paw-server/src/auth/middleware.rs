use axum::{
    body::Body,
    extract::State,
    http::{header, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use uuid::Uuid;

use crate::observability::RequestId;

use super::{jwt, AppState};

#[derive(Clone, Copy, Debug)]
pub struct UserId(pub Uuid);

#[derive(Clone, Copy, Debug)]
pub struct DeviceId(pub Option<Uuid>);

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request<Body>,
    next: Next,
) -> Response {
    let Some(auth_header) = request.headers().get(header::AUTHORIZATION) else {
        return unauthorized(
            "missing_authorization",
            "Authorization header is required",
            &request,
        );
    };

    let Ok(auth_str) = auth_header.to_str() else {
        return unauthorized(
            "invalid_authorization",
            "Authorization header is invalid",
            &request,
        );
    };

    let Some(token) = auth_str.strip_prefix("Bearer ") else {
        return unauthorized("invalid_authorization", "Expected Bearer token", &request);
    };

    let claims = match jwt::verify_token(token, &state.jwt_secret, Some(jwt::TOKEN_TYPE_ACCESS)) {
        Ok(claims) => claims,
        Err(_) => return unauthorized("invalid_token", "Access token is invalid", &request),
    };

    request.extensions_mut().insert(UserId(claims.sub));
    request.extensions_mut().insert(DeviceId(claims.device_id));

    next.run(request).await
}

fn unauthorized(code: &str, message: &str, request: &Request<Body>) -> Response {
    let request_id = request
        .extensions()
        .get::<RequestId>()
        .map(|req_id| req_id.0.as_str())
        .unwrap_or("-");
    tracing::warn!(
        request_id = %request_id,
        path = %request.uri().path(),
        code = %code,
        "auth failed"
    );

    (
        StatusCode::UNAUTHORIZED,
        Json(json!({
            "error": code,
            "message": message,
            "request_id": request_id,
        })),
    )
        .into_response()
}
