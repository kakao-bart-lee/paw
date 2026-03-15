use crate::admin::models::*;
use crate::db::DbPool;
use anyhow::Context;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// RBAC
// ---------------------------------------------------------------------------

/// Check whether `user_id` has at least `required` role level.
pub async fn require_role(
    db: &DbPool,
    user_id: Uuid,
    required: AdminRole,
) -> anyhow::Result<bool> {
    let role_str: Option<String> =
        sqlx::query_scalar("SELECT role FROM users WHERE id = $1 AND deleted_at IS NULL")
            .bind(user_id)
            .fetch_optional(db.as_ref())
            .await
            .context("check admin role")?;

    let Some(role_str) = role_str else {
        return Ok(false);
    };

    match AdminRole::from_str(&role_str) {
        Some(actual) if actual >= required => Ok(true),
        _ => Ok(false),
    }
}

// ---------------------------------------------------------------------------
// Audit log
// ---------------------------------------------------------------------------

pub async fn log_admin_action(
    db: &DbPool,
    actor_id: Uuid,
    action: &str,
    target_type: Option<&str>,
    target_id: Option<Uuid>,
    metadata: Option<serde_json::Value>,
) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO audit_logs (actor_id, action, target_type, target_id, metadata)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(actor_id)
    .bind(action)
    .bind(target_type)
    .bind(target_id)
    .bind(metadata)
    .execute(db.as_ref())
    .await
    .context("insert audit log")?;
    Ok(())
}

// ---------------------------------------------------------------------------
// User management
// ---------------------------------------------------------------------------

pub async fn list_users(
    db: &DbPool,
    page: i64,
    limit: i64,
    search: Option<&str>,
) -> anyhow::Result<(Vec<AdminUserRow>, i64)> {
    let offset = (page - 1).max(0) * limit;

    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM users
         WHERE deleted_at IS NULL
           AND ($1::TEXT IS NULL
                OR username ILIKE '%' || $1 || '%'
                OR display_name ILIKE '%' || $1 || '%'
                OR phone ILIKE '%' || $1 || '%')",
    )
    .bind(search)
    .fetch_one(db.as_ref())
    .await
    .context("count users")?;

    let rows = sqlx::query_as::<_, AdminUserRow>(
        "SELECT id, phone, username, display_name, role, created_at
         FROM users
         WHERE deleted_at IS NULL
           AND ($1::TEXT IS NULL
                OR username ILIKE '%' || $1 || '%'
                OR display_name ILIKE '%' || $1 || '%'
                OR phone ILIKE '%' || $1 || '%')
         ORDER BY created_at DESC
         LIMIT $2 OFFSET $3",
    )
    .bind(search)
    .bind(limit.min(100))
    .bind(offset)
    .fetch_all(db.as_ref())
    .await
    .context("list users")?;

    Ok((rows, total))
}

pub async fn get_user_detail(
    db: &DbPool,
    user_id: Uuid,
) -> anyhow::Result<Option<AdminUserRow>> {
    sqlx::query_as::<_, AdminUserRow>(
        "SELECT id, phone, username, display_name, role, created_at
         FROM users WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(user_id)
    .fetch_optional(db.as_ref())
    .await
    .context("get user detail")
}

pub async fn get_user_devices(
    db: &DbPool,
    user_id: Uuid,
) -> anyhow::Result<Vec<AdminUserDevice>> {
    sqlx::query_as::<_, AdminUserDevice>(
        "SELECT id, device_name, created_at FROM devices WHERE user_id = $1 ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(db.as_ref())
    .await
    .context("get user devices")
}

// ---------------------------------------------------------------------------
// Agent management
// ---------------------------------------------------------------------------

pub async fn list_pending_agents(db: &DbPool) -> anyhow::Result<Vec<PendingAgent>> {
    sqlx::query_as::<_, PendingAgent>(
        "SELECT id, name, description, owner_user_id, created_at
         FROM agent_tokens
         WHERE is_public = false AND revoked_at IS NULL AND manifest IS NOT NULL
         ORDER BY created_at ASC",
    )
    .fetch_all(db.as_ref())
    .await
    .context("list pending agents")
}

pub async fn approve_agent(db: &DbPool, agent_id: Uuid) -> anyhow::Result<bool> {
    let result = sqlx::query(
        "UPDATE agent_tokens SET is_public = true
         WHERE id = $1 AND revoked_at IS NULL AND is_public = false",
    )
    .bind(agent_id)
    .execute(db.as_ref())
    .await
    .context("approve agent")?;

    Ok(result.rows_affected() > 0)
}

pub async fn reject_agent(db: &DbPool, agent_id: Uuid) -> anyhow::Result<bool> {
    let result = sqlx::query(
        "UPDATE agent_tokens SET manifest = NULL
         WHERE id = $1 AND revoked_at IS NULL AND is_public = false",
    )
    .bind(agent_id)
    .execute(db.as_ref())
    .await
    .context("reject agent")?;

    Ok(result.rows_affected() > 0)
}

pub async fn deactivate_agent(db: &DbPool, agent_id: Uuid) -> anyhow::Result<bool> {
    let result = sqlx::query(
        "UPDATE agent_tokens SET revoked_at = NOW()
         WHERE id = $1 AND revoked_at IS NULL",
    )
    .bind(agent_id)
    .execute(db.as_ref())
    .await
    .context("deactivate agent")?;

    Ok(result.rows_affected() > 0)
}

// ---------------------------------------------------------------------------
// Report resolution
// ---------------------------------------------------------------------------

pub async fn resolve_report(
    db: &DbPool,
    report_id: Uuid,
    action: &str,
    reason: Option<&str>,
) -> anyhow::Result<bool> {
    let result = sqlx::query(
        "UPDATE reports SET status = $1, reason = COALESCE($2, reason)
         WHERE id = $3 AND status = 'pending'",
    )
    .bind(action)
    .bind(reason)
    .bind(report_id)
    .execute(db.as_ref())
    .await
    .context("resolve report")?;

    Ok(result.rows_affected() > 0)
}

// ---------------------------------------------------------------------------
// Spam patterns
// ---------------------------------------------------------------------------

pub async fn list_spam_patterns(db: &DbPool) -> anyhow::Result<Vec<SpamPattern>> {
    sqlx::query_as::<_, SpamPattern>(
        "SELECT id, keyword, created_at FROM spam_keywords ORDER BY created_at DESC",
    )
    .fetch_all(db.as_ref())
    .await
    .context("list spam patterns")
}

pub async fn create_spam_pattern(
    db: &DbPool,
    keyword: &str,
) -> anyhow::Result<CreateSpamPatternResponse> {
    let row = sqlx::query_as::<_, (i32, String)>(
        "INSERT INTO spam_keywords (keyword) VALUES ($1)
         ON CONFLICT (keyword) DO UPDATE SET keyword = EXCLUDED.keyword
         RETURNING id, keyword",
    )
    .bind(keyword)
    .fetch_one(db.as_ref())
    .await
    .context("create spam pattern")?;

    Ok(CreateSpamPatternResponse {
        id: row.0,
        keyword: row.1,
    })
}

pub async fn delete_spam_pattern(db: &DbPool, pattern_id: i32) -> anyhow::Result<bool> {
    let result = sqlx::query("DELETE FROM spam_keywords WHERE id = $1")
        .bind(pattern_id)
        .execute(db.as_ref())
        .await
        .context("delete spam pattern")?;

    Ok(result.rows_affected() > 0)
}

// ---------------------------------------------------------------------------
// Dashboard metrics
// ---------------------------------------------------------------------------

pub async fn get_dashboard_metrics(db: &DbPool) -> anyhow::Result<DashboardMetrics> {
    let total_users: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE deleted_at IS NULL")
            .fetch_one(db.as_ref())
            .await
            .context("count total users")?;

    let dau: i64 = sqlx::query_scalar(
        "SELECT COUNT(DISTINCT user_id) FROM devices
         WHERE last_seen_at >= NOW() - INTERVAL '1 day'",
    )
    .fetch_one(db.as_ref())
    .await
    .unwrap_or(0);

    let wau: i64 = sqlx::query_scalar(
        "SELECT COUNT(DISTINCT user_id) FROM devices
         WHERE last_seen_at >= NOW() - INTERVAL '7 days'",
    )
    .fetch_one(db.as_ref())
    .await
    .unwrap_or(0);

    let total_messages_today: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM messages
         WHERE created_at >= CURRENT_DATE",
    )
    .fetch_one(db.as_ref())
    .await
    .unwrap_or(0);

    let pending_reports: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM reports WHERE status = 'pending'")
            .fetch_one(db.as_ref())
            .await
            .unwrap_or(0);

    Ok(DashboardMetrics {
        total_users,
        dau,
        wau,
        total_messages_today,
        ws_connections: 0, // filled by handler from Hub
        pending_reports,
    })
}

// ---------------------------------------------------------------------------
// Audit logs query
// ---------------------------------------------------------------------------

pub async fn list_audit_logs(
    db: &DbPool,
    query: &AuditLogQuery,
) -> anyhow::Result<(Vec<AuditLogRow>, i64)> {
    let offset = (query.page - 1).max(0) * query.limit;

    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_logs
         WHERE ($1::UUID IS NULL OR actor_id = $1)
           AND ($2::TEXT IS NULL OR action = $2)
           AND ($3::TIMESTAMPTZ IS NULL OR created_at >= $3)
           AND ($4::TIMESTAMPTZ IS NULL OR created_at <= $4)",
    )
    .bind(query.actor_id)
    .bind(query.action.as_deref())
    .bind(query.from)
    .bind(query.to)
    .fetch_one(db.as_ref())
    .await
    .context("count audit logs")?;

    let rows = sqlx::query_as::<_, AuditLogRow>(
        "SELECT id, actor_id, action, target_type, target_id, metadata, created_at
         FROM audit_logs
         WHERE ($1::UUID IS NULL OR actor_id = $1)
           AND ($2::TEXT IS NULL OR action = $2)
           AND ($3::TIMESTAMPTZ IS NULL OR created_at >= $3)
           AND ($4::TIMESTAMPTZ IS NULL OR created_at <= $4)
         ORDER BY created_at DESC
         LIMIT $5 OFFSET $6",
    )
    .bind(query.actor_id)
    .bind(query.action.as_deref())
    .bind(query.from)
    .bind(query.to)
    .bind(query.limit.min(100))
    .bind(offset)
    .fetch_all(db.as_ref())
    .await
    .context("list audit logs")?;

    Ok((rows, total))
}
