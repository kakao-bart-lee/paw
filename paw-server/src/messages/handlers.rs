use crate::auth::{middleware::UserId, AppState};
use crate::i18n::{error_response, RequestLocale};
use crate::messages::{
    models::{
        AddMemberRequest, ConversationListItem, MediaAttachment, Message, MessageAttachment,
        RemoveMemberResponse, UpdateGroupNameRequest, UpdateMemberRoleRequest,
        UpdateMemberRoleResponse,
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
use chrono::Utc;
use paw_proto::{
    ContextConversationSettingsChangedMsg, ContextMemberJoinedMsg, ContextMemberLeftMsg,
    ContextMessageDeletedMsg, InboundContext, MessageFormat, MessageReceivedMsg, PROTOCOL_VERSION,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

type ApiResult<T> = Result<Json<T>, (StatusCode, Json<Value>)>;

#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub content: String,
    pub format: String,
    pub idempotency_key: Uuid,
    #[serde(default)]
    pub attachment_ids: Vec<Uuid>,
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

#[derive(Debug, Serialize)]
pub struct MessageAttachmentsResponse {
    pub attachments: Vec<MessageAttachment>,
}

#[derive(Debug, Clone, Copy)]
struct AttachmentLimits {
    max_text_plain_bytes: i64,
    max_image_bytes: i64,
}

const DEFAULT_MAX_TEXT_PLAIN_ATTACHMENT_BYTES: i64 = 256 * 1024;
const DEFAULT_MAX_IMAGE_ATTACHMENT_BYTES: i64 = 512 * 1024;
const ATTACHMENT_MAX_TEXT_PLAIN_BYTES_ENV: &str = "PAW_ATTACHMENT_MAX_TEXT_PLAIN_BYTES";
const ATTACHMENT_MAX_IMAGE_BYTES_ENV: &str = "PAW_ATTACHMENT_MAX_IMAGE_BYTES";

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

    if has_duplicates(&payload.attachment_ids) {
        return error(
            StatusCode::BAD_REQUEST,
            "invalid_attachment_ids",
            &locale,
            "attachment_ids must not contain duplicates",
        )
        .into_response();
    }

    let media_attachments = match service::fetch_media_attachments(
        &state.db,
        user_id,
        &payload.attachment_ids,
    )
    .await
    {
        Ok(attachments) => attachments,
        Err(err)
            if err
                .to_string()
                .contains("attachment_not_found_or_not_owned") =>
        {
            return error(
                StatusCode::BAD_REQUEST,
                "invalid_attachment_ids",
                &locale,
                "attachment_ids contains unknown attachments",
            )
            .into_response();
        }
        Err(err) => {
            tracing::error!(%err, conversation_id = %conv_id, sender_id = %user_id, "failed to load attachments");
            return error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "attachment_lookup_failed",
                &locale,
                "Could not process attachments",
            )
            .into_response();
        }
    };

    if let Err((code, message)) =
        validate_attachment_totals(&media_attachments, attachment_limits_from_env())
    {
        return error(StatusCode::PAYLOAD_TOO_LARGE, code, &locale, message).into_response();
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
        &media_attachments,
    )
    .await
    {
        Ok(created) => {
            crate::metrics::record_message_sent();
            let db = state.db.clone();
            let hub = state.hub.clone();
            let notify_state = state.clone();
            let context_engine = state.context_engine.clone();
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
                attachments: proto_attachments_from_media(&media_attachments),
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

            let context_message =
                crate::context_engine::message_created_event(MessageReceivedMsg {
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
                    attachments: proto_attachments_from_media(&media_attachments),
                });
            tokio::spawn(async move {
                if let Err(err) = context_engine.on_message_created(context_message).await {
                    tracing::error!(%err, conversation_id = %conv_id, "context hook failed");
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

    let message_ids: Vec<Uuid> = rows
        .iter()
        .map(|row| row.try_get::<Uuid, _>("id"))
        .collect::<Result<Vec<_>, _>>()?;
    let attachments_map = service::list_message_attachments_map(&state.db, &message_ids).await?;

    let mut recent_messages = Vec::with_capacity(rows.len());
    for row in rows {
        recent_messages.push(message_received_from_row(&row, &attachments_map)?);
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
    attachments_map: &HashMap<Uuid, Vec<MessageAttachment>>,
) -> Result<MessageReceivedMsg, sqlx::Error> {
    let id: Uuid = row.try_get("id")?;
    let format_raw: Option<String> = row.try_get::<Option<String>, _>("format")?;
    let blocks_raw: Option<serde_json::Value> =
        row.try_get::<Option<serde_json::Value>, _>("blocks")?;
    let blocks = match blocks_raw {
        Some(serde_json::Value::Array(values)) => values,
        _ => Vec::new(),
    };

    Ok(MessageReceivedMsg {
        v: PROTOCOL_VERSION,
        id,
        conversation_id: row.try_get("conversation_id")?,
        thread_id: row.try_get("thread_id")?,
        sender_id: row.try_get("sender_id")?,
        content: row.try_get("content")?,
        format: to_message_format(&format_raw.unwrap_or_else(|| "markdown".to_owned())),
        seq: row.try_get("seq")?,
        created_at: row.try_get("created_at")?,
        blocks,
        attachments: proto_attachments_from_message(
            attachments_map.get(&id).map(Vec::as_slice).unwrap_or(&[]),
        ),
    })
}

fn to_message_format(raw: &str) -> MessageFormat {
    match raw.to_ascii_lowercase().as_str() {
        "plain" => MessageFormat::Plain,
        _ => MessageFormat::Markdown,
    }
}

pub async fn get_message_attachments(
    State(state): State<AppState>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path(message_id): Path<Uuid>,
) -> Response {
    let conversation_id = match sqlx::query_scalar::<_, Uuid>(
        "SELECT conversation_id FROM messages WHERE id = $1",
    )
    .bind(message_id)
    .fetch_optional(state.db.as_ref())
    .await
    {
        Ok(Some(conversation_id)) => conversation_id,
        Ok(None) => {
            return error(
                StatusCode::NOT_FOUND,
                "message_not_found",
                &locale,
                "Message not found",
            )
            .into_response();
        }
        Err(err) => {
            tracing::error!(%err, message_id = %message_id, "failed to resolve conversation for message attachments");
            return error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "message_lookup_failed",
                &locale,
                "Could not fetch message attachments",
            )
            .into_response();
        }
    };

    match ensure_membership(&state, conversation_id, user_id, &locale).await {
        Ok(()) => {}
        Err(resp) => return resp,
    }

    match service::list_message_attachments(&state.db, message_id).await {
        Ok(attachments) => Json(MessageAttachmentsResponse { attachments }).into_response(),
        Err(err) => {
            tracing::error!(%err, message_id = %message_id, "failed to list message attachments");
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "attachment_list_failed",
                &locale,
                "Could not fetch message attachments",
            )
            .into_response()
        }
    }
}

fn attachment_limits_from_env() -> AttachmentLimits {
    AttachmentLimits {
        max_text_plain_bytes: parse_positive_env_i64(
            ATTACHMENT_MAX_TEXT_PLAIN_BYTES_ENV,
            DEFAULT_MAX_TEXT_PLAIN_ATTACHMENT_BYTES,
        ),
        max_image_bytes: parse_positive_env_i64(
            ATTACHMENT_MAX_IMAGE_BYTES_ENV,
            DEFAULT_MAX_IMAGE_ATTACHMENT_BYTES,
        ),
    }
}

fn parse_positive_env_i64(name: &str, default: i64) -> i64 {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse::<i64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(default)
}

fn validate_attachment_totals(
    attachments: &[MediaAttachment],
    limits: AttachmentLimits,
) -> Result<(), (&'static str, &'static str)> {
    let mut text_total = 0_i64;
    let mut image_total = 0_i64;

    for attachment in attachments {
        if attachment.mime_type.eq_ignore_ascii_case("text/plain") {
            text_total += attachment.file_size;
            continue;
        }

        if attachment
            .mime_type
            .to_ascii_lowercase()
            .starts_with("image/")
        {
            image_total += attachment.file_size;
        } else {
            text_total += attachment.file_size;
        }
    }

    if text_total > limits.max_text_plain_bytes {
        return Err((
            "attachment_size_exceeded",
            "Total text/file attachment size exceeds configured limit",
        ));
    }

    if image_total > limits.max_image_bytes {
        return Err((
            "attachment_size_exceeded",
            "Total image attachment size exceeds configured limit",
        ));
    }

    Ok(())
}

fn has_duplicates(values: &[Uuid]) -> bool {
    let mut set = HashSet::with_capacity(values.len());
    values.iter().any(|value| !set.insert(*value))
}

fn proto_attachments_from_media(
    attachments: &[MediaAttachment],
) -> Vec<paw_proto::MessageAttachment> {
    attachments
        .iter()
        .map(|attachment| paw_proto::MessageAttachment {
            id: attachment.id,
            file_type: attachment.media_type.clone(),
            file_url: attachment.s3_key.clone(),
            file_size: attachment.file_size,
            mime_type: attachment.mime_type.clone(),
            thumbnail_url: attachment.thumbnail_s3_key.clone(),
        })
        .collect()
}

fn proto_attachments_from_message(
    attachments: &[MessageAttachment],
) -> Vec<paw_proto::MessageAttachment> {
    attachments
        .iter()
        .map(|attachment| paw_proto::MessageAttachment {
            id: attachment.id,
            file_type: attachment.file_type.clone(),
            file_url: attachment.file_url.clone(),
            file_size: attachment.file_size,
            mime_type: attachment.mime_type.clone(),
            thumbnail_url: attachment.thumbnail_url.clone(),
        })
        .collect()
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
        let context_engine = state.context_engine.clone();
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

        tokio::spawn(async move {
            if let Err(err) = context_engine
                .on_message_deleted(ContextMessageDeletedMsg {
                    v: PROTOCOL_VERSION,
                    conversation_id: conv_id,
                    thread_id,
                    message_id,
                    deleted_by: user_id,
                    occurred_at: Utc::now(),
                })
                .await
            {
                tracing::error!(%err, conversation_id = %conv_id, message_id = %message_id, "context hook failed");
            }
        });
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
        Ok(added) => {
            if added {
                let context_engine = state.context_engine.clone();
                let member_id = payload.user_id;
                tokio::spawn(async move {
                    if let Err(err) = context_engine
                        .on_member_joined(ContextMemberJoinedMsg {
                            v: PROTOCOL_VERSION,
                            conversation_id,
                            member_id,
                            joined_by: user_id,
                            occurred_at: Utc::now(),
                        })
                        .await
                    {
                        tracing::error!(%err, conversation_id = %conversation_id, member_id = %member_id, "context hook failed");
                    }
                });
            }

            Json(AddMemberResponse { added }).into_response()
        }
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
        Ok(removed) => {
            if removed {
                let context_engine = state.context_engine.clone();
                tokio::spawn(async move {
                    if let Err(err) = context_engine
                        .on_member_left(ContextMemberLeftMsg {
                            v: PROTOCOL_VERSION,
                            conversation_id,
                            member_id: target_user_id,
                            left_by: user_id,
                            occurred_at: Utc::now(),
                        })
                        .await
                    {
                        tracing::error!(%err, conversation_id = %conversation_id, member_id = %target_user_id, "context hook failed");
                    }
                });
            }

            Json(RemoveMemberResponse { removed }).into_response()
        }
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
        Ok(updated) => {
            if updated {
                let context_engine = state.context_engine.clone();
                let title = payload.name.trim().to_owned();
                tokio::spawn(async move {
                    if let Err(err) = context_engine
                        .on_conversation_settings_changed(ContextConversationSettingsChangedMsg {
                            v: PROTOCOL_VERSION,
                            conversation_id,
                            changed_by: user_id,
                            occurred_at: Utc::now(),
                            changes: serde_json::json!({ "title": title }),
                        })
                        .await
                    {
                        tracing::error!(%err, conversation_id = %conversation_id, "context hook failed");
                    }
                });
            }

            Json(UpdateGroupNameResponse { updated }).into_response()
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn media_attachment(mime_type: &str, file_size: i64) -> MediaAttachment {
        MediaAttachment {
            id: Uuid::new_v4(),
            media_type: if mime_type.starts_with("image/") {
                "image".to_string()
            } else {
                "file".to_string()
            },
            mime_type: mime_type.to_string(),
            file_size,
            s3_key: "media/test/object".to_string(),
            thumbnail_s3_key: None,
        }
    }

    #[test]
    fn attachment_validation_enforces_text_and_image_limits_separately() {
        let limits = AttachmentLimits {
            max_text_plain_bytes: 10,
            max_image_bytes: 20,
        };

        let ok = vec![
            media_attachment("text/plain", 10),
            media_attachment("image/png", 20),
        ];
        assert!(validate_attachment_totals(&ok, limits).is_ok());

        let text_over = vec![media_attachment("application/pdf", 11)];
        assert!(validate_attachment_totals(&text_over, limits).is_err());

        let image_over = vec![media_attachment("image/jpeg", 21)];
        assert!(validate_attachment_totals(&image_over, limits).is_err());
    }

    #[test]
    fn duplicate_attachment_ids_are_rejected() {
        let duplicate = Uuid::new_v4();
        assert!(has_duplicates(&[duplicate, duplicate]));
        assert!(!has_duplicates(&[Uuid::new_v4(), Uuid::new_v4()]));
    }

    #[test]
    fn media_attachments_convert_to_proto_payload() {
        let attachment = MediaAttachment {
            id: Uuid::new_v4(),
            media_type: "image".to_string(),
            mime_type: "image/png".to_string(),
            file_size: 123,
            s3_key: "media/user/file.png".to_string(),
            thumbnail_s3_key: Some("media/user/file-thumb.png".to_string()),
        };

        let proto = proto_attachments_from_media(&[attachment]);
        assert_eq!(proto.len(), 1);
        assert_eq!(proto[0].file_type, "image");
        assert_eq!(proto[0].mime_type, "image/png");
        assert_eq!(proto[0].file_size, 123);
        assert_eq!(
            proto[0].thumbnail_url.as_deref(),
            Some("media/user/file-thumb.png")
        );
    }
}
