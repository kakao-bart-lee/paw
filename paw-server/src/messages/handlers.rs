use crate::auth::{AppState, middleware::UserId};
use crate::messages::{
    models::{
        AddMemberRequest, ConversationListItem, Message, RemoveMemberResponse, UpdateGroupNameRequest,
    },
    service::{self, GroupManagementError, Membership},
};
use crate::moderation;
use crate::push;
use axum::{
    Json,
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use paw_proto::{InboundContext, MessageFormat, MessageReceivedMsg, PROTOCOL_VERSION};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::Row;
use uuid::Uuid;

type ApiResult<T> = Result<Json<T>, (StatusCode, Json<Value>)>;

#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub content: String,
    pub format: String,
    pub idempotency_key: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct GetMessagesQuery {
    pub after_seq: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateConversationRequest {
    pub member_ids: Vec<Uuid>,
    pub name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct GetMessagesResponse {
    pub messages: Vec<Message>,
    pub has_more: bool,
}

#[derive(Debug, Serialize)]
pub struct ListConversationsResponse {
    pub conversations: Vec<ConversationListItem>,
}

#[derive(Debug, Serialize)]
pub struct CreateConversationResponse {
    pub id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct AddMemberResponse {
    pub added: bool,
}

#[derive(Debug, Serialize)]
pub struct UpdateGroupNameResponse {
    pub updated: bool,
}

pub async fn send_message(
    State(state): State<AppState>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path(conv_id): Path<Uuid>,
    Json(payload): Json<SendMessageRequest>,
) -> Response {
    if payload.content.trim().is_empty() {
        return error(StatusCode::BAD_REQUEST, "invalid_content", "Message content is required")
            .into_response();
    }

    let format = payload.format.to_ascii_lowercase();
    if format != "markdown" && format != "plain" {
        return error(
            StatusCode::BAD_REQUEST,
            "invalid_format",
            "format must be markdown or plain",
        )
        .into_response();
    }

    match ensure_membership(&state, conv_id, user_id).await {
        Ok(()) => {}
        Err(resp) => return resp,
    }

    if moderation::service::check_spam(&payload.content, &state.db).await {
        return error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "spam_detected",
            "Message contains prohibited content",
        )
        .into_response();
    }

    match service::get_idempotent_message(&state.db, conv_id, user_id, payload.idempotency_key).await {
        Ok(Some(existing)) => return Json(existing).into_response(),
        Ok(None) => {}
        Err(err) => {
            tracing::error!(%err, conversation_id = %conv_id, sender_id = %user_id, "failed idempotency lookup");
            return error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "message_lookup_failed",
                "Could not send message",
            )
            .into_response();
        }
    }

    match service::send_message(
        &state.db,
        conv_id,
        user_id,
        payload.content.trim(),
        &format,
        payload.idempotency_key,
    )
    .await
    {
        Ok(created) => {
            let db = state.db.clone();
            let hub = state.hub.clone();
            let notify_state = state.clone();
            let created_message = MessageReceivedMsg {
                v: PROTOCOL_VERSION,
                id: created.id,
                conversation_id: conv_id,
                sender_id: user_id,
                content: payload.content.trim().to_owned(),
                format: to_message_format(&format),
                seq: created.seq,
                created_at: created.created_at,
                blocks: Vec::new(),
            };

            tokio::spawn(async move {
                if let Err(err) =
                    push::service::send_push_notification(&db, &hub, conv_id, user_id).await
                {
                    tracing::error!(%err, conversation_id = %conv_id, "push notification failed");
                }
            });

            tokio::spawn(async move {
                if let Err(err) = notify_agents_of_message(notify_state, created_message).await {
                    tracing::error!(%err, conversation_id = %conv_id, "agent inbound notification failed");
                }
            });

            Json(created).into_response()
        }
        Err(err) => {
            tracing::warn!(%err, conversation_id = %conv_id, sender_id = %user_id, "message insert failed, checking idempotency replay");
            match service::get_idempotent_message(&state.db, conv_id, user_id, payload.idempotency_key)
                .await
            {
                Ok(Some(existing)) => Json(existing).into_response(),
                _ => error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "message_send_failed",
                    "Could not send message",
                )
                .into_response(),
            }
        }
    }
}

async fn notify_agents_of_message(state: AppState, message: MessageReceivedMsg) -> anyhow::Result<()> {
    let Some(nats) = state.nats.clone() else {
        return Ok(());
    };

    let agent_ids = sqlx::query_scalar::<_, Uuid>(
        "SELECT agent_id FROM conversation_agents WHERE conversation_id = $1",
    )
    .bind(message.conversation_id)
    .fetch_all(state.db.as_ref())
    .await?;

    if agent_ids.is_empty() {
        return Ok(());
    }

    let rows: Vec<sqlx::postgres::PgRow> = sqlx::query(
        "SELECT id, conversation_id, sender_id, content, format, seq, created_at, blocks\
         FROM messages\
         WHERE conversation_id = $1\
         ORDER BY seq DESC\
         LIMIT 10",
    )
    .bind(message.conversation_id)
    .fetch_all(state.db.as_ref())
    .await?;

    let mut recent_messages = Vec::with_capacity(rows.len());
    for row in rows {
        recent_messages.push(message_received_from_row(&row)?);
    }
    recent_messages.reverse();

    let conversation_id = message.conversation_id;
    let inbound = InboundContext {
        v: PROTOCOL_VERSION,
        message,
        conversation_id,
        recent_messages,
    };
    let payload = serde_json::to_vec(&inbound)?;

    for agent_id in agent_ids {
        let subject = format!("agent.inbound.{agent_id}");
        nats.publish(subject, payload.clone().into()).await?;
    }

    Ok(())
}

fn message_received_from_row(row: &sqlx::postgres::PgRow) -> Result<MessageReceivedMsg, sqlx::Error> {
    let format_raw: Option<String> = row.try_get::<Option<String>, _>("format")?;
    let blocks_raw: Option<serde_json::Value> = row.try_get::<Option<serde_json::Value>, _>("blocks")?;
    let blocks = match blocks_raw {
        Some(serde_json::Value::Array(values)) => values,
        _ => Vec::new(),
    };

    Ok(MessageReceivedMsg {
        v: PROTOCOL_VERSION,
        id: row.try_get("id")?,
        conversation_id: row.try_get("conversation_id")?,
        sender_id: row.try_get("sender_id")?,
        content: row.try_get("content")?,
        format: to_message_format(&format_raw.unwrap_or_else(|| "markdown".to_owned())),
        seq: row.try_get("seq")?,
        created_at: row.try_get("created_at")?,
        blocks,
    })
}

fn to_message_format(raw: &str) -> MessageFormat {
    match raw.to_ascii_lowercase().as_str() {
        "plain" => MessageFormat::Plain,
        _ => MessageFormat::Markdown,
    }
}

pub async fn get_messages(
    State(state): State<AppState>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path(conv_id): Path<Uuid>,
    Query(query): Query<GetMessagesQuery>,
) -> Response {
    match ensure_membership(&state, conv_id, user_id).await {
        Ok(()) => {}
        Err(resp) => return resp,
    }

    let after_seq = query.after_seq.unwrap_or(0);
    let limit = query.limit.unwrap_or(50).clamp(1, 50);

    let mut messages = match service::get_messages(&state.db, conv_id, after_seq, limit + 1).await {
        Ok(rows) => rows,
        Err(err) => {
            tracing::error!(%err, conversation_id = %conv_id, "failed to fetch messages");
            return error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "message_history_failed",
                "Could not fetch message history",
            )
            .into_response();
        }
    };

    let has_more = messages.len() as i64 > limit;
    if has_more {
        messages.truncate(limit as usize);
    }

    Json(GetMessagesResponse { messages, has_more }).into_response()
}

pub async fn list_conversations(
    State(state): State<AppState>,
    Extension(UserId(user_id)): Extension<UserId>,
) -> ApiResult<ListConversationsResponse> {
    let conversations = service::list_conversations(&state.db, user_id)
        .await
        .map_err(|err| {
            tracing::error!(%err, user_id = %user_id, "failed to list conversations");
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "conversation_list_failed",
                "Could not list conversations",
            )
        })?;

    Ok(Json(ListConversationsResponse { conversations }))
}

pub async fn create_conversation(
    State(state): State<AppState>,
    Extension(UserId(user_id)): Extension<UserId>,
    Json(payload): Json<CreateConversationRequest>,
) -> Response {
    if payload.member_ids.len() + 1 > service::MAX_GROUP_MEMBERS {
        return error(
            StatusCode::BAD_REQUEST,
            "too_many_members",
            "A conversation can have at most 100 members (including creator)",
        )
        .into_response();
    }

    let created = match service::create_conversation(&state.db, user_id, payload.member_ids, payload.name)
        .await
    {
        Ok(conversation) => conversation,
        Err(err) => {
            tracing::error!(%err, user_id = %user_id, "failed to create conversation");
            return error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "conversation_create_failed",
                "Could not create conversation",
            )
            .into_response();
        }
    };

    let response = CreateConversationResponse {
        id: created.id,
        created_at: created.created_at,
    };

    (StatusCode::CREATED, Json(response)).into_response()
}

pub async fn add_member_handler(
    State(state): State<AppState>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path(conversation_id): Path<Uuid>,
    Json(payload): Json<AddMemberRequest>,
) -> Response {
    match service::add_member(&state.db, conversation_id, user_id, payload.user_id).await {
        Ok(added) => Json(AddMemberResponse { added }).into_response(),
        Err(err) => group_management_error_to_response(err).into_response(),
    }
}

pub async fn remove_member_handler(
    State(state): State<AppState>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path((conversation_id, target_user_id)): Path<(Uuid, Uuid)>,
) -> Response {
    match service::remove_member(&state.db, conversation_id, user_id, target_user_id).await {
        Ok(removed) => Json(RemoveMemberResponse { removed }).into_response(),
        Err(err) => group_management_error_to_response(err).into_response(),
    }
}

pub async fn update_group_name_handler(
    State(state): State<AppState>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path(conversation_id): Path<Uuid>,
    Json(payload): Json<UpdateGroupNameRequest>,
) -> Response {
    match service::update_group_name(&state.db, conversation_id, user_id, &payload.name).await {
        Ok(updated) => Json(UpdateGroupNameResponse { updated }).into_response(),
        Err(err) => group_management_error_to_response(err).into_response(),
    }
}

async fn ensure_membership(state: &AppState, conv_id: Uuid, user_id: Uuid) -> Result<(), Response> {
    match service::check_member(&state.db, conv_id, user_id).await {
        Ok(Membership::Member) => Ok(()),
        Ok(Membership::NotMember) => Err(error(
            StatusCode::FORBIDDEN,
            "forbidden",
            "User is not a member of this conversation",
        )
        .into_response()),
        Ok(Membership::ConversationNotFound) => Err(error(
            StatusCode::NOT_FOUND,
            "conversation_not_found",
            "Conversation not found",
        )
        .into_response()),
        Err(err) => {
            tracing::error!(%err, conversation_id = %conv_id, user_id = %user_id, "failed membership check");
            Err(error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "membership_check_failed",
                "Could not validate conversation membership",
            )
            .into_response())
        }
    }
}

fn error(status: StatusCode, code: &str, message: &str) -> (StatusCode, Json<Value>) {
    (
        status,
        Json(json!({
            "error": code,
            "message": message,
        })),
    )
}

fn group_management_error_to_response(err: GroupManagementError) -> (StatusCode, Json<Value>) {
    match err {
        GroupManagementError::ConversationNotFound => error(
            StatusCode::NOT_FOUND,
            "conversation_not_found",
            "Conversation not found",
        ),
        GroupManagementError::NotAuthorized => {
            error(StatusCode::FORBIDDEN, "forbidden", "Not authorized for this action")
        }
        GroupManagementError::TooManyMembers => error(
            StatusCode::CONFLICT,
            "too_many_members",
            "Conversation reached maximum member limit",
        ),
        GroupManagementError::AlreadyMember => error(
            StatusCode::CONFLICT,
            "already_member",
            "User is already a member of this conversation",
        ),
        GroupManagementError::MemberNotFound => {
            error(StatusCode::NOT_FOUND, "member_not_found", "Conversation member not found")
        }
        GroupManagementError::CannotRemoveLastOwner => error(
            StatusCode::FORBIDDEN,
            "cannot_remove_last_owner",
            "Cannot remove the last owner from conversation",
        ),
        GroupManagementError::InvalidGroupName => {
            error(StatusCode::BAD_REQUEST, "invalid_group_name", "Group name is required")
        }
    }
}
