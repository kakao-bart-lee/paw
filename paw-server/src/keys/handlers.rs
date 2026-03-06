use crate::auth::{AppState, middleware::{DeviceId, UserId}};
use crate::keys::{models::UploadKeysRequest, service::{self, KeysError}};
use axum::{
    Json,
    extract::{Extension, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::{Value, json};
use uuid::Uuid;

pub async fn upload_keys_handler(
    State(state): State<AppState>,
    Extension(UserId(user_id)): Extension<UserId>,
    Extension(DeviceId(device_id)): Extension<DeviceId>,
    Json(payload): Json<UploadKeysRequest>,
) -> Response {
    match service::upload_keys(&state.db, user_id, device_id, payload).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(KeysError::InvalidBase64) => error(
            StatusCode::BAD_REQUEST,
            "invalid_base64",
            "One or more keys are not valid base64",
        )
        .into_response(),
        Err(KeysError::BundleNotFound) => error(
            StatusCode::NOT_FOUND,
            "bundle_not_found",
            "Prekey bundle not found",
        )
        .into_response(),
        Err(KeysError::Internal(err)) => {
            tracing::error!(%err, user_id = %user_id, "failed to upload prekey bundle");
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "keys_upload_failed",
                "Could not upload key bundle",
            )
            .into_response()
        }
    }
}

pub async fn get_key_bundle_handler(
    State(state): State<AppState>,
    Path(target_user_id): Path<Uuid>,
) -> Response {
    match service::get_key_bundle(&state.db, target_user_id).await {
        Ok(bundle) => Json(bundle).into_response(),
        Err(KeysError::BundleNotFound) => {
            error(StatusCode::NOT_FOUND, "bundle_not_found", "Prekey bundle not found")
                .into_response()
        }
        Err(KeysError::InvalidBase64) => error(
            StatusCode::BAD_REQUEST,
            "invalid_base64",
            "Stored key format is invalid",
        )
        .into_response(),
        Err(KeysError::Internal(err)) => {
            tracing::error!(%err, target_user_id = %target_user_id, "failed to get prekey bundle");
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "keys_fetch_failed",
                "Could not fetch key bundle",
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
