use crate::admin::service as admin_service;
use crate::auth::{middleware::UserId, AppState};
use crate::i18n::{error_response, RequestLocale};
use crate::moderation::{models::*, service};
use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::Value;
use uuid::Uuid;

pub async fn create_report(
    State(state): State<AppState>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Extension(UserId(user_id)): Extension<UserId>,
    Json(payload): Json<CreateReportRequest>,
) -> Response {
    let valid_types = ["message", "user", "agent"];
    if !valid_types.contains(&payload.target_type.as_str()) {
        return error(
            StatusCode::BAD_REQUEST,
            "invalid_target_type",
            &locale,
            "target_type must be message, user, or agent",
        )
        .into_response();
    }

    if payload.reason.trim().is_empty() {
        return error(
            StatusCode::BAD_REQUEST,
            "invalid_reason",
            &locale,
            "Reason is required",
        )
        .into_response();
    }

    match service::create_report(
        &state.db,
        user_id,
        &payload.target_type,
        payload.target_id,
        payload.reason.trim(),
    )
    .await
    {
        Ok(report) => {
            let resp = CreateReportResponse {
                id: report.id,
                created_at: report.created_at,
            };
            (StatusCode::CREATED, Json(resp)).into_response()
        }
        Err(err) => {
            tracing::error!(%err, "failed to create report");
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "report_failed",
                &locale,
                "Could not submit report",
            )
            .into_response()
        }
    }
}

pub async fn block_user(
    State(state): State<AppState>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path(target_id): Path<Uuid>,
) -> Response {
    if user_id == target_id {
        return error(
            StatusCode::BAD_REQUEST,
            "cannot_block_self",
            &locale,
            "Cannot block yourself",
        )
        .into_response();
    }

    match service::block_user(&state.db, user_id, target_id).await {
        Ok(blocked) => Json(BlockResponse { blocked }).into_response(),
        Err(err) => {
            tracing::error!(%err, "failed to block user");
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "block_failed",
                &locale,
                "Could not block user",
            )
            .into_response()
        }
    }
}

pub async fn unblock_user(
    State(state): State<AppState>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path(target_id): Path<Uuid>,
) -> Response {
    match service::unblock_user(&state.db, user_id, target_id).await {
        Ok(unblocked) => Json(UnblockResponse { unblocked }).into_response(),
        Err(err) => {
            tracing::error!(%err, "failed to unblock user");
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "unblock_failed",
                &locale,
                "Could not unblock user",
            )
            .into_response()
        }
    }
}

pub async fn list_blocked_users(
    State(state): State<AppState>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Extension(UserId(user_id)): Extension<UserId>,
) -> Response {
    match service::list_blocked_users(&state.db, user_id).await {
        Ok(blocked_users) => Json(BlockedUsersResponse { blocked_users }).into_response(),
        Err(err) => {
            tracing::error!(%err, "failed to list blocked users");
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "block_list_failed",
                &locale,
                "Could not list blocked users",
            )
            .into_response()
        }
    }
}

pub async fn suspend_user(
    State(state): State<AppState>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Extension(UserId(admin_id)): Extension<UserId>,
    Path(target_id): Path<Uuid>,
    Json(payload): Json<SuspendUserRequest>,
) -> Response {
    match service::is_admin(&state.db, admin_id).await {
        Ok(true) => {}
        Ok(false) => {
            return error(
                StatusCode::FORBIDDEN,
                "forbidden",
                &locale,
                "Admin access required",
            )
            .into_response()
        }
        Err(err) => {
            tracing::error!(%err, "admin check failed");
            return error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "admin_check_failed",
                &locale,
                "Could not verify admin status",
            )
            .into_response();
        }
    }

    match service::suspend_user(
        &state.db,
        target_id,
        payload.suspended_until,
        payload.reason.as_deref(),
        admin_id,
    )
    .await
    {
        Ok(suspended) => {
            if suspended {
                if let Err(err) = admin_service::log_admin_action(
                    &state.db,
                    admin_id,
                    "user.suspend",
                    Some("user"),
                    Some(target_id),
                    Some(serde_json::json!({ "reason": payload.reason })),
                )
                .await
                {
                    tracing::warn!(%err, "failed to write audit log");
                }
            }
            Json(SuspendResponse { suspended }).into_response()
        }
        Err(err) => {
            tracing::error!(%err, "failed to suspend user");
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "suspend_failed",
                &locale,
                "Could not suspend user",
            )
            .into_response()
        }
    }
}

pub async fn unsuspend_user(
    State(state): State<AppState>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Extension(UserId(admin_id)): Extension<UserId>,
    Path(target_id): Path<Uuid>,
) -> Response {
    match service::is_admin(&state.db, admin_id).await {
        Ok(true) => {}
        Ok(false) => {
            return error(
                StatusCode::FORBIDDEN,
                "forbidden",
                &locale,
                "Admin access required",
            )
            .into_response()
        }
        Err(err) => {
            tracing::error!(%err, "admin check failed");
            return error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "admin_check_failed",
                &locale,
                "Could not verify admin status",
            )
            .into_response();
        }
    }

    match service::unsuspend_user(&state.db, target_id).await {
        Ok(unsuspended) => {
            if unsuspended {
                if let Err(err) = admin_service::log_admin_action(
                    &state.db,
                    admin_id,
                    "user.unsuspend",
                    Some("user"),
                    Some(target_id),
                    None,
                )
                .await
                {
                    tracing::warn!(%err, "failed to write audit log");
                }
            }
            Json(UnsuspendResponse { unsuspended }).into_response()
        }
        Err(err) => {
            tracing::error!(%err, "failed to unsuspend user");
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "unsuspend_failed",
                &locale,
                "Could not unsuspend user",
            )
            .into_response()
        }
    }
}

pub async fn list_pending_reports(
    State(state): State<AppState>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Extension(UserId(admin_id)): Extension<UserId>,
) -> Response {
    match service::is_admin(&state.db, admin_id).await {
        Ok(true) => {}
        Ok(false) => {
            return error(
                StatusCode::FORBIDDEN,
                "forbidden",
                &locale,
                "Admin access required",
            )
            .into_response()
        }
        Err(err) => {
            tracing::error!(%err, "admin check failed");
            return error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "admin_check_failed",
                &locale,
                "Could not verify admin status",
            )
            .into_response();
        }
    }

    match service::list_pending_reports(&state.db).await {
        Ok(reports) => Json(AdminReportsResponse { reports }).into_response(),
        Err(err) => {
            tracing::error!(%err, "failed to list reports");
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "reports_failed",
                &locale,
                "Could not list reports",
            )
            .into_response()
        }
    }
}

fn error(status: StatusCode, code: &str, locale: &str, message: &str) -> (StatusCode, Json<Value>) {
    error_response(status, code, locale, message)
}
