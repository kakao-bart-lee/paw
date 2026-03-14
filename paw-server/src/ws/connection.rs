use crate::auth::AppState;
use crate::i18n::localized_message;
use crate::messages::service::{self, Membership};
use crate::ws::hub::WsSender;
use axum::extract::ws::{close_code, CloseFrame, Message, WebSocket};
use chrono::{DateTime, Utc};
use futures_util::{SinkExt, StreamExt};
use paw_proto::{
    ClientMessage, DeviceSyncResponse, HelloErrorMsg, HelloOkMsg, MessageFormat,
    MessageReceivedMsg, ServerMessage, PROTOCOL_VERSION,
};
use sqlx::Row;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use uuid::Uuid;

pub struct WsConnection {
    pub user_id: Uuid,
    pub device_id: Uuid,
    pub connected_at: DateTime<Utc>,
}

pub async fn handle_socket(
    socket: WebSocket,
    user_id: Uuid,
    device_id: Uuid,
    locale: String,
    state: AppState,
) {
    let connection = WsConnection {
        user_id,
        device_id,
        connected_at: Utc::now(),
    };

    tracing::debug!(
        user_id = %connection.user_id,
        device_id = %connection.device_id,
        connected_at = %connection.connected_at,
        "websocket connected"
    );

    crate::metrics::ws_connection_opened();

    let (outbound_tx, mut outbound_rx) = mpsc::unbounded_channel::<Message>();
    state
        .hub
        .register(connection.user_id, outbound_tx.clone())
        .await;

    if let Ok(frame) = serde_json::to_string(&ServerMessage::HelloOk(HelloOkMsg {
        v: PROTOCOL_VERSION,
        user_id: connection.user_id,
        server_time: Utc::now(),
    })) {
        let _ = outbound_tx.send(Message::Text(frame.into()));
    }

    let (mut ws_sender, mut ws_receiver) = socket.split();

    let writer = tokio::spawn(async move {
        while let Some(msg) = outbound_rx.recv().await {
            if ws_sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    let mut ping = tokio::time::interval(Duration::from_secs(crate::ws::HEARTBEAT_PING_SECONDS));
    let mut timeout_check =
        tokio::time::interval(Duration::from_secs(crate::ws::HEARTBEAT_PING_SECONDS));
    let mut last_pong = Instant::now();

    loop {
        tokio::select! {
            _ = ping.tick() => {
                if outbound_tx.send(Message::Ping(Vec::new().into())).is_err() {
                    break;
                }
            }
            _ = timeout_check.tick() => {
                if last_pong.elapsed() > Duration::from_secs(crate::ws::HEARTBEAT_TIMEOUT_SECONDS) {
                    let _ = outbound_tx.send(Message::Close(Some(CloseFrame {
                        code: close_code::NORMAL,
                        reason: "heartbeat timeout".into(),
                    })));
                    break;
                }
            }
            maybe_msg = ws_receiver.next() => {
                match maybe_msg {
                    Some(Ok(Message::Text(text))) => {
                        match serde_json::from_str::<ClientMessage>(&text) {
                            Ok(client_msg) => {
                                if let Err(err) = handle_client_message(&state, connection.user_id, &outbound_tx, client_msg, &locale).await {
                                    tracing::warn!(%err, user_id = %connection.user_id, "failed to handle client ws message");
                                    if err.to_string().starts_with("unsupported protocol version") {
                                        let _ = send_protocol_error(
                                            &outbound_tx,
                                            "unsupported_protocol_version",
                                            &locale,
                                            Some(&err.to_string()),
                                            "Unsupported protocol version",
                                        ).await;
                                    }
                                }
                            }
                            Err(err) => {
                                let _ = send_protocol_error(
                                    &outbound_tx,
                                    "invalid_frame",
                                    &locale,
                                    Some(&format!("invalid json frame: {err}")),
                                    "Invalid websocket frame",
                                ).await;
                            }
                        }
                    }
                    Some(Ok(Message::Pong(_))) => {
                        last_pong = Instant::now();
                    }
                    Some(Ok(Message::Ping(payload))) => {
                        let _ = outbound_tx.send(Message::Pong(payload));
                    }
                    Some(Ok(Message::Close(_))) => break,
                    Some(Ok(Message::Binary(_))) => {
                        let _ = send_protocol_error(
                            &outbound_tx,
                            "invalid_frame",
                            &locale,
                            Some("binary frames are not supported"),
                            "Invalid websocket frame",
                        ).await;
                    }
                    Some(Err(err)) => {
                        tracing::debug!(%err, user_id = %connection.user_id, "websocket receive error");
                        break;
                    }
                    None => break,
                }
            }
        }
    }

    state.hub.unregister(connection.user_id, &outbound_tx).await;
    crate::metrics::ws_connection_closed();
    drop(outbound_tx);
    let _ = writer.await;
}

async fn handle_client_message(
    state: &AppState,
    user_id: Uuid,
    outbound_tx: &WsSender,
    msg: ClientMessage,
    locale: &str,
) -> anyhow::Result<()> {
    match msg {
        ClientMessage::Connect(connect) => {
            require_v(connect.v)?;
        }
        ClientMessage::MessageSend(message_send) => {
            require_v(message_send.v)?;
            tracing::debug!(
                user_id = %user_id,
                conversation_id = %message_send.conversation_id,
                "message_send received; persistence is handled by HTTP flow"
            );
        }
        ClientMessage::TypingStart(mut typing) => {
            require_v(typing.v)?;
            typing.user_id = Some(user_id);
            let others: Vec<Uuid> = conversation_members(state, typing.conversation_id)
                .await?
                .into_iter()
                .filter(|&m| m != user_id)
                .collect();
            let payload = serde_json::to_string(&ServerMessage::TypingStart(typing))?;
            state.hub.broadcast_to_conversation(others, &payload).await;
        }
        ClientMessage::TypingStop(mut typing) => {
            require_v(typing.v)?;
            typing.user_id = Some(user_id);
            let others: Vec<Uuid> = conversation_members(state, typing.conversation_id)
                .await?
                .into_iter()
                .filter(|&m| m != user_id)
                .collect();
            let payload = serde_json::to_string(&ServerMessage::TypingStop(typing))?;
            state.hub.broadcast_to_conversation(others, &payload).await;
        }
        ClientMessage::MessageAck(ack) => {
            require_v(ack.v)?;
            sqlx::query(
                "UPDATE conversation_members SET last_read_seq = GREATEST(last_read_seq, $3) WHERE conversation_id = $1 AND user_id = $2",
            )
            .bind(ack.conversation_id)
            .bind(user_id)
            .bind(ack.last_seq)
            .execute(state.db.as_ref())
            .await?;
        }
        ClientMessage::Sync(sync) => {
            require_v(sync.v)?;
            match service::check_member(&state.db, sync.conversation_id, user_id).await? {
                Membership::Member => {}
                Membership::NotMember => {
                    let _ = send_protocol_error(
                        outbound_tx,
                        "forbidden",
                        locale,
                        None,
                        "User is not a member of this conversation",
                    )
                    .await;
                    return Ok(());
                }
                Membership::ConversationNotFound => {
                    let _ = send_protocol_error(
                        outbound_tx,
                        "not_found",
                        locale,
                        None,
                        "Conversation not found",
                    )
                    .await;
                    return Ok(());
                }
            }

            let messages =
                fetch_messages_after_seq(state, sync.conversation_id, sync.last_seq).await?;
            for message in messages {
                let payload = serde_json::to_string(&ServerMessage::MessageReceived(message))?;
                if outbound_tx.send(Message::Text(payload.into())).is_err() {
                    break;
                }
            }
        }
        ClientMessage::DeviceSync(request) => {
            require_v(request.v)?;

            let mut messages = Vec::new();
            let mut conversations = Vec::new();
            for conversation in request.conversations {
                match service::check_member(&state.db, conversation.conversation_id, user_id)
                    .await?
                {
                    Membership::Member => {
                        conversations.push(conversation.clone());
                        let mut missing = fetch_messages_after_seq(
                            state,
                            conversation.conversation_id,
                            conversation.last_seq,
                        )
                        .await?;
                        messages.append(&mut missing);
                    }
                    Membership::NotMember | Membership::ConversationNotFound => {
                        tracing::warn!(
                            user_id = %user_id,
                            conversation_id = %conversation.conversation_id,
                            "skipping unauthorized conversation in device_sync"
                        );
                    }
                }
            }

            conversations.sort_by_key(|conversation| conversation.conversation_id);
            messages.sort_by_key(|message| (message.conversation_id, message.seq));

            let payload =
                serde_json::to_string(&ServerMessage::DeviceSyncResponse(DeviceSyncResponse {
                    v: PROTOCOL_VERSION,
                    conversations,
                    messages,
                }))?;
            let _ = outbound_tx.send(Message::Text(payload.into()));
        }
    }

    Ok(())
}

async fn fetch_messages_after_seq(
    state: &AppState,
    conversation_id: Uuid,
    last_seq: i64,
) -> anyhow::Result<Vec<MessageReceivedMsg>> {
    let rows: Vec<sqlx::postgres::PgRow> = sqlx::query(
        "SELECT id, conversation_id, sender_id, content, format, seq, created_at, blocks \
         FROM messages \
         WHERE conversation_id = $1 AND seq > $2 \
         ORDER BY seq ASC \
         LIMIT 100",
    )
    .bind(conversation_id)
    .bind(last_seq)
    .fetch_all(state.db.as_ref())
    .await?;

    let mut messages = Vec::with_capacity(rows.len());
    for row in rows {
        let format_raw: Option<String> = row.try_get::<Option<String>, _>("format")?;
        let format = match format_raw
            .unwrap_or_else(|| "markdown".to_owned())
            .to_ascii_lowercase()
            .as_str()
        {
            "plain" => MessageFormat::Plain,
            _ => MessageFormat::Markdown,
        };

        let blocks: Option<serde_json::Value> =
            row.try_get::<Option<serde_json::Value>, _>("blocks")?;
        let blocks = match blocks {
            Some(serde_json::Value::Array(values)) => values,
            _ => Vec::new(),
        };

        messages.push(MessageReceivedMsg {
            v: PROTOCOL_VERSION,
            id: row.try_get("id")?,
            conversation_id: row.try_get("conversation_id")?,
            sender_id: row.try_get("sender_id")?,
            content: row.try_get("content")?,
            format,
            seq: row.try_get("seq")?,
            created_at: row.try_get("created_at")?,
            blocks,
        });
    }

    Ok(messages)
}

fn require_v(v: u8) -> anyhow::Result<()> {
    if v == PROTOCOL_VERSION {
        Ok(())
    } else {
        anyhow::bail!("unsupported protocol version: {v}")
    }
}

async fn conversation_members(
    state: &AppState,
    conversation_id: Uuid,
) -> anyhow::Result<Vec<Uuid>> {
    let rows: Vec<sqlx::postgres::PgRow> =
        sqlx::query("SELECT user_id FROM conversation_members WHERE conversation_id = $1")
            .bind(conversation_id)
            .fetch_all(state.db.as_ref())
            .await?;

    let mut users = Vec::with_capacity(rows.len());
    for row in rows {
        users.push(row.try_get::<Uuid, _>("user_id")?);
    }

    Ok(users)
}

async fn send_protocol_error(
    outbound_tx: &WsSender,
    code: &str,
    locale: &str,
    details: Option<&str>,
    fallback: &str,
) -> anyhow::Result<()> {
    let payload = serde_json::to_string(&ServerMessage::HelloError(HelloErrorMsg {
        v: PROTOCOL_VERSION,
        code: code.to_owned(),
        message: localized_message(code, locale, fallback).to_string(),
        details: details.map(ToOwned::to_owned),
    }))?;
    let _ = outbound_tx.send(Message::Text(payload.into()));
    Ok(())
}
