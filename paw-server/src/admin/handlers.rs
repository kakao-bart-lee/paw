use crate::admin::{models::*, service};
use crate::auth::{middleware::UserId, AppState};
use crate::i18n::{error_response, RequestLocale};
use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::Value;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn error(status: StatusCode, code: &str, locale: &str, message: &str) -> (StatusCode, Json<Value>) {
    error_response(status, code, locale, message)
}

/// Guard that checks role and returns early on failure.
macro_rules! require_role {
    ($state:expr, $admin_id:expr, $role:expr, $locale:expr) => {
        match service::require_role(&$state.db, $admin_id, $role).await {
            Ok(true) => {}
            Ok(false) => {
                return error(
                    StatusCode::FORBIDDEN,
                    "forbidden",
                    &$locale,
                    "Admin access required",
                )
                .into_response()
            }
            Err(err) => {
                tracing::error!(%err, "admin role check failed");
                return error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "admin_check_failed",
                    &$locale,
                    "Could not verify admin status",
                )
                .into_response();
            }
        }
    };
}

// ---------------------------------------------------------------------------
// User management
// ---------------------------------------------------------------------------

pub async fn list_users(
    State(state): State<AppState>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Extension(UserId(admin_id)): Extension<UserId>,
    Query(query): Query<AdminUserListQuery>,
) -> Response {
    require_role!(state, admin_id, AdminRole::Moderator, locale);

    match service::list_users(&state.db, query.page, query.limit, query.search.as_deref()).await {
        Ok((users, total)) => Json(AdminUserListResponse {
            users,
            total,
            page: query.page,
            limit: query.limit,
        })
        .into_response(),
        Err(err) => {
            tracing::error!(%err, "failed to list users");
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "query_failed",
                &locale,
                "Could not list users",
            )
            .into_response()
        }
    }
}

pub async fn get_user_detail(
    State(state): State<AppState>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Extension(UserId(admin_id)): Extension<UserId>,
    Path(target_id): Path<Uuid>,
) -> Response {
    require_role!(state, admin_id, AdminRole::Moderator, locale);

    let user = match service::get_user_detail(&state.db, target_id).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            return error(
                StatusCode::NOT_FOUND,
                "user_not_found",
                &locale,
                "User not found",
            )
            .into_response()
        }
        Err(err) => {
            tracing::error!(%err, "failed to get user detail");
            return error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "query_failed",
                &locale,
                "Could not get user",
            )
            .into_response();
        }
    };

    let devices = match service::get_user_devices(&state.db, target_id).await {
        Ok(d) => d,
        Err(err) => {
            tracing::error!(%err, "failed to get user devices");
            vec![]
        }
    };

    Json(AdminUserDetailResponse {
        id: user.id,
        phone: user.phone,
        username: user.username,
        display_name: user.display_name,
        role: user.role,
        created_at: user.created_at,
        devices,
    })
    .into_response()
}

// ---------------------------------------------------------------------------
// Agent management
// ---------------------------------------------------------------------------

pub async fn list_pending_agents(
    State(state): State<AppState>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Extension(UserId(admin_id)): Extension<UserId>,
) -> Response {
    require_role!(state, admin_id, AdminRole::Moderator, locale);

    match service::list_pending_agents(&state.db).await {
        Ok(agents) => Json(PendingAgentsResponse { agents }).into_response(),
        Err(err) => {
            tracing::error!(%err, "failed to list pending agents");
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "query_failed",
                &locale,
                "Could not list pending agents",
            )
            .into_response()
        }
    }
}

pub async fn approve_agent(
    State(state): State<AppState>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Extension(UserId(admin_id)): Extension<UserId>,
    Path(agent_id): Path<Uuid>,
) -> Response {
    require_role!(state, admin_id, AdminRole::Admin, locale);

    match service::approve_agent(&state.db, agent_id).await {
        Ok(true) => {
            if let Err(err) = service::log_admin_action(
                &state.db,
                admin_id,
                "agent.approve",
                Some("agent"),
                Some(agent_id),
                None,
            )
            .await
            {
                tracing::warn!(%err, "failed to write audit log");
            }
            Json(AgentActionResponse { success: true }).into_response()
        }
        Ok(false) => error(
            StatusCode::NOT_FOUND,
            "agent_not_found",
            &locale,
            "Agent not found or already public",
        )
        .into_response(),
        Err(err) => {
            tracing::error!(%err, "failed to approve agent");
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                &locale,
                "Could not approve agent",
            )
            .into_response()
        }
    }
}

pub async fn reject_agent(
    State(state): State<AppState>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Extension(UserId(admin_id)): Extension<UserId>,
    Path(agent_id): Path<Uuid>,
    Json(payload): Json<RejectAgentRequest>,
) -> Response {
    require_role!(state, admin_id, AdminRole::Admin, locale);

    match service::reject_agent(&state.db, agent_id).await {
        Ok(true) => {
            if let Err(err) = service::log_admin_action(
                &state.db,
                admin_id,
                "agent.reject",
                Some("agent"),
                Some(agent_id),
                Some(serde_json::json!({ "reason": payload.reason })),
            )
            .await
            {
                tracing::warn!(%err, "failed to write audit log");
            }
            Json(AgentActionResponse { success: true }).into_response()
        }
        Ok(false) => error(
            StatusCode::NOT_FOUND,
            "agent_not_found",
            &locale,
            "Agent not found",
        )
        .into_response(),
        Err(err) => {
            tracing::error!(%err, "failed to reject agent");
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                &locale,
                "Could not reject agent",
            )
            .into_response()
        }
    }
}

pub async fn deactivate_agent(
    State(state): State<AppState>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Extension(UserId(admin_id)): Extension<UserId>,
    Path(agent_id): Path<Uuid>,
) -> Response {
    require_role!(state, admin_id, AdminRole::Admin, locale);

    match service::deactivate_agent(&state.db, agent_id).await {
        Ok(true) => {
            if let Err(err) = service::log_admin_action(
                &state.db,
                admin_id,
                "agent.deactivate",
                Some("agent"),
                Some(agent_id),
                None,
            )
            .await
            {
                tracing::warn!(%err, "failed to write audit log");
            }
            Json(AgentActionResponse { success: true }).into_response()
        }
        Ok(false) => error(
            StatusCode::NOT_FOUND,
            "agent_not_found",
            &locale,
            "Agent not found",
        )
        .into_response(),
        Err(err) => {
            tracing::error!(%err, "failed to deactivate agent");
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                &locale,
                "Could not deactivate agent",
            )
            .into_response()
        }
    }
}

// ---------------------------------------------------------------------------
// Report resolution
// ---------------------------------------------------------------------------

pub async fn resolve_report(
    State(state): State<AppState>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Extension(UserId(admin_id)): Extension<UserId>,
    Path(report_id): Path<Uuid>,
    Json(payload): Json<ResolveReportRequest>,
) -> Response {
    require_role!(state, admin_id, AdminRole::Moderator, locale);

    let valid_actions = ["warn", "suspend", "delete", "dismiss"];
    if !valid_actions.contains(&payload.action.as_str()) {
        return error(
            StatusCode::BAD_REQUEST,
            "invalid_action",
            &locale,
            "Action must be warn, suspend, delete, or dismiss",
        )
        .into_response();
    }

    match service::resolve_report(
        &state.db,
        report_id,
        &payload.action,
        payload.reason.as_deref(),
    )
    .await
    {
        Ok(true) => {
            if let Err(err) = service::log_admin_action(
                &state.db,
                admin_id,
                "report.resolve",
                Some("report"),
                Some(report_id),
                Some(serde_json::json!({
                    "action": payload.action,
                    "reason": payload.reason,
                })),
            )
            .await
            {
                tracing::warn!(%err, "failed to write audit log");
            }
            Json(ResolveReportResponse { resolved: true }).into_response()
        }
        Ok(false) => error(
            StatusCode::NOT_FOUND,
            "not_found",
            &locale,
            "Report not found or already resolved",
        )
        .into_response(),
        Err(err) => {
            tracing::error!(%err, "failed to resolve report");
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                &locale,
                "Could not resolve report",
            )
            .into_response()
        }
    }
}

// ---------------------------------------------------------------------------
// Spam patterns
// ---------------------------------------------------------------------------

pub async fn list_spam_patterns(
    State(state): State<AppState>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Extension(UserId(admin_id)): Extension<UserId>,
) -> Response {
    require_role!(state, admin_id, AdminRole::Moderator, locale);

    match service::list_spam_patterns(&state.db).await {
        Ok(patterns) => Json(SpamPatternsResponse { patterns }).into_response(),
        Err(err) => {
            tracing::error!(%err, "failed to list spam patterns");
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "query_failed",
                &locale,
                "Could not list spam patterns",
            )
            .into_response()
        }
    }
}

pub async fn create_spam_pattern(
    State(state): State<AppState>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Extension(UserId(admin_id)): Extension<UserId>,
    Json(payload): Json<CreateSpamPatternRequest>,
) -> Response {
    require_role!(state, admin_id, AdminRole::Admin, locale);

    if payload.keyword.trim().is_empty() {
        return error(
            StatusCode::BAD_REQUEST,
            "invalid_keyword",
            &locale,
            "Keyword is required",
        )
        .into_response();
    }

    match service::create_spam_pattern(&state.db, payload.keyword.trim()).await {
        Ok(resp) => {
            if let Err(err) = service::log_admin_action(
                &state.db,
                admin_id,
                "spam_pattern.create",
                Some("spam_keyword"),
                None,
                Some(serde_json::json!({ "keyword": resp.keyword })),
            )
            .await
            {
                tracing::warn!(%err, "failed to write audit log");
            }
            (StatusCode::CREATED, Json(resp)).into_response()
        }
        Err(err) => {
            tracing::error!(%err, "failed to create spam pattern");
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                &locale,
                "Could not create spam pattern",
            )
            .into_response()
        }
    }
}

pub async fn delete_spam_pattern(
    State(state): State<AppState>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Extension(UserId(admin_id)): Extension<UserId>,
    Path(pattern_id): Path<i32>,
) -> Response {
    require_role!(state, admin_id, AdminRole::Admin, locale);

    match service::delete_spam_pattern(&state.db, pattern_id).await {
        Ok(true) => {
            if let Err(err) = service::log_admin_action(
                &state.db,
                admin_id,
                "spam_pattern.delete",
                Some("spam_keyword"),
                None,
                Some(serde_json::json!({ "pattern_id": pattern_id })),
            )
            .await
            {
                tracing::warn!(%err, "failed to write audit log");
            }
            StatusCode::NO_CONTENT.into_response()
        }
        Ok(false) => error(
            StatusCode::NOT_FOUND,
            "not_found",
            &locale,
            "Spam pattern not found",
        )
        .into_response(),
        Err(err) => {
            tracing::error!(%err, "failed to delete spam pattern");
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                &locale,
                "Could not delete spam pattern",
            )
            .into_response()
        }
    }
}

// ---------------------------------------------------------------------------
// Dashboard metrics
// ---------------------------------------------------------------------------

pub async fn dashboard_metrics(
    State(state): State<AppState>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Extension(UserId(admin_id)): Extension<UserId>,
) -> Response {
    require_role!(state, admin_id, AdminRole::Moderator, locale);

    match service::get_dashboard_metrics(&state.db).await {
        Ok(mut metrics) => {
            metrics.ws_connections = state.hub.total_connections().await;
            Json(metrics).into_response()
        }
        Err(err) => {
            tracing::error!(%err, "failed to get dashboard metrics");
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "query_failed",
                &locale,
                "Could not get metrics",
            )
            .into_response()
        }
    }
}

// ---------------------------------------------------------------------------
// Audit logs
// ---------------------------------------------------------------------------

pub async fn list_audit_logs(
    State(state): State<AppState>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Extension(UserId(admin_id)): Extension<UserId>,
    Query(query): Query<AuditLogQuery>,
) -> Response {
    require_role!(state, admin_id, AdminRole::SuperAdmin, locale);

    match service::list_audit_logs(&state.db, &query).await {
        Ok((logs, total)) => Json(AuditLogsResponse {
            logs,
            total,
            page: query.page,
            limit: query.limit,
        })
        .into_response(),
        Err(err) => {
            tracing::error!(%err, "failed to list audit logs");
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "query_failed",
                &locale,
                "Could not list audit logs",
            )
            .into_response()
        }
    }
}
