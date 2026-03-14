use std::{collections::BTreeMap, sync::Arc};

use chrono::Utc;
use paw_proto::{DeviceSyncResponse, ServerMessage};
use uuid::Uuid;

use crate::{
    auth::{
        run_session_reset, AuthBackendError, AuthClient, AuthUserProfile, SessionEvent,
        StoredTokens, TokenStore,
    },
    db::{AppDatabase, DbError, MessageRecord},
    events::{
        ConnectionSnapshot, ConversationCursorView, RecoveryCursorView, RuntimeSnapshot,
        StreamingSessionView,
    },
    sync::{
        ConversationSyncCursor, FinalizedStreamMessage, MessageSyncOutcome, StreamingSession,
        StreamingState, SyncEngine, SyncRequest, SyncService,
    },
    ws::{WsService, WsServiceError},
};

fn saturating_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

fn saturating_u64(value: u128) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}

#[derive(Debug, thiserror::Error)]
pub enum CoreRuntimeError {
    #[error(transparent)]
    Db(#[from] DbError),
    #[error(transparent)]
    Auth(#[from] AuthBackendError),
    #[error(transparent)]
    Ws(#[from] WsServiceError),
    #[error("invalid conversation id in local database: {0}")]
    InvalidConversationId(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeInitStep {
    DatabaseOpened,
    TokensRestored,
    BootstrapSkippedNoStoredTokens,
    SessionValidated,
    WsConnected,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeBootstrapReport {
    pub steps: Vec<RuntimeInitStep>,
    pub tokens: Option<StoredTokens>,
    pub profile: Option<AuthUserProfile>,
    pub connected_endpoint: Option<String>,
}

impl RuntimeBootstrapReport {
    fn db_only() -> Self {
        Self {
            steps: vec![RuntimeInitStep::DatabaseOpened],
            tokens: None,
            profile: None,
            connected_endpoint: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeEffect {
    BootstrapProgress(RuntimeBootstrapReport),
    ConnectionStateChanged(ConnectionSnapshot),
    ReconnectScheduled {
        delay_ms: u64,
        endpoint: String,
        attempt: u32,
    },
    ReconnectAttemptStarted {
        endpoint: String,
        attempt: u32,
    },
    ActiveStreamsCleared {
        count: u32,
    },
    SessionInvalidated(SessionEvent),
    SyncRequested(SyncRequest),
    AckRequested {
        conversation_id: Uuid,
        last_seq: i64,
    },
    DuplicateMessage {
        conversation_id: Uuid,
        received_seq: i64,
        last_seq: i64,
    },
    GapDetected {
        conversation_id: Uuid,
        expected_seq: i64,
        received_seq: i64,
        request_from_seq: i64,
    },
    DeviceSyncApplied {
        conversation_id: Uuid,
        applied_count: u32,
        highest_seq: i64,
    },
    DeviceSyncBatchProcessed {
        message_count: u32,
        conversation_count: u32,
        conversation_ids: Vec<Uuid>,
    },
    MessagePersisted(MessageRecord),
    StreamUpdated(StreamingSession),
    StreamFinalized(FinalizedStreamMessage),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeEffectDomain {
    Lifecycle,
    Connection,
    Sync,
    Streaming,
}

impl RuntimeEffect {
    pub fn domain(&self) -> RuntimeEffectDomain {
        match self {
            Self::BootstrapProgress(_)
            | Self::ActiveStreamsCleared { .. }
            | Self::SessionInvalidated(_) => RuntimeEffectDomain::Lifecycle,
            Self::ConnectionStateChanged(_)
            | Self::ReconnectScheduled { .. }
            | Self::ReconnectAttemptStarted { .. } => RuntimeEffectDomain::Connection,
            Self::SyncRequested(_)
            | Self::AckRequested { .. }
            | Self::DuplicateMessage { .. }
            | Self::GapDetected { .. }
            | Self::DeviceSyncApplied { .. }
            | Self::DeviceSyncBatchProcessed { .. }
            | Self::MessagePersisted(_) => RuntimeEffectDomain::Sync,
            Self::StreamUpdated(_) | Self::StreamFinalized(_) => RuntimeEffectDomain::Streaming,
        }
    }
}

pub struct CoreRuntime {
    db: Arc<AppDatabase>,
    sync_service: SyncService,
    sync_engine: SyncEngine,
    streaming: StreamingState,
    ws_service: WsService,
}

impl CoreRuntime {
    pub fn new(db: Arc<AppDatabase>, ws_service: WsService) -> Result<Self, CoreRuntimeError> {
        let sync_engine = SyncEngine::new(load_cursors(db.as_ref())?);
        let sync_service = SyncService::new(db.clone(), |_conversation_id, _last_seq| {});

        Ok(Self {
            db,
            sync_service,
            sync_engine,
            streaming: StreamingState::default(),
            ws_service,
        })
    }

    pub fn ws_service(&self) -> &WsService {
        &self.ws_service
    }

    pub fn db(&self) -> &Arc<AppDatabase> {
        &self.db
    }

    pub fn snapshot(&self) -> RuntimeSnapshot {
        RuntimeSnapshot {
            connection: ConnectionSnapshot::from(&self.ws_service),
            cursors: self
                .sync_engine
                .cursors()
                .iter()
                .map(ConversationCursorView::from)
                .collect(),
            pending_recoveries: self
                .sync_engine
                .pending_recoveries()
                .iter()
                .map(RecoveryCursorView::from)
                .collect(),
            active_streams: self
                .streaming
                .active_sessions()
                .iter()
                .map(StreamingSessionView::from)
                .collect(),
        }
    }

    pub async fn bootstrap(
        &mut self,
        token_store: &dyn TokenStore,
        auth_client: &dyn AuthClient,
    ) -> Result<RuntimeBootstrapReport, CoreRuntimeError> {
        let mut report = RuntimeBootstrapReport::db_only();

        let Some(tokens) = token_store.read().await else {
            report
                .steps
                .push(RuntimeInitStep::BootstrapSkippedNoStoredTokens);
            return Ok(report);
        };

        report.steps.push(RuntimeInitStep::TokensRestored);
        report.tokens = Some(tokens.clone());

        let profile = auth_client.get_me(&tokens.access_token).await?;
        report.steps.push(RuntimeInitStep::SessionValidated);
        report.profile = Some(profile);

        let uri = self
            .ws_service
            .connect_with_access_token(tokens.access_token.clone())
            .await?;
        report.steps.push(RuntimeInitStep::WsConnected);
        report.connected_endpoint = Some(crate::ws::public_endpoint_label(&uri));

        Ok(report)
    }

    pub async fn bootstrap_effects(
        &mut self,
        token_store: &dyn TokenStore,
        auth_client: &dyn AuthClient,
    ) -> Result<Vec<RuntimeEffect>, CoreRuntimeError> {
        let report = self.bootstrap(token_store, auth_client).await?;
        Ok(vec![
            RuntimeEffect::BootstrapProgress(report),
            RuntimeEffect::ConnectionStateChanged(ConnectionSnapshot::from(&self.ws_service)),
        ])
    }

    pub fn on_transport_error(&mut self) -> Vec<RuntimeEffect> {
        self.ws_service.on_transport_error();
        self.connection_transition_effects()
    }

    pub fn on_transport_closed(&mut self) -> Vec<RuntimeEffect> {
        self.ws_service.on_transport_closed();
        self.connection_transition_effects()
    }

    pub async fn handle_session_event(
        &mut self,
        token_store: &dyn TokenStore,
        event: SessionEvent,
    ) -> Result<Vec<RuntimeEffect>, CoreRuntimeError> {
        let cleared_streams = run_session_reset(token_store, || async {
            self.ws_service.clear_session().await?;
            Ok::<usize, WsServiceError>(self.streaming.clear())
        })
        .await?;

        let mut effects = Vec::with_capacity(3);
        if cleared_streams > 0 {
            effects.push(RuntimeEffect::ActiveStreamsCleared {
                count: saturating_u32(cleared_streams),
            });
        }
        effects.push(RuntimeEffect::SessionInvalidated(event));
        effects.push(RuntimeEffect::ConnectionStateChanged(
            ConnectionSnapshot::from(&self.ws_service),
        ));

        Ok(effects)
    }

    pub async fn reconnect_with_stored_token(
        &mut self,
    ) -> Result<Option<Vec<RuntimeEffect>>, CoreRuntimeError> {
        let Some(uri) = self.ws_service.connect_with_stored_token().await? else {
            return Ok(None);
        };
        Ok(Some(vec![
            RuntimeEffect::ReconnectAttemptStarted {
                endpoint: crate::ws::public_endpoint_label(&uri),
                attempt: saturating_u32(self.ws_service.attempts()),
            },
            RuntimeEffect::ConnectionStateChanged(ConnectionSnapshot::from(&self.ws_service)),
        ]))
    }

    pub async fn disconnect(&mut self) -> Result<RuntimeEffect, CoreRuntimeError> {
        self.ws_service.disconnect().await?;
        Ok(RuntimeEffect::ConnectionStateChanged(
            ConnectionSnapshot::from(&self.ws_service),
        ))
    }

    pub async fn handle_server_message(
        &mut self,
        msg: &ServerMessage,
    ) -> Result<Vec<RuntimeEffect>, CoreRuntimeError> {
        self.ws_service.handle_server_message(msg).await?;

        match msg {
            ServerMessage::HelloOk(_) => {
                self.sync_engine
                    .replace_cursors(load_cursors(self.db.as_ref())?);
                let mut effects = vec![RuntimeEffect::ConnectionStateChanged(
                    ConnectionSnapshot::from(&self.ws_service),
                )];
                effects.extend(
                    self.sync_service
                        .sync_all_conversations()?
                        .into_iter()
                        .map(RuntimeEffect::SyncRequested),
                );
                Ok(effects)
            }
            ServerMessage::MessageReceived(message) => self.handle_message_received(message),
            ServerMessage::DeviceSyncResponse(response) => {
                self.handle_device_sync_response(response)
            }
            ServerMessage::StreamStart(stream_start) => {
                self.streaming.handle_stream_start(stream_start);
                Ok(self
                    .streaming
                    .active_session(stream_start.stream_id)
                    .cloned()
                    .map(RuntimeEffect::StreamUpdated)
                    .into_iter()
                    .collect())
            }
            ServerMessage::ContentDelta(delta) => Ok(self
                .streaming
                .handle_content_delta(delta)
                .then(|| {
                    self.streaming
                        .active_session(delta.stream_id)
                        .cloned()
                        .map(RuntimeEffect::StreamUpdated)
                })
                .flatten()
                .into_iter()
                .collect()),
            ServerMessage::ToolStart(tool_start) => Ok(self
                .streaming
                .handle_tool_start(tool_start)
                .then(|| {
                    self.streaming
                        .active_session(tool_start.stream_id)
                        .cloned()
                        .map(RuntimeEffect::StreamUpdated)
                })
                .flatten()
                .into_iter()
                .collect()),
            ServerMessage::ToolEnd(tool_end) => Ok(self
                .streaming
                .handle_tool_end(tool_end)
                .then(|| {
                    self.streaming
                        .active_session(tool_end.stream_id)
                        .cloned()
                        .map(RuntimeEffect::StreamUpdated)
                })
                .flatten()
                .into_iter()
                .collect()),
            ServerMessage::StreamEnd(stream_end) => {
                let next_seq = self
                    .streaming
                    .active_session(stream_end.stream_id)
                    .map(|stream| self.sync_engine.last_seq(stream.conversation_id) + 1)
                    .unwrap_or(1);

                Ok(self
                    .streaming
                    .handle_stream_end(stream_end, next_seq, Utc::now())
                    .map(RuntimeEffect::StreamFinalized)
                    .into_iter()
                    .collect())
            }
            _ => Ok(Vec::new()),
        }
    }

    fn handle_message_received(
        &mut self,
        msg: &paw_proto::MessageReceivedMsg,
    ) -> Result<Vec<RuntimeEffect>, CoreRuntimeError> {
        let effects = match self.sync_engine.ingest_message(msg) {
            MessageSyncOutcome::DuplicateOrStale { ack_seq } => vec![
                RuntimeEffect::DuplicateMessage {
                    conversation_id: msg.conversation_id,
                    received_seq: msg.seq,
                    last_seq: ack_seq,
                },
                RuntimeEffect::AckRequested {
                    conversation_id: msg.conversation_id,
                    last_seq: ack_seq,
                },
            ],
            MessageSyncOutcome::GapDetected { request_from_seq } => {
                self.sync_engine
                    .mark_recovery_pending(msg.conversation_id, request_from_seq);
                vec![
                    RuntimeEffect::GapDetected {
                        conversation_id: msg.conversation_id,
                        expected_seq: request_from_seq + 1,
                        received_seq: msg.seq,
                        request_from_seq,
                    },
                    RuntimeEffect::SyncRequested(SyncRequest {
                        conversation_id: msg.conversation_id.to_string(),
                        last_seq: request_from_seq,
                    }),
                ]
            }
            MessageSyncOutcome::Applied { ack_seq } => {
                let record = self.sync_service.persist_message(msg)?;
                vec![
                    RuntimeEffect::MessagePersisted(record),
                    RuntimeEffect::AckRequested {
                        conversation_id: msg.conversation_id,
                        last_seq: ack_seq,
                    },
                ]
            }
        };

        Ok(effects)
    }

    fn handle_device_sync_response(
        &mut self,
        response: &DeviceSyncResponse,
    ) -> Result<Vec<RuntimeEffect>, CoreRuntimeError> {
        self.sync_engine
            .clear_recoveries(response.conversations.iter().map(|conversation| {
                ConversationSyncCursor {
                    conversation_id: conversation.conversation_id,
                    last_seq: conversation.last_seq,
                }
            }));
        self.sync_engine.apply_gap_fill(&response.messages);

        let mut effects = Vec::with_capacity(response.messages.len() * 2 + 1);
        let mut highest_seq_by_conversation: BTreeMap<Uuid, i64> = BTreeMap::new();
        let mut applied_count_by_conversation: BTreeMap<Uuid, u32> = BTreeMap::new();

        for message in &response.messages {
            let record = self.sync_service.persist_message(message)?;
            effects.push(RuntimeEffect::MessagePersisted(record));

            applied_count_by_conversation
                .entry(message.conversation_id)
                .and_modify(|count| *count += 1)
                .or_insert(1);
            highest_seq_by_conversation
                .entry(message.conversation_id)
                .and_modify(|seq| *seq = (*seq).max(message.seq))
                .or_insert(message.seq);
        }

        effects.push(RuntimeEffect::DeviceSyncBatchProcessed {
            message_count: saturating_u32(response.messages.len()),
            conversation_count: saturating_u32(response.conversations.len()),
            conversation_ids: response
                .conversations
                .iter()
                .map(|conversation| conversation.conversation_id)
                .collect(),
        });

        effects.extend(
            highest_seq_by_conversation
                .iter()
                .map(
                    |(conversation_id, highest_seq)| RuntimeEffect::DeviceSyncApplied {
                        conversation_id: *conversation_id,
                        applied_count: applied_count_by_conversation
                            .get(conversation_id)
                            .copied()
                            .unwrap_or_default(),
                        highest_seq: *highest_seq,
                    },
                ),
        );

        effects.extend(highest_seq_by_conversation.into_iter().map(
            |(conversation_id, last_seq)| RuntimeEffect::AckRequested {
                conversation_id,
                last_seq,
            },
        ));

        Ok(effects)
    }

    fn connection_transition_effects(&self) -> Vec<RuntimeEffect> {
        let mut effects = Vec::with_capacity(2);
        if let Some(plan) = self.ws_service.pending_reconnect() {
            effects.push(RuntimeEffect::ReconnectScheduled {
                delay_ms: saturating_u64(plan.delay.as_millis()),
                endpoint: crate::ws::public_endpoint_label(&plan.uri),
                attempt: saturating_u32(plan.attempt),
            });
        }
        effects.push(RuntimeEffect::ConnectionStateChanged(
            ConnectionSnapshot::from(&self.ws_service),
        ));
        effects
    }
}

fn load_cursors(db: &AppDatabase) -> Result<Vec<ConversationSyncCursor>, CoreRuntimeError> {
    db.get_all_conversations()?
        .into_iter()
        .map(|conversation| {
            let conversation_id = Uuid::parse_str(&conversation.id)
                .map_err(|_| CoreRuntimeError::InvalidConversationId(conversation.id.clone()))?;

            Ok(ConversationSyncCursor {
                conversation_id,
                last_seq: conversation.last_seq,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{Arc, Mutex},
        time::Duration,
    };

    use async_trait::async_trait;
    use chrono::{TimeZone, Utc};
    use paw_proto::{
        ContentDeltaMsg, ConvSyncState, DeviceSyncResponse, HelloOkMsg, MessageFormat,
        MessageReceivedMsg, StreamEndMsg, StreamStartMsg, ToolEndMsg, ToolStartMsg,
        PROTOCOL_VERSION,
    };
    use reqwest::Url;

    use crate::{
        auth::{InMemoryTokenStore, StoredTokens},
        db::ConversationRecord,
        events::ConnectionStateView,
        ws::{ReconnectionManager, WsConnectionState, WsTransport},
    };

    use super::*;

    #[derive(Default)]
    struct RecordingTransport {
        connections: Mutex<Vec<Url>>,
        sent: Mutex<Vec<paw_proto::ClientMessage>>,
        closes: Mutex<usize>,
    }

    #[async_trait]
    impl WsTransport for RecordingTransport {
        async fn connect(&self, uri: Url) -> Result<(), WsServiceError> {
            self.connections.lock().unwrap().push(uri);
            Ok(())
        }

        async fn send(&self, message: paw_proto::ClientMessage) -> Result<(), WsServiceError> {
            self.sent.lock().unwrap().push(message);
            Ok(())
        }

        async fn close(&self) -> Result<(), WsServiceError> {
            *self.closes.lock().unwrap() += 1;
            Ok(())
        }
    }

    #[derive(Clone)]
    struct StubAuthClient {
        calls: Arc<Mutex<Vec<&'static str>>>,
    }

    #[async_trait]
    impl AuthClient for StubAuthClient {
        async fn request_otp(&self, _phone: &str) -> Result<(), AuthBackendError> {
            unreachable!()
        }

        async fn verify_otp(
            &self,
            _phone: &str,
            _code: &str,
        ) -> Result<crate::auth::VerifyOtpResponse, AuthBackendError> {
            unreachable!()
        }

        async fn register_device(
            &self,
            _session_token: &str,
            _device_name: &str,
            _ed25519_public_key: &str,
        ) -> Result<crate::auth::RegisterDeviceResponse, AuthBackendError> {
            unreachable!()
        }

        async fn get_me(&self, access_token: &str) -> Result<AuthUserProfile, AuthBackendError> {
            self.calls.lock().unwrap().push("get_me");
            assert_eq!(access_token, "access-token");

            Ok(AuthUserProfile {
                username: "worker".into(),
                discoverable_by_phone: true,
            })
        }

        async fn update_me(
            &self,
            _access_token: &str,
            _username: &str,
            _discoverable_by_phone: bool,
        ) -> Result<AuthUserProfile, AuthBackendError> {
            unreachable!()
        }
    }

    fn runtime_with_db(
        db: Arc<AppDatabase>,
    ) -> (
        CoreRuntime,
        Arc<RecordingTransport>,
        Arc<Mutex<Vec<&'static str>>>,
    ) {
        let transport = Arc::new(RecordingTransport::default());
        let ws_service = WsService::new(
            "https://paw.example/api",
            transport.clone(),
            ReconnectionManager::new(3, vec![Duration::from_secs(1)]),
        );
        let calls = Arc::new(Mutex::new(Vec::new()));

        (CoreRuntime::new(db, ws_service).unwrap(), transport, calls)
    }

    #[tokio::test]
    async fn bootstrap_enforces_db_tokens_session_then_ws_order() {
        let db = Arc::new(AppDatabase::open_in_memory().unwrap());
        let (mut runtime, transport, calls) = runtime_with_db(db);
        let token_store = InMemoryTokenStore::new();
        token_store
            .write(StoredTokens::new("access-token", "refresh-token"))
            .await;
        let auth_client = StubAuthClient {
            calls: calls.clone(),
        };

        let report = runtime.bootstrap(&token_store, &auth_client).await.unwrap();

        assert_eq!(
            report.steps,
            vec![
                RuntimeInitStep::DatabaseOpened,
                RuntimeInitStep::TokensRestored,
                RuntimeInitStep::SessionValidated,
                RuntimeInitStep::WsConnected,
            ]
        );
        assert_eq!(*calls.lock().unwrap(), vec!["get_me"]);
        assert_eq!(
            report.connected_endpoint.as_deref(),
            Some("wss://paw.example/ws")
        );
        assert_eq!(
            runtime.ws_service().connection_state(),
            WsConnectionState::Connecting
        );
        assert!(matches!(
            transport.sent.lock().unwrap().first(),
            Some(paw_proto::ClientMessage::Connect(_))
        ));
    }

    #[tokio::test]
    async fn bootstrap_stops_when_no_tokens_are_available() {
        let db = Arc::new(AppDatabase::open_in_memory().unwrap());
        let (mut runtime, transport, calls) = runtime_with_db(db);
        let token_store = InMemoryTokenStore::new();
        let auth_client = StubAuthClient { calls };

        let report = runtime.bootstrap(&token_store, &auth_client).await.unwrap();

        assert_eq!(
            report.steps,
            vec![
                RuntimeInitStep::DatabaseOpened,
                RuntimeInitStep::BootstrapSkippedNoStoredTokens,
            ]
        );
        assert!(transport.connections.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn hello_ok_reloads_cursors_and_requests_sync() {
        let db = Arc::new(AppDatabase::open_in_memory().unwrap());
        let conversation_id = Uuid::new_v4();
        db.upsert_conversation(&ConversationRecord {
            id: conversation_id.to_string(),
            name: "hello".into(),
            avatar_url: None,
            last_seq: 4,
            unread_count: 0,
            updated_at: Utc.timestamp_opt(200, 0).single().unwrap(),
        })
        .unwrap();
        let (mut runtime, _, _) = runtime_with_db(db);

        let effects = runtime
            .handle_server_message(&ServerMessage::HelloOk(HelloOkMsg {
                v: PROTOCOL_VERSION,
                user_id: Uuid::new_v4(),
                server_time: Utc::now(),
            }))
            .await
            .unwrap();

        assert_eq!(
            effects,
            vec![
                RuntimeEffect::ConnectionStateChanged(ConnectionSnapshot {
                    state: ConnectionStateView::Connected,
                    attempts: 0,
                    pending_reconnect_delay_ms: None,
                    pending_reconnect_endpoint: None,
                }),
                RuntimeEffect::SyncRequested(SyncRequest {
                    conversation_id: conversation_id.to_string(),
                    last_seq: 0,
                }),
            ]
        );
        assert_eq!(
            runtime.ws_service().connection_state(),
            WsConnectionState::Connected
        );
    }

    #[tokio::test]
    async fn message_ingest_emits_ack_gap_and_persist_effects() {
        let db = Arc::new(AppDatabase::open_in_memory().unwrap());
        let conversation_id = Uuid::new_v4();
        db.upsert_conversation(&ConversationRecord {
            id: conversation_id.to_string(),
            name: "hello".into(),
            avatar_url: None,
            last_seq: 2,
            unread_count: 0,
            updated_at: Utc::now(),
        })
        .unwrap();
        let (mut runtime, _, _) = runtime_with_db(db.clone());

        let stale = MessageReceivedMsg {
            v: PROTOCOL_VERSION,
            id: Uuid::new_v4(),
            conversation_id,
            sender_id: Uuid::new_v4(),
            content: "stale".into(),
            format: MessageFormat::Markdown,
            seq: 2,
            created_at: Utc::now(),
            blocks: vec![],
        };
        let gap = MessageReceivedMsg {
            seq: 4,
            ..stale.clone()
        };
        let applied = MessageReceivedMsg {
            id: Uuid::new_v4(),
            content: "fresh".into(),
            seq: 3,
            ..stale.clone()
        };

        assert_eq!(
            runtime
                .handle_server_message(&ServerMessage::MessageReceived(stale))
                .await
                .unwrap(),
            vec![
                RuntimeEffect::DuplicateMessage {
                    conversation_id,
                    received_seq: 2,
                    last_seq: 2,
                },
                RuntimeEffect::AckRequested {
                    conversation_id,
                    last_seq: 2,
                },
            ]
        );
        assert_eq!(
            runtime
                .handle_server_message(&ServerMessage::MessageReceived(gap))
                .await
                .unwrap(),
            vec![
                RuntimeEffect::GapDetected {
                    conversation_id,
                    expected_seq: 3,
                    received_seq: 4,
                    request_from_seq: 2,
                },
                RuntimeEffect::SyncRequested(SyncRequest {
                    conversation_id: conversation_id.to_string(),
                    last_seq: 2,
                }),
            ]
        );
        assert_eq!(
            runtime.snapshot().pending_recoveries,
            vec![RecoveryCursorView {
                conversation_id: conversation_id.to_string(),
                request_from_seq: 2,
            }]
        );

        let effects = runtime
            .handle_server_message(&ServerMessage::MessageReceived(applied.clone()))
            .await
            .unwrap();

        assert!(matches!(
            &effects[..],
            [
                RuntimeEffect::MessagePersisted(MessageRecord { content, seq, .. }),
                RuntimeEffect::AckRequested { conversation_id: ack_id, last_seq }
            ] if content == "fresh" && *seq == 3 && *ack_id == conversation_id && *last_seq == 3
        ));
        assert_eq!(db.get_last_seq(&conversation_id.to_string()).unwrap(), 3);
    }

    #[tokio::test]
    async fn device_sync_gap_fill_persists_messages_and_acks_highest_seq() {
        let db = Arc::new(AppDatabase::open_in_memory().unwrap());
        let conversation_id = Uuid::new_v4();
        db.upsert_conversation(&ConversationRecord {
            id: conversation_id.to_string(),
            name: "hello".into(),
            avatar_url: None,
            last_seq: 0,
            unread_count: 0,
            updated_at: Utc::now(),
        })
        .unwrap();
        let (mut runtime, _, _) = runtime_with_db(db.clone());
        let message = |seq, content: &str| MessageReceivedMsg {
            v: PROTOCOL_VERSION,
            id: Uuid::new_v4(),
            conversation_id,
            sender_id: Uuid::new_v4(),
            content: content.into(),
            format: MessageFormat::Markdown,
            seq,
            created_at: Utc::now(),
            blocks: vec![],
        };

        let effects = runtime
            .handle_server_message(&ServerMessage::DeviceSyncResponse(DeviceSyncResponse {
                v: PROTOCOL_VERSION,
                conversations: vec![ConvSyncState {
                    conversation_id,
                    last_seq: 0,
                }],
                messages: vec![message(1, "one"), message(3, "three"), message(2, "two")],
            }))
            .await
            .unwrap();

        assert_eq!(db.get_last_seq(&conversation_id.to_string()).unwrap(), 3);
        assert!(runtime.snapshot().pending_recoveries.is_empty());
        assert!(effects.iter().any(|effect| {
            matches!(
                effect,
                RuntimeEffect::DeviceSyncBatchProcessed {
                    message_count: 3,
                    conversation_count: 1,
                    conversation_ids,
                }
                    if conversation_ids == &vec![conversation_id]
            )
        }));
        assert_eq!(
            effects.last(),
            Some(&RuntimeEffect::AckRequested {
                conversation_id,
                last_seq: 3,
            })
        );
        assert!(effects.iter().any(|effect| {
            matches!(
                effect,
                RuntimeEffect::DeviceSyncApplied {
                    conversation_id: effect_conversation_id,
                    applied_count: 3,
                    highest_seq: 3,
                } if *effect_conversation_id == conversation_id
            )
        }));
    }

    #[tokio::test]
    async fn empty_device_sync_response_still_surfaces_batch_completion() {
        let db = Arc::new(AppDatabase::open_in_memory().unwrap());
        let (mut runtime, _, _) = runtime_with_db(db);
        let conversation_id = Uuid::new_v4();
        runtime
            .sync_engine
            .mark_recovery_pending(conversation_id, 4);

        let effects = runtime
            .handle_server_message(&ServerMessage::DeviceSyncResponse(DeviceSyncResponse {
                v: PROTOCOL_VERSION,
                conversations: vec![ConvSyncState {
                    conversation_id,
                    last_seq: 4,
                }],
                messages: vec![],
            }))
            .await
            .unwrap();

        assert_eq!(
            effects,
            vec![RuntimeEffect::DeviceSyncBatchProcessed {
                message_count: 0,
                conversation_count: 1,
                conversation_ids: vec![conversation_id],
            }]
        );
        assert!(runtime.snapshot().pending_recoveries.is_empty());
    }

    #[tokio::test]
    async fn streaming_effects_track_updates_and_finalization() {
        let db = Arc::new(AppDatabase::open_in_memory().unwrap());
        let conversation_id = Uuid::new_v4();
        db.upsert_conversation(&ConversationRecord {
            id: conversation_id.to_string(),
            name: "hello".into(),
            avatar_url: None,
            last_seq: 7,
            unread_count: 0,
            updated_at: Utc::now(),
        })
        .unwrap();
        let (mut runtime, _, _) = runtime_with_db(db);
        let stream_id = Uuid::new_v4();
        let agent_id = Uuid::new_v4();

        let start = runtime
            .handle_server_message(&ServerMessage::StreamStart(StreamStartMsg {
                v: PROTOCOL_VERSION,
                conversation_id,
                agent_id,
                stream_id,
            }))
            .await
            .unwrap();
        assert!(matches!(
            &start[..],
            [RuntimeEffect::StreamUpdated(StreamingSession { content, .. })] if content.is_empty()
        ));

        runtime
            .handle_server_message(&ServerMessage::ContentDelta(ContentDeltaMsg {
                v: PROTOCOL_VERSION,
                stream_id,
                delta: "Hello".into(),
            }))
            .await
            .unwrap();
        runtime
            .handle_server_message(&ServerMessage::ToolStart(ToolStartMsg {
                v: PROTOCOL_VERSION,
                stream_id,
                tool: "search".into(),
                label: "Searching".into(),
            }))
            .await
            .unwrap();
        runtime
            .handle_server_message(&ServerMessage::ToolEnd(ToolEndMsg {
                v: PROTOCOL_VERSION,
                stream_id,
                tool: "search".into(),
            }))
            .await
            .unwrap();
        runtime
            .handle_server_message(&ServerMessage::ContentDelta(ContentDeltaMsg {
                v: PROTOCOL_VERSION,
                stream_id,
                delta: " world".into(),
            }))
            .await
            .unwrap();

        let end = runtime
            .handle_server_message(&ServerMessage::StreamEnd(StreamEndMsg {
                v: PROTOCOL_VERSION,
                stream_id,
                tokens: 12,
                duration_ms: 250,
            }))
            .await
            .unwrap();

        assert!(matches!(
            &end[..],
            [RuntimeEffect::StreamFinalized(FinalizedStreamMessage {
                conversation_id: finalized_conversation_id,
                sender_id,
                content,
                seq,
                tool_calls,
                ..
            })]
            if *finalized_conversation_id == conversation_id
                && *sender_id == agent_id
                && content == "Hello world"
                && *seq == 8
                && tool_calls.len() == 1
        ));
    }

    #[tokio::test]
    async fn repeated_transport_errors_transition_runtime_to_exhausted() {
        let db = Arc::new(AppDatabase::open_in_memory().unwrap());
        let transport = Arc::new(RecordingTransport::default());
        let ws_service = WsService::new(
            "https://paw.example/api",
            transport,
            ReconnectionManager::new(1, vec![Duration::from_secs(1)]),
        );
        let mut runtime = CoreRuntime::new(db, ws_service).unwrap();

        runtime
            .ws_service
            .connect("https://paw.example/api", "access-token")
            .await
            .unwrap();

        let first = runtime.on_transport_error();
        assert!(matches!(
            &first[..],
            [
                RuntimeEffect::ReconnectScheduled {
                    delay_ms: 1_000,
                    attempt: 1,
                    ..
                },
                RuntimeEffect::ConnectionStateChanged(ConnectionSnapshot {
                    state: ConnectionStateView::Retrying,
                    ..
                })
            ]
        ));

        let second = runtime.on_transport_error();
        assert!(matches!(
            &second[..],
            [RuntimeEffect::ConnectionStateChanged(ConnectionSnapshot {
                state: ConnectionStateView::Exhausted,
                pending_reconnect_delay_ms: None,
                pending_reconnect_endpoint: None,
                ..
            })]
        ));
    }

    #[tokio::test]
    async fn session_invalidation_clears_tokens_and_disconnects_runtime() {
        let db = Arc::new(AppDatabase::open_in_memory().unwrap());
        let (mut runtime, transport, calls) = runtime_with_db(db);
        let token_store = InMemoryTokenStore::new();
        token_store
            .write(StoredTokens::new("access-token", "refresh-token"))
            .await;
        let auth_client = StubAuthClient {
            calls: calls.clone(),
        };

        runtime.bootstrap(&token_store, &auth_client).await.unwrap();

        let effects = runtime
            .handle_session_event(
                &token_store,
                SessionEvent {
                    reason: crate::auth::SessionExpiryReason::Unauthorized,
                },
            )
            .await
            .unwrap();

        assert_eq!(token_store.snapshot().await, None);
        assert_eq!(
            effects,
            vec![
                RuntimeEffect::SessionInvalidated(SessionEvent {
                    reason: crate::auth::SessionExpiryReason::Unauthorized,
                }),
                RuntimeEffect::ConnectionStateChanged(ConnectionSnapshot {
                    state: ConnectionStateView::Disconnected,
                    attempts: 0,
                    pending_reconnect_delay_ms: None,
                    pending_reconnect_endpoint: None,
                }),
            ]
        );
        assert_eq!(*transport.closes.lock().unwrap(), 2);
        assert_eq!(
            runtime.ws_service().connection_state(),
            WsConnectionState::Disconnected
        );
    }

    #[tokio::test]
    async fn session_invalidation_discards_active_streams_before_notifying_ui() {
        let db = Arc::new(AppDatabase::open_in_memory().unwrap());
        let (mut runtime, _transport, _calls) = runtime_with_db(db);
        let token_store = InMemoryTokenStore::new();
        let conversation_id = Uuid::new_v4();
        let agent_id = Uuid::new_v4();
        let stream_id = Uuid::new_v4();

        runtime
            .handle_server_message(&ServerMessage::StreamStart(paw_proto::StreamStartMsg {
                v: paw_proto::PROTOCOL_VERSION,
                conversation_id,
                agent_id,
                stream_id,
            }))
            .await
            .unwrap();

        assert_eq!(runtime.snapshot().active_streams.len(), 1);

        let effects = runtime
            .handle_session_event(
                &token_store,
                SessionEvent {
                    reason: crate::auth::SessionExpiryReason::Unauthorized,
                },
            )
            .await
            .unwrap();

        assert_eq!(
            effects.first(),
            Some(&RuntimeEffect::ActiveStreamsCleared { count: 1 })
        );
        assert!(runtime.snapshot().active_streams.is_empty());
    }

    #[tokio::test]
    async fn session_invalidation_prevents_reconnect_with_stale_in_memory_token() {
        let db = Arc::new(AppDatabase::open_in_memory().unwrap());
        let (mut runtime, _transport, calls) = runtime_with_db(db);
        let token_store = InMemoryTokenStore::new();
        token_store
            .write(StoredTokens::new("access-token", "refresh-token"))
            .await;
        let auth_client = StubAuthClient {
            calls: calls.clone(),
        };

        runtime.bootstrap(&token_store, &auth_client).await.unwrap();
        runtime
            .handle_session_event(
                &token_store,
                SessionEvent {
                    reason: crate::auth::SessionExpiryReason::Unauthorized,
                },
            )
            .await
            .unwrap();

        let reconnect = runtime.reconnect_with_stored_token().await.unwrap();
        assert_eq!(reconnect, None);
        assert_eq!(
            runtime.ws_service().connection_state(),
            WsConnectionState::Disconnected
        );
    }

    #[tokio::test]
    async fn reconnect_with_stored_token_surfaces_attempt_metadata() {
        let db = Arc::new(AppDatabase::open_in_memory().unwrap());
        let (mut runtime, _transport, calls) = runtime_with_db(db);
        let token_store = InMemoryTokenStore::new();
        token_store
            .write(StoredTokens::new("access-token", "refresh-token"))
            .await;
        let auth_client = StubAuthClient {
            calls: calls.clone(),
        };

        runtime.bootstrap(&token_store, &auth_client).await.unwrap();
        let effects = runtime
            .reconnect_with_stored_token()
            .await
            .unwrap()
            .expect("stored token reconnect effects");

        assert!(matches!(
            &effects[..],
            [
                RuntimeEffect::ReconnectAttemptStarted { attempt: 0, .. },
                RuntimeEffect::ConnectionStateChanged(ConnectionSnapshot {
                    state: ConnectionStateView::Connecting,
                    ..
                })
            ]
        ));
    }

    #[test]
    fn runtime_effect_domain_groups_recovery_events_by_semantics() {
        assert_eq!(
            RuntimeEffect::ReconnectScheduled {
                delay_ms: 1_000,
                endpoint: "wss://paw.example/ws".into(),
                attempt: 1,
            }
            .domain(),
            RuntimeEffectDomain::Connection
        );
        assert_eq!(
            RuntimeEffect::DeviceSyncBatchProcessed {
                message_count: 0,
                conversation_count: 0,
                conversation_ids: vec![],
            }
            .domain(),
            RuntimeEffectDomain::Sync
        );
        assert_eq!(
            RuntimeEffect::SessionInvalidated(SessionEvent {
                reason: crate::auth::SessionExpiryReason::Unauthorized,
            })
            .domain(),
            RuntimeEffectDomain::Lifecycle
        );
    }
}
