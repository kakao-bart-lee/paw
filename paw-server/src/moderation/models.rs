use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateReportRequest {
    pub target_type: String,
    pub target_id: Uuid,
    pub reason: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Report {
    pub id: Uuid,
    pub reporter_id: Uuid,
    pub target_type: String,
    pub target_id: Uuid,
    pub reason: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct CreateReportResponse {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserBlock {
    pub blocker_id: Uuid,
    pub blocked_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct BlockResponse {
    pub blocked: bool,
}

#[derive(Debug, Serialize)]
pub struct UnblockResponse {
    pub unblocked: bool,
}

#[derive(Debug, Serialize)]
pub struct BlockedUsersResponse {
    pub blocked_users: Vec<BlockedUserItem>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct BlockedUserItem {
    pub blocked_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SuspendUserRequest {
    pub suspended_until: DateTime<Utc>,
    pub reason: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserSuspension {
    pub user_id: Uuid,
    pub suspended_until: DateTime<Utc>,
    pub reason: Option<String>,
    pub suspended_by: Uuid,
}

#[derive(Debug, Serialize)]
pub struct SuspendResponse {
    pub suspended: bool,
}

#[derive(Debug, Serialize)]
pub struct UnsuspendResponse {
    pub unsuspended: bool,
}

#[derive(Debug, Serialize)]
pub struct AdminReportsResponse {
    pub reports: Vec<Report>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_request_serde_roundtrip() {
        let req = CreateReportRequest {
            target_type: "message".into(),
            target_id: Uuid::nil(),
            reason: "Spam content".into(),
        };

        let json = serde_json::to_string(&req).unwrap();
        let parsed: CreateReportRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.target_type, "message");
        assert_eq!(parsed.target_id, Uuid::nil());
        assert_eq!(parsed.reason, "Spam content");
    }

    #[test]
    fn report_request_rejects_missing_fields() {
        let incomplete = r#"{"target_type":"user"}"#;
        let result = serde_json::from_str::<CreateReportRequest>(incomplete);
        assert!(result.is_err());
    }

    #[test]
    fn block_item_serde_roundtrip() {
        let item = BlockedUserItem {
            blocked_id: Uuid::nil(),
            created_at: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&item).unwrap();
        let parsed: BlockedUserItem = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.blocked_id, Uuid::nil());
    }

    #[test]
    fn suspension_model_serde_roundtrip() {
        let suspension = UserSuspension {
            user_id: Uuid::nil(),
            suspended_until: chrono::Utc::now(),
            reason: Some("Violation of terms".into()),
            suspended_by: Uuid::nil(),
        };

        let json = serde_json::to_string(&suspension).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert!(parsed["user_id"].is_string());
        assert!(parsed["suspended_until"].is_string());
        assert_eq!(parsed["reason"], "Violation of terms");
        assert!(parsed["suspended_by"].is_string());

        let roundtripped: UserSuspension = serde_json::from_str(&json).unwrap();
        assert_eq!(roundtripped.reason, Some("Violation of terms".into()));
    }

    #[test]
    fn suspension_model_with_null_reason() {
        let suspension = UserSuspension {
            user_id: Uuid::nil(),
            suspended_until: chrono::Utc::now(),
            reason: None,
            suspended_by: Uuid::nil(),
        };

        let json = serde_json::to_string(&suspension).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed["reason"].is_null());

        let roundtripped: UserSuspension = serde_json::from_str(&json).unwrap();
        assert_eq!(roundtripped.reason, None);
    }

    #[test]
    fn suspend_request_serde_roundtrip() {
        let req = SuspendUserRequest {
            suspended_until: chrono::Utc::now(),
            reason: Some("Repeated spam".into()),
        };

        let json = serde_json::to_string(&req).unwrap();
        let parsed: SuspendUserRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.reason, Some("Repeated spam".into()));
    }
}
