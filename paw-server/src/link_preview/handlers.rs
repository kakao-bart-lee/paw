use crate::auth::{middleware::UserId, AppState};
use crate::i18n::{error_response, RequestLocale};
use crate::link_preview;
use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct TriggerMessagePreviewResponse {
    pub queued: bool,
}

#[derive(Debug, Serialize)]
pub struct GetMessagePreviewResponse {
    pub previews: Vec<link_preview::LinkPreviewRecord>,
}

pub async fn trigger_message_preview(
    State(state): State<AppState>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path(message_id): Path<Uuid>,
) -> Response {
    let message = match link_preview::find_message_for_user(&state.db, message_id, user_id).await {
        Ok(Some(row)) => row,
        Ok(None) => {
            return error(
                StatusCode::NOT_FOUND,
                "not_found",
                &locale,
                "Message not found",
            )
            .into_response();
        }
        Err(err) => {
            tracing::error!(%err, message_id = %message_id, user_id = %user_id, "failed to load message for link preview trigger");
            return error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "preview_trigger_failed",
                &locale,
                "Could not queue message preview",
            )
            .into_response();
        }
    };

    let preview_state = state.clone();
    tokio::spawn(async move {
        if let Err(err) = preview_state
            .link_preview_service
            .generate_and_store_for_message(&preview_state.db, message.message_id, &message.content)
            .await
        {
            tracing::warn!(%err, message_id = %message.message_id, "manual link preview generation failed");
        }
    });

    (
        StatusCode::ACCEPTED,
        Json(TriggerMessagePreviewResponse { queued: true }),
    )
        .into_response()
}

pub async fn get_message_preview(
    State(state): State<AppState>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path(message_id): Path<Uuid>,
) -> Response {
    match link_preview::find_message_for_user(&state.db, message_id, user_id).await {
        Ok(Some(_)) => {}
        Ok(None) => {
            return error(
                StatusCode::NOT_FOUND,
                "not_found",
                &locale,
                "Message not found",
            )
            .into_response();
        }
        Err(err) => {
            tracing::error!(%err, message_id = %message_id, user_id = %user_id, "failed to load message for link preview read");
            return error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "preview_fetch_failed",
                &locale,
                "Could not load message preview",
            )
            .into_response();
        }
    }

    let previews = match link_preview::list_cached_previews(&state.db, message_id).await {
        Ok(rows) => rows,
        Err(err) => {
            tracing::error!(%err, message_id = %message_id, "failed to query link previews");
            return error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "preview_fetch_failed",
                &locale,
                "Could not load message preview",
            )
            .into_response();
        }
    };

    Json(GetMessagePreviewResponse { previews }).into_response()
}

fn error(status: StatusCode, code: &str, locale: &str, message: &str) -> (StatusCode, Json<Value>) {
    error_response(status, code, locale, message)
}
