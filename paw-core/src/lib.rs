#![allow(clippy::empty_line_after_doc_comments)]

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
    ConnectionStateView, ConversationCursorView, CoreEvent, DeviceSyncAppliedView,
    DeviceSyncBatchProcessedView, DuplicateMessageView, FinalizedStreamMessageView,
    GapDetectedView, MessageRecordView, ReconnectAttemptStartedView, ReconnectScheduledView,
    RuntimeBootstrapReportView, RuntimeInitStepView, RuntimeSnapshot, SessionEventView,
    SessionExpiryReasonView, StreamingSessionView, SyncRequestView, ToolCallView,
};
pub use http::{
    AddMemberResponse, ApiClient, ApiError, ApiErrorKind, ApiResult, AuthTokens,
    ConversationListItem, CreateConversationResponse, ErrorPayload, GetMessagesResponse,
    HttpClientError, KeyBundle, MediaAttachment, MessageRecord as HttpMessageRecord, OneTimeKey,
    RefreshTokenResponse, RegisterDeviceRequest, RemoveMemberResponse, RequestOtpResponse,
    SendMessageRequest, SendMessageResponse, UpdateMeRequest, UploadKeysRequest, UserProfile,
    VerifyOtpResponse as HttpVerifyOtpResponse,
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
            pending_reconnect_uri: None,
        },
        cursors: Vec::new(),
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
