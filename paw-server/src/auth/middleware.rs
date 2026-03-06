use axum::{
    Json,
    body::Body,
    extract::State,
    http::{Request, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde_json::json;
use uuid::Uuid;

use super::{AppState, jwt};

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
        return unauthorized("missing_authorization", "Authorization header is required");
    };

    let Ok(auth_str) = auth_header.to_str() else {
        return unauthorized("invalid_authorization", "Authorization header is invalid");
    };

    let Some(token) = auth_str.strip_prefix("Bearer ") else {
        return unauthorized("invalid_authorization", "Expected Bearer token");
    };

    let claims = match jwt::verify_token(token, &state.jwt_secret, Some(jwt::TOKEN_TYPE_ACCESS)) {
        Ok(claims) => claims,
        Err(_) => return unauthorized("invalid_token", "Access token is invalid"),
    };

    request.extensions_mut().insert(UserId(claims.sub));
    request.extensions_mut().insert(DeviceId(claims.device_id));

    next.run(request).await
}

fn unauthorized(code: &str, message: &str) -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(json!({
            "error": code,
            "message": message,
        })),
    )
        .into_response()
}
