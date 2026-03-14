use crate::auth::jwt;
use crate::auth::AppState;
use crate::i18n::{error_response, RequestLocale};
use crate::ws::connection::handle_socket;
use axum::extract::{Extension, Query, State, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use std::collections::HashMap;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Query(params): Query<HashMap<String, String>>,
    State(state): State<AppState>,
) -> Response {
    let token = params.get("token").cloned().unwrap_or_default();
    let (user_id, device_id) = match validate_jwt(&token, &state.jwt_secret) {
        Ok(ids) => ids,
        Err(_) => {
            return error_response(
                StatusCode::UNAUTHORIZED,
                "invalid_token",
                &locale,
                "Access token is invalid",
            )
            .into_response()
        }
    };

    ws.on_upgrade(move |socket| handle_socket(socket, user_id, device_id, locale, state))
}

fn validate_jwt(token: &str, secret: &str) -> anyhow::Result<(uuid::Uuid, uuid::Uuid)> {
    let claims = jwt::verify_token(token, secret, Some(jwt::TOKEN_TYPE_ACCESS))
        .map_err(|message| anyhow::anyhow!(message))?;
    let device_id = claims
        .device_id
        .ok_or_else(|| anyhow::anyhow!("missing device_id in access token"))?;
    Ok((claims.sub, device_id))
}
