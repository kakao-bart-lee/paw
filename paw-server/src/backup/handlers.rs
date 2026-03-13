use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::{json, Value};
use uuid::Uuid;

use super::models::{BackupSettings, ListBackupsResponse, RestoreBackupResponse};
use super::service;
use crate::auth::middleware::UserId;
use crate::auth::AppState;

fn error(status: StatusCode, code: &str, message: &str) -> (StatusCode, Json<Value>) {
    (
        status,
        Json(json!({
            "error": code,
            "message": message,
        })),
    )
}

pub async fn initiate_backup(
    State(state): State<AppState>,
    Extension(UserId(user_id)): Extension<UserId>,
) -> Response {
    match service::initiate_backup(&state.db, &state.media_service, user_id).await {
        Ok(resp) => (StatusCode::CREATED, Json(resp)).into_response(),
        Err(err) => {
            tracing::error!(%err, %user_id, "backup initiate failed");
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "backup_initiate_failed",
                "Could not initiate backup",
            )
            .into_response()
        }
    }
}

pub async fn list_backups(
    State(state): State<AppState>,
    Extension(UserId(user_id)): Extension<UserId>,
) -> Response {
    match service::list_backups(&state.db, user_id).await {
        Ok(backups) => Json(ListBackupsResponse { backups }).into_response(),
        Err(err) => {
            tracing::error!(%err, %user_id, "backup list failed");
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "backup_list_failed",
                "Could not list backups",
            )
            .into_response()
        }
    }
}

pub async fn restore_backup(
    State(state): State<AppState>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path(backup_id): Path<Uuid>,
) -> Response {
    match service::restore_backup(&state.db, &state.media_service, user_id, backup_id).await {
        Ok(Some(download_url)) => Json(RestoreBackupResponse { download_url }).into_response(),
        Ok(None) => error(
            StatusCode::NOT_FOUND,
            "backup_not_found",
            "Backup not found",
        )
        .into_response(),
        Err(err) => {
            tracing::error!(%err, %user_id, %backup_id, "backup restore failed");
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "backup_restore_failed",
                "Could not restore backup",
            )
            .into_response()
        }
    }
}

pub async fn delete_backup(
    State(state): State<AppState>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path(backup_id): Path<Uuid>,
) -> Response {
    match service::delete_backup(&state.db, &state.media_service, user_id, backup_id).await {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => error(
            StatusCode::NOT_FOUND,
            "backup_not_found",
            "Backup not found",
        )
        .into_response(),
        Err(err) => {
            tracing::error!(%err, %user_id, %backup_id, "backup delete failed");
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "backup_delete_failed",
                "Could not delete backup",
            )
            .into_response()
        }
    }
}

pub async fn get_settings(
    State(state): State<AppState>,
    Extension(UserId(user_id)): Extension<UserId>,
) -> Response {
    match service::get_backup_settings(&state.db, user_id).await {
        Ok(settings) => Json(settings).into_response(),
        Err(err) => {
            tracing::error!(%err, %user_id, "backup settings get failed");
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "backup_settings_failed",
                "Could not get backup settings",
            )
            .into_response()
        }
    }
}

pub async fn update_settings(
    State(state): State<AppState>,
    Extension(UserId(user_id)): Extension<UserId>,
    Json(settings): Json<BackupSettings>,
) -> Response {
    match service::update_backup_settings(&state.db, user_id, &settings).await {
        Ok(()) => Json(settings).into_response(),
        Err(err) => {
            tracing::error!(%err, %user_id, "backup settings update failed");
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "backup_settings_update_failed",
                "Could not update backup settings",
            )
            .into_response()
        }
    }
}
