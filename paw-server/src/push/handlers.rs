use crate::auth::{
    middleware::{DeviceId, UserId},
    AppState,
};
use crate::push::{models, service};
use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::{json, Value};
use uuid::Uuid;

pub async fn register_push_token(
    State(state): State<AppState>,
    Extension(UserId(user_id)): Extension<UserId>,
    Extension(DeviceId(device_id)): Extension<DeviceId>,
    Json(payload): Json<models::RegisterPushTokenRequest>,
) -> Response {
    let Some(device_id) = device_id else {
        return error(
            StatusCode::BAD_REQUEST,
            "missing_device_id",
            "Access token must contain a device_id",
        )
        .into_response();
    };

    if payload.token.trim().is_empty() {
        return error(
            StatusCode::BAD_REQUEST,
            "invalid_token",
            "Push token must not be empty",
        )
        .into_response();
    }

    match service::register_push_token(
        &state.db,
        user_id,
        device_id,
        &payload.platform,
        payload.token.trim(),
    )
    .await
    {
        Ok(()) => Json(models::RegisterPushTokenResponse { registered: true }).into_response(),
        Err(err) => {
            tracing::error!(%err, %user_id, %device_id, "failed to register push token");
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "push_register_failed",
                "Could not register push token",
            )
            .into_response()
        }
    }
}

pub async fn unregister_push_token(
    State(state): State<AppState>,
    Extension(UserId(user_id)): Extension<UserId>,
    Extension(DeviceId(device_id)): Extension<DeviceId>,
) -> Response {
    let Some(device_id) = device_id else {
        return error(
            StatusCode::BAD_REQUEST,
            "missing_device_id",
            "Access token must contain a device_id",
        )
        .into_response();
    };

    match service::unregister_push_token(&state.db, device_id).await {
        Ok(removed) => Json(models::UnregisterPushTokenResponse {
            unregistered: removed,
        })
        .into_response(),
        Err(err) => {
            tracing::error!(%err, %user_id, %device_id, "failed to unregister push token");
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "push_unregister_failed",
                "Could not unregister push token",
            )
            .into_response()
        }
    }
}

pub async fn mute_conversation(
    State(state): State<AppState>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path(conversation_id): Path<Uuid>,
    Json(payload): Json<models::MuteConversationRequest>,
) -> Response {
    let muted_until = if payload.forever.unwrap_or(false) {
        None
    } else if let Some(minutes) = payload.duration_minutes {
        if minutes <= 0 {
            return error(
                StatusCode::BAD_REQUEST,
                "invalid_duration",
                "duration_minutes must be positive",
            )
            .into_response();
        }
        Some(chrono::Utc::now() + chrono::Duration::minutes(minutes))
    } else {
        return error(
            StatusCode::BAD_REQUEST,
            "invalid_mute_request",
            "Must specify duration_minutes or forever",
        )
        .into_response();
    };

    match service::mute_conversation(&state.db, user_id, conversation_id, muted_until).await {
        Ok(()) => Json(models::MuteConversationResponse {
            muted: true,
            muted_until,
        })
        .into_response(),
        Err(err) => {
            tracing::error!(%err, %user_id, %conversation_id, "failed to mute conversation");
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "mute_failed",
                "Could not mute conversation",
            )
            .into_response()
        }
    }
}

pub async fn unmute_conversation(
    State(state): State<AppState>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path(conversation_id): Path<Uuid>,
) -> Response {
    match service::unmute_conversation(&state.db, user_id, conversation_id).await {
        Ok(removed) => {
            Json(models::UnmuteConversationResponse { unmuted: removed }).into_response()
        }
        Err(err) => {
            tracing::error!(%err, %user_id, %conversation_id, "failed to unmute conversation");
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "unmute_failed",
                "Could not unmute conversation",
            )
            .into_response()
        }
    }
}

fn error(status: StatusCode, code: &str, message: &str) -> (StatusCode, Json<Value>) {
    (
        status,
        Json(json!({
            "error": code,
            "message": message,
        })),
    )
}
