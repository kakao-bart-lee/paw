use crate::db::DbPool;
use crate::ws::hub::Hub;
use paw_proto::{
    ForwardedFrom, MessageAttachment, MessageFormat, MessageForwardedMsg, MessageReceivedMsg,
    ServerMessage, PROTOCOL_VERSION,
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

        let message = match enrich_with_attachments(pool.as_ref(), message).await {
            Ok(message) => message,
            Err(err) => {
                tracing::error!(%err, "failed to enrich message attachments for ws payload");
                continue;
            }
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

async fn enrich_with_attachments(
    pool: &sqlx::PgPool,
    message: ServerMessage,
) -> anyhow::Result<ServerMessage> {
    let message_id = match &message {
        ServerMessage::MessageReceived(frame) => frame.id,
        ServerMessage::MessageForwarded(frame) => frame.id,
        _ => return Ok(message),
    };

    let rows = sqlx::query(
        "SELECT id, file_type, file_url, file_size, mime_type, thumbnail_url
         FROM message_attachments
         WHERE message_id = $1
         ORDER BY created_at ASC, id ASC",
    )
    .bind(message_id)
    .fetch_all(pool)
    .await?;

    let attachments = rows
        .into_iter()
        .map(|row| MessageAttachment {
            id: row.try_get("id").unwrap_or_else(|_| Uuid::new_v4()),
            file_type: row
                .try_get::<String, _>("file_type")
                .unwrap_or_else(|_| "file".to_owned()),
            file_url: row.try_get::<String, _>("file_url").unwrap_or_default(),
            file_size: row.try_get::<i64, _>("file_size").unwrap_or_default(),
            mime_type: row
                .try_get::<String, _>("mime_type")
                .unwrap_or_else(|_| "application/octet-stream".to_owned()),
            thumbnail_url: row
                .try_get::<Option<String>, _>("thumbnail_url")
                .ok()
                .flatten(),
        })
        .collect::<Vec<_>>();

    Ok(match message {
        ServerMessage::MessageReceived(mut frame) => {
            frame.attachments = attachments;
            ServerMessage::MessageReceived(frame)
        }
        ServerMessage::MessageForwarded(mut frame) => {
            frame.attachments = attachments;
            ServerMessage::MessageForwarded(frame)
        }
        other => other,
    })
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
        assert!(matches!(
            parsed,
            paw_proto::ServerMessage::MessageForwarded(_)
        ));
    }
}
