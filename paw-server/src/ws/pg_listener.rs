use crate::db::DbPool;
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

        let frame = if let Some(message) = payload_to_forwarded_message(&payload) {
            match serde_json::to_string(&ServerMessage::MessageForwarded(message)) {
                Ok(frame) => frame,
                Err(err) => {
                    tracing::error!(%err, "failed to serialize message_forwarded frame");
                    continue;
                }
            }
        } else {
            let Some(message) = payload_to_message(&payload) else {
                tracing::warn!(
                    payload = notification.payload(),
                    "pg_notify payload missing fields"
                );
                continue;
            };

            match serde_json::to_string(&ServerMessage::MessageReceived(message)) {
                Ok(frame) => frame,
                Err(err) => {
                    tracing::error!(%err, "failed to serialize message_received frame");
                    continue;
                }
            }
        };

        let conversation_id = match payload
            .get("conversation_id")
            .and_then(|value| value.as_str())
            .and_then(parse_uuid)
        {
            Some(conversation_id) => conversation_id,
            None => {
                tracing::warn!(
                    payload = notification.payload(),
                    "pg_notify payload missing conversation_id"
                );
                continue;
            }
        };

        let members = match conversation_members(pool.as_ref(), conversation_id).await {
            Ok(members) => members,
            Err(err) => {
                tracing::error!(%err, conversation_id = %conversation_id, "failed to load conversation members");
                continue;
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
    })
}

fn payload_to_forwarded_message(payload: &serde_json::Value) -> Option<MessageForwardedMsg> {
    let forwarded_from_value = payload.get("forwarded_from")?;
    let forwarded_from = parse_forwarded_from(forwarded_from_value)?;
    let base = payload_to_message(payload)?;

    Some(MessageForwardedMsg {
        v: base.v,
        id: base.id,
        conversation_id: base.conversation_id,
        thread_id: base.thread_id,
        sender_id: base.sender_id,
        content: base.content,
        format: base.format,
        seq: base.seq,
        created_at: base.created_at,
        blocks: base.blocks,
        forwarded_from,
    })
}

fn parse_forwarded_from(value: &serde_json::Value) -> Option<ForwardedFromMsg> {
    Some(ForwardedFromMsg {
        message_id: value.get("message_id")?.as_str().and_then(parse_uuid)?,
        conversation_id: value
            .get("conversation_id")?
            .as_str()
            .and_then(parse_uuid)?,
        sender_id: value.get("sender_id")?.as_str().and_then(parse_uuid)?,
    })
}

fn parse_uuid(value: &str) -> Option<Uuid> {
    Uuid::parse_str(value).ok()
}
