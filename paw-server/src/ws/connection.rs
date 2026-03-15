use crate::auth::AppState;
use crate::i18n::localized_message;
use crate::messages::service::{self, Membership};
use crate::ws::hub::WsSender;
use axum::extract::ws::{close_code, CloseFrame, Message, WebSocket};
use chrono::{DateTime, Utc};
use futures_util::{SinkExt, StreamExt};
use paw_proto::{
    ClientMessage, DeviceSyncResponse, ErrorMsg, HelloOkMsg, MessageAttachment, MessageFormat,
    MessageReceivedMsg, ServerMessage, ThreadMessageSendMsg, ThreadSubscriptionMsg,
    ThreadTypingMsg, PROTOCOL_VERSION,
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

#[tracing::instrument(
    name = "ws_connection",
    skip(socket, locale, state),
    fields(user_id = %user_id, device_id = %device_id)
)]
pub async fn handle_socket(
    socket: WebSocket,
    user_id: Uuid,
    device_id: Uuid,
    locale: String,
    state: AppState,
) {
    let mut socket = socket;

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
    if !state
        .hub
        .try_register_with_limit(
            connection.user_id,
            outbound_tx.clone(),
            crate::ws::MAX_WS_CONNECTIONS_PER_USER,
        )
        .await
    {
        if let Ok(payload) = serde_json::to_string(&ServerMessage::Error(ErrorMsg {
            v: PROTOCOL_VERSION,
            code: "too_many_connections".to_string(),
            ref_type: "connection".to_string(),
            message: localized_message(
                "too_many_connections",
                &locale,
                "Too many concurrent websocket connections",
            )
            .to_string(),
        })) {
            let _ = socket.send(Message::Text(payload.into())).await;
        }
        let _ = socket
            .send(Message::Close(Some(CloseFrame {
                code: close_code::POLICY,
                reason: "too many connections".into(),
            })))
            .await;
        return;
    }

    if let Ok(frame) = serde_json::to_string(&ServerMessage::HelloOk(HelloOkMsg {
        v: PROTOCOL_VERSION,
        user_id: connection.user_id,
        server_time: Utc::now(),
        // Threads stay disabled until server-side routing/broadcast support is implemented.
        capabilities: None,
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
                        if exceeds_ws_message_size(text.len()) {
                            let _ = send_protocol_error(
                                &outbound_tx,
                                "message_too_large",
                                "frame",
                                &locale,
                                Some("websocket text frame exceeds max size"),
                                "WebSocket frame exceeds size limit",
                            )
                            .await;
                            let _ = outbound_tx.send(Message::Close(Some(CloseFrame {
                                code: close_code::POLICY,
                                reason: "message too large".into(),
                            })));
                            break;
                        }

                        match serde_json::from_str::<ClientMessage>(&text) {
                            Ok(client_msg) => {
                                if let Err(err) = handle_client_message(&state, connection.user_id, &outbound_tx, client_msg, &locale).await {
                                    tracing::warn!(%err, user_id = %connection.user_id, "failed to handle client ws message");
                                    if err.to_string().starts_with("unsupported protocol version") {
                                        let _ = send_protocol_error(
                                            &outbound_tx,
                                            "unsupported_protocol_version",
                                            "frame",
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
                                    "frame",
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
                        if exceeds_ws_message_size(payload.len()) {
                            let _ = outbound_tx.send(Message::Close(Some(CloseFrame {
                                code: close_code::POLICY,
                                reason: "message too large".into(),
                            })));
                            break;
                        }
                        let _ = outbound_tx.send(Message::Pong(payload));
                    }
                    Some(Ok(Message::Close(_))) => break,
                    Some(Ok(Message::Binary(payload))) => {
                        if exceeds_ws_message_size(payload.len()) {
                            let _ = send_protocol_error(
                                &outbound_tx,
                                "message_too_large",
                                "frame",
                                &locale,
                                Some("websocket binary frame exceeds max size"),
                                "WebSocket frame exceeds size limit",
                            )
                            .await;
                            let _ = outbound_tx.send(Message::Close(Some(CloseFrame {
                                code: close_code::POLICY,
                                reason: "message too large".into(),
                            })));
                            break;
                        }

                        let _ = send_protocol_error(
                            &outbound_tx,
                            "invalid_frame",
                            "frame",
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
            if let Some(thread_id) = message_send.thread_id {
                handle_thread_message_send(
                    state,
                    user_id,
                    outbound_tx,
                    locale,
                    ThreadMessageSendMsg {
                        v: message_send.v,
                        conversation_id: message_send.conversation_id,
                        thread_id,
                        content: message_send.content,
                        format: message_send.format,
                        blocks: message_send.blocks,
                        idempotency_key: message_send.idempotency_key,
                    },
                )
                .await?;
            } else {
                tracing::debug!(
                    user_id = %user_id,
                    conversation_id = %message_send.conversation_id,
                    "message_send received; persistence is handled by HTTP flow"
                );
            }
        }
        ClientMessage::SendThreadMessage(thread_message_send) => {
            require_v(thread_message_send.v)?;
            handle_thread_message_send(state, user_id, outbound_tx, locale, thread_message_send)
                .await?;
        }
        ClientMessage::TypingStart(mut typing) => {
            require_v(typing.v)?;
            typing.user_id = Some(user_id);
            handle_typing_message(state, user_id, outbound_tx, locale, typing, true).await?;
        }
        ClientMessage::TypingStop(mut typing) => {
            require_v(typing.v)?;
            typing.user_id = Some(user_id);
            handle_typing_message(state, user_id, outbound_tx, locale, typing, false).await?;
        }
        ClientMessage::TypingThreadStart(typing_thread_start) => {
            require_v(typing_thread_start.v)?;
            handle_thread_typing_message(
                state,
                user_id,
                outbound_tx,
                locale,
                typing_thread_start,
                true,
            )
            .await?;
        }
        ClientMessage::TypingThreadEnd(typing_thread_end) => {
            require_v(typing_thread_end.v)?;
            handle_thread_typing_message(
                state,
                user_id,
                outbound_tx,
                locale,
                typing_thread_end,
                false,
            )
            .await?;
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
            if let Some(thread_id) = sync.thread_id {
                if !ensure_thread_scope(
                    state,
                    user_id,
                    outbound_tx,
                    locale,
                    sync.conversation_id,
                    thread_id,
                    "sync",
                )
                .await?
                {
                    return Ok(());
                }
            } else {
                match service::check_member(&state.db, sync.conversation_id, user_id).await? {
                    Membership::Member => {}
                    Membership::NotMember => {
                        let _ = send_protocol_error(
                            outbound_tx,
                            "forbidden",
                            "sync",
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
                            "sync",
                            locale,
                            None,
                            "Conversation not found",
                        )
                        .await;
                        return Ok(());
                    }
                }
            }

            let messages = fetch_messages_after_seq(
                state,
                sync.conversation_id,
                sync.thread_id,
                sync.last_seq,
            )
            .await?;
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
                            None,
                            conversation.last_seq,
                        )
                        .await?;
                        messages.append(&mut missing);

                        for thread in &conversation.threads {
                            if !thread_exists(state, conversation.conversation_id, thread.thread_id)
                                .await?
                            {
                                tracing::warn!(
                                    user_id = %user_id,
                                    conversation_id = %conversation.conversation_id,
                                    thread_id = %thread.thread_id,
                                    "skipping unknown thread in device_sync"
                                );
                                continue;
                            }

                            let mut thread_missing = fetch_messages_after_seq(
                                state,
                                conversation.conversation_id,
                                Some(thread.thread_id),
                                thread.last_seq,
                            )
                            .await?;
                            messages.append(&mut thread_missing);
                        }
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
            messages
                .sort_by_key(|message| (message.conversation_id, message.thread_id, message.seq));

            let payload =
                serde_json::to_string(&ServerMessage::DeviceSyncResponse(DeviceSyncResponse {
                    v: PROTOCOL_VERSION,
                    conversations,
                    messages,
                }))?;
            let _ = outbound_tx.send(Message::Text(payload.into()));
        }
        ClientMessage::ThreadSubscribe(thread_subscribe) => {
            require_v(thread_subscribe.v)?;
            handle_thread_subscription(state, user_id, outbound_tx, locale, thread_subscribe, true)
                .await?;
        }
        ClientMessage::ThreadUnsubscribe(thread_unsubscribe) => {
            require_v(thread_unsubscribe.v)?;
            handle_thread_subscription(
                state,
                user_id,
                outbound_tx,
                locale,
                thread_unsubscribe,
                false,
            )
            .await?;
        }
        ClientMessage::ThreadCreate(thread_create) => {
            require_v(thread_create.v)?;
            let _ = send_protocol_error(
                outbound_tx,
                "invalid_frame",
                "thread_create",
                locale,
                Some("thread_create is not implemented on this server yet"),
                "Thread operations are not available yet",
            )
            .await;
        }
        ClientMessage::ThreadBindAgent(thread_bind_agent) => {
            require_v(thread_bind_agent.v)?;
            let _ = send_protocol_error(
                outbound_tx,
                "invalid_frame",
                "thread_bind_agent",
                locale,
                Some("thread_bind_agent is not implemented on this server yet"),
                "Thread operations are not available yet",
            )
            .await;
        }
        ClientMessage::ThreadUnbindAgent(thread_unbind_agent) => {
            require_v(thread_unbind_agent.v)?;
            let _ = send_protocol_error(
                outbound_tx,
                "invalid_frame",
                "thread_unbind_agent",
                locale,
                Some("thread_unbind_agent is not implemented on this server yet"),
                "Thread operations are not available yet",
            )
            .await;
        }
        ClientMessage::ThreadDelete(thread_delete) => {
            require_v(thread_delete.v)?;
            let _ = send_protocol_error(
                outbound_tx,
                "invalid_frame",
                "thread_delete",
                locale,
                Some("thread_delete is not implemented on this server yet"),
                "Thread operations are not available yet",
            )
            .await;
        }
    }

    Ok(())
}

async fn handle_thread_subscription(
    state: &AppState,
    user_id: Uuid,
    outbound_tx: &WsSender,
    locale: &str,
    subscription: ThreadSubscriptionMsg,
    subscribe: bool,
) -> anyhow::Result<()> {
    if !ensure_thread_scope(
        state,
        user_id,
        outbound_tx,
        locale,
        subscription.conversation_id,
        subscription.thread_id,
        if subscribe {
            "thread_subscribe"
        } else {
            "thread_unsubscribe"
        },
    )
    .await?
    {
        return Ok(());
    }

    if subscribe {
        state
            .hub
            .subscribe_thread(
                user_id,
                outbound_tx,
                subscription.conversation_id,
                subscription.thread_id,
            )
            .await;
    } else {
        state
            .hub
            .unsubscribe_thread(
                user_id,
                outbound_tx,
                subscription.conversation_id,
                subscription.thread_id,
            )
            .await;
    }

    Ok(())
}

async fn handle_thread_message_send(
    state: &AppState,
    user_id: Uuid,
    outbound_tx: &WsSender,
    locale: &str,
    message: ThreadMessageSendMsg,
) -> anyhow::Result<()> {
    if !ensure_thread_scope(
        state,
        user_id,
        outbound_tx,
        locale,
        message.conversation_id,
        message.thread_id,
        "send_thread_message",
    )
    .await?
    {
        return Ok(());
    }

    let trimmed_content = message.content.trim();
    if trimmed_content.is_empty() {
        let _ = send_protocol_error(
            outbound_tx,
            "invalid_content",
            "send_thread_message",
            locale,
            None,
            "Message content is required",
        )
        .await;
        return Ok(());
    }

    let format = match message.format {
        MessageFormat::Plain => "plain",
        MessageFormat::Markdown => "markdown",
    };

    match service::get_idempotent_message(
        &state.db,
        message.conversation_id,
        user_id,
        message.idempotency_key,
    )
    .await
    {
        Ok(Some(_)) => return Ok(()),
        Ok(None) => {}
        Err(err) => {
            tracing::error!(
                %err,
                user_id = %user_id,
                conversation_id = %message.conversation_id,
                thread_id = %message.thread_id,
                "failed idempotency lookup for thread message"
            );
            let _ = send_protocol_error(
                outbound_tx,
                "message_lookup_failed",
                "send_thread_message",
                locale,
                None,
                "Could not send thread message",
            )
            .await;
            return Ok(());
        }
    }

    if let Err(err) = service::send_message(
        &state.db,
        message.conversation_id,
        user_id,
        Some(message.thread_id),
        trimmed_content,
        format,
        message.idempotency_key,
    )
    .await
    {
        tracing::error!(
            %err,
            user_id = %user_id,
            conversation_id = %message.conversation_id,
            thread_id = %message.thread_id,
            "failed to persist thread websocket message"
        );
        let _ = send_protocol_error(
            outbound_tx,
            "message_send_failed",
            "send_thread_message",
            locale,
            None,
            "Could not send thread message",
        )
        .await;
    }

    Ok(())
}

async fn handle_typing_message(
    state: &AppState,
    user_id: Uuid,
    outbound_tx: &WsSender,
    locale: &str,
    typing: paw_proto::TypingMsg,
    is_start: bool,
) -> anyhow::Result<()> {
    if let Some(thread_id) = typing.thread_id {
        handle_thread_typing_message(
            state,
            user_id,
            outbound_tx,
            locale,
            ThreadTypingMsg {
                v: typing.v,
                conversation_id: typing.conversation_id,
                thread_id,
                user_id: typing.user_id,
            },
            is_start,
        )
        .await?;
        return Ok(());
    }

    let others: Vec<Uuid> = conversation_members(state, typing.conversation_id)
        .await?
        .into_iter()
        .filter(|&member_id| member_id != user_id)
        .collect();
    let payload = if is_start {
        serde_json::to_string(&ServerMessage::TypingStart(typing))?
    } else {
        serde_json::to_string(&ServerMessage::TypingStop(typing))?
    };
    state.hub.broadcast_to_conversation(others, &payload).await;
    Ok(())
}

async fn handle_thread_typing_message(
    state: &AppState,
    user_id: Uuid,
    outbound_tx: &WsSender,
    locale: &str,
    mut typing: ThreadTypingMsg,
    is_start: bool,
) -> anyhow::Result<()> {
    if !ensure_thread_scope(
        state,
        user_id,
        outbound_tx,
        locale,
        typing.conversation_id,
        typing.thread_id,
        if is_start {
            "typing_thread_start"
        } else {
            "typing_thread_end"
        },
    )
    .await?
    {
        return Ok(());
    }

    typing.user_id = Some(user_id);
    let conversation_id = typing.conversation_id;
    let thread_id = typing.thread_id;
    let others: Vec<Uuid> = conversation_members(state, typing.conversation_id)
        .await?
        .into_iter()
        .filter(|&member_id| member_id != user_id)
        .collect();
    let payload = if is_start {
        serde_json::to_string(&ServerMessage::TypingThreadStart(typing))?
    } else {
        serde_json::to_string(&ServerMessage::TypingThreadEnd(typing))?
    };
    state
        .hub
        .send_to_thread(conversation_id, thread_id, others, &payload)
        .await;
    Ok(())
}

async fn ensure_thread_scope(
    state: &AppState,
    user_id: Uuid,
    outbound_tx: &WsSender,
    locale: &str,
    conversation_id: Uuid,
    thread_id: Uuid,
    ref_type: &str,
) -> anyhow::Result<bool> {
    match service::check_member(&state.db, conversation_id, user_id).await? {
        Membership::Member => {}
        Membership::NotMember => {
            let _ = send_protocol_error(
                outbound_tx,
                "forbidden",
                ref_type,
                locale,
                None,
                "User is not a member of this conversation",
            )
            .await;
            return Ok(false);
        }
        Membership::ConversationNotFound => {
            let _ = send_protocol_error(
                outbound_tx,
                "conversation_not_found",
                ref_type,
                locale,
                None,
                "Conversation not found",
            )
            .await;
            return Ok(false);
        }
    }

    if !thread_exists(state, conversation_id, thread_id).await? {
        let _ = send_protocol_error(
            outbound_tx,
            "thread_not_found",
            ref_type,
            locale,
            None,
            "Thread not found",
        )
        .await;
        return Ok(false);
    }

    Ok(true)
}

async fn thread_exists(
    state: &AppState,
    conversation_id: Uuid,
    thread_id: Uuid,
) -> anyhow::Result<bool> {
    sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(
             SELECT 1
             FROM threads
             WHERE id = $1 AND conversation_id = $2 AND archived_at IS NULL
         )",
    )
    .bind(thread_id)
    .bind(conversation_id)
    .fetch_one(state.db.as_ref())
    .await
    .map_err(Into::into)
}

async fn fetch_messages_after_seq(
    state: &AppState,
    conversation_id: Uuid,
    thread_id: Option<Uuid>,
    last_seq: i64,
) -> anyhow::Result<Vec<MessageReceivedMsg>> {
    let rows: Vec<sqlx::postgres::PgRow> = match thread_id {
        Some(thread_id) => {
            sqlx::query(
                "SELECT id, conversation_id, thread_id, sender_id, content, format, seq, created_at, blocks \
                 FROM messages \
                 WHERE conversation_id = $1 AND thread_id = $2 AND seq > $3 AND is_deleted = FALSE \
                 ORDER BY seq ASC \
                 LIMIT 100",
            )
            .bind(conversation_id)
            .bind(thread_id)
            .bind(last_seq)
            .fetch_all(state.db.as_ref())
            .await?
        }
        None => {
            sqlx::query(
                "SELECT id, conversation_id, thread_id, sender_id, content, format, seq, created_at, blocks \
                 FROM messages \
                 WHERE conversation_id = $1 AND thread_id IS NULL AND seq > $2 AND is_deleted = FALSE \
                 ORDER BY seq ASC \
                 LIMIT 100",
            )
            .bind(conversation_id)
            .bind(last_seq)
            .fetch_all(state.db.as_ref())
            .await?
        }
    };

    let message_ids = rows
        .iter()
        .filter_map(|row| row.try_get::<Uuid, _>("id").ok())
        .collect::<Vec<_>>();
    let mut attachments_by_message =
        service::get_message_attachments_for_messages(&state.db, &message_ids).await?;

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

        let message_id: Uuid = row.try_get("id")?;
        let attachments = attachments_by_message
            .remove(&message_id)
            .unwrap_or_default()
            .into_iter()
            .map(|record| MessageAttachment {
                id: record.id,
                file_type: record.file_type,
                file_url: record.file_url,
                file_size: record.file_size,
                mime_type: record.mime_type,
                thumbnail_url: record.thumbnail_url,
            })
            .collect();

        messages.push(MessageReceivedMsg {
            v: PROTOCOL_VERSION,
            id: message_id,
            conversation_id: row.try_get("conversation_id")?,
            thread_id: row.try_get("thread_id")?,
            sender_id: row.try_get("sender_id")?,
            content: row.try_get("content")?,
            format,
            seq: row.try_get("seq")?,
            created_at: row.try_get("created_at")?,
            blocks,
            attachments,
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
    ref_type: &str,
    locale: &str,
    details: Option<&str>,
    fallback: &str,
) -> anyhow::Result<()> {
    let payload = serde_json::to_string(&ServerMessage::Error(ErrorMsg {
        v: PROTOCOL_VERSION,
        code: code.to_owned(),
        ref_type: ref_type.to_owned(),
        message: details
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| localized_message(code, locale, fallback).to_string()),
    }))?;
    let _ = outbound_tx.send(Message::Text(payload.into()));
    Ok(())
}

fn exceeds_ws_message_size(frame_len: usize) -> bool {
    frame_len > crate::ws::MAX_WS_MESSAGE_SIZE_BYTES
}

#[cfg(test)]
mod tests {
    use super::exceeds_ws_message_size;

    #[test]
    fn websocket_message_size_limit_is_64kb() {
        assert!(!exceeds_ws_message_size(64 * 1024));
        assert!(exceeds_ws_message_size((64 * 1024) + 1));
    }

    #[test]
    fn websocket_connection_limit_is_five_per_user() {
        assert_eq!(crate::ws::MAX_WS_CONNECTIONS_PER_USER, 5);
    }
}
