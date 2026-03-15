use crate::auth::{middleware::UserId, AppState};
use crate::i18n::{error_response, RequestLocale};
use crate::messages::models::Message;
use crate::messages::service::{self as message_service, Membership};
use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use paw_proto::{
    ContextThreadCreatedMsg, MessageFormat, MessageReceivedMsg, ServerMessage, ThreadAgentBoundMsg,
    ThreadAgentUnboundMsg, ThreadCreatedMsg, ThreadDeletedMsg, PROTOCOL_VERSION,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Thread {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub root_message_id: Uuid,
    pub title: Option<String>,
    pub created_by: Uuid,
    pub message_count: i32,
    pub last_message_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ThreadAgent {
    pub thread_id: Uuid,
    pub agent_id: Uuid,
    pub bound_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateThreadRequest {
    pub root_message_id: Uuid,
    pub title: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SendThreadMessageRequest {
    pub content: String,
    pub format: String,
    pub idempotency_key: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct BindAgentRequest {
    pub agent_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct ListThreadMessagesQuery {
    pub before: Option<chrono::DateTime<chrono::Utc>>,
    pub limit: Option<i64>,
}

fn error(status: StatusCode, code: &str, locale: &str, message: &str) -> (StatusCode, Json<Value>) {
    error_response(status, code, locale, message)
}

async fn ensure_membership(
    state: &AppState,
    conversation_id: Uuid,
    user_id: Uuid,
    locale: &str,
) -> Result<(), Response> {
    match message_service::check_member(&state.db, conversation_id, user_id).await {
        Ok(Membership::Member) => Ok(()),
        Ok(Membership::NotMember) => Err(error(
            StatusCode::FORBIDDEN,
            "forbidden",
            locale,
            "User is not a member of this conversation",
        )
        .into_response()),
        Ok(Membership::ConversationNotFound) => Err(error(
            StatusCode::NOT_FOUND,
            "conversation_not_found",
            locale,
            "Conversation not found",
        )
        .into_response()),
        Err(err) => {
            tracing::error!(%err, conversation_id = %conversation_id, user_id = %user_id, "failed membership check");
            Err(error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "membership_check_failed",
                locale,
                "Could not validate conversation membership",
            )
            .into_response())
        }
    }
}

async fn is_group_conversation(state: &AppState, conversation_id: Uuid) -> anyhow::Result<bool> {
    let value =
        sqlx::query_scalar::<_, Option<String>>("SELECT type FROM conversations WHERE id = $1")
            .bind(conversation_id)
            .fetch_optional(state.db.as_ref())
            .await?;

    Ok(matches!(value.flatten().as_deref(), Some("group")))
}

async fn get_thread_row(
    state: &AppState,
    conversation_id: Uuid,
    thread_id: Uuid,
) -> anyhow::Result<Option<Thread>> {
    sqlx::query_as::<_, Thread>(
        "SELECT id, conversation_id, root_message_id, title, created_by, message_count, last_message_at, created_at
         FROM threads
         WHERE id = $1 AND conversation_id = $2",
    )
    .bind(thread_id)
    .bind(conversation_id)
    .fetch_optional(state.db.as_ref())
    .await
    .map_err(Into::into)
}

async fn conversation_member_ids(
    state: &AppState,
    conversation_id: Uuid,
) -> anyhow::Result<Vec<Uuid>> {
    sqlx::query_scalar::<_, Uuid>(
        "SELECT user_id FROM conversation_members WHERE conversation_id = $1",
    )
    .bind(conversation_id)
    .fetch_all(state.db.as_ref())
    .await
    .map_err(Into::into)
}

async fn broadcast_to_conversation(
    state: &AppState,
    conversation_id: Uuid,
    message: &ServerMessage,
) -> anyhow::Result<()> {
    let user_ids = conversation_member_ids(state, conversation_id).await?;
    let msg = serde_json::to_string(message)?;
    state.hub.broadcast_to_conversation(user_ids, &msg).await;
    Ok(())
}

pub async fn list_threads(
    State(state): State<AppState>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path(conversation_id): Path<Uuid>,
) -> Response {
    match ensure_membership(&state, conversation_id, user_id, &locale).await {
        Ok(()) => {}
        Err(resp) => return resp,
    }

    match sqlx::query_as::<_, Thread>(
        "SELECT id, conversation_id, root_message_id, title, created_by, message_count, last_message_at, created_at
         FROM threads
         WHERE conversation_id = $1
         ORDER BY created_at ASC",
    )
    .bind(conversation_id)
    .fetch_all(state.db.as_ref())
    .await
    {
        Ok(rows) => Json(rows).into_response(),
        Err(err) => {
            tracing::error!(%err, conversation_id = %conversation_id, "failed to list threads");
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "query_failed",
                &locale,
                "Could not list threads",
            )
            .into_response()
        }
    }
}

pub async fn create_thread(
    State(state): State<AppState>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path(conversation_id): Path<Uuid>,
    Json(payload): Json<CreateThreadRequest>,
) -> Response {
    match ensure_membership(&state, conversation_id, user_id, &locale).await {
        Ok(()) => {}
        Err(resp) => return resp,
    }

    match is_group_conversation(&state, conversation_id).await {
        Ok(true) => {}
        Ok(false) => {
            return error(
                StatusCode::BAD_REQUEST,
                "threads_not_allowed",
                &locale,
                "Threads are only allowed in group conversations",
            )
            .into_response();
        }
        Err(err) => {
            tracing::error!(%err, conversation_id = %conversation_id, "failed to determine conversation type");
            return error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "query_failed",
                &locale,
                "Could not create thread",
            )
            .into_response();
        }
    }

    let thread_count = match sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*)::BIGINT FROM threads WHERE conversation_id = $1",
    )
    .bind(conversation_id)
    .fetch_one(state.db.as_ref())
    .await
    {
        Ok(count) => count,
        Err(err) => {
            tracing::error!(%err, conversation_id = %conversation_id, "failed counting threads");
            return error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "query_failed",
                &locale,
                "Could not create thread",
            )
            .into_response();
        }
    };

    if thread_count >= 100 {
        return error(
            StatusCode::CONFLICT,
            "thread_limit_exceeded",
            &locale,
            "Maximum 100 threads per conversation reached",
        )
        .into_response();
    }

    let root_message = match sqlx::query(
        "SELECT thread_id
         FROM messages
         WHERE id = $1 AND conversation_id = $2",
    )
    .bind(payload.root_message_id)
    .bind(conversation_id)
    .fetch_optional(state.db.as_ref())
    .await
    {
        Ok(Some(row)) => row,
        Ok(None) => {
            return error(
                StatusCode::BAD_REQUEST,
                "message_not_found",
                &locale,
                "Root message not found in this conversation",
            )
            .into_response();
        }
        Err(err) => {
            tracing::error!(%err, conversation_id = %conversation_id, root_message_id = %payload.root_message_id, "failed to validate root message");
            return error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "query_failed",
                &locale,
                "Could not create thread",
            )
            .into_response();
        }
    };

    let root_thread_id: Option<Uuid> = root_message.try_get("thread_id").unwrap_or(None);
    if root_thread_id.is_some() {
        return error(
            StatusCode::BAD_REQUEST,
            "message_not_found",
            &locale,
            "Root message must belong to the main timeline",
        )
        .into_response();
    }

    let created = match sqlx::query_as::<_, Thread>(
        "INSERT INTO threads (conversation_id, root_message_id, title, created_by)
         VALUES ($1, $2, $3, $4)
         RETURNING id, conversation_id, root_message_id, title, created_by, message_count, last_message_at, created_at",
    )
    .bind(conversation_id)
    .bind(payload.root_message_id)
    .bind(payload.title.as_deref().map(str::trim).filter(|value| !value.is_empty()))
    .bind(user_id)
    .fetch_one(state.db.as_ref())
    .await
    {
        Ok(thread) => thread,
        Err(sqlx::Error::Database(db_err)) if db_err.code().as_deref() == Some("23505") => {
            return error(
                StatusCode::CONFLICT,
                "thread_not_empty",
                &locale,
                "Thread already exists for this root message",
            )
            .into_response();
        }
        Err(err) => {
            tracing::error!(%err, conversation_id = %conversation_id, "failed to create thread");
            return error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "query_failed",
                &locale,
                "Could not create thread",
            )
            .into_response();
        }
    };

    let event = ServerMessage::ThreadCreated(ThreadCreatedMsg {
        v: PROTOCOL_VERSION,
        conversation_id,
        thread_id: created.id,
        root_message_id: created.root_message_id,
        title: created.title.clone(),
        created_by: user_id,
        created_at: created.created_at,
    });
    let _ = broadcast_to_conversation(&state, conversation_id, &event).await;

    let context_engine = state.context_engine.clone();
    let thread_id = created.id;
    let root_message_id = created.root_message_id;
    let title = created.title.clone();
    let occurred_at = created.created_at;
    tokio::spawn(async move {
        if let Err(err) = context_engine
            .on_thread_created(ContextThreadCreatedMsg {
                v: PROTOCOL_VERSION,
                conversation_id,
                thread_id,
                root_message_id,
                title,
                created_by: user_id,
                occurred_at,
            })
            .await
        {
            tracing::error!(%err, conversation_id = %conversation_id, thread_id = %thread_id, "context hook failed");
        }
    });

    (StatusCode::CREATED, Json(created)).into_response()
}

pub async fn get_thread(
    State(state): State<AppState>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path((conversation_id, thread_id)): Path<(Uuid, Uuid)>,
) -> Response {
    match ensure_membership(&state, conversation_id, user_id, &locale).await {
        Ok(()) => {}
        Err(resp) => return resp,
    }

    match get_thread_row(&state, conversation_id, thread_id).await {
        Ok(Some(thread)) => Json(thread).into_response(),
        Ok(None) => error(
            StatusCode::NOT_FOUND,
            "thread_not_found",
            &locale,
            "Thread not found",
        )
        .into_response(),
        Err(err) => {
            tracing::error!(%err, conversation_id = %conversation_id, thread_id = %thread_id, "failed to load thread");
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "query_failed",
                &locale,
                "Could not load thread",
            )
            .into_response()
        }
    }
}

pub async fn delete_thread(
    State(state): State<AppState>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path((conversation_id, thread_id)): Path<(Uuid, Uuid)>,
) -> Response {
    match ensure_membership(&state, conversation_id, user_id, &locale).await {
        Ok(()) => {}
        Err(resp) => return resp,
    }

    if get_thread_row(&state, conversation_id, thread_id)
        .await
        .ok()
        .flatten()
        .is_none()
    {
        return error(
            StatusCode::NOT_FOUND,
            "thread_not_found",
            &locale,
            "Thread not found",
        )
        .into_response();
    }

    let message_count = match sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*)::BIGINT FROM messages WHERE conversation_id = $1 AND thread_id = $2",
    )
    .bind(conversation_id)
    .bind(thread_id)
    .fetch_one(state.db.as_ref())
    .await
    {
        Ok(count) => count,
        Err(err) => {
            tracing::error!(%err, conversation_id = %conversation_id, thread_id = %thread_id, "failed checking thread message count");
            return error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "query_failed",
                &locale,
                "Could not delete thread",
            )
            .into_response();
        }
    };

    if message_count > 0 {
        return error(
            StatusCode::CONFLICT,
            "thread_not_empty",
            &locale,
            "Cannot delete a thread that still contains messages",
        )
        .into_response();
    }

    let unbound = match sqlx::query_as::<_, ThreadAgent>(
        "DELETE FROM thread_agents
         WHERE thread_id = $1 AND conversation_id = $2
         RETURNING thread_id, agent_id, bound_at",
    )
    .bind(thread_id)
    .bind(conversation_id)
    .fetch_all(state.db.as_ref())
    .await
    {
        Ok(bindings) => bindings,
        Err(err) => {
            tracing::error!(%err, conversation_id = %conversation_id, thread_id = %thread_id, "failed to unbind thread agents before delete");
            return error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "query_failed",
                &locale,
                "Could not delete thread",
            )
            .into_response();
        }
    };

    let deleted = match sqlx::query("DELETE FROM threads WHERE id = $1 AND conversation_id = $2")
        .bind(thread_id)
        .bind(conversation_id)
        .execute(state.db.as_ref())
        .await
    {
        Ok(result) => result.rows_affected() > 0,
        Err(err) => {
            tracing::error!(%err, conversation_id = %conversation_id, thread_id = %thread_id, "failed to delete thread");
            return error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "query_failed",
                &locale,
                "Could not delete thread",
            )
            .into_response();
        }
    };

    if !deleted {
        return error(
            StatusCode::NOT_FOUND,
            "thread_not_found",
            &locale,
            "Thread not found",
        )
        .into_response();
    }

    for binding in unbound {
        let event = ServerMessage::ThreadAgentUnbound(ThreadAgentUnboundMsg {
            v: PROTOCOL_VERSION,
            conversation_id,
            thread_id,
            agent_id: binding.agent_id,
            unbound_at: chrono::Utc::now(),
        });
        let _ = broadcast_to_conversation(&state, conversation_id, &event).await;
    }

    let event = ServerMessage::ThreadDeleted(ThreadDeletedMsg {
        v: PROTOCOL_VERSION,
        conversation_id,
        thread_id,
        deleted_by: user_id,
        deleted_at: chrono::Utc::now(),
    });
    let _ = broadcast_to_conversation(&state, conversation_id, &event).await;

    Json(serde_json::json!({ "deleted": true })).into_response()
}

pub async fn list_thread_messages(
    State(state): State<AppState>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path((conversation_id, thread_id)): Path<(Uuid, Uuid)>,
    Query(query): Query<ListThreadMessagesQuery>,
) -> Response {
    match ensure_membership(&state, conversation_id, user_id, &locale).await {
        Ok(()) => {}
        Err(resp) => return resp,
    }

    if get_thread_row(&state, conversation_id, thread_id)
        .await
        .ok()
        .flatten()
        .is_none()
    {
        return error(
            StatusCode::NOT_FOUND,
            "thread_not_found",
            &locale,
            "Thread not found",
        )
        .into_response();
    }

    let limit = query.limit.unwrap_or(50).clamp(1, 50);
    let rows = match sqlx::query_as::<_, Message>(
        "SELECT id, conversation_id, thread_id, sender_id, content, format, seq, created_at
         FROM messages
         WHERE conversation_id = $1
           AND thread_id = $2
           AND ($3::timestamptz IS NULL OR created_at < $3)
         ORDER BY seq DESC
         LIMIT $4",
    )
    .bind(conversation_id)
    .bind(thread_id)
    .bind(query.before)
    .bind(limit)
    .fetch_all(state.db.as_ref())
    .await
    {
        Ok(rows) => rows,
        Err(err) => {
            tracing::error!(%err, conversation_id = %conversation_id, thread_id = %thread_id, "failed to list thread messages");
            return error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "query_failed",
                &locale,
                "Could not load thread messages",
            )
            .into_response();
        }
    };

    Json(rows).into_response()
}

pub async fn send_thread_message(
    State(state): State<AppState>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path((conversation_id, thread_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<SendThreadMessageRequest>,
) -> Response {
    match ensure_membership(&state, conversation_id, user_id, &locale).await {
        Ok(()) => {}
        Err(resp) => return resp,
    }

    if payload.content.trim().is_empty() {
        return error(
            StatusCode::BAD_REQUEST,
            "invalid_content",
            &locale,
            "Message content is required",
        )
        .into_response();
    }

    if get_thread_row(&state, conversation_id, thread_id)
        .await
        .ok()
        .flatten()
        .is_none()
    {
        return error(
            StatusCode::NOT_FOUND,
            "thread_not_found",
            &locale,
            "Thread not found",
        )
        .into_response();
    }

    let format = payload.format.to_ascii_lowercase();
    if format != "markdown" && format != "plain" {
        return error(
            StatusCode::BAD_REQUEST,
            "invalid_format",
            &locale,
            "format must be markdown or plain",
        )
        .into_response();
    }

    let idempotency_key = payload.idempotency_key.unwrap_or_else(Uuid::new_v4);
    let created = match message_service::send_message(
        &state.db,
        conversation_id,
        user_id,
        Some(thread_id),
        payload.content.trim(),
        &format,
        idempotency_key,
    )
    .await
    {
        Ok(message) => message,
        Err(err) => {
            tracing::error!(%err, conversation_id = %conversation_id, thread_id = %thread_id, "failed to send thread message");
            return error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "message_send_failed",
                &locale,
                "Could not send thread message",
            )
            .into_response();
        }
    };

    if let Err(err) = sqlx::query(
        "UPDATE threads
         SET message_count = message_count + 1,
             last_message_at = $3
         WHERE id = $1 AND conversation_id = $2",
    )
    .bind(thread_id)
    .bind(conversation_id)
    .bind(created.created_at)
    .execute(state.db.as_ref())
    .await
    {
        tracing::error!(%err, conversation_id = %conversation_id, thread_id = %thread_id, "failed to update thread counters after message insert");
    }

    let response = Message {
        id: created.id,
        conversation_id,
        thread_id: Some(thread_id),
        sender_id: user_id,
        content: payload.content.trim().to_owned(),
        format,
        seq: created.seq,
        created_at: created.created_at,
    };

    let event = ServerMessage::MessageReceived(MessageReceivedMsg {
        v: PROTOCOL_VERSION,
        id: response.id,
        conversation_id,
        thread_id: Some(thread_id),
        sender_id: user_id,
        content: response.content.clone(),
        format: match response.format.as_str() {
            "plain" => MessageFormat::Plain,
            _ => MessageFormat::Markdown,
        },
        seq: response.seq,
        created_at: response.created_at,
        blocks: vec![],
    });
    let _ = broadcast_to_conversation(&state, conversation_id, &event).await;

    let context_engine = state.context_engine.clone();
    let context_message = crate::context_engine::message_created_event(MessageReceivedMsg {
        v: PROTOCOL_VERSION,
        id: response.id,
        conversation_id,
        thread_id: Some(thread_id),
        sender_id: user_id,
        content: response.content.clone(),
        format: match response.format.as_str() {
            "plain" => MessageFormat::Plain,
            _ => MessageFormat::Markdown,
        },
        seq: response.seq,
        created_at: response.created_at,
        blocks: Vec::new(),
    });
    tokio::spawn(async move {
        if let Err(err) = context_engine.on_message_created(context_message).await {
            tracing::error!(%err, conversation_id = %conversation_id, thread_id = %thread_id, "context hook failed");
        }
    });

    (StatusCode::CREATED, Json(response)).into_response()
}

pub async fn list_thread_agents(
    State(state): State<AppState>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path((conversation_id, thread_id)): Path<(Uuid, Uuid)>,
) -> Response {
    match ensure_membership(&state, conversation_id, user_id, &locale).await {
        Ok(()) => {}
        Err(resp) => return resp,
    }

    if get_thread_row(&state, conversation_id, thread_id)
        .await
        .ok()
        .flatten()
        .is_none()
    {
        return error(
            StatusCode::NOT_FOUND,
            "thread_not_found",
            &locale,
            "Thread not found",
        )
        .into_response();
    }

    match sqlx::query_as::<_, ThreadAgent>(
        "SELECT thread_id, agent_id, bound_at
         FROM thread_agents
         WHERE conversation_id = $1 AND thread_id = $2
         ORDER BY bound_at ASC",
    )
    .bind(conversation_id)
    .bind(thread_id)
    .fetch_all(state.db.as_ref())
    .await
    {
        Ok(rows) => Json(rows).into_response(),
        Err(err) => {
            tracing::error!(%err, conversation_id = %conversation_id, thread_id = %thread_id, "failed to list thread agents");
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "query_failed",
                &locale,
                "Could not list thread agents",
            )
            .into_response()
        }
    }
}

pub async fn bind_agent(
    State(state): State<AppState>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path((conversation_id, thread_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<BindAgentRequest>,
) -> Response {
    match ensure_membership(&state, conversation_id, user_id, &locale).await {
        Ok(()) => {}
        Err(resp) => return resp,
    }

    if get_thread_row(&state, conversation_id, thread_id)
        .await
        .ok()
        .flatten()
        .is_none()
    {
        return error(
            StatusCode::NOT_FOUND,
            "thread_not_found",
            &locale,
            "Thread not found",
        )
        .into_response();
    }

    let invited = match sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(
            SELECT 1 FROM conversation_agents
            WHERE conversation_id = $1 AND agent_id = $2
         )",
    )
    .bind(conversation_id)
    .bind(payload.agent_id)
    .fetch_one(state.db.as_ref())
    .await
    {
        Ok(value) => value,
        Err(err) => {
            tracing::error!(%err, conversation_id = %conversation_id, agent_id = %payload.agent_id, "failed to check conversation agent membership");
            return error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "query_failed",
                &locale,
                "Could not bind agent",
            )
            .into_response();
        }
    };

    if !invited {
        return error(
            StatusCode::BAD_REQUEST,
            "agent_not_in_conversation",
            &locale,
            "Agent is not invited to this conversation",
        )
        .into_response();
    }

    let existing = match sqlx::query_scalar::<_, Option<Uuid>>(
        "SELECT thread_id
         FROM thread_agents
         WHERE conversation_id = $1 AND agent_id = $2
         LIMIT 1",
    )
    .bind(conversation_id)
    .bind(payload.agent_id)
    .fetch_optional(state.db.as_ref())
    .await
    {
        Ok(binding) => binding.flatten(),
        Err(err) => {
            tracing::error!(%err, conversation_id = %conversation_id, agent_id = %payload.agent_id, "failed to query existing binding");
            return error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "query_failed",
                &locale,
                "Could not bind agent",
            )
            .into_response();
        }
    };

    if existing.is_some() {
        return error(
            StatusCode::CONFLICT,
            "agent_already_bound",
            &locale,
            "Agent is already bound to another thread in this conversation",
        )
        .into_response();
    }

    let binding = match sqlx::query_as::<_, ThreadAgent>(
        "INSERT INTO thread_agents (thread_id, conversation_id, agent_id)
         VALUES ($1, $2, $3)
         RETURNING thread_id, agent_id, bound_at",
    )
    .bind(thread_id)
    .bind(conversation_id)
    .bind(payload.agent_id)
    .fetch_one(state.db.as_ref())
    .await
    {
        Ok(binding) => binding,
        Err(sqlx::Error::Database(db_err)) if db_err.code().as_deref() == Some("23505") => {
            return error(
                StatusCode::CONFLICT,
                "agent_already_bound",
                &locale,
                "Agent is already bound to another thread in this conversation",
            )
            .into_response();
        }
        Err(err) => {
            tracing::error!(%err, conversation_id = %conversation_id, thread_id = %thread_id, agent_id = %payload.agent_id, "failed to bind agent");
            return error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "query_failed",
                &locale,
                "Could not bind agent",
            )
            .into_response();
        }
    };

    let event = ServerMessage::ThreadAgentBound(ThreadAgentBoundMsg {
        v: PROTOCOL_VERSION,
        conversation_id,
        thread_id,
        agent_id: payload.agent_id,
        bound_at: binding.bound_at,
    });
    let _ = broadcast_to_conversation(&state, conversation_id, &event).await;

    Json(binding).into_response()
}

pub async fn unbind_agent(
    State(state): State<AppState>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path((conversation_id, thread_id, agent_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Response {
    match ensure_membership(&state, conversation_id, user_id, &locale).await {
        Ok(()) => {}
        Err(resp) => return resp,
    }

    let deleted = match sqlx::query_as::<_, ThreadAgent>(
        "DELETE FROM thread_agents
         WHERE conversation_id = $1 AND thread_id = $2 AND agent_id = $3
         RETURNING thread_id, agent_id, bound_at",
    )
    .bind(conversation_id)
    .bind(thread_id)
    .bind(agent_id)
    .fetch_optional(state.db.as_ref())
    .await
    {
        Ok(binding) => binding,
        Err(err) => {
            tracing::error!(%err, conversation_id = %conversation_id, thread_id = %thread_id, agent_id = %agent_id, "failed to unbind agent");
            return error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "query_failed",
                &locale,
                "Could not unbind agent",
            )
            .into_response();
        }
    };

    let Some(binding) = deleted else {
        return error(
            StatusCode::BAD_REQUEST,
            "agent_not_in_conversation",
            &locale,
            "Thread or agent binding not found",
        )
        .into_response();
    };

    let event = ServerMessage::ThreadAgentUnbound(ThreadAgentUnboundMsg {
        v: PROTOCOL_VERSION,
        conversation_id,
        thread_id,
        agent_id,
        unbound_at: chrono::Utc::now(),
    });
    let _ = broadcast_to_conversation(&state, conversation_id, &event).await;

    Json(binding).into_response()
}
