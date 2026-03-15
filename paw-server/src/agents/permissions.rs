use crate::db::DbPool;
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentPermission {
    ReadMessages,
    SendMessages,
    ManageThread,
    AccessHistory,
    UseTools,
}

impl AgentPermission {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ReadMessages => "read_messages",
            Self::SendMessages => "send_messages",
            Self::ManageThread => "manage_thread",
            Self::AccessHistory => "access_history",
            Self::UseTools => "use_tools",
        }
    }

    fn from_db(value: &str) -> anyhow::Result<Self> {
        match value {
            "read_messages" => Ok(Self::ReadMessages),
            "send_messages" => Ok(Self::SendMessages),
            "manage_thread" => Ok(Self::ManageThread),
            "access_history" => Ok(Self::AccessHistory),
            "use_tools" => Ok(Self::UseTools),
            _ => Err(anyhow!("unknown permission value: {value}")),
        }
    }
}

pub const DEFAULT_INVITE_PERMISSIONS: [AgentPermission; 2] =
    [AgentPermission::ReadMessages, AgentPermission::SendMessages];

#[derive(Debug, Deserialize)]
pub struct UpdateAgentPermissionsRequest {
    pub permissions: Vec<AgentPermission>,
}

#[derive(Debug, Serialize)]
pub struct AgentPermissionsResponse {
    pub conversation_id: Uuid,
    pub agent_id: Uuid,
    pub permissions: Vec<AgentPermission>,
}

pub fn normalize_permissions(permissions: &[AgentPermission]) -> Vec<AgentPermission> {
    let mut seen = HashSet::new();
    let mut normalized = Vec::new();

    for permission in permissions {
        if seen.insert(*permission) {
            normalized.push(*permission);
        }
    }

    normalized
}

pub async fn list_agent_permissions(
    pool: &DbPool,
    conversation_id: Uuid,
    agent_id: Uuid,
) -> anyhow::Result<Option<Vec<AgentPermission>>> {
    let is_invited = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(
            SELECT 1
            FROM conversation_agents
            WHERE conversation_id = $1 AND agent_id = $2
        )",
    )
    .bind(conversation_id)
    .bind(agent_id)
    .fetch_one(pool.as_ref())
    .await?;

    if !is_invited {
        return Ok(None);
    }

    let rows = sqlx::query_scalar::<_, String>(
        "SELECT permission
         FROM agent_permissions
         WHERE conversation_id = $1 AND agent_id = $2
         ORDER BY granted_at ASC",
    )
    .bind(conversation_id)
    .bind(agent_id)
    .fetch_all(pool.as_ref())
    .await?;

    let mut permissions = Vec::with_capacity(rows.len());
    for permission in rows {
        permissions.push(AgentPermission::from_db(&permission)?);
    }

    Ok(Some(permissions))
}

pub async fn replace_agent_permissions(
    pool: &DbPool,
    conversation_id: Uuid,
    agent_id: Uuid,
    granted_by: Uuid,
    permissions: &[AgentPermission],
) -> anyhow::Result<Option<Vec<AgentPermission>>> {
    let normalized = normalize_permissions(permissions);
    let mut tx = pool.begin().await?;

    let is_invited = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(
            SELECT 1
            FROM conversation_agents
            WHERE conversation_id = $1 AND agent_id = $2
        )",
    )
    .bind(conversation_id)
    .bind(agent_id)
    .fetch_one(&mut *tx)
    .await?;

    if !is_invited {
        tx.rollback().await?;
        return Ok(None);
    }

    sqlx::query(
        "DELETE FROM agent_permissions
         WHERE conversation_id = $1 AND agent_id = $2",
    )
    .bind(conversation_id)
    .bind(agent_id)
    .execute(&mut *tx)
    .await?;

    for permission in &normalized {
        sqlx::query(
            "INSERT INTO agent_permissions (id, agent_id, conversation_id, permission, granted_by)
             VALUES (gen_random_uuid(), $1, $2, $3, $4)",
        )
        .bind(agent_id)
        .bind(conversation_id)
        .bind(permission.as_str())
        .bind(granted_by)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(Some(normalized))
}

pub async fn check_agent_permission(
    pool: &DbPool,
    conversation_id: Uuid,
    agent_id: Uuid,
    permission: AgentPermission,
) -> anyhow::Result<bool> {
    sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(
            SELECT 1
            FROM agent_permissions
            WHERE conversation_id = $1
              AND agent_id = $2
              AND permission = $3
        )",
    )
    .bind(conversation_id)
    .bind(agent_id)
    .bind(permission.as_str())
    .fetch_one(pool.as_ref())
    .await
    .map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permission_serde_roundtrip() {
        let permission = AgentPermission::AccessHistory;
        let json = serde_json::to_string(&permission).expect("serialize permission");
        assert_eq!(json, "\"access_history\"");
        let parsed: AgentPermission = serde_json::from_str(&json).expect("deserialize permission");
        assert_eq!(parsed, permission);
    }

    #[test]
    fn normalize_permissions_deduplicates_preserving_order() {
        let normalized = normalize_permissions(&[
            AgentPermission::ReadMessages,
            AgentPermission::SendMessages,
            AgentPermission::ReadMessages,
            AgentPermission::UseTools,
            AgentPermission::SendMessages,
        ]);

        assert_eq!(
            normalized,
            vec![
                AgentPermission::ReadMessages,
                AgentPermission::SendMessages,
                AgentPermission::UseTools,
            ]
        );
    }
}
