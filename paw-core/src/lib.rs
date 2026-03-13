#![allow(clippy::empty_line_after_doc_comments)]

pub mod auth;
pub mod core;
pub mod crypto;
pub mod db;
pub mod events;
pub mod http;
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
    CoreRuntime, CoreRuntimeError, RuntimeBootstrapReport, RuntimeEffect, RuntimeInitStep,
};
pub use crypto::{create_account, decrypt, encrypt, AccountKeys};
pub use db::{AppDatabase, ConversationRecord, DbError, DbResult, MessageRecord};
pub use events::{
    AckRequestView, AuthStateView, AuthStepView, ConnectionSnapshot, ConnectionStateView,
    ConversationCursorView, CoreEvent, FinalizedStreamMessageView, MessageRecordView,
    RuntimeBootstrapReportView, RuntimeInitStepView, RuntimeSnapshot, StreamingSessionView,
    SyncRequestView, ToolCallView,
};
pub use http::{
    AddMemberResponse, ApiClient, ApiError, ApiErrorKind, ApiResult, AuthTokens,
    ConversationListItem, CreateConversationResponse, ErrorPayload, GetMessagesResponse,
    HttpClientError, KeyBundle, MediaAttachment, MessageRecord as HttpMessageRecord, OneTimeKey,
    RefreshTokenResponse, RegisterDeviceRequest, RemoveMemberResponse, RequestOtpResponse,
    SendMessageRequest, SendMessageResponse, UpdateMeRequest, UploadKeysRequest, UserProfile,
    VerifyOtpResponse as HttpVerifyOtpResponse,
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

uniffi::include_scaffolding!("paw_core");
