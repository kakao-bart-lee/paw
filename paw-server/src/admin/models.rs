use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// RBAC
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AdminRole {
    Moderator = 1,
    Admin = 2,
    SuperAdmin = 3,
}

impl AdminRole {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "moderator" => Some(Self::Moderator),
            "admin" => Some(Self::Admin),
            "super_admin" => Some(Self::SuperAdmin),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// User management
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct AdminUserListQuery {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_limit")]
    pub limit: i64,
    pub search: Option<String>,
}

fn default_page() -> i64 {
    1
}
fn default_limit() -> i64 {
    20
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct AdminUserRow {
    pub id: Uuid,
    pub phone: String,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub role: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct AdminUserListResponse {
    pub users: Vec<AdminUserRow>,
    pub total: i64,
    pub page: i64,
    pub limit: i64,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct AdminUserDevice {
    pub id: Uuid,
    pub device_name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct AdminUserDetailResponse {
    pub id: Uuid,
    pub phone: String,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub role: String,
    pub created_at: DateTime<Utc>,
    pub devices: Vec<AdminUserDevice>,
}

// ---------------------------------------------------------------------------
// Agent management
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct PendingAgent {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub owner_user_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct PendingAgentsResponse {
    pub agents: Vec<PendingAgent>,
}

#[derive(Debug, Deserialize)]
pub struct RejectAgentRequest {
    pub reason: String,
}

#[derive(Debug, Serialize)]
pub struct AgentActionResponse {
    pub success: bool,
}

// ---------------------------------------------------------------------------
// Report resolution
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct ResolveReportRequest {
    pub action: String,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ResolveReportResponse {
    pub resolved: bool,
}

// ---------------------------------------------------------------------------
// Spam patterns
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct SpamPattern {
    pub id: i32,
    pub keyword: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct SpamPatternsResponse {
    pub patterns: Vec<SpamPattern>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSpamPatternRequest {
    pub keyword: String,
}

#[derive(Debug, Serialize)]
pub struct CreateSpamPatternResponse {
    pub id: i32,
    pub keyword: String,
}

// ---------------------------------------------------------------------------
// Dashboard metrics
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct DashboardMetrics {
    pub total_users: i64,
    pub dau: i64,
    pub wau: i64,
    pub total_messages_today: i64,
    pub ws_connections: usize,
    pub pending_reports: i64,
}

// ---------------------------------------------------------------------------
// Audit logs
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct AuditLogQuery {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_limit")]
    pub limit: i64,
    pub actor_id: Option<Uuid>,
    pub action: Option<String>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct AuditLogRow {
    pub id: Uuid,
    pub actor_id: Uuid,
    pub action: String,
    pub target_type: Option<String>,
    pub target_id: Option<Uuid>,
    pub metadata: Option<Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct AuditLogsResponse {
    pub logs: Vec<AuditLogRow>,
    pub total: i64,
    pub page: i64,
    pub limit: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admin_role_ordering() {
        assert!(AdminRole::SuperAdmin > AdminRole::Admin);
        assert!(AdminRole::Admin > AdminRole::Moderator);
    }

    #[test]
    fn admin_role_from_str() {
        assert_eq!(AdminRole::from_str("moderator"), Some(AdminRole::Moderator));
        assert_eq!(AdminRole::from_str("admin"), Some(AdminRole::Admin));
        assert_eq!(
            AdminRole::from_str("super_admin"),
            Some(AdminRole::SuperAdmin)
        );
        assert_eq!(AdminRole::from_str("user"), None);
        assert_eq!(AdminRole::from_str(""), None);
    }

    #[test]
    fn user_list_query_defaults() {
        let q: AdminUserListQuery = serde_json::from_str("{}").unwrap();
        assert_eq!(q.page, 1);
        assert_eq!(q.limit, 20);
        assert!(q.search.is_none());
    }

    #[test]
    fn resolve_report_request_serde() {
        let req: ResolveReportRequest =
            serde_json::from_str(r#"{"action":"warn","reason":"test"}"#).unwrap();
        assert_eq!(req.action, "warn");
        assert_eq!(req.reason, Some("test".into()));
    }

    #[test]
    fn audit_log_query_defaults() {
        let q: AuditLogQuery = serde_json::from_str("{}").unwrap();
        assert_eq!(q.page, 1);
        assert_eq!(q.limit, 20);
        assert!(q.actor_id.is_none());
    }
}
