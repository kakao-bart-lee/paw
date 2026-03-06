use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub capabilities: Vec<String>,
    pub permissions: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config_schema: Option<Value>,
}

impl AgentManifest {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.name.is_empty() {
            return Err("manifest name is required");
        }
        if self.version.is_empty() {
            return Err("manifest version is required");
        }
        if self.description.is_empty() {
            return Err("manifest description is required");
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct MarketplaceSearchQuery {
    pub q: Option<String>,
    pub category: Option<String>,
    #[serde(default = "default_sort")]
    pub sort: String,
}

fn default_sort() -> String {
    "popular".to_string()
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct MarketplaceAgent {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub avatar_url: Option<String>,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub rating_avg: f64,
    pub install_count: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct MarketplaceSearchResponse {
    pub agents: Vec<MarketplaceAgent>,
    pub count: usize,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct MarketplaceAgentDetail {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub avatar_url: Option<String>,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub rating_avg: f64,
    pub install_count: i32,
    pub manifest: Option<Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct InstallAgentResponse {
    pub installed: bool,
    pub agent_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct UninstallAgentResponse {
    pub uninstalled: bool,
    pub agent_id: Uuid,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct InstalledAgent {
    pub agent_id: Uuid,
    pub agent_name: String,
    pub agent_description: Option<String>,
    pub agent_avatar_url: Option<String>,
    pub installed_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct InstalledAgentsResponse {
    pub agents: Vec<InstalledAgent>,
}

#[derive(Debug, Deserialize)]
pub struct PublishAgentRequest {
    pub manifest: AgentManifest,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct PublishAgentResponse {
    pub published: bool,
    pub agent_id: Uuid,
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

    #[test]
    fn marketplace_search_response_serde_roundtrip() {
        let response = MarketplaceSearchResponse {
            agents: vec![MarketplaceAgent {
                id: Uuid::nil(),
                name: "TestBot".into(),
                description: Some("A test agent".into()),
                avatar_url: None,
                category: Some("productivity".into()),
                tags: vec!["test".into(), "demo".into()],
                rating_avg: 4.5,
                install_count: 100,
                created_at: chrono::Utc::now(),
            }],
            count: 1,
        };

        let json = serde_json::to_string(&response).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["count"], 1);
        assert_eq!(parsed["agents"][0]["name"], "TestBot");
        assert_eq!(parsed["agents"][0]["rating_avg"], 4.5);
        assert_eq!(parsed["agents"][0]["tags"][0], "test");
    }

    #[test]
    fn install_response_serde_roundtrip() {
        let response = InstallAgentResponse {
            installed: true,
            agent_id: Uuid::nil(),
        };

        let json = serde_json::to_string(&response).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["installed"], true);
        assert!(parsed["agent_id"].is_string());

        let uninstall = UninstallAgentResponse {
            uninstalled: true,
            agent_id: Uuid::nil(),
        };
        let json2 = serde_json::to_string(&uninstall).unwrap();
        let parsed2: serde_json::Value = serde_json::from_str(&json2).unwrap();
        assert_eq!(parsed2["uninstalled"], true);
    }

    #[test]
    fn manifest_validation() {
        let valid = AgentManifest {
            name: "TestBot".into(),
            version: "1.0.0".into(),
            description: "A test agent".into(),
            capabilities: vec!["chat".into()],
            permissions: vec!["read_messages".into()],
            config_schema: None,
        };
        assert!(valid.validate().is_ok());

        let roundtripped: AgentManifest =
            serde_json::from_str(&serde_json::to_string(&valid).unwrap()).unwrap();
        assert_eq!(valid, roundtripped);

        let empty_name = AgentManifest {
            name: "".into(),
            ..valid.clone()
        };
        assert_eq!(empty_name.validate(), Err("manifest name is required"));

        let empty_version = AgentManifest {
            version: "".into(),
            ..valid.clone()
        };
        assert_eq!(
            empty_version.validate(),
            Err("manifest version is required")
        );

        let empty_desc = AgentManifest {
            description: "".into(),
            ..valid
        };
        assert_eq!(
            empty_desc.validate(),
            Err("manifest description is required")
        );
    }
}
