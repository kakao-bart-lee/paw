use crate::db::DbPool;
use crate::messages::models::{
    ConversationCreateResult, ConversationListItem, MediaAttachment, Message, MessageAttachment,
    MessageSendResult,
};
use anyhow::{anyhow, Context};
use sqlx::{Postgres, Transaction};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

pub const MAX_GROUP_MEMBERS: usize = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Membership {
    Member,
    NotMember,
    ConversationNotFound,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupManagementError {
    ConversationNotFound,
    NotAuthorized,
    TooManyMembers,
    AlreadyMember,
    MemberNotFound,
    CannotRemoveLastOwner,
    InvalidGroupName,
}

pub async fn check_member(
    pool: &DbPool,
    conversation_id: Uuid,
    user_id: Uuid,
) -> anyhow::Result<Membership> {
    let exists =
        sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM conversations WHERE id = $1)")
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
    thread_id: Option<Uuid>,
    content: &str,
    format: &str,
    idempotency_key: Uuid,
    attachments: &[MediaAttachment],
) -> anyhow::Result<MessageSendResult> {
    let mut tx = pool.begin().await.context("begin message transaction")?;

    let created = sqlx::query_as::<_, MessageSendResult>(
        "INSERT INTO messages (conversation_id, sender_id, thread_id, seq, content, format, idempotency_key)
         VALUES ($1, $2, $3, next_message_seq($1), $4, $5, $6)
         RETURNING id, seq, created_at",
    )
    .bind(conversation_id)
    .bind(sender_id)
    .bind(thread_id)
    .bind(content)
    .bind(format)
    .bind(idempotency_key)
    .fetch_one(&mut *tx)
    .await
    .context("insert message")?;

    for attachment in attachments {
        sqlx::query(
            "INSERT INTO message_attachments (
                 message_id,
                 file_type,
                 file_url,
                 file_size,
                 mime_type,
                 thumbnail_url
             )
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(created.id)
        .bind(&attachment.media_type)
        .bind(&attachment.s3_key)
        .bind(attachment.file_size)
        .bind(&attachment.mime_type)
        .bind(&attachment.thumbnail_s3_key)
        .execute(&mut *tx)
        .await
        .context("insert message attachment")?;
    }

    tx.commit().await.context("commit message transaction")?;
    Ok(created)
}

pub async fn fetch_media_attachments(
    pool: &DbPool,
    uploader_id: Uuid,
    attachment_ids: &[Uuid],
) -> anyhow::Result<Vec<MediaAttachment>> {
    if attachment_ids.is_empty() {
        return Ok(Vec::new());
    }

    let rows = sqlx::query_as::<_, MediaAttachment>(
        "SELECT id, media_type, mime_type, file_size, s3_key, thumbnail_s3_key
         FROM media_attachments
         WHERE uploader_id = $1 AND id = ANY($2)",
    )
    .bind(uploader_id)
    .bind(attachment_ids)
    .fetch_all(pool.as_ref())
    .await
    .context("load media attachments")?;

    let by_id: HashMap<Uuid, MediaAttachment> = rows.into_iter().map(|row| (row.id, row)).collect();

    let mut ordered = Vec::with_capacity(attachment_ids.len());
    for attachment_id in attachment_ids {
        let Some(attachment) = by_id.get(attachment_id) else {
            return Err(anyhow!("attachment_not_found_or_not_owned"));
        };
        ordered.push(attachment.clone());
    }

    Ok(ordered)
}

pub async fn list_message_attachments(
    pool: &DbPool,
    message_id: Uuid,
) -> anyhow::Result<Vec<MessageAttachment>> {
    sqlx::query_as::<_, MessageAttachment>(
        "SELECT id, message_id, file_type, file_url, file_size, mime_type, thumbnail_url, created_at
         FROM message_attachments
         WHERE message_id = $1
         ORDER BY created_at ASC",
    )
    .bind(message_id)
    .fetch_all(pool.as_ref())
    .await
    .context("list message attachments")
}

pub async fn list_message_attachments_map(
    pool: &DbPool,
    message_ids: &[Uuid],
) -> anyhow::Result<HashMap<Uuid, Vec<MessageAttachment>>> {
    if message_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let rows = sqlx::query_as::<_, MessageAttachment>(
        "SELECT id, message_id, file_type, file_url, file_size, mime_type, thumbnail_url, created_at
         FROM message_attachments
         WHERE message_id = ANY($1)
         ORDER BY created_at ASC",
    )
    .bind(message_ids)
    .fetch_all(pool.as_ref())
    .await
    .context("list message attachments map")?;

    let mut by_message_id: HashMap<Uuid, Vec<MessageAttachment>> = HashMap::new();
    for row in rows {
        by_message_id.entry(row.message_id).or_default().push(row);
    }

    Ok(by_message_id)
}

pub async fn get_messages(
    pool: &DbPool,
    conversation_id: Uuid,
    after_seq: i64,
    limit: i64,
) -> anyhow::Result<Vec<Message>> {
    let max_limit = limit.clamp(1, 50);

    sqlx::query_as::<_, Message>(
        "SELECT id, conversation_id, thread_id, sender_id, content, format, seq, created_at
         FROM messages
         WHERE conversation_id = $1 AND seq > $2 AND is_deleted = FALSE
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
            WHERE conversation_id = c.id AND is_deleted = FALSE
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

    if all_members.len() > MAX_GROUP_MEMBERS {
        return Err(anyhow!("too_many_members"));
    }

    insert_members(&mut tx, created.id, creator_id, &all_members).await?;

    tx.commit()
        .await
        .context("commit conversation transaction")?;
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

pub async fn add_member(
    pool: &DbPool,
    conversation_id: Uuid,
    requester_id: Uuid,
    new_user_id: Uuid,
) -> Result<bool, GroupManagementError> {
    let requester_role = get_role(pool, conversation_id, requester_id)
        .await
        .map_err(|_| GroupManagementError::ConversationNotFound)?;

    match requester_role.as_deref() {
        Some("owner") | Some("admin") => {}
        Some(_) | None => return Err(GroupManagementError::NotAuthorized),
    }

    let conv_row = sqlx::query_as::<_, (i32, i64)>(
        "SELECT c.max_members, COUNT(cm.user_id)::BIGINT
         FROM conversations c
         LEFT JOIN conversation_members cm ON cm.conversation_id = c.id
         WHERE c.id = $1
         GROUP BY c.max_members",
    )
    .bind(conversation_id)
    .fetch_optional(pool.as_ref())
    .await
    .map_err(|_| GroupManagementError::ConversationNotFound)?;

    let Some((max_members, current_count)) = conv_row else {
        return Err(GroupManagementError::ConversationNotFound);
    };

    if current_count >= i64::from(max_members) {
        return Err(GroupManagementError::TooManyMembers);
    }

    let inserted = sqlx::query(
        "INSERT INTO conversation_members (conversation_id, user_id, role)
         VALUES ($1, $2, 'member')
         ON CONFLICT (conversation_id, user_id) DO NOTHING",
    )
    .bind(conversation_id)
    .bind(new_user_id)
    .execute(pool.as_ref())
    .await
    .map_err(|_| GroupManagementError::ConversationNotFound)?
    .rows_affected();

    if inserted == 0 {
        return Err(GroupManagementError::AlreadyMember);
    }

    Ok(true)
}

pub async fn remove_member(
    pool: &DbPool,
    conversation_id: Uuid,
    requester_id: Uuid,
    target_user_id: Uuid,
) -> Result<bool, GroupManagementError> {
    let requester_role = get_role(pool, conversation_id, requester_id)
        .await
        .map_err(|_| GroupManagementError::ConversationNotFound)?;

    if requester_id != target_user_id && requester_role.as_deref() != Some("owner") {
        return Err(GroupManagementError::NotAuthorized);
    }

    let target_role = get_role(pool, conversation_id, target_user_id)
        .await
        .map_err(|_| GroupManagementError::ConversationNotFound)?;

    let Some(target_role) = target_role else {
        return Err(GroupManagementError::MemberNotFound);
    };

    if target_role == "owner" {
        let owner_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*)::BIGINT
             FROM conversation_members
             WHERE conversation_id = $1 AND role = 'owner'",
        )
        .bind(conversation_id)
        .fetch_one(pool.as_ref())
        .await
        .map_err(|_| GroupManagementError::ConversationNotFound)?;

        if owner_count <= 1 {
            return Err(GroupManagementError::CannotRemoveLastOwner);
        }
    }

    let removed = sqlx::query(
        "DELETE FROM conversation_members
         WHERE conversation_id = $1 AND user_id = $2",
    )
    .bind(conversation_id)
    .bind(target_user_id)
    .execute(pool.as_ref())
    .await
    .map_err(|_| GroupManagementError::ConversationNotFound)?
    .rows_affected();

    if removed == 0 {
        return Err(GroupManagementError::MemberNotFound);
    }

    Ok(true)
}

pub async fn update_group_name(
    pool: &DbPool,
    conversation_id: Uuid,
    requester_id: Uuid,
    name: &str,
) -> Result<bool, GroupManagementError> {
    let requester_role = get_role(pool, conversation_id, requester_id)
        .await
        .map_err(|_| GroupManagementError::ConversationNotFound)?;

    match requester_role.as_deref() {
        Some("owner") | Some("admin") => {}
        Some(_) | None => return Err(GroupManagementError::NotAuthorized),
    }

    let normalized = name.trim();
    if normalized.is_empty() {
        return Err(GroupManagementError::InvalidGroupName);
    }

    let updated = sqlx::query(
        "UPDATE conversations
         SET title = $2
         WHERE id = $1 AND type = 'group'",
    )
    .bind(conversation_id)
    .bind(normalized)
    .execute(pool.as_ref())
    .await
    .map_err(|_| GroupManagementError::ConversationNotFound)?
    .rows_affected();

    if updated == 0 {
        return Err(GroupManagementError::ConversationNotFound);
    }

    Ok(true)
}

async fn get_role(
    pool: &DbPool,
    conversation_id: Uuid,
    user_id: Uuid,
) -> anyhow::Result<Option<String>> {
    let conversation_exists =
        sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM conversations WHERE id = $1)")
            .bind(conversation_id)
            .fetch_one(pool.as_ref())
            .await
            .context("check conversation exists")?;

    if !conversation_exists {
        return Err(anyhow!("conversation_not_found"));
    }

    sqlx::query_scalar::<_, String>(
        "SELECT role FROM conversation_members WHERE conversation_id = $1 AND user_id = $2",
    )
    .bind(conversation_id)
    .bind(user_id)
    .fetch_optional(pool.as_ref())
    .await
    .context("load member role")
}

pub async fn delete_message(
    pool: &DbPool,
    conversation_id: Uuid,
    message_id: Uuid,
) -> anyhow::Result<bool> {
    let deleted = sqlx::query(
        "DELETE FROM messages
         WHERE id = $1 AND conversation_id = $2",
    )
    .bind(message_id)
    .bind(conversation_id)
    .execute(pool.as_ref())
    .await
    .context("delete message")?
    .rows_affected();

    Ok(deleted > 0)
}
