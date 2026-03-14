use std::{collections::BTreeMap, sync::Arc};

use chrono::Utc;
use paw_proto::{DeviceSyncResponse, ServerMessage};
use reqwest::Url;
use uuid::Uuid;

use crate::{
    auth::{AuthBackendError, AuthClient, AuthUserProfile, StoredTokens, TokenStore},
    db::{AppDatabase, DbError, MessageRecord},
    events::{ConnectionSnapshot, ConversationCursorView, RuntimeSnapshot, StreamingSessionView},
    sync::{
        ConversationSyncCursor, FinalizedStreamMessage, MessageSyncOutcome, StreamingSession,
        StreamingState, SyncEngine, SyncRequest, SyncService,
    },
    ws::{WsService, WsServiceError},
};

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
    SessionValidated,
    WsConnected,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeBootstrapReport {
    pub steps: Vec<RuntimeInitStep>,
    pub tokens: Option<StoredTokens>,
    pub profile: Option<AuthUserProfile>,
    pub connected_uri: Option<Url>,
}

impl RuntimeBootstrapReport {
    fn db_only() -> Self {
        Self {
            steps: vec![RuntimeInitStep::DatabaseOpened],
            tokens: None,
            profile: None,
            connected_uri: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeEffect {
    SyncRequested(SyncRequest),
    AckRequested {
        conversation_id: Uuid,
        last_seq: i64,
    },
    MessagePersisted(MessageRecord),
    StreamUpdated(StreamingSession),
    StreamFinalized(FinalizedStreamMessage),
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
        report.connected_uri = Some(uri);

        Ok(report)
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
                Ok(self
                    .sync_service
                    .sync_all_conversations()?
                    .into_iter()
                    .map(RuntimeEffect::SyncRequested)
                    .collect())
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
        let effect = match self.sync_engine.ingest_message(msg) {
            MessageSyncOutcome::DuplicateOrStale { ack_seq } => RuntimeEffect::AckRequested {
                conversation_id: msg.conversation_id,
                last_seq: ack_seq,
            },
            MessageSyncOutcome::GapDetected { request_from_seq } => {
                RuntimeEffect::SyncRequested(SyncRequest {
                    conversation_id: msg.conversation_id.to_string(),
                    last_seq: request_from_seq,
                })
            }
            MessageSyncOutcome::Applied { ack_seq } => {
                let record = self.sync_service.persist_message(msg)?;
                return Ok(vec![
                    RuntimeEffect::MessagePersisted(record),
                    RuntimeEffect::AckRequested {
                        conversation_id: msg.conversation_id,
                        last_seq: ack_seq,
                    },
                ]);
            }
        };

        Ok(vec![effect])
    }

    fn handle_device_sync_response(
        &mut self,
        response: &DeviceSyncResponse,
    ) -> Result<Vec<RuntimeEffect>, CoreRuntimeError> {
        self.sync_engine.apply_gap_fill(&response.messages);

        let mut effects = Vec::with_capacity(response.messages.len() * 2);
        let mut highest_seq_by_conversation: BTreeMap<Uuid, i64> = BTreeMap::new();

        for message in &response.messages {
            let record = self.sync_service.persist_message(message)?;
            effects.push(RuntimeEffect::MessagePersisted(record));

            highest_seq_by_conversation
                .entry(message.conversation_id)
                .and_modify(|seq| *seq = (*seq).max(message.seq))
                .or_insert(message.seq);
        }

        effects.extend(highest_seq_by_conversation.into_iter().map(
            |(conversation_id, last_seq)| RuntimeEffect::AckRequested {
                conversation_id,
                last_seq,
            },
        ));

        Ok(effects)
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
        ContentDeltaMsg, DeviceSyncResponse, HelloOkMsg, MessageFormat, MessageReceivedMsg,
        StreamEndMsg, StreamStartMsg, ToolEndMsg, ToolStartMsg, PROTOCOL_VERSION,
    };

    use crate::{
        auth::{InMemoryTokenStore, StoredTokens},
        db::ConversationRecord,
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
            report.connected_uri.unwrap().as_str(),
            "wss://paw.example/ws?token=access-token"
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

        assert_eq!(report.steps, vec![RuntimeInitStep::DatabaseOpened]);
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
            vec![RuntimeEffect::SyncRequested(SyncRequest {
                conversation_id: conversation_id.to_string(),
                last_seq: 0,
            })]
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
            vec![RuntimeEffect::AckRequested {
                conversation_id,
                last_seq: 2,
            }]
        );
        assert_eq!(
            runtime
                .handle_server_message(&ServerMessage::MessageReceived(gap))
                .await
                .unwrap(),
            vec![RuntimeEffect::SyncRequested(SyncRequest {
                conversation_id: conversation_id.to_string(),
                last_seq: 2,
            })]
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
                messages: vec![message(1, "one"), message(3, "three"), message(2, "two")],
            }))
            .await
            .unwrap();

        assert_eq!(db.get_last_seq(&conversation_id.to_string()).unwrap(), 3);
        assert_eq!(
            effects.last(),
            Some(&RuntimeEffect::AckRequested {
                conversation_id,
                last_seq: 3,
            })
        );
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
}
