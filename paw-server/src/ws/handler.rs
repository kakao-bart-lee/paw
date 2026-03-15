use crate::auth::jwt;
use crate::auth::AppState;
use crate::i18n::{error_response, lookup_user_preferred_locale, RequestLocale};
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
    let (user_id, device_id) = match validate_jwt(&token, &state).await {
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

    let effective_locale = match lookup_user_preferred_locale(&state.db, user_id).await {
        Ok(Some(preferred_locale)) => preferred_locale,
        Ok(None) => locale,
        Err(err) => {
            tracing::warn!(%err, user_id = %user_id, "failed to load preferred locale for websocket");
            locale
        }
    };

    ws.on_upgrade(move |socket| handle_socket(socket, user_id, device_id, effective_locale, state))
}

async fn validate_jwt(token: &str, state: &AppState) -> anyhow::Result<(uuid::Uuid, uuid::Uuid)> {
    let claims = jwt::verify_token_with_revocation(
        token,
        &state.jwt_secret,
        Some(jwt::TOKEN_TYPE_ACCESS),
        &state.db,
    )
    .await
    .map_err(|message| anyhow::anyhow!(message))?;
    let device_id = claims
        .device_id
        .ok_or_else(|| anyhow::anyhow!("missing device_id in access token"))?;
    Ok((claims.sub, device_id))
}
