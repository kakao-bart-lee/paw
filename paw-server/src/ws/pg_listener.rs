use crate::db::DbPool;
use crate::ws::hub::Hub;
use paw_proto::{
    ForwardedFrom, MessageFormat, MessageForwardedMsg, MessageReceivedMsg, ServerMessage,
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

        let Some(message) = payload_to_message(&payload) else {
            tracing::warn!(
                payload = notification.payload(),
                "pg_notify payload missing fields"
            );
            continue;
        };

        let conversation_id = match &message {
            ServerMessage::MessageReceived(frame) => frame.conversation_id,
            ServerMessage::MessageForwarded(frame) => frame.conversation_id,
            _ => continue,
        };

        let members = match conversation_members(pool.as_ref(), conversation_id).await {
            Ok(members) => members,
            Err(err) => {
                tracing::error!(%err, conversation_id = %conversation_id, "failed to load conversation members");
                continue;
            }
        };

        let frame = match serde_json::to_string(&message) {
            Ok(frame) => frame,
            Err(err) => {
                tracing::error!(%err, "failed to serialize message_received frame");
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

fn payload_to_message(payload: &serde_json::Value) -> Option<ServerMessage> {
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

    let message_received = MessageReceivedMsg {
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
    };

    let forwarded_from = payload
        .get("forwarded_from")
        .and_then(|value| serde_json::from_value::<ForwardedFrom>(value.clone()).ok());

    match forwarded_from {
        Some(forwarded_from) => Some(ServerMessage::MessageForwarded(MessageForwardedMsg {
            v: message_received.v,
            id: message_received.id,
            conversation_id: message_received.conversation_id,
            thread_id: message_received.thread_id,
            sender_id: message_received.sender_id,
            content: message_received.content,
            format: message_received.format,
            seq: message_received.seq,
            created_at: message_received.created_at,
            blocks: message_received.blocks,
            attachments: message_received.attachments,
            forwarded_from,
        })),
        None => Some(ServerMessage::MessageReceived(message_received)),
    }
}

fn parse_uuid(value: &str) -> Option<Uuid> {
    Uuid::parse_str(value).ok()
}

#[cfg(test)]
mod tests {
    use super::payload_to_message;

    #[test]
    fn forwarded_payload_maps_to_message_forwarded_variant() {
        let payload = serde_json::json!({
            "id": uuid::Uuid::new_v4().to_string(),
            "conversation_id": uuid::Uuid::new_v4().to_string(),
            "sender_id": uuid::Uuid::new_v4().to_string(),
            "seq": 3,
            "content": "forwarded",
            "format": "plain",
            "blocks": [],
            "created_at": chrono::Utc::now().to_rfc3339(),
            "forwarded_from": {
                "original_message_id": uuid::Uuid::new_v4().to_string(),
                "source_conversation_id": uuid::Uuid::new_v4().to_string()
            }
        });

        let parsed = payload_to_message(&payload).expect("payload should parse");
        assert!(matches!(parsed, paw_proto::ServerMessage::MessageForwarded(_)));
    }
}
