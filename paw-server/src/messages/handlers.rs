use crate::agents::permissions::{check_agent_permission, AgentPermission};
use crate::auth::{middleware::UserId, AppState};
use crate::context_engine::models::{
    ConversationSettingsChangedHook, MemberJoinedHook, MemberLeftHook, MessageCreatedHook,
    MessageDeletedHook,
};
use crate::context_engine::LifecycleHooks;
use crate::i18n::{error_response, RequestLocale};
use crate::messages::{
    models::{
        AddMemberRequest, ConversationListItem, ForwardedFromMetadata, Message,
        MessageAttachmentRecord, RemoveMemberResponse, UpdateGroupNameRequest,
        UpdateMemberRoleRequest, UpdateMemberRoleResponse,
    },
    service::{self, GroupManagementError, Membership, NewMessageAttachment},
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
    InboundContext, MessageAttachment, MessageFormat, MessageReceivedMsg, PROTOCOL_VERSION,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use std::collections::HashSet;
use std::sync::OnceLock;
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
pub struct ForwardMessageRequest {
    pub original_message_id: Uuid,
    pub source_conversation_id: Uuid,
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
pub struct MessageAttachmentsResponse {
    pub attachments: Vec<MessageAttachment>,
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

const DEFAULT_TEXT_ATTACHMENT_MAX_BYTES: usize = 256 * 1024;
const DEFAULT_IMAGE_ATTACHMENT_MAX_BYTES: i64 = 512 * 1024;

static TEXT_ATTACHMENT_MAX_BYTES: OnceLock<usize> = OnceLock::new();
static IMAGE_ATTACHMENT_MAX_BYTES: OnceLock<i64> = OnceLock::new();

fn text_attachment_max_bytes() -> usize {
    *TEXT_ATTACHMENT_MAX_BYTES.get_or_init(|| {
        std::env::var("PAW_TEXT_ATTACHMENT_MAX_BYTES")
            .ok()
            .and_then(|raw| raw.parse::<usize>().ok())
            .filter(|&value| value > 0)
            .unwrap_or(DEFAULT_TEXT_ATTACHMENT_MAX_BYTES)
    })
}

fn image_attachment_max_bytes() -> i64 {
    *IMAGE_ATTACHMENT_MAX_BYTES.get_or_init(|| {
        std::env::var("PAW_IMAGE_ATTACHMENT_MAX_BYTES")
            .ok()
            .and_then(|raw| raw.parse::<i64>().ok())
            .filter(|&value| value > 0)
            .unwrap_or(DEFAULT_IMAGE_ATTACHMENT_MAX_BYTES)
    })
}

fn infer_message_file_type(media_type: &str, mime_type: &str) -> String {
    if media_type.eq_ignore_ascii_case("image") || mime_type.starts_with("image/") {
        return "image".to_owned();
    }

    if media_type.eq_ignore_ascii_case("video") || mime_type.starts_with("video/") {
        return "video".to_owned();
    }

    if media_type.eq_ignore_ascii_case("audio") || mime_type.starts_with("audio/") {
        return "audio".to_owned();
    }

    if mime_type.starts_with("text/") {
        return "text".to_owned();
    }

    "file".to_owned()
}

fn is_text_attachment(file_type: &str, mime_type: &str) -> bool {
    file_type.eq_ignore_ascii_case("text") || mime_type.starts_with("text/")
}

fn to_protocol_attachment(record: MessageAttachmentRecord) -> MessageAttachment {
    MessageAttachment {
        id: record.id,
        file_type: record.file_type,
        file_url: record.file_url,
        file_size: record.file_size,
        mime_type: record.mime_type,
        thumbnail_url: record.thumbnail_url,
    }
}

pub async fn send_message(
    State(state): State<AppState>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path(conv_id): Path<Uuid>,
    Json(payload): Json<SendMessageRequest>,
) -> Response {
    let trimmed_content = payload.content.trim();
    if trimmed_content.is_empty() && payload.attachment_ids.is_empty() {
        return error(
            StatusCode::BAD_REQUEST,
            "invalid_content",
            &locale,
            "Message content or attachments are required",
        )
        .into_response();
    }

    if trimmed_content.len() > text_attachment_max_bytes() {
        return error(
            StatusCode::PAYLOAD_TOO_LARGE,
            "content_too_large",
            &locale,
            "Message content exceeds configured text size limit",
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

    let mut attachments_for_insert = Vec::new();
    let mut attachment_frames = Vec::new();
    if !payload.attachment_ids.is_empty() {
        let mut unique_ids = Vec::with_capacity(payload.attachment_ids.len());
        let mut seen = HashSet::with_capacity(payload.attachment_ids.len());
        for id in &payload.attachment_ids {
            if seen.insert(*id) {
                unique_ids.push(*id);
            }
        }

        let media_rows = match sqlx::query(
            "SELECT id, media_type, mime_type, file_size, s3_key, thumbnail_s3_key
             FROM media_attachments
             WHERE id = ANY($1) AND uploader_id = $2",
        )
        .bind(&unique_ids)
        .bind(user_id)
        .fetch_all(state.db.as_ref())
        .await
        {
            Ok(rows) => rows,
            Err(err) => {
                tracing::error!(%err, sender_id = %user_id, "failed to resolve attachment ids");
                return error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "attachment_lookup_failed",
                    &locale,
                    "Could not resolve attachments",
                )
                .into_response();
            }
        };

        if media_rows.len() != unique_ids.len() {
            return error(
                StatusCode::BAD_REQUEST,
                "invalid_attachment_ids",
                &locale,
                "One or more attachment ids are invalid",
            )
            .into_response();
        }

        for row in media_rows {
            let media_type = row
                .try_get::<String, _>("media_type")
                .unwrap_or_else(|_| "file".to_owned());
            let mime_type = row
                .try_get::<String, _>("mime_type")
                .unwrap_or_else(|_| "application/octet-stream".to_owned());
            let file_size = row.try_get::<i64, _>("file_size").unwrap_or(0);
            let file_url = row
                .try_get::<String, _>("s3_key")
                .unwrap_or_else(|_| String::new());
            let thumbnail_url = row
                .try_get::<Option<String>, _>("thumbnail_s3_key")
                .ok()
                .flatten();
            let file_type = infer_message_file_type(&media_type, &mime_type);

            if file_type.eq_ignore_ascii_case("image") && file_size > image_attachment_max_bytes() {
                return error(
                    StatusCode::PAYLOAD_TOO_LARGE,
                    "attachment_too_large",
                    &locale,
                    "Image attachment exceeds configured size limit",
                )
                .into_response();
            }

            if is_text_attachment(&file_type, &mime_type)
                && file_size > text_attachment_max_bytes() as i64
            {
                return error(
                    StatusCode::PAYLOAD_TOO_LARGE,
                    "attachment_too_large",
                    &locale,
                    "Text attachment exceeds configured size limit",
                )
                .into_response();
            }

            attachments_for_insert.push(NewMessageAttachment {
                file_type: file_type.clone(),
                file_url: file_url.clone(),
                file_size,
                mime_type: mime_type.clone(),
                thumbnail_url: thumbnail_url.clone(),
            });

            attachment_frames.push(MessageAttachment {
                id: row.try_get("id").unwrap_or_else(|_| Uuid::new_v4()),
                file_type,
                file_url,
                file_size,
                mime_type,
                thumbnail_url,
            });
        }
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

    match service::send_message_with_attachments(
        &state.db,
        conv_id,
        user_id,
        None,
        trimmed_content,
        &format,
        payload.idempotency_key,
        &attachments_for_insert,
    )
    .await
    {
        Ok(created) => {
            crate::metrics::record_message_sent();
            let saved_attachments = service::get_message_attachments(&state.db, created.id)
                .await
                .unwrap_or_else(|_| {
                    attachment_frames
                        .into_iter()
                        .map(|attachment| MessageAttachmentRecord {
                            id: attachment.id,
                            message_id: created.id,
                            file_type: attachment.file_type,
                            file_url: attachment.file_url,
                            file_size: attachment.file_size,
                            mime_type: attachment.mime_type,
                            thumbnail_url: attachment.thumbnail_url,
                            created_at: created.created_at,
                        })
                        .collect()
                });
            let created_attachments = saved_attachments
                .into_iter()
                .map(to_protocol_attachment)
                .collect::<Vec<_>>();
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
                content: trimmed_content.to_owned(),
                format: to_message_format(&format),
                seq: created.seq,
                created_at: created.created_at,
                blocks: Vec::new(),
                attachments: created_attachments,
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

            context_engine
                .on_message_created(MessageCreatedHook {
                    conversation_id: conv_id,
                    message_id: created.id,
                    thread_id: None,
                    sender_id: user_id,
                    content: trimmed_content.to_owned(),
                    format,
                    seq: created.seq,
                    timestamp: created.created_at,
                })
                .await;

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

    let message_ids = rows
        .iter()
        .filter_map(|row| row.try_get::<Uuid, _>("id").ok())
        .collect::<Vec<_>>();
    let mut attachments_by_message =
        service::get_message_attachments_for_messages(&state.db, &message_ids).await?;

    let mut recent_messages = Vec::with_capacity(rows.len());
    for row in rows {
        let message_id = row.try_get::<Uuid, _>("id")?;
        let attachments = attachments_by_message
            .remove(&message_id)
            .unwrap_or_default()
            .into_iter()
            .map(to_protocol_attachment)
            .collect::<Vec<_>>();
        recent_messages.push(message_received_from_row(&row, attachments)?);
    }
    recent_messages.reverse();

    let conversation_id = message.conversation_id;

    for agent_id in agent_ids {
        let can_read = check_agent_permission(
            &state.db,
            conversation_id,
            agent_id,
            AgentPermission::ReadMessages,
        )
        .await?;

        if !can_read {
            continue;
        }

        let can_access_history = check_agent_permission(
            &state.db,
            conversation_id,
            agent_id,
            AgentPermission::AccessHistory,
        )
        .await?;

        let inbound = InboundContext {
            v: PROTOCOL_VERSION,
            message: message.clone(),
            conversation_id,
            recent_messages: if can_access_history {
                recent_messages.clone()
            } else {
                Vec::new()
            },
        };
        let payload = serde_json::to_vec(&inbound)?;

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
    attachments: Vec<MessageAttachment>,
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
        attachments,
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

pub async fn get_message_attachments(
    State(state): State<AppState>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path(message_id): Path<Uuid>,
) -> Response {
    let conversation_id = match sqlx::query_scalar::<_, Uuid>(
        "SELECT conversation_id
         FROM messages
         WHERE id = $1",
    )
    .bind(message_id)
    .fetch_optional(state.db.as_ref())
    .await
    {
        Ok(Some(value)) => value,
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
            tracing::error!(%err, message_id = %message_id, "failed to load message conversation");
            return error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "attachment_lookup_failed",
                &locale,
                "Could not load message attachments",
            )
            .into_response();
        }
    };

    match ensure_membership(&state, conversation_id, user_id, &locale).await {
        Ok(()) => {}
        Err(resp) => return resp,
    }

    let attachments = match service::get_message_attachments(&state.db, message_id).await {
        Ok(rows) => rows
            .into_iter()
            .map(to_protocol_attachment)
            .collect::<Vec<_>>(),
        Err(err) => {
            tracing::error!(%err, message_id = %message_id, "failed to load message attachments");
            return error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "attachment_lookup_failed",
                &locale,
                "Could not load message attachments",
            )
            .into_response();
        }
    };

    Json(MessageAttachmentsResponse { attachments }).into_response()
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

    let original = match sqlx::query(
        "SELECT content, format
         FROM messages
         WHERE id = $1
           AND conversation_id = $2
           AND is_deleted = FALSE",
    )
    .bind(payload.original_message_id)
    .bind(payload.source_conversation_id)
    .fetch_optional(state.db.as_ref())
    .await
    {
        Ok(Some(row)) => row,
        Ok(None) => {
            return error(
                StatusCode::NOT_FOUND,
                "message_not_found",
                &locale,
                "Original message not found",
            )
            .into_response();
        }
        Err(err) => {
            tracing::error!(
                %err,
                conversation_id = %payload.source_conversation_id,
                message_id = %payload.original_message_id,
                "failed to load source message for forwarding"
            );
            return error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "message_forward_failed",
                &locale,
                "Could not forward message",
            )
            .into_response();
        }
    };

    let content = match original.try_get::<String, _>("content") {
        Ok(value) => value,
        Err(err) => {
            tracing::error!(%err, "source message row missing content");
            return error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "message_forward_failed",
                &locale,
                "Could not forward message",
            )
            .into_response();
        }
    };
    let format = original
        .try_get::<Option<String>, _>("format")
        .ok()
        .flatten()
        .unwrap_or_else(|| "markdown".to_owned());

    let forwarded_from = ForwardedFromMetadata {
        original_message_id: payload.original_message_id,
        source_conversation_id: payload.source_conversation_id,
    };

    let forwarded_json = match serde_json::to_value(&forwarded_from) {
        Ok(value) => value,
        Err(err) => {
            tracing::error!(%err, "failed to serialize forwarded_from metadata");
            return error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "message_forward_failed",
                &locale,
                "Could not forward message",
            )
            .into_response();
        }
    };

    match service::send_forwarded_message(
        &state.db,
        target_conv_id,
        user_id,
        &content,
        &format,
        Uuid::new_v4(),
        forwarded_json,
    )
    .await
    {
        Ok(created) => {
            crate::metrics::record_message_sent();
            Json(created).into_response()
        }
        Err(err) => {
            tracing::error!(%err, conversation_id = %target_conv_id, sender_id = %user_id, "forwarded message insert failed");
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
        state
            .context_engine
            .on_message_deleted(MessageDeletedHook {
                conversation_id: conv_id,
                message_id,
                thread_id,
                deleted_by: user_id,
                timestamp: Utc::now(),
            })
            .await;

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
        Ok(added) => {
            if added {
                state
                    .context_engine
                    .on_member_joined(MemberJoinedHook {
                        conversation_id,
                        member_id: payload.user_id,
                        joined_by: user_id,
                        timestamp: Utc::now(),
                    })
                    .await;
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
                state
                    .context_engine
                    .on_member_left(MemberLeftHook {
                        conversation_id,
                        member_id: target_user_id,
                        left_by: user_id,
                        timestamp: Utc::now(),
                    })
                    .await;
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
                state
                    .context_engine
                    .on_conversation_settings_changed(ConversationSettingsChangedHook {
                        conversation_id,
                        changed_by: user_id,
                        changes: serde_json::json!({"title": payload.name}),
                        timestamp: Utc::now(),
                    })
                    .await;
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
        Ok(updated) => {
            if updated {
                state
                    .context_engine
                    .on_conversation_settings_changed(ConversationSettingsChangedHook {
                        conversation_id,
                        changed_by: user_id,
                        changes: serde_json::json!({
                            "member_role": {
                                "user_id": target_user_id,
                                "role": payload.role,
                            }
                        }),
                        timestamp: Utc::now(),
                    })
                    .await;
            }

            Json(UpdateMemberRoleResponse { updated }).into_response()
        }
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
    use super::{infer_message_file_type, is_text_attachment};

    #[test]
    fn infer_message_file_type_detects_images() {
        assert_eq!(infer_message_file_type("image", "image/png"), "image");
        assert_eq!(infer_message_file_type("file", "image/jpeg"), "image");
    }

    #[test]
    fn infer_message_file_type_detects_text() {
        assert_eq!(infer_message_file_type("file", "text/plain"), "text");
    }

    #[test]
    fn text_attachment_detection_uses_file_type_or_mime() {
        assert!(is_text_attachment("text", "application/octet-stream"));
        assert!(is_text_attachment("file", "text/markdown"));
        assert!(!is_text_attachment("image", "image/png"));
    }
}
