use crate::db::DbPool;
use anyhow::anyhow;
use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

use super::models::{
    is_valid_agent_token_format, AgentManifest, AgentProfile, InstalledAgent, MarketplaceAgent,
    MarketplaceAgentDetail, RegisterAgentRequest, RegisterAgentResponse, RevokeAgentResponse,
    RotateAgentKeyResponse,
};
use super::permissions::DEFAULT_INVITE_PERMISSIONS;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversationAgentManageAccess {
    Allowed,
    Forbidden,
    ConversationNotFound,
}

#[derive(Debug, Clone)]
pub struct AgentDelegationRecord {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub from_agent_id: Uuid,
    pub target_agent_id: Uuid,
    pub delegated_at: DateTime<Utc>,
}

pub fn generate_agent_token() -> String {
    format!("paw_agent_{}", Uuid::new_v4())
}

pub fn hash_token(raw_token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw_token.as_bytes());
    hex::encode(hasher.finalize())
}

fn generate_agent_token_pair() -> (String, String) {
    let raw_token = generate_agent_token();
    let token_hash = hash_token(&raw_token);
    (raw_token, token_hash)
}

pub async fn register_agent(
    db: &PgPool,
    owner_user_id: Uuid,
    req: RegisterAgentRequest,
) -> Result<RegisterAgentResponse, sqlx::Error> {
    let (raw_token, token_hash) = generate_agent_token_pair();

    let row = sqlx::query_as::<_, (Uuid,)>(
        "INSERT INTO agent_tokens (name, description, avatar_url, token_hash, owner_user_id) \
         VALUES ($1, $2, $3, $4, $5) RETURNING id",
    )
    .bind(&req.name)
    .bind(&req.description)
    .bind(&req.avatar_url)
    .bind(&token_hash)
    .bind(owner_user_id)
    .fetch_one(db)
    .await?;

    Ok(RegisterAgentResponse {
        agent_id: row.0,
        token: raw_token,
        name: req.name,
    })
}

pub async fn get_agent_profile(
    db: &PgPool,
    agent_id: Uuid,
) -> Result<Option<AgentProfile>, sqlx::Error> {
    sqlx::query_as::<_, AgentProfile>(
        "SELECT id, name, description, avatar_url, created_at \
         FROM agent_tokens WHERE id = $1 AND revoked_at IS NULL",
    )
    .bind(agent_id)
    .fetch_optional(db)
    .await
}

pub async fn revoke_agent_token(
    db: &PgPool,
    agent_id: Uuid,
    owner_user_id: Uuid,
) -> Result<Option<RevokeAgentResponse>, sqlx::Error> {
    let result = sqlx::query_as::<_, (Uuid,)>(
        "UPDATE agent_tokens SET revoked_at = NOW() \
         WHERE id = $1 AND owner_user_id = $2 AND revoked_at IS NULL \
         RETURNING id",
    )
    .bind(agent_id)
    .bind(owner_user_id)
    .fetch_optional(db)
    .await?;

    Ok(result.map(|r| RevokeAgentResponse {
        agent_id: r.0,
        revoked: true,
    }))
}

pub async fn rotate_agent_token(
    db: &PgPool,
    agent_id: Uuid,
    owner_user_id: Uuid,
) -> Result<Option<RotateAgentKeyResponse>, sqlx::Error> {
    let (raw_token, token_hash) = generate_agent_token_pair();

    let result = sqlx::query_as::<_, (Uuid,)>(
        "UPDATE agent_tokens SET token_hash = $1, last_used_at = NULL \
         WHERE id = $2 AND owner_user_id = $3 AND revoked_at IS NULL \
         RETURNING id",
    )
    .bind(&token_hash)
    .bind(agent_id)
    .bind(owner_user_id)
    .fetch_optional(db)
    .await?;

    Ok(result.map(|row| RotateAgentKeyResponse {
        agent_id: row.0,
        rotated: true,
        api_key: raw_token,
    }))
}

pub async fn verify_agent_token(db: &PgPool, raw_token: &str) -> Result<Option<Uuid>, sqlx::Error> {
    if !is_valid_agent_token_format(raw_token) {
        return Ok(None);
    }

    let token_hash = hash_token(raw_token);

    let result = sqlx::query_as::<_, (Uuid,)>(
        "UPDATE agent_tokens SET last_used_at = NOW() \
         WHERE token_hash = $1 AND revoked_at IS NULL RETURNING id",
    )
    .bind(&token_hash)
    .fetch_optional(db)
    .await?;

    Ok(result.map(|r| r.0))
}

pub async fn invite_agent_to_conversation(
    pool: &DbPool,
    conversation_id: Uuid,
    agent_id: Uuid,
    invited_by: Uuid,
) -> anyhow::Result<bool> {
    let mut tx = pool.begin().await?;

    let outcome = sqlx::query_scalar::<_, i32>(
        "WITH inserted AS (\
            INSERT INTO conversation_agents (conversation_id, agent_id, invited_by)\
            SELECT $1, at.id, $3\
            FROM agent_tokens at\
            WHERE at.id = $2 AND at.revoked_at IS NULL\
            ON CONFLICT (conversation_id, agent_id) DO NOTHING\
            RETURNING 1\
        )\
        SELECT CASE\
            WHEN EXISTS (SELECT 1 FROM inserted) THEN 1\
            WHEN EXISTS (SELECT 1 FROM conversation_agents WHERE conversation_id = $1 AND agent_id = $2) THEN 0\
            ELSE -1\
        END",
    )
    .bind(conversation_id)
    .bind(agent_id)
    .bind(invited_by)
    .fetch_one(&mut *tx)
    .await?;

    match outcome {
        1 => {
            for permission in DEFAULT_INVITE_PERMISSIONS {
                sqlx::query(
                    "INSERT INTO agent_permissions (id, agent_id, conversation_id, permission, granted_by)
                     VALUES (gen_random_uuid(), $1, $2, $3, $4)
                     ON CONFLICT (agent_id, conversation_id, permission) DO NOTHING",
                )
                .bind(agent_id)
                .bind(conversation_id)
                .bind(permission.as_str())
                .bind(invited_by)
                .execute(&mut *tx)
                .await?;
            }
            tx.commit().await?;
            Ok(true)
        }
        0 => {
            tx.commit().await?;
            Ok(false)
        }
        _ => Err(anyhow!("agent_not_found_or_revoked")),
    }
}

#[allow(dead_code)]
pub async fn can_manage_conversation_agents(
    pool: &DbPool,
    conversation_id: Uuid,
    user_id: Uuid,
) -> anyhow::Result<bool> {
    Ok(matches!(
        conversation_agent_manage_access(pool, conversation_id, user_id).await?,
        ConversationAgentManageAccess::Allowed
    ))
}

pub async fn conversation_agent_manage_access(
    pool: &DbPool,
    conversation_id: Uuid,
    user_id: Uuid,
) -> anyhow::Result<ConversationAgentManageAccess> {
    let exists = conversation_exists(pool, conversation_id).await?;
    if !exists {
        return Ok(ConversationAgentManageAccess::ConversationNotFound);
    }

    sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(
            SELECT 1
            FROM conversations c
            WHERE c.id = $1
              AND (
                  (c.is_agent_only = TRUE AND c.created_by = $2)
                  OR EXISTS(
                      SELECT 1
                      FROM conversation_members cm
                      WHERE cm.conversation_id = c.id
                        AND cm.user_id = $2
                        AND cm.role = 'admin'
                  )
              )
        )",
    )
    .bind(conversation_id)
    .bind(user_id)
    .fetch_one(pool.as_ref())
    .await
    .map(|allowed| {
        if allowed {
            ConversationAgentManageAccess::Allowed
        } else {
            ConversationAgentManageAccess::Forbidden
        }
    })
    .map_err(Into::into)
}

pub async fn conversation_exists(pool: &DbPool, conversation_id: Uuid) -> anyhow::Result<bool> {
    sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM conversations WHERE id = $1)")
        .bind(conversation_id)
        .fetch_one(pool.as_ref())
        .await
        .map_err(Into::into)
}

pub async fn remove_agent_from_conversation(
    pool: &DbPool,
    conversation_id: Uuid,
    agent_id: Uuid,
    requester_id: Uuid,
) -> anyhow::Result<bool> {
    let is_admin = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(\
            SELECT 1 FROM conversation_members\
            WHERE conversation_id = $1 AND user_id = $2 AND role = 'admin'\
        )",
    )
    .bind(conversation_id)
    .bind(requester_id)
    .fetch_one(pool.as_ref())
    .await?;

    if !is_admin {
        return Err(anyhow!("not_admin"));
    }

    let removed =
        sqlx::query("DELETE FROM conversation_agents WHERE conversation_id = $1 AND agent_id = $2")
            .bind(conversation_id)
            .bind(agent_id)
            .execute(pool.as_ref())
            .await?
            .rows_affected();

    Ok(removed > 0)
}

pub async fn delegate_agent_task(
    pool: &DbPool,
    conversation_id: Uuid,
    from_agent_id: Uuid,
    target_agent_id: Uuid,
    delegated_by: Uuid,
    task_description: &str,
) -> anyhow::Result<AgentDelegationRecord> {
    let normalized_task = task_description.trim();
    if normalized_task.is_empty() {
        return Err(anyhow!("{}", "invalid_task_description"));
    }
    if from_agent_id == target_agent_id {
        return Err(anyhow!("{}", "same_agent"));
    }

    let membership = sqlx::query_as::<_, (bool, bool, bool)>(
        "SELECT
            EXISTS(SELECT 1 FROM conversations WHERE id = $1) AS conversation_exists,
            EXISTS(
                SELECT 1 FROM conversation_agents
                WHERE conversation_id = $1 AND agent_id = $2
            ) AS source_in_conversation,
            EXISTS(
                SELECT 1 FROM conversation_agents
                WHERE conversation_id = $1 AND agent_id = $3
            ) AS target_in_conversation",
    )
    .bind(conversation_id)
    .bind(from_agent_id)
    .bind(target_agent_id)
    .fetch_one(pool.as_ref())
    .await?;

    if !membership.0 {
        return Err(anyhow!("{}", "conversation_not_found"));
    }
    if !membership.1 {
        return Err(anyhow!("{}", "source_agent_not_in_conversation"));
    }
    if !membership.2 {
        return Err(anyhow!("{}", "target_agent_not_in_conversation"));
    }

    let (delegation_id, delegated_at) = sqlx::query_as::<_, (Uuid, DateTime<Utc>)>(
        "INSERT INTO agent_delegations (
            conversation_id,
            from_agent_id,
            target_agent_id,
            delegated_by,
            task_description
        )
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, created_at",
    )
    .bind(conversation_id)
    .bind(from_agent_id)
    .bind(target_agent_id)
    .bind(delegated_by)
    .bind(normalized_task)
    .fetch_one(pool.as_ref())
    .await?;

    Ok(AgentDelegationRecord {
        id: delegation_id,
        conversation_id,
        from_agent_id,
        target_agent_id,
        delegated_at,
    })
}

pub async fn search_marketplace_agents(
    db: &PgPool,
    q: Option<&str>,
    category: Option<&str>,
    sort: &str,
) -> Result<Vec<MarketplaceAgent>, sqlx::Error> {
    let order_clause = match sort {
        "newest" => "created_at DESC",
        "rating" => "rating_avg DESC",
        _ => "install_count DESC",
    };

    let query = format!(
        "SELECT id, name, description, avatar_url, category, tags, \
                rating_avg, install_count, created_at \
         FROM agent_tokens \
         WHERE is_public = true AND revoked_at IS NULL \
           AND ($1::TEXT IS NULL OR name ILIKE '%' || $1 || '%' OR description ILIKE '%' || $1 || '%') \
           AND ($2::TEXT IS NULL OR category = $2) \
         ORDER BY {order_clause} \
         LIMIT 50"
    );

    sqlx::query_as::<_, MarketplaceAgent>(&query)
        .bind(q)
        .bind(category)
        .fetch_all(db)
        .await
}

pub async fn get_marketplace_agent_detail(
    db: &PgPool,
    agent_id: Uuid,
) -> Result<Option<MarketplaceAgentDetail>, sqlx::Error> {
    sqlx::query_as::<_, MarketplaceAgentDetail>(
        "SELECT id, name, description, avatar_url, category, tags, \
                rating_avg, install_count, manifest, created_at \
         FROM agent_tokens \
         WHERE id = $1 AND is_public = true AND revoked_at IS NULL",
    )
    .bind(agent_id)
    .fetch_optional(db)
    .await
}

pub async fn install_agent(db: &PgPool, user_id: Uuid, agent_id: Uuid) -> anyhow::Result<bool> {
    let is_public = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM agent_tokens WHERE id = $1 AND is_public = true AND revoked_at IS NULL)",
    )
    .bind(agent_id)
    .fetch_one(db)
    .await?;

    if !is_public {
        return Err(anyhow!("agent_not_found"));
    }

    let result = sqlx::query(
        "INSERT INTO user_installed_agents (user_id, agent_id) \
         VALUES ($1, $2) ON CONFLICT DO NOTHING",
    )
    .bind(user_id)
    .bind(agent_id)
    .execute(db)
    .await?;

    if result.rows_affected() > 0 {
        sqlx::query("UPDATE agent_tokens SET install_count = install_count + 1 WHERE id = $1")
            .bind(agent_id)
            .execute(db)
            .await?;
    }

    Ok(result.rows_affected() > 0)
}

pub async fn uninstall_agent(db: &PgPool, user_id: Uuid, agent_id: Uuid) -> anyhow::Result<bool> {
    let result =
        sqlx::query("DELETE FROM user_installed_agents WHERE user_id = $1 AND agent_id = $2")
            .bind(user_id)
            .bind(agent_id)
            .execute(db)
            .await?;

    if result.rows_affected() > 0 {
        sqlx::query(
            "UPDATE agent_tokens SET install_count = GREATEST(install_count - 1, 0) WHERE id = $1",
        )
        .bind(agent_id)
        .execute(db)
        .await?;
    }

    Ok(result.rows_affected() > 0)
}

pub async fn list_installed_agents(
    db: &PgPool,
    user_id: Uuid,
) -> Result<Vec<InstalledAgent>, sqlx::Error> {
    sqlx::query_as::<_, InstalledAgent>(
        "SELECT uia.agent_id, at.name AS agent_name, at.description AS agent_description, \
                at.avatar_url AS agent_avatar_url, uia.installed_at \
         FROM user_installed_agents uia \
         JOIN agent_tokens at ON at.id = uia.agent_id \
         WHERE uia.user_id = $1 \
         ORDER BY uia.installed_at DESC",
    )
    .bind(user_id)
    .fetch_all(db)
    .await
}

pub async fn publish_agent(
    db: &PgPool,
    agent_id: Uuid,
    owner_user_id: Uuid,
    manifest: &AgentManifest,
    category: Option<&str>,
    tags: Option<&[String]>,
) -> anyhow::Result<bool> {
    let manifest_json =
        serde_json::to_value(manifest).map_err(|e| anyhow!("invalid manifest: {e}"))?;

    let empty_tags: Vec<String> = vec![];
    let tag_slice = tags.unwrap_or(&empty_tags);

    let result = sqlx::query(
        "UPDATE agent_tokens SET is_public = true, manifest = $1, category = $2, tags = $3 \
         WHERE id = $4 AND owner_user_id = $5 AND revoked_at IS NULL",
    )
    .bind(&manifest_json)
    .bind(category)
    .bind(tag_slice)
    .bind(agent_id)
    .bind(owner_user_id)
    .execute(db)
    .await?;

    Ok(result.rows_affected() > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_token_pair_matches_hash() {
        let (raw_token, token_hash) = generate_agent_token_pair();
        assert!(is_valid_agent_token_format(&raw_token));
        assert_eq!(token_hash, hash_token(&raw_token));
    }

    #[test]
    fn generated_token_pair_is_unique_per_rotation() {
        let (raw_a, hash_a) = generate_agent_token_pair();
        let (raw_b, hash_b) = generate_agent_token_pair();

        assert_ne!(raw_a, raw_b);
        assert_ne!(hash_a, hash_b);
    }
}
