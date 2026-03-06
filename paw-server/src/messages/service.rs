use crate::db::DbPool;
use crate::messages::models::{
    ConversationCreateResult, ConversationListItem, Message, MessageSendResult,
};
use anyhow::Context;
use sqlx::{Postgres, Transaction};
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Membership {
    Member,
    NotMember,
    ConversationNotFound,
}

pub async fn check_member(
    pool: &DbPool,
    conversation_id: Uuid,
    user_id: Uuid,
) -> anyhow::Result<Membership> {
    let exists = sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM conversations WHERE id = $1)")
        .bind(conversation_id)
        .fetch_one(pool.as_ref())
        .await
        .context("check conversation exists")?;

    if !exists {
        return Ok(Membership::ConversationNotFound);
    }

    let is_member = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM conversation_members WHERE conversation_id = $1 AND user_id = $2)",
    )
    .bind(conversation_id)
    .bind(user_id)
    .fetch_one(pool.as_ref())
    .await
    .context("check conversation membership")?;

    if is_member {
        Ok(Membership::Member)
    } else {
        Ok(Membership::NotMember)
    }
}

pub async fn get_idempotent_message(
    pool: &DbPool,
    conversation_id: Uuid,
    sender_id: Uuid,
    idempotency_key: Uuid,
) -> anyhow::Result<Option<MessageSendResult>> {
    sqlx::query_as::<_, MessageSendResult>(
        "SELECT id, seq, created_at
         FROM messages
         WHERE conversation_id = $1 AND sender_id = $2 AND idempotency_key = $3
         LIMIT 1",
    )
    .bind(conversation_id)
    .bind(sender_id)
    .bind(idempotency_key)
    .fetch_optional(pool.as_ref())
    .await
    .context("load idempotent message")
}

pub async fn send_message(
    pool: &DbPool,
    conversation_id: Uuid,
    sender_id: Uuid,
    content: &str,
    format: &str,
    idempotency_key: Uuid,
) -> anyhow::Result<MessageSendResult> {
    sqlx::query_as::<_, MessageSendResult>(
        "INSERT INTO messages (conversation_id, sender_id, seq, content, format, idempotency_key)
         VALUES ($1, $2, next_message_seq($1), $3, $4, $5)
         RETURNING id, seq, created_at",
    )
    .bind(conversation_id)
    .bind(sender_id)
    .bind(content)
    .bind(format)
    .bind(idempotency_key)
    .fetch_one(pool.as_ref())
    .await
    .context("insert message")
}

pub async fn get_messages(
    pool: &DbPool,
    conversation_id: Uuid,
    after_seq: i64,
    limit: i64,
) -> anyhow::Result<Vec<Message>> {
    let max_limit = limit.clamp(1, 50);

    sqlx::query_as::<_, Message>(
        "SELECT id, conversation_id, sender_id, content, format, seq, created_at
         FROM messages
         WHERE conversation_id = $1 AND seq > $2
         ORDER BY seq ASC
         LIMIT $3",
    )
    .bind(conversation_id)
    .bind(after_seq)
    .bind(max_limit)
    .fetch_all(pool.as_ref())
    .await
    .context("fetch conversation messages")
}

pub async fn list_conversations(
    pool: &DbPool,
    user_id: Uuid,
) -> anyhow::Result<Vec<ConversationListItem>> {
    sqlx::query_as::<_, ConversationListItem>(
        "SELECT
            c.id,
            c.title AS name,
            lm.content AS last_message,
            COALESCE((
                SELECT COUNT(*)::BIGINT
                FROM messages m2
                WHERE m2.conversation_id = c.id
                  AND m2.seq > cm.last_read_seq
            ), 0) AS unread_count
         FROM conversation_members cm
         JOIN conversations c ON c.id = cm.conversation_id
         LEFT JOIN LATERAL (
            SELECT content
            FROM messages
            WHERE conversation_id = c.id
            ORDER BY seq DESC
            LIMIT 1
         ) lm ON true
         WHERE cm.user_id = $1
         ORDER BY c.last_message_at DESC NULLS LAST, c.created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool.as_ref())
    .await
    .context("list user conversations")
}

pub async fn create_conversation(
    pool: &DbPool,
    creator_id: Uuid,
    member_ids: Vec<Uuid>,
    name: Option<String>,
) -> anyhow::Result<ConversationCreateResult> {
    let mut tx = pool.begin().await.context("begin transaction")?;

    let mut all_members: HashSet<Uuid> = member_ids.into_iter().collect();
    all_members.insert(creator_id);
    let total_members = all_members.len();

    let conv_type = if total_members == 2 && name.as_deref().unwrap_or_default().is_empty() {
        "direct"
    } else {
        "group"
    };

    let normalized_name = name.and_then(|value| {
        let trimmed = value.trim().to_owned();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    });

    let created = sqlx::query_as::<_, ConversationCreateResult>(
        "INSERT INTO conversations (type, title, created_by)
         VALUES ($1, $2, $3)
         RETURNING id, created_at",
    )
    .bind(conv_type)
    .bind(normalized_name)
    .bind(creator_id)
    .fetch_one(&mut *tx)
    .await
    .context("insert conversation")?;

    insert_members(&mut tx, created.id, creator_id, &all_members).await?;

    tx.commit().await.context("commit conversation transaction")?;
    Ok(created)
}

async fn insert_members(
    tx: &mut Transaction<'_, Postgres>,
    conversation_id: Uuid,
    creator_id: Uuid,
    members: &HashSet<Uuid>,
) -> anyhow::Result<()> {
    for member_id in members {
        let role = if *member_id == creator_id {
            "owner"
        } else {
            "member"
        };

        sqlx::query(
            "INSERT INTO conversation_members (conversation_id, user_id, role)
             VALUES ($1, $2, $3)
             ON CONFLICT (conversation_id, user_id) DO NOTHING",
        )
        .bind(conversation_id)
        .bind(member_id)
        .bind(role)
        .execute(&mut **tx)
        .await
        .context("insert conversation member")?;
    }

    Ok(())
}
