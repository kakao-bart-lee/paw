use crate::db::DbPool;
use crate::push::models::{PushPayload, PushPlatform, PushTokenRow};
use crate::ws::hub::Hub;
use anyhow::Context;
use chrono::{DateTime, Utc};
use uuid::Uuid;

pub async fn register_push_token(
    pool: &DbPool,
    user_id: Uuid,
    device_id: Uuid,
    platform: &PushPlatform,
    token: &str,
) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO push_tokens (user_id, device_id, platform, token)
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (device_id) DO UPDATE SET platform = $3, token = $4, created_at = NOW()",
    )
    .bind(user_id)
    .bind(device_id)
    .bind(platform.as_str())
    .bind(token)
    .execute(pool.as_ref())
    .await
    .context("register push token")?;

    Ok(())
}

pub async fn unregister_push_token(pool: &DbPool, device_id: Uuid) -> anyhow::Result<bool> {
    let rows = sqlx::query("DELETE FROM push_tokens WHERE device_id = $1")
        .bind(device_id)
        .execute(pool.as_ref())
        .await
        .context("unregister push token")?
        .rows_affected();

    Ok(rows > 0)
}

pub async fn get_push_tokens_for_user(
    pool: &DbPool,
    user_id: Uuid,
) -> anyhow::Result<Vec<PushTokenRow>> {
    sqlx::query_as::<_, PushTokenRow>(
        "SELECT id, user_id, device_id, platform, token, created_at
         FROM push_tokens WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_all(pool.as_ref())
    .await
    .context("get push tokens for user")
}

pub async fn mute_conversation(
    pool: &DbPool,
    user_id: Uuid,
    conversation_id: Uuid,
    muted_until: Option<DateTime<Utc>>,
) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO conversation_mutes (user_id, conversation_id, muted_until)
         VALUES ($1, $2, $3)
         ON CONFLICT (user_id, conversation_id) DO UPDATE SET muted_until = $3",
    )
    .bind(user_id)
    .bind(conversation_id)
    .bind(muted_until)
    .execute(pool.as_ref())
    .await
    .context("mute conversation")?;

    Ok(())
}

pub async fn unmute_conversation(
    pool: &DbPool,
    user_id: Uuid,
    conversation_id: Uuid,
) -> anyhow::Result<bool> {
    let rows =
        sqlx::query("DELETE FROM conversation_mutes WHERE user_id = $1 AND conversation_id = $2")
            .bind(user_id)
            .bind(conversation_id)
            .execute(pool.as_ref())
            .await
            .context("unmute conversation")?
            .rows_affected();

    Ok(rows > 0)
}

pub async fn is_conversation_muted(
    pool: &DbPool,
    user_id: Uuid,
    conversation_id: Uuid,
) -> anyhow::Result<bool> {
    let muted = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(
            SELECT 1 FROM conversation_mutes
            WHERE user_id = $1 AND conversation_id = $2
              AND (muted_until IS NULL OR muted_until > NOW())
        )",
    )
    .bind(user_id)
    .bind(conversation_id)
    .fetch_one(pool.as_ref())
    .await
    .context("check conversation mute")?;

    Ok(muted)
}

pub async fn get_conversation_member_ids(
    pool: &DbPool,
    conversation_id: Uuid,
) -> anyhow::Result<Vec<Uuid>> {
    sqlx::query_scalar::<_, Uuid>(
        "SELECT user_id FROM conversation_members WHERE conversation_id = $1",
    )
    .bind(conversation_id)
    .fetch_all(pool.as_ref())
    .await
    .context("get conversation member ids")
}

/// SECURITY: Push payload contains only metadata (conversation_id, sender_id).
/// No message content is included — clients must fetch & decrypt via E2EE.
pub async fn send_push_notification(
    pool: &DbPool,
    hub: &Hub,
    conversation_id: Uuid,
    sender_id: Uuid,
) -> anyhow::Result<()> {
    let member_ids = get_conversation_member_ids(pool, conversation_id).await?;

    let payload = PushPayload {
        payload_type: "new_message".to_string(),
        conversation_id,
        sender_id,
    };
    let payload_json = serde_json::to_string(&payload).context("serialize push payload")?;

    for member_id in member_ids {
        if member_id == sender_id {
            continue;
        }

        if hub.is_user_connected(member_id).await {
            continue;
        }

        if is_conversation_muted(pool, member_id, conversation_id)
            .await
            .unwrap_or(false)
        {
            continue;
        }

        let tokens = get_push_tokens_for_user(pool, member_id).await?;
        for token_row in &tokens {
            tracing::info!(
                "[PUSH] would send to {}: {}",
                token_row.platform,
                payload_json
            );
        }
    }

    Ok(())
}
