use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

use super::models::{
    AgentProfile, RegisterAgentRequest, RegisterAgentResponse, RevokeAgentResponse,
    is_valid_agent_token_format,
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
