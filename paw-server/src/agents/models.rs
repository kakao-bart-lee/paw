use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct RegisterAgentRequest {
    pub name: String,
    pub description: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RegisterAgentResponse {
    pub agent_id: Uuid,
    pub token: String,
    pub name: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct AgentProfile {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub avatar_url: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct RevokeAgentResponse {
    pub agent_id: Uuid,
    pub revoked: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InviteAgentRequest {
    pub agent_id: Uuid,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InviteAgentResponse {
    pub invited: bool,
}

pub const AGENT_TOKEN_PREFIX: &str = "paw_agent_";

pub fn is_valid_agent_token_format(raw_token: &str) -> bool {
    let Some(suffix) = raw_token.strip_prefix(AGENT_TOKEN_PREFIX) else {
        return false;
    };
    Uuid::parse_str(suffix).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_token_format() {
        let token = format!("paw_agent_{}", Uuid::new_v4());
        assert!(is_valid_agent_token_format(&token));
    }

    #[test]
    fn rejects_missing_prefix() {
        let token = format!("agent_{}", Uuid::new_v4());
        assert!(!is_valid_agent_token_format(&token));
    }

    #[test]
    fn rejects_wrong_prefix() {
        let token = format!("paw_bot_{}", Uuid::new_v4());
        assert!(!is_valid_agent_token_format(&token));
    }

    #[test]
    fn rejects_non_uuid_suffix() {
        assert!(!is_valid_agent_token_format("paw_agent_not-a-uuid"));
    }

    #[test]
    fn rejects_empty_string() {
        assert!(!is_valid_agent_token_format(""));
    }

    #[test]
    fn generated_token_passes_validation() {
        let token = crate::agents::service::generate_agent_token();
        assert!(is_valid_agent_token_format(&token));
    }
}
