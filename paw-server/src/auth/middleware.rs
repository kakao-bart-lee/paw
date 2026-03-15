use axum::{
    body::Body,
    extract::State,
    http::{header, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use crate::i18n::{error_response_with_request_id, lookup_user_preferred_locale, RequestLocale};
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

    let claims = match jwt::verify_token_with_revocation(
        token,
        &state.jwt_secret,
        Some(jwt::TOKEN_TYPE_ACCESS),
        &state.db,
    )
    .await
    {
        Ok(claims) => claims,
        Err(_) => return unauthorized("invalid_token", "Access token is invalid", &request),
    };

    request.extensions_mut().insert(UserId(claims.sub));
    request.extensions_mut().insert(DeviceId(claims.device_id));
    match lookup_user_preferred_locale(&state.db, claims.sub).await {
        Ok(Some(preferred_locale)) => {
            request
                .extensions_mut()
                .insert(RequestLocale(preferred_locale));
        }
        Ok(None) => {}
        Err(err) => {
            tracing::warn!(%err, user_id = %claims.sub, "failed to load preferred locale");
        }
    }

    let response_locale = request
        .extensions()
        .get::<RequestLocale>()
        .cloned()
        .unwrap_or_else(|| RequestLocale(state.default_locale.clone()));

    let mut response = next.run(request).await;
    response.extensions_mut().insert(response_locale);
    response
}

fn unauthorized(code: &str, message: &str, request: &Request<Body>) -> Response {
    let request_id = request
        .extensions()
        .get::<RequestId>()
        .map(|req_id| req_id.0.as_str())
        .unwrap_or("-");
    let locale = request
        .extensions()
        .get::<RequestLocale>()
        .map(|locale| locale.0.as_str())
        .unwrap_or("ko-KR");
    tracing::warn!(
        request_id = %request_id,
        path = %request.uri().path(),
        code = %code,
        "auth failed"
    );

    error_response_with_request_id(
        StatusCode::UNAUTHORIZED,
        code,
        locale,
        Some(request_id),
        message,
    )
    .into_response()
}
