#![allow(clippy::empty_line_after_doc_comments)]

use chrono::{DateTime, Utc};
use uuid::Uuid;

pub mod auth;
pub mod core;
pub mod crypto;
pub mod db;
pub mod events;
pub mod http;
pub mod platform;
pub mod search;
pub mod sync;
pub mod ws;

pub use auth::{
    AuthBackendError, AuthClient, AuthState, AuthStateMachine, AuthStateMachineError, AuthStep,
    AuthUserProfile, InMemoryTokenStore, NoopSessionTransport, RegisterDeviceResponse,
    SessionEvent, SessionExpiryReason, SessionTransport, StoredTokens, TokenStore,
    VerifyOtpResponse,
};
pub use core::{
    CoreRuntime, CoreRuntimeError, RuntimeBootstrapReport, RuntimeEffect, RuntimeEffectDomain,
    RuntimeInitStep,
};
pub use crypto::{create_account, decrypt, encrypt, AccountKeys};
pub use db::{AppDatabase, ConversationRecord, DbError, DbResult, MessageRecord};
pub use events::{
    AckRequestView, ActiveStreamsClearedView, AuthStateView, AuthStepView, ConnectionSnapshot,
    ConnectionStateView, ConversationCursorView, CoreEvent, CoreEventDomain, DeviceSyncAppliedView,
    DeviceSyncBatchProcessedView, DuplicateMessageView, FinalizedStreamMessageView,
    GapDetectedView, MessageRecordView, ReconnectAttemptStartedView, ReconnectScheduledView,
    RecoveryCursorView, RuntimeBootstrapReportView, RuntimeInitStepView, RuntimeSnapshot,
    SessionEventView, SessionExpiryReasonView, StreamingSessionView, SyncRequestView, ToolCallView,
};
pub use http::{
    AddMemberResponse, ApiClient, ApiError, ApiErrorKind, ApiResult, AuthTokens,
    ConversationListItem, CreateConversationResponse, ErrorPayload, GetMessagesResponse,
    HttpClientError, KeyBundle, MediaAttachment, MessageRecord as HttpMessageRecord, OneTimeKey,
    RefreshTokenResponse, RegisterDeviceRequest, RemoveMemberResponse, RequestOtpResponse,
    SendMessageRequest, SendMessageResponse, ThreadRecord as HttpThreadRecord,
    ThreadStateSnapshot as HttpThreadStateSnapshot, UpdateMeRequest, UploadKeysRequest,
    UserProfile, VerifyOtpResponse as HttpVerifyOtpResponse,
};
pub use platform::{
    lifecycle_hints_for, DeviceKeyMaterial, DeviceKeyStore, InMemoryDeviceKeyStore,
    InMemoryLifecycleBridge, InMemorySecureTokenVault, LifecycleBridge, LifecycleEvent,
    LifecycleHint, LifecycleState, NoopPushRegistrar, PushPlatform, PushRegistrar,
    PushRegistrationError, PushRegistrationErrorCode, PushRegistrationState,
    PushRegistrationStatus, PushTokenRegistration, SecureStorageAvailability,
    SecureStorageCapabilities, SecureStorageError, SecureStorageErrorCode, SecureStorageProvider,
    SecureTokenVault,
};
pub use search::{SearchResult, SearchService};
pub use sync::{
    ConversationSyncCursor, FinalizedStreamMessage, MessageSyncOutcome, StreamingSession,
    StreamingState, SyncEngine, SyncRequest, SyncService, ToolCallRecord,
};
pub use ws::{
    ReconnectPlan, ReconnectionManager, WsConnectionState, WsService, WsServiceError, WsTransport,
};

#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct CoreApiConfig {
    pub base_url: String,
    pub access_token: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct Thread {
    pub id: String,
    pub conversation_id: String,
    pub root_message_id: String,
    pub title: Option<String>,
    pub created_by: String,
    pub message_count: i32,
    pub last_seq: Option<i64>,
    pub last_message_at_ms: Option<i64>,
    pub created_at_ms: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct Message {
    pub id: String,
    pub conversation_id: String,
    pub thread_id: Option<String>,
    pub thread_seq: Option<i64>,
    pub sender_id: String,
    pub content: String,
    pub format: String,
    pub seq: i64,
    pub created_at_ms: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct Conversation {
    pub id: String,
    pub name: Option<String>,
    pub last_message: Option<String>,
    pub unread_count: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct User {
    pub id: String,
    pub phone: Option<String>,
    pub username: Option<String>,
    pub preferred_locale: Option<String>,
    pub discoverable_by_phone: bool,
    pub phone_verified_at_ms: Option<i64>,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub created_at_ms: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq, uniffi::Enum)]
pub enum AgentPermission {
    ReadMessages,
    SendMessages,
    ManageThread,
    AccessHistory,
    UseTools,
}

#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct CreateThreadInput {
    pub conversation_id: String,
    pub root_message_id: String,
    pub title: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct SendMessageInput {
    pub conversation_id: String,
    pub content: String,
    pub format: String,
    pub idempotency_key: Option<String>,
    pub attachment_ids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct ListMessagesInput {
    pub conversation_id: String,
    pub thread_id: Option<String>,
    pub after_seq: Option<i64>,
    pub limit: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct SyncThreadStateInput {
    pub conversation_id: String,
    pub thread_id: String,
    pub since_seq: Option<i64>,
    pub limit: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct ThreadSyncState {
    pub thread: Thread,
    pub participant_ids: Vec<String>,
    pub message_count: i32,
    pub last_seq: i64,
    pub last_message_at_ms: Option<i64>,
    pub messages: Vec<Message>,
}

#[derive(Clone, Debug, PartialEq, Eq, uniffi::Enum)]
pub enum PawCoreApiErrorKind {
    InvalidInput,
    Unauthorized,
    Forbidden,
    NotFound,
    Server,
    Network,
    Timeout,
    Client,
    InvalidResponse,
    Unknown,
}

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum PawCoreApiError {
    #[error("{message}")]
    RequestFailed {
        kind: PawCoreApiErrorKind,
        status_code: Option<u16>,
        code: Option<String>,
        message: String,
    },
}

impl From<ConversationListItem> for Conversation {
    fn from(value: ConversationListItem) -> Self {
        Self {
            id: value.id.to_string(),
            name: value.name,
            last_message: value.last_message,
            unread_count: value.unread_count,
        }
    }
}

impl From<HttpThreadRecord> for Thread {
    fn from(value: HttpThreadRecord) -> Self {
        Self {
            id: value.id.to_string(),
            conversation_id: value.conversation_id.to_string(),
            root_message_id: value.root_message_id.to_string(),
            title: value.title,
            created_by: value.created_by.to_string(),
            message_count: value.message_count,
            last_seq: value.last_seq,
            last_message_at_ms: optional_datetime_to_millis(value.last_message_at),
            created_at_ms: datetime_to_millis(value.created_at),
        }
    }
}

impl From<HttpMessageRecord> for Message {
    fn from(value: HttpMessageRecord) -> Self {
        Self {
            id: value.id.to_string(),
            conversation_id: value.conversation_id.to_string(),
            thread_id: value.thread_id.map(|thread_id| thread_id.to_string()),
            thread_seq: value.thread_seq,
            sender_id: value.sender_id.to_string(),
            content: value.content,
            format: value.format,
            seq: value.seq,
            created_at_ms: datetime_to_millis(value.created_at),
        }
    }
}

impl From<UserProfile> for User {
    fn from(value: UserProfile) -> Self {
        Self {
            id: value.id.to_string(),
            phone: value.phone,
            username: value.username,
            preferred_locale: value.preferred_locale,
            discoverable_by_phone: value.discoverable_by_phone,
            phone_verified_at_ms: optional_datetime_to_millis(value.phone_verified_at),
            display_name: value.display_name,
            avatar_url: value.avatar_url,
            created_at_ms: optional_datetime_to_millis(value.created_at),
        }
    }
}

impl From<HttpClientError> for PawCoreApiError {
    fn from(value: HttpClientError) -> Self {
        Self::RequestFailed {
            kind: match value.kind() {
                ApiErrorKind::Unauthorized => PawCoreApiErrorKind::Unauthorized,
                ApiErrorKind::Forbidden => PawCoreApiErrorKind::Forbidden,
                ApiErrorKind::NotFound => PawCoreApiErrorKind::NotFound,
                ApiErrorKind::Server => PawCoreApiErrorKind::Server,
                ApiErrorKind::Network => PawCoreApiErrorKind::Network,
                ApiErrorKind::Timeout => PawCoreApiErrorKind::Timeout,
                ApiErrorKind::Client => PawCoreApiErrorKind::Client,
                ApiErrorKind::InvalidResponse => PawCoreApiErrorKind::InvalidResponse,
                ApiErrorKind::Unknown => PawCoreApiErrorKind::Unknown,
            },
            status_code: value.status_code(),
            code: value.code().map(str::to_owned),
            message: value.message().to_owned(),
        }
    }
}

fn datetime_to_millis(value: DateTime<Utc>) -> i64 {
    value.timestamp_millis()
}

fn optional_datetime_to_millis(value: Option<DateTime<Utc>>) -> Option<i64> {
    value.map(datetime_to_millis)
}

fn invalid_input(field: &str, value: impl Into<String>) -> PawCoreApiError {
    PawCoreApiError::RequestFailed {
        kind: PawCoreApiErrorKind::InvalidInput,
        status_code: None,
        code: Some(format!("invalid_{field}")),
        message: value.into(),
    }
}

fn api_client(config: &CoreApiConfig) -> Result<ApiClient, PawCoreApiError> {
    if config.base_url.trim().is_empty() {
        return Err(invalid_input("base_url", "base_url must not be empty"));
    }

    let mut client = ApiClient::new(&config.base_url).map_err(PawCoreApiError::from)?;
    if let Some(token) = config
        .access_token
        .as_deref()
        .map(str::trim)
        .filter(|token| !token.is_empty())
    {
        client.set_access_token(token.to_owned());
    }
    Ok(client)
}

fn parse_uuid(field: &str, value: &str) -> Result<Uuid, PawCoreApiError> {
    Uuid::parse_str(value)
        .map_err(|_| invalid_input(field, format!("{field} must be a valid UUID")))
}

fn normalize_limit(limit: Option<u32>) -> i64 {
    i64::from(limit.unwrap_or(50).clamp(1, 100))
}

fn resolve_idempotency_key(value: Option<&str>) -> Result<Uuid, PawCoreApiError> {
    match value.map(str::trim).filter(|value| !value.is_empty()) {
        Some(value) => parse_uuid("idempotency_key", value),
        None => Ok(Uuid::new_v4()),
    }
}

fn parse_attachment_ids(values: &[String]) -> Result<Vec<Uuid>, PawCoreApiError> {
    values
        .iter()
        .enumerate()
        .map(|(index, value)| parse_uuid(&format!("attachment_ids[{index}]"), value))
        .collect()
}

#[uniffi::export]
pub async fn get_me(config: CoreApiConfig) -> Result<User, PawCoreApiError> {
    let client = api_client(&config)?;
    let user = client.get_me().await.map_err(PawCoreApiError::from)?;
    Ok(user.into())
}

#[uniffi::export]
pub async fn list_conversations(
    config: CoreApiConfig,
) -> Result<Vec<Conversation>, PawCoreApiError> {
    let client = api_client(&config)?;
    let conversations = client
        .get_conversations()
        .await
        .map_err(PawCoreApiError::from)?;
    Ok(conversations.into_iter().map(Conversation::from).collect())
}

#[uniffi::export]
pub async fn create_thread(
    config: CoreApiConfig,
    input: CreateThreadInput,
) -> Result<Thread, PawCoreApiError> {
    let client = api_client(&config)?;
    let conversation_id = parse_uuid("conversation_id", &input.conversation_id)?;
    let root_message_id = parse_uuid("root_message_id", &input.root_message_id)?;
    let thread = client
        .create_thread(conversation_id, root_message_id, input.title)
        .await
        .map_err(PawCoreApiError::from)?;
    Ok(thread.into())
}

#[uniffi::export]
pub async fn send_message(
    config: CoreApiConfig,
    input: SendMessageInput,
) -> Result<Message, PawCoreApiError> {
    let client = api_client(&config)?;
    let conversation_id = parse_uuid("conversation_id", &input.conversation_id)?;
    let idempotency_key = resolve_idempotency_key(input.idempotency_key.as_deref())?;
    let request = SendMessageRequest {
        content: input.content,
        format: input.format,
        idempotency_key,
        attachment_ids: parse_attachment_ids(&input.attachment_ids)?,
    };

    let sent = client
        .send_message(conversation_id, &request)
        .await
        .map_err(PawCoreApiError::from)?;
    let history = client
        .get_messages(conversation_id, sent.seq.saturating_sub(1), 20)
        .await
        .map_err(PawCoreApiError::from)?;

    history
        .messages
        .into_iter()
        .find(|message| message.id == sent.id)
        .map(Message::from)
        .ok_or_else(|| {
            invalid_input(
                "message_lookup",
                format!(
                    "message {} was accepted but not returned by list_messages",
                    sent.id
                ),
            )
        })
}

#[uniffi::export]
pub async fn list_messages(
    config: CoreApiConfig,
    input: ListMessagesInput,
) -> Result<Vec<Message>, PawCoreApiError> {
    let client = api_client(&config)?;
    let conversation_id = parse_uuid("conversation_id", &input.conversation_id)?;
    let after_seq = input.after_seq.unwrap_or(0).max(0);
    let limit = normalize_limit(input.limit);

    let response = match input.thread_id {
        Some(thread_id) => {
            let thread_id = parse_uuid("thread_id", &thread_id)?;
            client
                .get_thread_messages(conversation_id, thread_id, after_seq, limit)
                .await
        }
        None => client.get_messages(conversation_id, after_seq, limit).await,
    }
    .map_err(PawCoreApiError::from)?;

    Ok(response.messages.into_iter().map(Message::from).collect())
}

#[uniffi::export]
pub async fn sync_thread_state(
    config: CoreApiConfig,
    input: SyncThreadStateInput,
) -> Result<ThreadSyncState, PawCoreApiError> {
    let client = api_client(&config)?;
    let conversation_id = parse_uuid("conversation_id", &input.conversation_id)?;
    let thread_id = parse_uuid("thread_id", &input.thread_id)?;
    let since_seq = input.since_seq.unwrap_or(0).max(0);
    let limit = normalize_limit(input.limit);

    let thread = client
        .get_thread(conversation_id, thread_id)
        .await
        .map_err(PawCoreApiError::from)?;
    let state = client
        .get_thread_state(conversation_id, thread_id)
        .await
        .map_err(PawCoreApiError::from)?;
    let messages = client
        .get_thread_messages(conversation_id, thread_id, since_seq, limit)
        .await
        .map_err(PawCoreApiError::from)?;

    Ok(ThreadSyncState {
        thread: thread.into(),
        participant_ids: state
            .participants
            .into_iter()
            .map(|participant| participant.to_string())
            .collect(),
        message_count: state.message_count,
        last_seq: state.last_seq,
        last_message_at_ms: optional_datetime_to_millis(state.last_message_at),
        messages: messages.messages.into_iter().map(Message::from).collect(),
    })
}

pub fn ping() -> String {
    "paw-core-ok".to_string()
}

pub fn initial_auth_state_view() -> AuthStateView {
    AuthStateView::from(&auth::AuthState::initial())
}

pub fn empty_runtime_snapshot() -> RuntimeSnapshot {
    RuntimeSnapshot {
        connection: ConnectionSnapshot {
            state: ConnectionStateView::Disconnected,
            attempts: 0,
            pending_reconnect_delay_ms: None,
            pending_reconnect_endpoint: None,
        },
        cursors: Vec::new(),
        pending_recoveries: Vec::new(),
        active_streams: Vec::new(),
    }
}

pub fn core_event_json(event: CoreEvent) -> String {
    serde_json::to_string(&event).expect("core event should serialize")
}

pub fn memory_fallback_secure_storage_capabilities() -> SecureStorageCapabilities {
    SecureStorageCapabilities::memory_fallback()
}

pub fn empty_push_registration_state() -> PushRegistrationState {
    PushRegistrationState::unregistered()
}

pub fn lifecycle_hints(event: LifecycleEvent) -> Vec<LifecycleHint> {
    lifecycle_hints_for(&event)
}

uniffi::include_scaffolding!("paw_core");
