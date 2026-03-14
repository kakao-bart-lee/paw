use crate::auth::middleware::UserId;
use crate::auth::AppState;
use crate::devices::models::Device;
use crate::i18n::{error_response, RequestLocale};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use serde_json::{json, Value};
use uuid::Uuid;

fn error(status: StatusCode, code: &str, locale: &str, message: &str) -> (StatusCode, Json<Value>) {
    error_response(status, code, locale, message)
}

pub async fn list_devices(
    Extension(user_id): Extension<UserId>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    State(state): State<AppState>,
) -> (StatusCode, Json<Value>) {
    match sqlx::query_as::<_, Device>(
        "SELECT id, user_id, device_name, platform, last_active_at, created_at \
         FROM devices \
         WHERE user_id = $1 \
         ORDER BY created_at DESC",
    )
    .bind(user_id.0)
    .fetch_all(state.db.as_ref())
    .await
    {
        Ok(devices) => (
            StatusCode::OK,
            Json(json!({
                "devices": devices,
            })),
        ),
        Err(_) => error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "query_failed",
            &locale,
            "Failed to list devices",
        ),
    }
}

pub async fn delete_device(
    Extension(user_id): Extension<UserId>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    State(state): State<AppState>,
    Path(device_id): Path<Uuid>,
) -> (StatusCode, Json<Value>) {
    match sqlx::query("DELETE FROM devices WHERE id = $1 AND user_id = $2")
        .bind(device_id)
        .bind(user_id.0)
        .execute(state.db.as_ref())
        .await
    {
        Ok(result) if result.rows_affected() == 1 => {
            (StatusCode::OK, Json(json!({ "deleted": true })))
        }
        Ok(_) => error(
            StatusCode::NOT_FOUND,
            "device_not_found",
            &locale,
            "Device not found",
        ),
        Err(_) => error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "delete_failed",
            &locale,
            "Failed to delete device",
        ),
    }
}
