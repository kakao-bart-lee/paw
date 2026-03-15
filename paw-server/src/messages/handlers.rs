use crate::auth::{middleware::UserId, AppState};
use crate::i18n::{error_response, RequestLocale};
use crate::messages::{
    models::{
        AddMemberRequest, ConversationListItem, Message, RemoveMemberResponse,
        UpdateGroupNameRequest, UpdateMemberRoleRequest, UpdateMemberRoleResponse,
    },
    service::{self, GroupManagementError, Membership},
};
use crate::moderation;
use crate::push;
use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use paw_proto::{InboundContext, MessageFormat, MessageReceivedMsg, PROTOCOL_VERSION};
use serde::{Deserialize, Serialize};
use serde_json::Value;
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
pub struct ForwardMessageRequest {
    pub original_message_id: Uuid,
    pub source_conversation_id: Uuid,
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
pub struct DeleteMessageResponse {
    pub deleted: bool,
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
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path(conv_id): Path<Uuid>,
    Json(payload): Json<SendMessageRequest>,
) -> Response {
    if payload.content.trim().is_empty() {
        return error(
            StatusCode::BAD_REQUEST,
            "invalid_content",
            &locale,
            "Message content is required",
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

    match ensure_membership(&state, conv_id, user_id, &locale).await {
        Ok(()) => {}
        Err(resp) => return resp,
    }

    if moderation::service::check_spam(&payload.content, &state.db).await {
        return error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "spam_detected",
            &locale,
            "Message contains prohibited content",
        )
        .into_response();
    }

    match service::get_idempotent_message(&state.db, conv_id, user_id, payload.idempotency_key)
        .await
    {
        Ok(Some(existing)) => return Json(existing).into_response(),
        Ok(None) => {}
        Err(err) => {
            tracing::error!(%err, conversation_id = %conv_id, sender_id = %user_id, "failed idempotency lookup");
            return error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "message_lookup_failed",
                &locale,
                "Could not send message",
            )
            .into_response();
        }
    }

    match service::send_message(
        &state.db,
        conv_id,
        user_id,
        None,
        payload.content.trim(),
        &format,
        payload.idempotency_key,
    )
    .await
    {
        Ok(created) => {
            crate::metrics::record_message_sent();
            let db = state.db.clone();
            let hub = state.hub.clone();
            let notify_state = state.clone();
            let created_message = MessageReceivedMsg {
                v: PROTOCOL_VERSION,
                id: created.id,
                conversation_id: conv_id,
                thread_id: None,
                sender_id: user_id,
                content: payload.content.trim().to_owned(),
                format: to_message_format(&format),
                seq: created.seq,
                created_at: created.created_at,
                blocks: Vec::new(),
                attachments: Vec::new(),
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
            match service::get_idempotent_message(
                &state.db,
                conv_id,
                user_id,
                payload.idempotency_key,
            )
            .await
            {
                Ok(Some(existing)) => Json(existing).into_response(),
                _ => error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "message_send_failed",
                    &locale,
                    "Could not send message",
                )
                .into_response(),
            }
        }
    }
}

pub async fn forward_message(
    State(state): State<AppState>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path(target_conv_id): Path<Uuid>,
    Json(payload): Json<ForwardMessageRequest>,
) -> Response {
    match ensure_membership(&state, target_conv_id, user_id, &locale).await {
        Ok(()) => {}
        Err(resp) => return resp,
    }

    match ensure_membership(&state, payload.source_conversation_id, user_id, &locale).await {
        Ok(()) => {}
        Err(resp) => return resp,
    }

    match service::forward_message(
        &state.db,
        target_conv_id,
        payload.source_conversation_id,
        user_id,
        payload.original_message_id,
    )
    .await
    {
        Ok(Some(created)) => Json(created).into_response(),
        Ok(None) => error(
            StatusCode::NOT_FOUND,
            "not_found",
            &locale,
            "Original message not found",
        )
        .into_response(),
        Err(err) => {
            tracing::error!(
                %err,
                target_conversation_id = %target_conv_id,
                source_conversation_id = %payload.source_conversation_id,
                original_message_id = %payload.original_message_id,
                sender_id = %user_id,
                "message forward failed"
            );
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "message_forward_failed",
                &locale,
                "Could not forward message",
            )
            .into_response()
        }
    }
}

async fn notify_agents_of_message(
    state: AppState,
    message: MessageReceivedMsg,
) -> anyhow::Result<()> {
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
        "SELECT id, conversation_id, thread_id, sender_id, content, format, seq, created_at, blocks\
         FROM messages\
         WHERE conversation_id = $1 AND is_deleted = FALSE\
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
        match nats.publish(subject, payload.clone().into()).await {
            Ok(()) => crate::metrics::record_agent_gateway_call(true),
            Err(err) => {
                crate::metrics::record_agent_gateway_call(false);
                return Err(err.into());
            }
        }
    }

    Ok(())
}

fn message_received_from_row(
    row: &sqlx::postgres::PgRow,
) -> Result<MessageReceivedMsg, sqlx::Error> {
    let format_raw: Option<String> = row.try_get::<Option<String>, _>("format")?;
    let blocks_raw: Option<serde_json::Value> =
        row.try_get::<Option<serde_json::Value>, _>("blocks")?;
    let blocks = match blocks_raw {
        Some(serde_json::Value::Array(values)) => values,
        _ => Vec::new(),
    };

    Ok(MessageReceivedMsg {
        v: PROTOCOL_VERSION,
        id: row.try_get("id")?,
        conversation_id: row.try_get("conversation_id")?,
        thread_id: row.try_get("thread_id")?,
        sender_id: row.try_get("sender_id")?,
        content: row.try_get("content")?,
        format: to_message_format(&format_raw.unwrap_or_else(|| "markdown".to_owned())),
        seq: row.try_get("seq")?,
        created_at: row.try_get("created_at")?,
        blocks,
        attachments: Vec::new(),
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
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path(conv_id): Path<Uuid>,
    Query(query): Query<GetMessagesQuery>,
) -> Response {
    match ensure_membership(&state, conv_id, user_id, &locale).await {
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
                &locale,
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

pub async fn delete_message(
    State(state): State<AppState>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path((conv_id, message_id)): Path<(Uuid, Uuid)>,
) -> Response {
    match ensure_membership(&state, conv_id, user_id, &locale).await {
        Ok(()) => {}
        Err(resp) => return resp,
    }

    let message_row = match sqlx::query(
        "SELECT thread_id
         FROM messages
         WHERE id = $1 AND conversation_id = $2",
    )
    .bind(message_id)
    .bind(conv_id)
    .fetch_optional(state.db.as_ref())
    .await
    {
        Ok(Some(row)) => row,
        Ok(None) => {
            return error(
                StatusCode::NOT_FOUND,
                "not_found",
                &locale,
                "Message not found",
            )
            .into_response();
        }
        Err(err) => {
            tracing::error!(%err, conversation_id = %conv_id, message_id = %message_id, "failed to load message for delete");
            return error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "delete_failed",
                &locale,
                "Could not delete message",
            )
            .into_response();
        }
    };

    let protected_root = match sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(
            SELECT 1 FROM threads
            WHERE conversation_id = $1 AND root_message_id = $2
         )",
    )
    .bind(conv_id)
    .bind(message_id)
    .fetch_one(state.db.as_ref())
    .await
    {
        Ok(value) => value,
        Err(err) => {
            tracing::error!(%err, conversation_id = %conv_id, message_id = %message_id, "failed to check root protection");
            return error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "delete_failed",
                &locale,
                "Could not delete message",
            )
            .into_response();
        }
    };

    if protected_root {
        return error(
            StatusCode::CONFLICT,
            "root_message_protected",
            &locale,
            "Cannot delete a thread root message while the thread exists",
        )
        .into_response();
    }

    let thread_id: Option<Uuid> = message_row.try_get("thread_id").unwrap_or(None);
    let deleted = match service::delete_message(&state.db, conv_id, message_id).await {
        Ok(value) => value,
        Err(err) => {
            tracing::error!(%err, conversation_id = %conv_id, message_id = %message_id, "delete message failed");
            return error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "delete_failed",
                &locale,
                "Could not delete message",
            )
            .into_response();
        }
    };

    if deleted {
        if let Some(thread_id) = thread_id {
            let latest = sqlx::query_scalar::<_, Option<chrono::DateTime<chrono::Utc>>>(
                "SELECT MAX(created_at)
                 FROM messages
                 WHERE conversation_id = $1 AND thread_id = $2",
            )
            .bind(conv_id)
            .bind(thread_id)
            .fetch_one(state.db.as_ref())
            .await
            .ok()
            .flatten();

            let _ = sqlx::query(
                "UPDATE threads
                 SET message_count = GREATEST(message_count - 1, 0),
                     last_message_at = $3
                 WHERE id = $1 AND conversation_id = $2",
            )
            .bind(thread_id)
            .bind(conv_id)
            .bind(latest)
            .execute(state.db.as_ref())
            .await;
        }
    }

    Json(DeleteMessageResponse { deleted }).into_response()
}

pub async fn list_conversations(
    State(state): State<AppState>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Extension(UserId(user_id)): Extension<UserId>,
) -> ApiResult<ListConversationsResponse> {
    let conversations = service::list_conversations(&state.db, user_id)
        .await
        .map_err(|err| {
            tracing::error!(%err, user_id = %user_id, "failed to list conversations");
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "conversation_list_failed",
                &locale,
                "Could not list conversations",
            )
        })?;

    Ok(Json(ListConversationsResponse { conversations }))
}

pub async fn create_conversation(
    State(state): State<AppState>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Extension(UserId(user_id)): Extension<UserId>,
    Json(payload): Json<CreateConversationRequest>,
) -> Response {
    if payload.member_ids.len() + 1 > service::MAX_GROUP_MEMBERS {
        return error(
            StatusCode::BAD_REQUEST,
            "too_many_members",
            &locale,
            "A conversation can have at most 100 members (including creator)",
        )
        .into_response();
    }

    let created =
        match service::create_conversation(&state.db, user_id, payload.member_ids, payload.name)
            .await
        {
            Ok(conversation) => conversation,
            Err(err) => {
                tracing::error!(%err, user_id = %user_id, "failed to create conversation");
                return error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "conversation_create_failed",
                    &locale,
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
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path(conversation_id): Path<Uuid>,
    Json(payload): Json<AddMemberRequest>,
) -> Response {
    match service::add_member(&state.db, conversation_id, user_id, payload.user_id).await {
        Ok(added) => Json(AddMemberResponse { added }).into_response(),
        Err(err) => group_management_error_to_response(err, &locale).into_response(),
    }
}

pub async fn remove_member_handler(
    State(state): State<AppState>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path((conversation_id, target_user_id)): Path<(Uuid, Uuid)>,
) -> Response {
    match service::remove_member(&state.db, conversation_id, user_id, target_user_id).await {
        Ok(removed) => Json(RemoveMemberResponse { removed }).into_response(),
        Err(err) => group_management_error_to_response(err, &locale).into_response(),
    }
}

pub async fn update_group_name_handler(
    State(state): State<AppState>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path(conversation_id): Path<Uuid>,
    Json(payload): Json<UpdateGroupNameRequest>,
) -> Response {
    match service::update_group_name(&state.db, conversation_id, user_id, &payload.name).await {
        Ok(updated) => Json(UpdateGroupNameResponse { updated }).into_response(),
        Err(err) => group_management_error_to_response(err, &locale).into_response(),
    }
}

pub async fn update_member_role_handler(
    State(state): State<AppState>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path((conversation_id, target_user_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<UpdateMemberRoleRequest>,
) -> Response {
    match service::update_member_role(
        &state.db,
        conversation_id,
        user_id,
        target_user_id,
        &payload.role,
    )
    .await
    {
        Ok(updated) => Json(UpdateMemberRoleResponse { updated }).into_response(),
        Err(err) => group_management_error_to_response(err, &locale).into_response(),
    }
}

async fn ensure_membership(
    state: &AppState,
    conv_id: Uuid,
    user_id: Uuid,
    locale: &str,
) -> Result<(), Response> {
    match service::check_member(&state.db, conv_id, user_id).await {
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
            tracing::error!(%err, conversation_id = %conv_id, user_id = %user_id, "failed membership check");
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

fn error(status: StatusCode, code: &str, locale: &str, message: &str) -> (StatusCode, Json<Value>) {
    error_response(status, code, locale, message)
}

fn group_management_error_to_response(
    err: GroupManagementError,
    locale: &str,
) -> (StatusCode, Json<Value>) {
    match err {
        GroupManagementError::ConversationNotFound => error(
            StatusCode::NOT_FOUND,
            "conversation_not_found",
            locale,
            "Conversation not found",
        ),
        GroupManagementError::NotGroupConversation => error(
            StatusCode::BAD_REQUEST,
            "not_group_conversation",
            locale,
            "This action is only available for group conversations",
        ),
        GroupManagementError::NotAuthorized => error(
            StatusCode::FORBIDDEN,
            "forbidden",
            locale,
            "Not authorized for this action",
        ),
        GroupManagementError::TooManyMembers => error(
            StatusCode::CONFLICT,
            "too_many_members",
            locale,
            "Conversation reached maximum member limit",
        ),
        GroupManagementError::AlreadyMember => error(
            StatusCode::CONFLICT,
            "already_member",
            locale,
            "User is already a member of this conversation",
        ),
        GroupManagementError::MemberNotFound => error(
            StatusCode::NOT_FOUND,
            "member_not_found",
            locale,
            "Conversation member not found",
        ),
        GroupManagementError::CannotRemoveLastAdmin => error(
            StatusCode::FORBIDDEN,
            "cannot_remove_last_admin",
            locale,
            "Cannot remove the last admin from conversation",
        ),
        GroupManagementError::CannotDemoteLastAdmin => error(
            StatusCode::FORBIDDEN,
            "cannot_demote_last_admin",
            locale,
            "Cannot demote the last admin in conversation",
        ),
        GroupManagementError::InvalidGroupName => error(
            StatusCode::BAD_REQUEST,
            "invalid_group_name",
            locale,
            "Group name is required",
        ),
        GroupManagementError::InvalidRole => error(
            StatusCode::BAD_REQUEST,
            "invalid_role",
            locale,
            "Conversation role is invalid",
        ),
    }
}
