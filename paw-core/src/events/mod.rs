use serde::{Deserialize, Serialize};

use crate::{
    auth::{AuthState, AuthStep, SessionEvent, SessionExpiryReason},
    core::{CoreRuntime, RuntimeBootstrapReport, RuntimeEffect, RuntimeInitStep},
    db::MessageRecord,
    sync::{
        FinalizedStreamMessage, ScopedSyncCursor, StreamingSession, SyncRequest, ToolCallRecord,
    },
    ws::{WsConnectionState, WsService},
};

fn saturating_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

fn saturating_u64(value: u128) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, uniffi::Enum)]
pub enum AuthStepView {
    AuthMethodSelect,
    PhoneInput,
    OtpVerify,
    DeviceName,
    UsernameSetup,
    Authenticated,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct AuthStateView {
    pub step: AuthStepView,
    pub phone: String,
    pub device_name: String,
    pub username: String,
    pub discoverable_by_phone: bool,
    pub has_session_token: bool,
    pub has_access_token: bool,
    pub has_refresh_token: bool,
    pub is_loading: bool,
    pub error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, uniffi::Enum)]
pub enum ConnectionStateView {
    Disconnected,
    Connecting,
    Connected,
    Retrying,
    Exhausted,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct ConnectionSnapshot {
    pub state: ConnectionStateView,
    pub attempts: u32,
    pub pending_reconnect_delay_ms: Option<u64>,
    pub pending_reconnect_endpoint: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct ReconnectScheduledView {
    pub delay_ms: u64,
    pub endpoint: String,
    pub attempt: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct ReconnectAttemptStartedView {
    pub endpoint: String,
    pub attempt: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct ActiveStreamsClearedView {
    pub count: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct ConversationCursorView {
    pub conversation_id: String,
    pub thread_id: Option<String>,
    pub last_seq: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct RecoveryCursorView {
    pub conversation_id: String,
    pub thread_id: Option<String>,
    pub request_from_seq: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct ToolCallView {
    pub tool: String,
    pub label: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct StreamingSessionView {
    pub stream_id: String,
    pub conversation_id: String,
    pub agent_id: String,
    pub content: String,
    pub current_tool: Option<String>,
    pub current_tool_label: Option<String>,
    pub tool_complete: bool,
    pub is_complete: bool,
    pub tool_history: Vec<ToolCallView>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct MessageRecordView {
    pub id: String,
    pub conversation_id: String,
    pub thread_id: Option<String>,
    pub sender_id: String,
    pub content: String,
    pub format: String,
    pub seq: i64,
    pub created_at_ms: i64,
    pub is_me: bool,
    pub is_agent: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct FinalizedStreamMessageView {
    pub id: String,
    pub conversation_id: String,
    pub sender_id: String,
    pub content: String,
    pub format: String,
    pub seq: i64,
    pub created_at_ms: i64,
    pub tool_calls: Vec<ToolCallView>,
    pub tokens: u32,
    pub duration_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct SyncRequestView {
    pub conversation_id: String,
    pub thread_id: Option<String>,
    pub last_seq: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct AckRequestView {
    pub conversation_id: String,
    pub thread_id: Option<String>,
    pub last_seq: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct DuplicateMessageView {
    pub conversation_id: String,
    pub thread_id: Option<String>,
    pub received_seq: i64,
    pub last_seq: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct GapDetectedView {
    pub conversation_id: String,
    pub thread_id: Option<String>,
    pub expected_seq: i64,
    pub received_seq: i64,
    pub request_from_seq: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct DeviceSyncAppliedView {
    pub conversation_id: String,
    pub thread_id: Option<String>,
    pub applied_count: u32,
    pub highest_seq: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct ThreadCreatedView {
    pub conversation_id: String,
    pub thread_id: String,
    pub root_message_id: String,
    pub title: Option<String>,
    pub created_by: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct ThreadDeletedView {
    pub conversation_id: String,
    pub thread_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct ThreadAgentBindingView {
    pub conversation_id: String,
    pub thread_id: String,
    pub agent_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct DeviceSyncBatchProcessedView {
    pub message_count: u32,
    pub conversation_count: u32,
    pub conversation_ids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, uniffi::Enum)]
pub enum RuntimeInitStepView {
    DatabaseOpened,
    TokensRestored,
    BootstrapSkippedNoStoredTokens,
    SessionValidated,
    WsConnected,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, uniffi::Enum)]
pub enum SessionExpiryReasonView {
    Unauthorized,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct SessionEventView {
    pub reason: SessionExpiryReasonView,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct RuntimeBootstrapReportView {
    pub steps: Vec<RuntimeInitStepView>,
    pub has_tokens: bool,
    pub has_profile: bool,
    pub connected_endpoint: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct RuntimeSnapshot {
    pub connection: ConnectionSnapshot,
    pub cursors: Vec<ConversationCursorView>,
    pub pending_recoveries: Vec<RecoveryCursorView>,
    pub active_streams: Vec<StreamingSessionView>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, uniffi::Enum)]
#[serde(tag = "type", content = "payload")]
pub enum CoreEvent {
    AuthStateChanged(AuthStateView),
    BootstrapProgress(RuntimeBootstrapReportView),
    ConnectionStateChanged(ConnectionSnapshot),
    ReconnectScheduled(ReconnectScheduledView),
    ReconnectAttemptStarted(ReconnectAttemptStartedView),
    ActiveStreamsCleared(ActiveStreamsClearedView),
    SessionInvalidated(SessionEventView),
    SyncRequested(SyncRequestView),
    AckRequested(AckRequestView),
    DuplicateMessage(DuplicateMessageView),
    GapDetected(GapDetectedView),
    DeviceSyncApplied(DeviceSyncAppliedView),
    DeviceSyncBatchProcessed(DeviceSyncBatchProcessedView),
    MessagePersisted(MessageRecordView),
    ThreadCreated(ThreadCreatedView),
    ThreadDeleted(ThreadDeletedView),
    ThreadAgentBound(ThreadAgentBindingView),
    ThreadAgentUnbound(ThreadAgentBindingView),
    StreamUpdated(StreamingSessionView),
    StreamFinalized(FinalizedStreamMessageView),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, uniffi::Enum)]
pub enum CoreEventDomain {
    Lifecycle,
    Connection,
    Sync,
    Streaming,
}

impl CoreEvent {
    pub fn domain(&self) -> CoreEventDomain {
        match self {
            Self::AuthStateChanged(_)
            | Self::BootstrapProgress(_)
            | Self::ActiveStreamsCleared(_)
            | Self::SessionInvalidated(_) => CoreEventDomain::Lifecycle,
            Self::ConnectionStateChanged(_)
            | Self::ReconnectScheduled(_)
            | Self::ReconnectAttemptStarted(_) => CoreEventDomain::Connection,
            Self::SyncRequested(_)
            | Self::AckRequested(_)
            | Self::DuplicateMessage(_)
            | Self::GapDetected(_)
            | Self::DeviceSyncApplied(_)
            | Self::DeviceSyncBatchProcessed(_)
            | Self::MessagePersisted(_) => CoreEventDomain::Sync,
            Self::ThreadCreated(_)
            | Self::ThreadDeleted(_)
            | Self::ThreadAgentBound(_)
            | Self::ThreadAgentUnbound(_) => CoreEventDomain::Lifecycle,
            Self::StreamUpdated(_) | Self::StreamFinalized(_) => CoreEventDomain::Streaming,
        }
    }
}

impl From<&AuthStep> for AuthStepView {
    fn from(value: &AuthStep) -> Self {
        match value {
            AuthStep::AuthMethodSelect => Self::AuthMethodSelect,
            AuthStep::PhoneInput => Self::PhoneInput,
            AuthStep::OtpVerify => Self::OtpVerify,
            AuthStep::DeviceName => Self::DeviceName,
            AuthStep::UsernameSetup => Self::UsernameSetup,
            AuthStep::Authenticated => Self::Authenticated,
        }
    }
}

impl From<&AuthState> for AuthStateView {
    fn from(value: &AuthState) -> Self {
        Self {
            step: (&value.step).into(),
            phone: value.phone.clone(),
            device_name: value.device_name.clone(),
            username: value.username.clone(),
            discoverable_by_phone: value.discoverable_by_phone,
            has_session_token: value.session_token.is_some(),
            has_access_token: value.access_token.is_some(),
            has_refresh_token: value.refresh_token.is_some(),
            is_loading: value.is_loading,
            error: value.error.clone(),
        }
    }
}

impl From<WsConnectionState> for ConnectionStateView {
    fn from(value: WsConnectionState) -> Self {
        match value {
            WsConnectionState::Disconnected => Self::Disconnected,
            WsConnectionState::Connecting => Self::Connecting,
            WsConnectionState::Connected => Self::Connected,
            WsConnectionState::Retrying => Self::Retrying,
            WsConnectionState::Exhausted => Self::Exhausted,
        }
    }
}

impl From<&WsService> for ConnectionSnapshot {
    fn from(value: &WsService) -> Self {
        let pending = value.pending_reconnect();
        Self {
            state: value.connection_state().into(),
            attempts: saturating_u32(value.attempts()),
            pending_reconnect_delay_ms: pending.map(|plan| saturating_u64(plan.delay.as_millis())),
            pending_reconnect_endpoint: pending
                .map(|plan| crate::ws::public_endpoint_label(&plan.uri)),
        }
    }
}

impl From<&crate::sync::ConversationSyncCursor> for ConversationCursorView {
    fn from(value: &crate::sync::ConversationSyncCursor) -> Self {
        Self {
            conversation_id: value.conversation_id.to_string(),
            thread_id: None,
            last_seq: value.last_seq,
        }
    }
}

impl From<&crate::sync::ConversationSyncCursor> for RecoveryCursorView {
    fn from(value: &crate::sync::ConversationSyncCursor) -> Self {
        Self {
            conversation_id: value.conversation_id.to_string(),
            thread_id: None,
            request_from_seq: value.last_seq,
        }
    }
}

impl From<&ScopedSyncCursor> for ConversationCursorView {
    fn from(value: &ScopedSyncCursor) -> Self {
        Self {
            conversation_id: value.conversation_id.to_string(),
            thread_id: value
                .thread_id
                .map(|thread_id: uuid::Uuid| thread_id.to_string()),
            last_seq: value.last_seq,
        }
    }
}

impl From<&ScopedSyncCursor> for RecoveryCursorView {
    fn from(value: &ScopedSyncCursor) -> Self {
        Self {
            conversation_id: value.conversation_id.to_string(),
            thread_id: value
                .thread_id
                .map(|thread_id: uuid::Uuid| thread_id.to_string()),
            request_from_seq: value.last_seq,
        }
    }
}

impl From<&ToolCallRecord> for ToolCallView {
    fn from(value: &ToolCallRecord) -> Self {
        Self {
            tool: value.tool.clone(),
            label: value.label.clone(),
        }
    }
}

impl From<&StreamingSession> for StreamingSessionView {
    fn from(value: &StreamingSession) -> Self {
        Self {
            stream_id: value.stream_id.to_string(),
            conversation_id: value.conversation_id.to_string(),
            agent_id: value.agent_id.to_string(),
            content: value.content.clone(),
            current_tool: value.current_tool.clone(),
            current_tool_label: value.current_tool_label.clone(),
            tool_complete: value.tool_complete,
            is_complete: value.is_complete,
            tool_history: value.tool_history.iter().map(Into::into).collect(),
        }
    }
}

impl From<&MessageRecord> for MessageRecordView {
    fn from(value: &MessageRecord) -> Self {
        Self {
            id: value.id.clone(),
            conversation_id: value.conversation_id.clone(),
            thread_id: value.thread_id.clone(),
            sender_id: value.sender_id.clone(),
            content: value.content.clone(),
            format: value.format.clone(),
            seq: value.seq,
            created_at_ms: value.created_at.timestamp_millis(),
            is_me: value.is_me,
            is_agent: value.is_agent,
        }
    }
}

impl From<&FinalizedStreamMessage> for FinalizedStreamMessageView {
    fn from(value: &FinalizedStreamMessage) -> Self {
        Self {
            id: value.id.to_string(),
            conversation_id: value.conversation_id.to_string(),
            sender_id: value.sender_id.to_string(),
            content: value.content.clone(),
            format: format!("{:?}", value.format).to_lowercase(),
            seq: value.seq,
            created_at_ms: value.created_at.timestamp_millis(),
            tool_calls: value.tool_calls.iter().map(Into::into).collect(),
            tokens: value.tokens,
            duration_ms: value.duration_ms,
        }
    }
}

impl From<&SyncRequest> for SyncRequestView {
    fn from(value: &SyncRequest) -> Self {
        Self {
            conversation_id: value.conversation_id.clone(),
            thread_id: value.thread_id.clone(),
            last_seq: value.last_seq,
        }
    }
}

impl From<&RuntimeEffect> for DuplicateMessageView {
    fn from(value: &RuntimeEffect) -> Self {
        match value {
            RuntimeEffect::DuplicateMessage {
                conversation_id,
                thread_id,
                received_seq,
                last_seq,
            } => Self {
                conversation_id: conversation_id.to_string(),
                thread_id: thread_id.map(|thread_id| thread_id.to_string()),
                received_seq: *received_seq,
                last_seq: *last_seq,
            },
            other => unreachable!(
                "invariant: matched DuplicateMessage effect before conversion, got {other:?}"
            ),
        }
    }
}

impl From<&RuntimeEffect> for GapDetectedView {
    fn from(value: &RuntimeEffect) -> Self {
        match value {
            RuntimeEffect::GapDetected {
                conversation_id,
                thread_id,
                expected_seq,
                received_seq,
                request_from_seq,
            } => Self {
                conversation_id: conversation_id.to_string(),
                thread_id: thread_id.map(|thread_id| thread_id.to_string()),
                expected_seq: *expected_seq,
                received_seq: *received_seq,
                request_from_seq: *request_from_seq,
            },
            other => unreachable!(
                "invariant: matched GapDetected effect before conversion, got {other:?}"
            ),
        }
    }
}

impl From<&RuntimeEffect> for DeviceSyncAppliedView {
    fn from(value: &RuntimeEffect) -> Self {
        match value {
            RuntimeEffect::DeviceSyncApplied {
                conversation_id,
                thread_id,
                applied_count,
                highest_seq,
            } => Self {
                conversation_id: conversation_id.to_string(),
                thread_id: thread_id.map(|thread_id| thread_id.to_string()),
                applied_count: *applied_count,
                highest_seq: *highest_seq,
            },
            other => unreachable!(
                "invariant: matched DeviceSyncApplied effect before conversion, got {other:?}"
            ),
        }
    }
}

impl From<&RuntimeEffect> for DeviceSyncBatchProcessedView {
    fn from(value: &RuntimeEffect) -> Self {
        match value {
            RuntimeEffect::DeviceSyncBatchProcessed {
                message_count,
                conversation_count,
                conversation_ids,
            } => Self {
                message_count: *message_count,
                conversation_count: *conversation_count,
                conversation_ids: conversation_ids.iter().map(ToString::to_string).collect(),
            },
            other => unreachable!("invariant: matched DeviceSyncBatchProcessed effect before conversion, got {other:?}"),
        }
    }
}

impl From<&RuntimeInitStep> for RuntimeInitStepView {
    fn from(value: &RuntimeInitStep) -> Self {
        match value {
            RuntimeInitStep::DatabaseOpened => Self::DatabaseOpened,
            RuntimeInitStep::TokensRestored => Self::TokensRestored,
            RuntimeInitStep::BootstrapSkippedNoStoredTokens => Self::BootstrapSkippedNoStoredTokens,
            RuntimeInitStep::SessionValidated => Self::SessionValidated,
            RuntimeInitStep::WsConnected => Self::WsConnected,
        }
    }
}

impl From<&RuntimeBootstrapReport> for RuntimeBootstrapReportView {
    fn from(value: &RuntimeBootstrapReport) -> Self {
        Self {
            steps: value.steps.iter().map(Into::into).collect(),
            has_tokens: value.tokens.is_some(),
            has_profile: value.profile.is_some(),
            connected_endpoint: value.connected_endpoint.clone(),
        }
    }
}

impl From<&SessionExpiryReason> for SessionExpiryReasonView {
    fn from(value: &SessionExpiryReason) -> Self {
        match value {
            SessionExpiryReason::Unauthorized => Self::Unauthorized,
        }
    }
}

impl From<&SessionEvent> for SessionEventView {
    fn from(value: &SessionEvent) -> Self {
        Self {
            reason: (&value.reason).into(),
        }
    }
}

impl From<&RuntimeEffect> for CoreEvent {
    fn from(value: &RuntimeEffect) -> Self {
        match value {
            RuntimeEffect::BootstrapProgress(report) => Self::BootstrapProgress(report.into()),
            RuntimeEffect::ConnectionStateChanged(snapshot) => {
                Self::ConnectionStateChanged(snapshot.clone())
            }
            RuntimeEffect::ReconnectScheduled {
                delay_ms,
                endpoint,
                attempt,
            } => Self::ReconnectScheduled(ReconnectScheduledView {
                delay_ms: *delay_ms,
                endpoint: endpoint.clone(),
                attempt: *attempt,
            }),
            RuntimeEffect::ReconnectAttemptStarted { endpoint, attempt } => {
                Self::ReconnectAttemptStarted(ReconnectAttemptStartedView {
                    endpoint: endpoint.clone(),
                    attempt: *attempt,
                })
            }
            RuntimeEffect::ActiveStreamsCleared { count } => {
                Self::ActiveStreamsCleared(ActiveStreamsClearedView { count: *count })
            }
            RuntimeEffect::SessionInvalidated(event) => Self::SessionInvalidated(event.into()),
            RuntimeEffect::SyncRequested(request) => Self::SyncRequested(request.into()),
            RuntimeEffect::AckRequested {
                conversation_id,
                thread_id,
                last_seq,
            } => Self::AckRequested(AckRequestView {
                conversation_id: conversation_id.to_string(),
                thread_id: thread_id.map(|thread_id| thread_id.to_string()),
                last_seq: *last_seq,
            }),
            RuntimeEffect::DuplicateMessage { .. } => Self::DuplicateMessage(value.into()),
            RuntimeEffect::GapDetected { .. } => Self::GapDetected(value.into()),
            RuntimeEffect::DeviceSyncApplied { .. } => Self::DeviceSyncApplied(value.into()),
            RuntimeEffect::DeviceSyncBatchProcessed { .. } => {
                Self::DeviceSyncBatchProcessed(value.into())
            }
            RuntimeEffect::MessagePersisted(record) => Self::MessagePersisted(record.into()),
            RuntimeEffect::ThreadCreated {
                conversation_id,
                thread_id,
                root_message_id,
                title,
                created_by,
            } => Self::ThreadCreated(ThreadCreatedView {
                conversation_id: conversation_id.to_string(),
                thread_id: thread_id.to_string(),
                root_message_id: root_message_id.to_string(),
                title: title.clone(),
                created_by: created_by.to_string(),
            }),
            RuntimeEffect::ThreadDeleted {
                conversation_id,
                thread_id,
            } => Self::ThreadDeleted(ThreadDeletedView {
                conversation_id: conversation_id.to_string(),
                thread_id: thread_id.to_string(),
            }),
            RuntimeEffect::ThreadAgentBound {
                conversation_id,
                thread_id,
                agent_id,
            } => Self::ThreadAgentBound(ThreadAgentBindingView {
                conversation_id: conversation_id.to_string(),
                thread_id: thread_id.to_string(),
                agent_id: agent_id.to_string(),
            }),
            RuntimeEffect::ThreadAgentUnbound {
                conversation_id,
                thread_id,
                agent_id,
            } => Self::ThreadAgentUnbound(ThreadAgentBindingView {
                conversation_id: conversation_id.to_string(),
                thread_id: thread_id.to_string(),
                agent_id: agent_id.to_string(),
            }),
            RuntimeEffect::StreamUpdated(stream) => Self::StreamUpdated(stream.into()),
            RuntimeEffect::StreamFinalized(message) => Self::StreamFinalized(message.into()),
        }
    }
}

impl RuntimeSnapshot {
    pub fn capture(runtime: &CoreRuntime) -> Self {
        runtime.snapshot()
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, time::Duration};

    use chrono::{TimeZone, Utc};
    use paw_proto::MessageFormat;
    use reqwest::Url;
    use uuid::Uuid;

    use crate::{
        core::RuntimeEffect,
        sync::{FinalizedStreamMessage, ToolCallRecord},
        ws::{ReconnectionManager, WsService},
    };

    use super::*;

    #[test]
    fn auth_view_hides_secret_values_but_preserves_step() {
        let state = AuthState {
            step: AuthStep::UsernameSetup,
            phone: "+821012345678".into(),
            device_name: "paw".into(),
            username: "friend".into(),
            discoverable_by_phone: true,
            session_token: Some("session".into()),
            access_token: Some("access".into()),
            refresh_token: Some("refresh".into()),
            is_loading: false,
            error: None,
        };

        let view = AuthStateView::from(&state);
        assert_eq!(view.step, AuthStepView::UsernameSetup);
        assert!(view.has_session_token);
        assert!(view.has_access_token);
        assert!(view.has_refresh_token);
    }

    #[test]
    fn runtime_effects_convert_to_serializable_core_events() {
        let effect = RuntimeEffect::DuplicateMessage {
            conversation_id: Uuid::nil(),
            thread_id: None,
            received_seq: 7,
            last_seq: 9,
        };

        let event = CoreEvent::from(&effect);
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"DuplicateMessage\""));
        assert!(json.contains("\"received_seq\":7"));
        assert!(json.contains("\"last_seq\":9"));
    }

    #[test]
    fn session_invalidated_effects_convert_to_serializable_core_events() {
        let effect = RuntimeEffect::SessionInvalidated(SessionEvent {
            reason: SessionExpiryReason::Unauthorized,
        });

        let event = CoreEvent::from(&effect);
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"SessionInvalidated\""));
        assert!(json.contains("\"Unauthorized\""));
    }

    #[test]
    fn active_streams_cleared_effects_convert_to_serializable_core_events() {
        let effect = RuntimeEffect::ActiveStreamsCleared { count: 2 };

        let event = CoreEvent::from(&effect);
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"ActiveStreamsCleared\""));
        assert!(json.contains("\"count\":2"));
    }

    #[test]
    fn reconnect_scheduled_effects_convert_to_serializable_core_events() {
        let effect = RuntimeEffect::ReconnectScheduled {
            delay_ms: 1_000,
            endpoint: "wss://paw.example/ws".into(),
            attempt: 2,
        };

        let event = CoreEvent::from(&effect);
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"ReconnectScheduled\""));
        assert!(json.contains("\"delay_ms\":1000"));
        assert!(json.contains("\"attempt\":2"));
    }

    #[test]
    fn reconnect_attempt_started_effects_convert_to_serializable_core_events() {
        let effect = RuntimeEffect::ReconnectAttemptStarted {
            endpoint: "wss://paw.example/ws".into(),
            attempt: 3,
        };

        let event = CoreEvent::from(&effect);
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"ReconnectAttemptStarted\""));
        assert!(json.contains("\"attempt\":3"));
    }

    #[test]
    fn device_sync_batch_processed_effects_convert_to_serializable_core_events() {
        let effect = RuntimeEffect::DeviceSyncBatchProcessed {
            message_count: 3,
            conversation_count: 1,
            conversation_ids: vec![uuid::Uuid::nil()],
        };

        let event = CoreEvent::from(&effect);
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"DeviceSyncBatchProcessed\""));
        assert!(json.contains("\"message_count\":3"));
        assert!(json.contains("\"conversation_count\":1"));
        assert!(json.contains("\"conversation_ids\""));
    }

    #[test]
    fn thread_lifecycle_effects_convert_to_serializable_core_events() {
        let conversation_id = Uuid::new_v4();
        let thread_id = Uuid::new_v4();
        let root_message_id = Uuid::new_v4();
        let agent_id = Uuid::new_v4();
        let created_by = Uuid::new_v4();

        let created = CoreEvent::from(&RuntimeEffect::ThreadCreated {
            conversation_id,
            thread_id,
            root_message_id,
            title: Some("planning".into()),
            created_by,
        });
        let bound = CoreEvent::from(&RuntimeEffect::ThreadAgentBound {
            conversation_id,
            thread_id,
            agent_id,
        });

        let created_json = serde_json::to_string(&created).unwrap();
        let bound_json = serde_json::to_string(&bound).unwrap();

        assert!(created_json.contains("\"ThreadCreated\""));
        assert!(created_json.contains(&thread_id.to_string()));
        assert!(bound_json.contains("\"ThreadAgentBound\""));
        assert!(bound_json.contains(&agent_id.to_string()));
    }

    #[test]
    fn core_event_domain_groups_exported_events_by_semantics() {
        assert_eq!(
            CoreEvent::ReconnectScheduled(ReconnectScheduledView {
                delay_ms: 1_000,
                endpoint: "wss://paw.example/ws".into(),
                attempt: 1,
            })
            .domain(),
            CoreEventDomain::Connection
        );
        assert_eq!(
            CoreEvent::DeviceSyncBatchProcessed(DeviceSyncBatchProcessedView {
                message_count: 0,
                conversation_count: 0,
                conversation_ids: vec![],
            })
            .domain(),
            CoreEventDomain::Sync
        );
        assert_eq!(
            CoreEvent::SessionInvalidated(SessionEventView {
                reason: SessionExpiryReasonView::Unauthorized,
            })
            .domain(),
            CoreEventDomain::Lifecycle
        );
    }

    #[test]
    fn connection_snapshot_tracks_retry_plan_metadata() {
        struct NoopTransport;

        #[async_trait::async_trait]
        impl crate::ws::WsTransport for NoopTransport {
            async fn connect(&self, _uri: Url) -> Result<(), crate::ws::WsServiceError> {
                Ok(())
            }
            async fn send(
                &self,
                _message: paw_proto::ClientMessage,
            ) -> Result<(), crate::ws::WsServiceError> {
                Ok(())
            }
            async fn close(&self) -> Result<(), crate::ws::WsServiceError> {
                Ok(())
            }
        }

        let transport = Arc::new(NoopTransport);
        let mut service = WsService::new(
            "https://example.com",
            transport,
            ReconnectionManager::new(3, vec![Duration::from_secs(1)]),
        );
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(service.connect_with_access_token("token"))
            .unwrap();
        service.on_transport_error();

        let snapshot = ConnectionSnapshot::from(&service);
        assert_eq!(snapshot.state, ConnectionStateView::Retrying);
        assert_eq!(snapshot.attempts, 1);
        assert_eq!(snapshot.pending_reconnect_delay_ms, Some(1_000));
        assert_eq!(
            snapshot.pending_reconnect_endpoint.as_deref(),
            Some("wss://example.com/ws")
        );
    }

    #[test]
    fn finalized_stream_message_view_uses_platform_friendly_scalars() {
        let message = FinalizedStreamMessage {
            id: Uuid::nil(),
            conversation_id: Uuid::nil(),
            sender_id: Uuid::nil(),
            content: "hello".into(),
            format: MessageFormat::Markdown,
            seq: 4,
            created_at: Utc.timestamp_opt(10, 0).single().unwrap(),
            tool_calls: vec![ToolCallRecord {
                tool: "search".into(),
                label: "Searching".into(),
            }],
            tokens: 12,
            duration_ms: 300,
        };

        let view = FinalizedStreamMessageView::from(&message);
        assert_eq!(view.format, "markdown");
        assert_eq!(view.created_at_ms, 10_000);
        assert_eq!(view.tool_calls.len(), 1);
    }
}
