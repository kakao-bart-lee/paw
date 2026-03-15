//! Paw WebSocket Protocol Types v1
//!
//! All messages MUST include the `v` field (currently `1`).
//! This enables future protocol evolution without breaking clients.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const PROTOCOL_VERSION: u8 = 1;

/// All WebSocket frames implement this trait
pub trait PawMessage {
    fn version(&self) -> u8 {
        PROTOCOL_VERSION
    }
    fn message_type(&self) -> &str;
}

// ─── Envelope ────────────────────────────────────────────────────────────
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    Connect(ConnectMsg),
    MessageSend(MessageSendMsg),
    TypingStart(TypingMsg),
    TypingStop(TypingMsg),
    MessageAck(MessageAckMsg),
    Sync(SyncMsg),
    DeviceSync(DeviceSyncRequest),
    ThreadCreate(ThreadCreateMsg),
    ThreadBindAgent(ThreadBindAgentMsg),
    ThreadUnbindAgent(ThreadUnbindAgentMsg),
    ThreadDelete(ThreadDeleteMsg),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    HelloOk(HelloOkMsg),
    HelloError(HelloErrorMsg),
    Error(ErrorMsg),
    MessageReceived(MessageReceivedMsg),
    MessageForwarded(MessageForwardedMsg),
    DeviceSyncResponse(DeviceSyncResponse),
    TypingStart(TypingMsg),
    TypingStop(TypingMsg),
    PresenceUpdate(PresenceUpdateMsg),
    ThreadCreated(ThreadCreatedMsg),
    ThreadAgentBound(ThreadAgentBoundMsg),
    ThreadAgentUnbound(ThreadAgentUnboundMsg),
    ThreadDeleted(ThreadDeletedMsg),
    // Phase 2 streaming (reserved, not implemented in Phase 1)
    StreamStart(StreamStartMsg),
    ContentDelta(ContentDeltaMsg),
    ToolStart(ToolStartMsg),
    ToolEnd(ToolEndMsg),
    StreamEnd(StreamEndMsg),
}

// ─── Client Messages ─────────────────────────────────────────────────────
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectMsg {
    pub v: u8,
    pub token: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub capabilities: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageSendMsg {
    pub v: u8,
    pub conversation_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub thread_id: Option<Uuid>,
    pub content: String,
    pub format: MessageFormat,
    #[serde(default)]
    pub blocks: Vec<serde_json::Value>,
    pub idempotency_key: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypingMsg {
    pub v: u8,
    pub conversation_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub thread_id: Option<Uuid>,
    /// Injected by server before fan-out; absent in client→server direction.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub user_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageAckMsg {
    pub v: u8,
    pub conversation_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub thread_id: Option<Uuid>,
    pub last_seq: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncMsg {
    pub v: u8,
    pub conversation_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub thread_id: Option<Uuid>,
    pub last_seq: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadSyncEntry {
    pub thread_id: Uuid,
    pub last_seq: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvSyncState {
    pub conversation_id: Uuid,
    pub last_seq: i64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub threads: Vec<ThreadSyncEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceSyncRequest {
    pub v: u8,
    pub conversations: Vec<ConvSyncState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadCreateMsg {
    pub v: u8,
    pub conversation_id: Uuid,
    pub root_message_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadBindAgentMsg {
    pub v: u8,
    pub conversation_id: Uuid,
    pub thread_id: Uuid,
    pub agent_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadUnbindAgentMsg {
    pub v: u8,
    pub conversation_id: Uuid,
    pub thread_id: Uuid,
    pub agent_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadDeleteMsg {
    pub v: u8,
    pub conversation_id: Uuid,
    pub thread_id: Uuid,
}

// ─── Server Messages ──────────────────────────────────────────────────────
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelloOkMsg {
    pub v: u8,
    pub user_id: Uuid,
    pub server_time: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub capabilities: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelloErrorMsg {
    pub v: u8,
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub details: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorMsg {
    pub v: u8,
    pub code: String,
    pub ref_type: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageReceivedMsg {
    pub v: u8,
    pub id: Uuid,
    pub conversation_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub thread_id: Option<Uuid>,
    pub sender_id: Uuid,
    pub content: String,
    pub format: MessageFormat,
    pub seq: i64,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub blocks: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForwardedFromMsg {
    pub message_id: Uuid,
    pub conversation_id: Uuid,
    pub sender_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageForwardedMsg {
    pub v: u8,
    pub id: Uuid,
    pub conversation_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub thread_id: Option<Uuid>,
    pub sender_id: Uuid,
    pub content: String,
    pub format: MessageFormat,
    pub seq: i64,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub blocks: Vec<serde_json::Value>,
    pub forwarded_from: ForwardedFromMsg,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceSyncResponse {
    pub v: u8,
    #[serde(default)]
    pub conversations: Vec<ConvSyncState>,
    pub messages: Vec<MessageReceivedMsg>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceUpdateMsg {
    pub v: u8,
    pub user_id: Uuid,
    pub online: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadCreatedMsg {
    pub v: u8,
    pub conversation_id: Uuid,
    pub thread_id: Uuid,
    pub root_message_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub title: Option<String>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadAgentBoundMsg {
    pub v: u8,
    pub conversation_id: Uuid,
    pub thread_id: Uuid,
    pub agent_id: Uuid,
    pub bound_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadAgentUnboundMsg {
    pub v: u8,
    pub conversation_id: Uuid,
    pub thread_id: Uuid,
    pub agent_id: Uuid,
    pub unbound_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadDeletedMsg {
    pub v: u8,
    pub conversation_id: Uuid,
    pub thread_id: Uuid,
    pub deleted_by: Uuid,
    pub deleted_at: DateTime<Utc>,
}

// ─── Phase 2 Streaming (types reserved, not used in Phase 1) ─────────────
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamStartMsg {
    pub v: u8,
    pub conversation_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub thread_id: Option<Uuid>,
    pub agent_id: Uuid,
    pub stream_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentDeltaMsg {
    pub v: u8,
    pub stream_id: Uuid,
    pub delta: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolStartMsg {
    pub v: u8,
    pub stream_id: Uuid,
    pub tool: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolEndMsg {
    pub v: u8,
    pub stream_id: Uuid,
    pub tool: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamEndMsg {
    pub v: u8,
    pub stream_id: Uuid,
    pub tokens: u32,
    pub duration_ms: u64,
}

// ─── Agent Gateway ───────────────────────────────────────────────────────

/// Agent Gateway: context sent to agent when a user sends a message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboundContext {
    pub v: u8,
    pub message: MessageReceivedMsg,
    pub conversation_id: Uuid,
    pub recent_messages: Vec<MessageReceivedMsg>,
}

/// Agent → Server: agent's response message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponseMsg {
    pub v: u8,
    pub conversation_id: Uuid,
    pub content: String,
    pub format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContextEvent {
    MessageCreated(ContextMessageCreatedMsg),
    MessageEdited(ContextMessageEditedMsg),
    MessageDeleted(ContextMessageDeletedMsg),
    MemberJoined(ContextMemberJoinedMsg),
    MemberLeft(ContextMemberLeftMsg),
    ThreadCreated(ContextThreadCreatedMsg),
    ConversationSettingsChanged(ContextConversationSettingsChangedMsg),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMessageCreatedMsg {
    pub v: u8,
    pub conversation_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub message: MessageReceivedMsg,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMessageEditedMsg {
    pub v: u8,
    pub conversation_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub thread_id: Option<Uuid>,
    pub message_id: Uuid,
    pub edited_by: Uuid,
    pub content: String,
    pub format: MessageFormat,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMessageDeletedMsg {
    pub v: u8,
    pub conversation_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub thread_id: Option<Uuid>,
    pub message_id: Uuid,
    pub deleted_by: Uuid,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMemberJoinedMsg {
    pub v: u8,
    pub conversation_id: Uuid,
    pub member_id: Uuid,
    pub joined_by: Uuid,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMemberLeftMsg {
    pub v: u8,
    pub conversation_id: Uuid,
    pub member_id: Uuid,
    pub left_by: Uuid,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextThreadCreatedMsg {
    pub v: u8,
    pub conversation_id: Uuid,
    pub thread_id: Uuid,
    pub root_message_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub title: Option<String>,
    pub created_by: Uuid,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextConversationSettingsChangedMsg {
    pub v: u8,
    pub conversation_id: Uuid,
    pub changed_by: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub changes: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentStreamMsg {
    StreamStart(StreamStartMsg),
    ContentDelta(ContentDeltaMsg),
    ToolStart(ToolStartMsg),
    ToolEnd(ToolEndMsg),
    StreamEnd(StreamEndMsg),
}

// ─── Shared Enums ────────────────────────────────────────────────────────
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum MessageFormat {
    #[default]
    Markdown,
    Plain,
}

// ─── Tests ───────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_connect_msg_has_version() {
        let msg = ConnectMsg {
            v: 1,
            token: "test".into(),
            capabilities: None,
        };
        let json = serde_json::to_value(&msg).unwrap();
        assert_eq!(json["v"], 1, "All messages must include v field");
    }

    #[test]
    fn test_message_send_roundtrip() {
        let msg = MessageSendMsg {
            v: 1,
            conversation_id: Uuid::new_v4(),
            thread_id: None,
            content: "Hello, Paw!".into(),
            format: MessageFormat::Markdown,
            blocks: vec![],
            idempotency_key: Uuid::new_v4(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let _: MessageSendMsg = serde_json::from_str(&json).unwrap();
    }

    #[test]
    fn test_message_forwarded_roundtrip() {
        let msg = ServerMessage::MessageForwarded(MessageForwardedMsg {
            v: PROTOCOL_VERSION,
            id: Uuid::new_v4(),
            conversation_id: Uuid::new_v4(),
            thread_id: None,
            sender_id: Uuid::new_v4(),
            content: "Forwarded content".to_string(),
            format: MessageFormat::Markdown,
            seq: 12,
            created_at: "2026-03-16T00:00:00Z".parse::<DateTime<Utc>>().unwrap(),
            blocks: Vec::new(),
            forwarded_from: ForwardedFromMsg {
                message_id: Uuid::new_v4(),
                conversation_id: Uuid::new_v4(),
                sender_id: Uuid::new_v4(),
            },
        });

        let json = serde_json::to_value(&msg).unwrap();
        assert_eq!(json["type"], "message_forwarded");
        assert!(json["forwarded_from"]["message_id"].is_string());
        assert!(json["forwarded_from"]["conversation_id"].is_string());
        assert!(json["forwarded_from"]["sender_id"].is_string());

        let parsed: ServerMessage = serde_json::from_value(json).unwrap();
        match parsed {
            ServerMessage::MessageForwarded(forwarded) => {
                assert_eq!(forwarded.v, PROTOCOL_VERSION);
                assert_eq!(forwarded.seq, 12);
                assert!(!forwarded.content.is_empty());
            }
            _ => panic!("expected MessageForwarded variant"),
        }
    }

    #[test]
    fn test_server_message_version_required() {
        let json = r#"{"type": "hello_ok", "user_id": "550e8400-e29b-41d4-a716-446655440000", "server_time": "2026-01-01T00:00:00Z"}"#;
        // Without v field - should fail (v is required)
        let result: Result<HelloOkMsg, _> = serde_json::from_str(json);
        assert!(result.is_err(), "Messages without v field must be rejected");
    }

    #[test]
    fn test_device_sync_request_roundtrip() {
        let conversation_id = Uuid::new_v4();
        let msg = ClientMessage::DeviceSync(DeviceSyncRequest {
            v: PROTOCOL_VERSION,
            conversations: vec![ConvSyncState {
                conversation_id,
                last_seq: 17,
                threads: vec![ThreadSyncEntry {
                    thread_id: Uuid::new_v4(),
                    last_seq: 3,
                }],
            }],
        });

        let json = serde_json::to_value(&msg).unwrap();
        assert_eq!(json["type"], "device_sync");
        assert_eq!(json["v"], PROTOCOL_VERSION);

        let parsed: ClientMessage = serde_json::from_value(json).unwrap();
        match parsed {
            ClientMessage::DeviceSync(request) => {
                assert_eq!(request.v, PROTOCOL_VERSION);
                assert_eq!(request.conversations.len(), 1);
                assert_eq!(request.conversations[0].conversation_id, conversation_id);
                assert_eq!(request.conversations[0].last_seq, 17);
                assert_eq!(request.conversations[0].threads.len(), 1);
                assert_eq!(request.conversations[0].threads[0].last_seq, 3);
            }
            _ => panic!("expected DeviceSync variant"),
        }
    }

    #[test]
    fn test_thread_create_roundtrip() {
        let conversation_id = Uuid::new_v4();
        let root_message_id = Uuid::new_v4();
        let msg = ClientMessage::ThreadCreate(ThreadCreateMsg {
            v: PROTOCOL_VERSION,
            conversation_id,
            root_message_id,
            title: Some("A thread".into()),
        });

        let json = serde_json::to_value(&msg).unwrap();
        assert_eq!(json["type"], "thread_create");
        assert_eq!(json["conversation_id"], conversation_id.to_string());
        assert_eq!(json["root_message_id"], root_message_id.to_string());
        assert_eq!(json["title"], "A thread");

        let parsed: ClientMessage = serde_json::from_value(json).unwrap();
        match parsed {
            ClientMessage::ThreadCreate(thread_create) => {
                assert_eq!(thread_create.v, PROTOCOL_VERSION);
                assert_eq!(thread_create.conversation_id, conversation_id);
                assert_eq!(thread_create.root_message_id, root_message_id);
                assert_eq!(thread_create.title.as_deref(), Some("A thread"));
            }
            _ => panic!("expected ThreadCreate variant"),
        }
    }

    #[test]
    fn test_thread_created_roundtrip() {
        let conversation_id = Uuid::new_v4();
        let thread_id = Uuid::new_v4();
        let root_message_id = Uuid::new_v4();
        let created_by = Uuid::new_v4();
        let created_at = "2026-03-15T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        let msg = ServerMessage::ThreadCreated(ThreadCreatedMsg {
            v: PROTOCOL_VERSION,
            conversation_id,
            thread_id,
            root_message_id,
            title: Some("Threads".into()),
            created_by,
            created_at,
        });

        let json = serde_json::to_value(&msg).unwrap();
        assert_eq!(json["type"], "thread_created");
        assert_eq!(json["thread_id"], thread_id.to_string());
        assert_eq!(json["created_by"], created_by.to_string());

        let parsed: ServerMessage = serde_json::from_value(json).unwrap();
        match parsed {
            ServerMessage::ThreadCreated(thread_created) => {
                assert_eq!(thread_created.v, PROTOCOL_VERSION);
                assert_eq!(thread_created.conversation_id, conversation_id);
                assert_eq!(thread_created.thread_id, thread_id);
                assert_eq!(thread_created.root_message_id, root_message_id);
                assert_eq!(thread_created.title.as_deref(), Some("Threads"));
                assert_eq!(thread_created.created_by, created_by);
                assert_eq!(thread_created.created_at, created_at);
            }
            _ => panic!("expected ThreadCreated variant"),
        }
    }

    #[test]
    fn test_error_roundtrip() {
        let msg = ServerMessage::Error(ErrorMsg {
            v: PROTOCOL_VERSION,
            code: "agent_already_bound".into(),
            ref_type: "thread_bind_agent".into(),
            message: "Agent is already bound to another thread in this conversation".into(),
        });

        let json = serde_json::to_value(&msg).unwrap();
        assert_eq!(json["type"], "error");
        assert_eq!(json["code"], "agent_already_bound");
        assert_eq!(json["ref_type"], "thread_bind_agent");

        let parsed: ServerMessage = serde_json::from_value(json).unwrap();
        match parsed {
            ServerMessage::Error(error) => {
                assert_eq!(error.v, PROTOCOL_VERSION);
                assert_eq!(error.code, "agent_already_bound");
                assert_eq!(error.ref_type, "thread_bind_agent");
            }
            _ => panic!("expected Error variant"),
        }
    }

    #[test]
    fn test_optional_thread_id_and_capabilities_deserialize_when_absent() {
        let connect_json = r#"{"v":1,"token":"jwt-token"}"#;
        let connect: ConnectMsg = serde_json::from_str(connect_json).unwrap();
        assert!(connect.capabilities.is_none());

        let sync_json =
            r#"{"v":1,"conversation_id":"550e8400-e29b-41d4-a716-446655440000","last_seq":42}"#;
        let sync: SyncMsg = serde_json::from_str(sync_json).unwrap();
        assert!(sync.thread_id.is_none());
    }

    #[test]
    fn test_context_event_message_created_roundtrip() {
        let conversation_id = Uuid::new_v4();
        let message = MessageReceivedMsg {
            v: PROTOCOL_VERSION,
            id: Uuid::new_v4(),
            conversation_id,
            thread_id: None,
            sender_id: Uuid::new_v4(),
            content: "hello agent".into(),
            format: MessageFormat::Markdown,
            seq: 7,
            created_at: "2026-03-16T00:00:00Z".parse::<DateTime<Utc>>().unwrap(),
            blocks: Vec::new(),
        };
        let event = ContextEvent::MessageCreated(ContextMessageCreatedMsg {
            v: PROTOCOL_VERSION,
            conversation_id,
            occurred_at: message.created_at,
            message: message.clone(),
        });

        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["type"], "message_created");
        assert_eq!(json["conversation_id"], conversation_id.to_string());
        assert_eq!(json["message"]["id"], message.id.to_string());

        let parsed: ContextEvent = serde_json::from_value(json).unwrap();
        match parsed {
            ContextEvent::MessageCreated(created) => {
                assert_eq!(created.v, PROTOCOL_VERSION);
                assert_eq!(created.conversation_id, conversation_id);
                assert_eq!(created.message.id, message.id);
                assert_eq!(created.message.content, "hello agent");
            }
            _ => panic!("expected message_created variant"),
        }
    }

    #[test]
    fn test_context_event_settings_changed_roundtrip() {
        let conversation_id = Uuid::new_v4();
        let changed_by = Uuid::new_v4();
        let occurred_at = "2026-03-16T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        let event =
            ContextEvent::ConversationSettingsChanged(ContextConversationSettingsChangedMsg {
                v: PROTOCOL_VERSION,
                conversation_id,
                changed_by,
                occurred_at,
                changes: json!({ "title": "Design Sync" }),
            });

        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["type"], "conversation_settings_changed");
        assert_eq!(json["changes"]["title"], "Design Sync");

        let parsed: ContextEvent = serde_json::from_value(json).unwrap();
        match parsed {
            ContextEvent::ConversationSettingsChanged(changed) => {
                assert_eq!(changed.v, PROTOCOL_VERSION);
                assert_eq!(changed.conversation_id, conversation_id);
                assert_eq!(changed.changed_by, changed_by);
                assert_eq!(changed.changes["title"], "Design Sync");
            }
            _ => panic!("expected conversation_settings_changed variant"),
        }
    }
}
