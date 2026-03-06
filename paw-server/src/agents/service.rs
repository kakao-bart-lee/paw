use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

use super::models::{RegisterAgentRequest, RegisterAgentResponse};

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
        "INSERT INTO agent_tokens (name, description, token_hash, owner_user_id) \
         VALUES ($1, $2, $3, $4) RETURNING id",
    )
    .bind(&req.name)
    .bind(&req.description)
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

pub async fn verify_agent_token(
    db: &PgPool,
    raw_token: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let token_hash = hash_token(raw_token);

    let result = sqlx::query_as::<_, (Uuid,)>(
        "UPDATE agent_tokens SET last_used_at = NOW() \
         WHERE token_hash = $1 RETURNING id",
    )
    .bind(&token_hash)
    .fetch_optional(db)
    .await?;

    Ok(result.map(|r| r.0))
}
