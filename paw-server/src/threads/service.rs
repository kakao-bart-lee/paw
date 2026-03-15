use crate::db::DbPool;
use crate::messages::models::Message;

use sqlx::Row;
use uuid::Uuid;

use super::models::{Thread, ThreadStateSnapshot};

pub const MAX_THREADS_PER_CONVERSATION: i64 = 100;
pub const MAX_THREAD_MESSAGE_HISTORY_LIMIT: i64 = 100;

#[derive(Debug)]
pub enum CreateThreadError {
    ConversationNotFound,
    ThreadsNotAllowed,
    RootMessageNotFound,
    RootMessageMustBeMainTimeline,
    ThreadAlreadyExists,
    ThreadLimitExceeded,
    Database(sqlx::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadReadError {
    ThreadNotFound,
    Database,
}

impl From<sqlx::Error> for CreateThreadError {
    fn from(error: sqlx::Error) -> Self {
        Self::Database(error)
    }
}

pub async fn list_threads(
    pool: &DbPool,
    conversation_id: Uuid,
) -> Result<Vec<Thread>, sqlx::Error> {
    sqlx::query_as::<_, Thread>(
        "SELECT id, conversation_id, root_message_id, title, created_by, message_count, last_seq, last_message_at, created_at
         FROM threads
         WHERE conversation_id = $1 AND archived_at IS NULL
         ORDER BY created_at ASC",
    )
    .bind(conversation_id)
    .fetch_all(pool.as_ref())
    .await
}

pub async fn get_thread(
    pool: &DbPool,
    conversation_id: Uuid,
    thread_id: Uuid,
) -> Result<Option<Thread>, sqlx::Error> {
    sqlx::query_as::<_, Thread>(
        "SELECT id, conversation_id, root_message_id, title, created_by, message_count, last_seq, last_message_at, created_at
         FROM threads
         WHERE id = $1 AND conversation_id = $2 AND archived_at IS NULL",
    )
    .bind(thread_id)
    .bind(conversation_id)
    .fetch_optional(pool.as_ref())
    .await
}

pub async fn create_thread(
    pool: &DbPool,
    conversation_id: Uuid,
    root_message_id: Uuid,
    created_by: Uuid,
    title: Option<String>,
) -> Result<Thread, CreateThreadError> {
    let conversation_type =
        sqlx::query_scalar::<_, Option<String>>("SELECT type FROM conversations WHERE id = $1")
            .bind(conversation_id)
            .fetch_optional(pool.as_ref())
            .await?
            .flatten();

    match conversation_type.as_deref() {
        Some("group") => {}
        Some(_) => return Err(CreateThreadError::ThreadsNotAllowed),
        None => return Err(CreateThreadError::ConversationNotFound),
    }

    let mut transaction = pool.begin().await?;

    let thread_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*)::BIGINT FROM threads WHERE conversation_id = $1 AND archived_at IS NULL",
    )
    .bind(conversation_id)
    .fetch_one(&mut *transaction)
    .await?;

    if thread_count >= MAX_THREADS_PER_CONVERSATION {
        return Err(CreateThreadError::ThreadLimitExceeded);
    }

    let root_message = sqlx::query(
        "SELECT thread_id
         FROM messages
         WHERE id = $1 AND conversation_id = $2 AND is_deleted = FALSE",
    )
    .bind(root_message_id)
    .bind(conversation_id)
    .fetch_optional(&mut *transaction)
    .await?;

    let Some(root_message) = root_message else {
        return Err(CreateThreadError::RootMessageNotFound);
    };

    let root_thread_id: Option<Uuid> = root_message.try_get("thread_id").unwrap_or(None);
    if root_thread_id.is_some() {
        return Err(CreateThreadError::RootMessageMustBeMainTimeline);
    }

    let thread = sqlx::query_as::<_, Thread>(
        "INSERT INTO threads (conversation_id, root_message_id, title, created_by)
         VALUES ($1, $2, $3, $4)
         RETURNING id, conversation_id, root_message_id, title, created_by, message_count, last_seq, last_message_at, created_at",
    )
    .bind(conversation_id)
    .bind(root_message_id)
    .bind(normalize_title(title))
    .bind(created_by)
    .fetch_one(&mut *transaction)
    .await
    .map_err(|error| match error {
        sqlx::Error::Database(db_error) if db_error.code().as_deref() == Some("23505") => {
            CreateThreadError::ThreadAlreadyExists
        }
        other => CreateThreadError::Database(other),
    })?;

    sqlx::query(
        "INSERT INTO thread_read_state (thread_id, user_id, last_read_seq, updated_at)
         VALUES ($1, $2, 0, NOW())
         ON CONFLICT (thread_id, user_id)
         DO UPDATE SET updated_at = NOW()",
    )
    .bind(thread.id)
    .bind(created_by)
    .execute(&mut *transaction)
    .await?;

    transaction.commit().await?;

    Ok(thread)
}

pub async fn update_thread_title(
    pool: &DbPool,
    conversation_id: Uuid,
    thread_id: Uuid,
    title: Option<String>,
) -> Result<Option<Thread>, sqlx::Error> {
    sqlx::query_as::<_, Thread>(
        "UPDATE threads
         SET title = $3
         WHERE id = $1 AND conversation_id = $2 AND archived_at IS NULL
         RETURNING id, conversation_id, root_message_id, title, created_by, message_count, last_seq, last_message_at, created_at",
    )
    .bind(thread_id)
    .bind(conversation_id)
    .bind(normalize_title(title))
    .fetch_optional(pool.as_ref())
    .await
}

pub async fn archive_thread(
    pool: &DbPool,
    conversation_id: Uuid,
    thread_id: Uuid,
    archived_by: Uuid,
) -> Result<bool, sqlx::Error> {
    let mut transaction = pool.begin().await?;

    let archived = sqlx::query(
        "UPDATE threads
         SET archived_at = NOW(),
             archived_by = $3
         WHERE id = $1 AND conversation_id = $2 AND archived_at IS NULL",
    )
    .bind(thread_id)
    .bind(conversation_id)
    .bind(archived_by)
    .execute(&mut *transaction)
    .await?
    .rows_affected()
        > 0;

    if archived {
        sqlx::query(
            "DELETE FROM thread_agents
             WHERE thread_id = $1 AND conversation_id = $2",
        )
        .bind(thread_id)
        .bind(conversation_id)
        .execute(&mut *transaction)
        .await?;
    }

    transaction.commit().await?;

    Ok(archived)
}

pub async fn get_thread_messages(
    pool: &DbPool,
    conversation_id: Uuid,
    thread_id: Uuid,
    since_seq: i64,
    limit: i64,
) -> Result<Vec<Message>, sqlx::Error> {
    sqlx::query_as::<_, Message>(
        "SELECT id, conversation_id, thread_id, thread_seq, sender_id, content, format, seq, created_at, forwarded_from
         FROM messages
         WHERE conversation_id = $1
           AND thread_id = $2
           AND thread_seq > $3
           AND is_deleted = FALSE
         ORDER BY thread_seq ASC, seq ASC
         LIMIT $4",
    )
    .bind(conversation_id)
    .bind(thread_id)
    .bind(sanitize_seq(since_seq))
    .bind(clamp_thread_message_limit(limit))
    .fetch_all(pool.as_ref())
    .await
}

pub async fn get_thread_state(
    pool: &DbPool,
    conversation_id: Uuid,
    thread_id: Uuid,
) -> Result<Option<ThreadStateSnapshot>, sqlx::Error> {
    let thread = sqlx::query(
        "SELECT id, message_count, COALESCE(last_seq, 0) AS last_seq, last_message_at
         FROM threads
         WHERE id = $1 AND conversation_id = $2 AND archived_at IS NULL",
    )
    .bind(thread_id)
    .bind(conversation_id)
    .fetch_optional(pool.as_ref())
    .await?;

    let Some(thread) = thread else {
        return Ok(None);
    };

    let participant_rows = sqlx::query(
        "SELECT user_id
         FROM thread_read_state
         WHERE thread_id = $1
         ORDER BY updated_at ASC, user_id ASC",
    )
    .bind(thread_id)
    .fetch_all(pool.as_ref())
    .await?;

    let participants = participant_rows
        .into_iter()
        .filter_map(|row| row.try_get("user_id").ok())
        .collect();

    Ok(Some(ThreadStateSnapshot {
        thread_id: thread.try_get("id")?,
        message_count: thread.try_get("message_count")?,
        last_seq: thread.try_get("last_seq")?,
        participants,
        last_message_at: thread.try_get("last_message_at")?,
    }))
}

pub async fn join_thread(
    pool: &DbPool,
    conversation_id: Uuid,
    thread_id: Uuid,
    user_id: Uuid,
) -> Result<bool, sqlx::Error> {
    if !thread_exists(pool, conversation_id, thread_id).await? {
        return Ok(false);
    }

    sqlx::query(
        "INSERT INTO thread_read_state (thread_id, user_id, last_read_seq, updated_at)
         VALUES ($1, $2, 0, NOW())
         ON CONFLICT (thread_id, user_id)
         DO UPDATE SET updated_at = NOW()",
    )
    .bind(thread_id)
    .bind(user_id)
    .execute(pool.as_ref())
    .await?;

    Ok(true)
}

pub async fn leave_thread(
    pool: &DbPool,
    conversation_id: Uuid,
    thread_id: Uuid,
    user_id: Uuid,
) -> Result<bool, sqlx::Error> {
    if !thread_exists(pool, conversation_id, thread_id).await? {
        return Ok(false);
    }

    sqlx::query(
        "DELETE FROM thread_read_state
         WHERE thread_id = $1 AND user_id = $2",
    )
    .bind(thread_id)
    .bind(user_id)
    .execute(pool.as_ref())
    .await?;

    Ok(true)
}

pub async fn mark_thread_read(
    pool: &DbPool,
    conversation_id: Uuid,
    thread_id: Uuid,
    user_id: Uuid,
    last_read_seq: i64,
) -> Result<(), ThreadReadError> {
    let mut transaction = pool.begin().await.map_err(|_| ThreadReadError::Database)?;

    let thread_last_seq = sqlx::query_scalar::<_, Option<i64>>(
        "SELECT last_seq
         FROM threads
         WHERE id = $1 AND conversation_id = $2 AND archived_at IS NULL",
    )
    .bind(thread_id)
    .bind(conversation_id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|_| ThreadReadError::Database)?
    .flatten()
    .ok_or(ThreadReadError::ThreadNotFound)?;

    let bounded_last_read_seq = sanitize_seq(last_read_seq).min(thread_last_seq.max(0));

    sqlx::query(
        "INSERT INTO thread_read_state (thread_id, user_id, last_read_seq, updated_at)
         VALUES ($1, $2, $3, NOW())
         ON CONFLICT (thread_id, user_id)
         DO UPDATE SET
             last_read_seq = GREATEST(thread_read_state.last_read_seq, EXCLUDED.last_read_seq),
             updated_at = NOW()",
    )
    .bind(thread_id)
    .bind(user_id)
    .bind(bounded_last_read_seq)
    .execute(&mut *transaction)
    .await
    .map_err(|_| ThreadReadError::Database)?;

    let conversation_seq = sqlx::query_scalar::<_, Option<i64>>(
        "SELECT seq
         FROM messages
         WHERE conversation_id = $1
           AND thread_id = $2
           AND thread_seq <= $3
           AND is_deleted = FALSE
         ORDER BY thread_seq DESC, seq DESC
         LIMIT 1",
    )
    .bind(conversation_id)
    .bind(thread_id)
    .bind(bounded_last_read_seq)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|_| ThreadReadError::Database)?
    .flatten();

    if let Some(conversation_seq) = conversation_seq {
        sqlx::query(
            "UPDATE conversation_members
             SET last_read_seq = GREATEST(last_read_seq, $3)
             WHERE conversation_id = $1 AND user_id = $2",
        )
        .bind(conversation_id)
        .bind(user_id)
        .bind(conversation_seq)
        .execute(&mut *transaction)
        .await
        .map_err(|_| ThreadReadError::Database)?;
    }

    transaction
        .commit()
        .await
        .map_err(|_| ThreadReadError::Database)
}

pub fn clamp_thread_message_limit(limit: i64) -> i64 {
    limit.clamp(1, MAX_THREAD_MESSAGE_HISTORY_LIMIT)
}

fn sanitize_seq(seq: i64) -> i64 {
    seq.max(0)
}

async fn thread_exists(pool: &DbPool, conversation_id: Uuid, thread_id: Uuid) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(
             SELECT 1
             FROM threads
             WHERE id = $1 AND conversation_id = $2 AND archived_at IS NULL
         )",
    )
    .bind(thread_id)
    .bind(conversation_id)
    .fetch_one(pool.as_ref())
    .await
}

fn normalize_title(title: Option<String>) -> Option<String> {
    title.and_then(|value| {
        let trimmed = value.trim().to_owned();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

#[cfg(test)]
mod tests {
    use super::{clamp_thread_message_limit, normalize_title};

    #[test]
    fn normalize_title_trims_surrounding_whitespace() {
        assert_eq!(
            normalize_title(Some("  thread title  ".to_string())),
            Some("thread title".to_string())
        );
    }

    #[test]
    fn normalize_title_returns_none_for_blank_values() {
        assert_eq!(normalize_title(Some("   \n\t  ".to_string())), None);
    }

    #[test]
    fn normalize_title_preserves_none() {
        assert_eq!(normalize_title(None), None);
    }

    #[test]
    fn clamp_thread_message_limit_stays_within_supported_range() {
        assert_eq!(clamp_thread_message_limit(-5), 1);
        assert_eq!(clamp_thread_message_limit(25), 25);
        assert_eq!(clamp_thread_message_limit(500), 100);
    }
}
