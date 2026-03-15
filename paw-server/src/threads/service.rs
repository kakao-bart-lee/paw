use crate::db::DbPool;

use sqlx::Row;
use uuid::Uuid;

use super::models::Thread;

pub const MAX_THREADS_PER_CONVERSATION: i64 = 100;

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

    let thread_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*)::BIGINT FROM threads WHERE conversation_id = $1 AND archived_at IS NULL",
    )
    .bind(conversation_id)
    .fetch_one(pool.as_ref())
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
    .fetch_optional(pool.as_ref())
    .await?;

    let Some(root_message) = root_message else {
        return Err(CreateThreadError::RootMessageNotFound);
    };

    let root_thread_id: Option<Uuid> = root_message.try_get("thread_id").unwrap_or(None);
    if root_thread_id.is_some() {
        return Err(CreateThreadError::RootMessageMustBeMainTimeline);
    }

    sqlx::query_as::<_, Thread>(
        "INSERT INTO threads (conversation_id, root_message_id, title, created_by)
         VALUES ($1, $2, $3, $4)
         RETURNING id, conversation_id, root_message_id, title, created_by, message_count, last_seq, last_message_at, created_at",
    )
    .bind(conversation_id)
    .bind(root_message_id)
    .bind(normalize_title(title))
    .bind(created_by)
    .fetch_one(pool.as_ref())
    .await
    .map_err(|error| match error {
        sqlx::Error::Database(db_error) if db_error.code().as_deref() == Some("23505") => {
            CreateThreadError::ThreadAlreadyExists
        }
        other => CreateThreadError::Database(other),
    })
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
    use super::normalize_title;

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
}
