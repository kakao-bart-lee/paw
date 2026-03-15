use crate::db::DbPool;
use crate::messages::{models::MessageAttachment, service as message_service};
use crate::ws::hub::Hub;
use paw_proto::{
    ForwardedFromMsg, MessageFormat, MessageForwardedMsg, MessageReceivedMsg, ServerMessage,
    PROTOCOL_VERSION,
};
use sqlx::Row;
use std::sync::Arc;
use uuid::Uuid;

pub async fn start_pg_listener(pool: DbPool, hub: Arc<Hub>) {
    let mut listener = match sqlx::postgres::PgListener::connect_with(pool.as_ref()).await {
        Ok(listener) => listener,
        Err(err) => {
            tracing::error!(%err, "pg listener failed to connect");
            return;
        }
    };

    if let Err(err) = listener.listen("new_message").await {
        tracing::error!(%err, "failed to LISTEN new_message");
        return;
    }

    loop {
        let notification = match listener.recv().await {
            Ok(notification) => notification,
            Err(err) => {
                tracing::error!(%err, "pg listener recv failed");
                continue;
            }
        };

        let payload: serde_json::Value = match serde_json::from_str(notification.payload()) {
            Ok(payload) => payload,
            Err(err) => {
                tracing::warn!(%err, payload = notification.payload(), "invalid pg_notify payload");
                continue;
            }
        };

        let Some(mut message) = payload_to_message(&payload) else {
            tracing::warn!(
                payload = notification.payload(),
                "pg_notify payload missing fields"
            );
            continue;
        };

        let attachments = match message_service::list_message_attachments(&pool, message.id).await {
            Ok(attachments) => attachments,
            Err(err) => {
                tracing::error!(%err, message_id = %message.id, "failed to load message attachments");
                continue;
            }
        };
        message.attachments = proto_attachments_from_message(&attachments);

        let members = match conversation_members(pool.as_ref(), message.conversation_id).await {
            Ok(members) => members,
            Err(err) => {
                tracing::error!(%err, conversation_id = %message.conversation_id, "failed to load conversation members");
                continue;
            }
        };

        let frame = if let Some(forwarded_from) = parse_forwarded_from(&payload) {
            let forwarded = MessageForwardedMsg {
                v: message.v,
                id: message.id,
                conversation_id: message.conversation_id,
                thread_id: message.thread_id,
                sender_id: message.sender_id,
                content: message.content,
                format: message.format,
                seq: message.seq,
                created_at: message.created_at,
                blocks: message.blocks,
                attachments: message.attachments,
                forwarded_from,
            };

            match serde_json::to_string(&ServerMessage::MessageForwarded(forwarded)) {
                Ok(frame) => frame,
                Err(err) => {
                    tracing::error!(%err, "failed to serialize message_forwarded frame");
                    continue;
                }
            }
        } else {
            match serde_json::to_string(&ServerMessage::MessageReceived(message)) {
                Ok(frame) => frame,
                Err(err) => {
                    tracing::error!(%err, "failed to serialize message_received frame");
                    continue;
                }
            }
        };

        hub.broadcast_to_conversation(members, &frame).await;
    }
}

async fn conversation_members(
    pool: &sqlx::PgPool,
    conversation_id: Uuid,
) -> anyhow::Result<Vec<Uuid>> {
    let rows = sqlx::query("SELECT user_id FROM conversation_members WHERE conversation_id = $1")
        .bind(conversation_id)
        .fetch_all(pool)
        .await?;

    let mut user_ids = Vec::with_capacity(rows.len());
    for row in rows {
        user_ids.push(row.try_get("user_id")?);
    }
    Ok(user_ids)
}

fn payload_to_message(payload: &serde_json::Value) -> Option<MessageReceivedMsg> {
    let id = payload.get("id")?.as_str().and_then(parse_uuid)?;
    let conversation_id = payload
        .get("conversation_id")?
        .as_str()
        .and_then(parse_uuid)?;
    let sender_id = payload.get("sender_id")?.as_str().and_then(parse_uuid)?;
    let content = payload.get("content")?.as_str()?.to_owned();
    let seq = payload.get("seq")?.as_i64()?;

    let created_at = payload
        .get("created_at")?
        .as_str()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc))?;

    let format = match payload
        .get("format")
        .and_then(|v| v.as_str())
        .unwrap_or("markdown")
        .to_ascii_lowercase()
        .as_str()
    {
        "plain" => MessageFormat::Plain,
        _ => MessageFormat::Markdown,
    };

    let blocks = match payload.get("blocks") {
        Some(serde_json::Value::Array(values)) => values.clone(),
        _ => Vec::new(),
    };

    Some(MessageReceivedMsg {
        v: PROTOCOL_VERSION,
        id,
        conversation_id,
        thread_id: None,
        sender_id,
        content,
        format,
        seq,
        created_at,
        blocks,
        attachments: Vec::new(),
    })
}

fn parse_uuid(value: &str) -> Option<Uuid> {
    Uuid::parse_str(value).ok()
}

fn parse_forwarded_from(payload: &serde_json::Value) -> Option<ForwardedFromMsg> {
    let value = payload.get("forwarded_from")?;
    Some(ForwardedFromMsg {
        message_id: value.get("message_id")?.as_str().and_then(parse_uuid)?,
        conversation_id: value
            .get("conversation_id")?
            .as_str()
            .and_then(parse_uuid)?,
        sender_id: value.get("sender_id")?.as_str().and_then(parse_uuid)?,
    })
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
