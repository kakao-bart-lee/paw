use crate::db::DbPool;
use anyhow::{anyhow, Context};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentPermission {
    ReadMessages,
    SendMessages,
    ManageThread,
    AccessHistory,
    UseTools,
}

impl AgentPermission {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ReadMessages => "read_messages",
            Self::SendMessages => "send_messages",
            Self::ManageThread => "manage_thread",
            Self::AccessHistory => "access_history",
            Self::UseTools => "use_tools",
        }
    }

    pub const fn default_on_invite() -> [Self; 2] {
        [Self::ReadMessages, Self::SendMessages]
    }
}

impl FromStr for AgentPermission {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "read_messages" => Ok(Self::ReadMessages),
            "send_messages" => Ok(Self::SendMessages),
            "manage_thread" => Ok(Self::ManageThread),
            "access_history" => Ok(Self::AccessHistory),
            "use_tools" => Ok(Self::UseTools),
            _ => Err(anyhow!("invalid_permission")),
        }
    }
}

pub async fn check_agent_permission(
    pool: &DbPool,
    agent_id: Uuid,
    conversation_id: Uuid,
    required_permission: AgentPermission,
) -> anyhow::Result<bool> {
    sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(\
            SELECT 1\
            FROM agent_permissions\
            WHERE agent_id = $1 AND conversation_id = $2 AND permission = $3\
        )",
    )
    .bind(agent_id)
    .bind(conversation_id)
    .bind(required_permission.as_str())
    .fetch_one(pool.as_ref())
    .await
    .context("check agent permission")
}

pub async fn agent_in_conversation(
    pool: &DbPool,
    conversation_id: Uuid,
    agent_id: Uuid,
) -> anyhow::Result<bool> {
    sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(\
            SELECT 1 FROM conversation_agents\
            WHERE conversation_id = $1 AND agent_id = $2\
        )",
    )
    .bind(conversation_id)
    .bind(agent_id)
    .fetch_one(pool.as_ref())
    .await
    .context("check agent membership in conversation")
}

pub async fn list_permissions(
    pool: &DbPool,
    conversation_id: Uuid,
    agent_id: Uuid,
) -> anyhow::Result<Vec<AgentPermission>> {
    let rows = sqlx::query_scalar::<_, String>(
        "SELECT permission\
         FROM agent_permissions\
         WHERE conversation_id = $1 AND agent_id = $2\
         ORDER BY permission ASC",
    )
    .bind(conversation_id)
    .bind(agent_id)
    .fetch_all(pool.as_ref())
    .await
    .context("list agent permissions")?;

    rows.into_iter()
        .map(|permission| AgentPermission::from_str(&permission))
        .collect()
}

pub async fn replace_permissions(
    pool: &DbPool,
    conversation_id: Uuid,
    agent_id: Uuid,
    granted_by: Uuid,
    permissions: &[AgentPermission],
) -> anyhow::Result<Vec<AgentPermission>> {
    let unique_permissions: BTreeSet<AgentPermission> = permissions.iter().copied().collect();
    let mut tx = pool.begin().await.context("begin permission update tx")?;

    sqlx::query("DELETE FROM agent_permissions WHERE conversation_id = $1 AND agent_id = $2")
        .bind(conversation_id)
        .bind(agent_id)
        .execute(&mut *tx)
        .await
        .context("clear existing permissions")?;

    for permission in &unique_permissions {
        sqlx::query(
            "INSERT INTO agent_permissions (agent_id, conversation_id, permission, granted_by)\
             VALUES ($1, $2, $3, $4)",
        )
        .bind(agent_id)
        .bind(conversation_id)
        .bind(permission.as_str())
        .bind(granted_by)
        .execute(&mut *tx)
        .await
        .context("insert agent permission")?;
    }

    tx.commit().await.context("commit permission update tx")?;

    Ok(unique_permissions.into_iter().collect())
}

pub async fn can_manage_permissions(
    pool: &DbPool,
    conversation_id: Uuid,
    user_id: Uuid,
) -> anyhow::Result<bool> {
    sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(\
            SELECT 1\
            FROM conversation_members\
            WHERE conversation_id = $1 AND user_id = $2 AND role IN ('owner', 'admin')\
        )",
    )
    .bind(conversation_id)
    .bind(user_id)
    .fetch_one(pool.as_ref())
    .await
    .context("check conversation permission manager role")
}

#[cfg(test)]
mod tests {
    use super::AgentPermission;
    use std::str::FromStr;

    #[test]
    fn default_on_invite_contains_read_and_send() {
        let defaults = AgentPermission::default_on_invite();
        assert_eq!(defaults.len(), 2);
        assert!(defaults.contains(&AgentPermission::ReadMessages));
        assert!(defaults.contains(&AgentPermission::SendMessages));
    }

    #[test]
    fn permission_string_roundtrip() {
        let parsed = AgentPermission::from_str("use_tools").unwrap();
        assert_eq!(parsed, AgentPermission::UseTools);
        assert_eq!(parsed.as_str(), "use_tools");
    }
}
