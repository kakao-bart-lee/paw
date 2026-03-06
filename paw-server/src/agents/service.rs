use crate::db::DbPool;
use anyhow::anyhow;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

use super::models::{
    AgentManifest, AgentProfile, InstalledAgent, MarketplaceAgent, MarketplaceAgentDetail,
    RegisterAgentRequest, RegisterAgentResponse, RevokeAgentResponse, is_valid_agent_token_format,
};

pub fn generate_agent_token() -> String {
    format!("paw_agent_{}", Uuid::new_v4())
}

pub fn hash_token(raw_token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw_token.as_bytes());
    hex::encode(hasher.finalize())
}

pub async fn register_agent(
    db: &PgPool,
    owner_user_id: Uuid,
    req: RegisterAgentRequest,
) -> Result<RegisterAgentResponse, sqlx::Error> {
    let raw_token = generate_agent_token();
    let token_hash = hash_token(&raw_token);

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

pub async fn verify_agent_token(
    db: &PgPool,
    raw_token: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
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
    .fetch_one(pool.as_ref())
    .await?;

    match outcome {
        1 => Ok(true),
        0 => Ok(false),
        _ => Err(anyhow!("agent_not_found_or_revoked")),
    }
}

pub async fn remove_agent_from_conversation(
    pool: &DbPool,
    conversation_id: Uuid,
    agent_id: Uuid,
    requester_id: Uuid,
) -> anyhow::Result<bool> {
    let is_owner = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(\
            SELECT 1 FROM conversation_members\
            WHERE conversation_id = $1 AND user_id = $2 AND role = 'owner'\
        )",
    )
    .bind(conversation_id)
    .bind(requester_id)
    .fetch_one(pool.as_ref())
    .await?;

    if !is_owner {
        return Err(anyhow!("not_owner"));
    }

    let removed = sqlx::query(
        "DELETE FROM conversation_agents WHERE conversation_id = $1 AND agent_id = $2",
    )
    .bind(conversation_id)
    .bind(agent_id)
    .execute(pool.as_ref())
    .await?
    .rows_affected();

    Ok(removed > 0)
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

pub async fn install_agent(
    db: &PgPool,
    user_id: Uuid,
    agent_id: Uuid,
) -> anyhow::Result<bool> {
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

pub async fn uninstall_agent(
    db: &PgPool,
    user_id: Uuid,
    agent_id: Uuid,
) -> anyhow::Result<bool> {
    let result = sqlx::query(
        "DELETE FROM user_installed_agents WHERE user_id = $1 AND agent_id = $2",
    )
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
    let manifest_json = serde_json::to_value(manifest)
        .map_err(|e| anyhow!("invalid manifest: {e}"))?;

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
