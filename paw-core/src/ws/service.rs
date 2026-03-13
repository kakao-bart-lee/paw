use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use paw_proto::{
    ClientMessage, ConnectMsg, HelloErrorMsg, HelloOkMsg, MessageAckMsg, ServerMessage, SyncMsg,
    TypingMsg, PROTOCOL_VERSION,
};
use reqwest::Url;

use super::ReconnectionManager;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WsConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Retrying,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReconnectPlan {
    pub delay: Duration,
    pub uri: Url,
    pub attempt: usize,
}

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum WsServiceError {
    #[error("invalid websocket url: {0}")]
    InvalidUrl(String),
    #[error("transport error: {0}")]
    Transport(String),
}

#[async_trait]
pub trait WsTransport: Send + Sync {
    async fn connect(&self, uri: Url) -> Result<(), WsServiceError>;
    async fn send(&self, message: ClientMessage) -> Result<(), WsServiceError>;
    async fn close(&self) -> Result<(), WsServiceError>;
}

pub struct WsService {
    server_url: String,
    transport: Arc<dyn WsTransport>,
    reconnection_manager: ReconnectionManager,
    connected: bool,
    manual_disconnect: bool,
    access_token: Option<String>,
    connection_state: WsConnectionState,
    pending_reconnect: Option<ReconnectPlan>,
    sync_all: Option<Box<dyn Fn() -> Result<(), WsServiceError> + Send + Sync>>,
}

impl WsService {
    pub fn new(
        server_url: impl Into<String>,
        transport: Arc<dyn WsTransport>,
        reconnection_manager: ReconnectionManager,
    ) -> Self {
        Self {
            server_url: server_url.into(),
            transport,
            reconnection_manager,
            connected: false,
            manual_disconnect: false,
            access_token: None,
            connection_state: WsConnectionState::Disconnected,
            pending_reconnect: None,
            sync_all: None,
        }
    }

    pub fn set_sync_all(
        &mut self,
        hook: impl Fn() -> Result<(), WsServiceError> + Send + Sync + 'static,
    ) {
        self.sync_all = Some(Box::new(hook));
    }

    pub fn is_connected(&self) -> bool {
        self.connected
    }

    pub fn connection_state(&self) -> WsConnectionState {
        self.connection_state
    }

    pub fn attempts(&self) -> usize {
        self.reconnection_manager.attempts()
    }

    pub fn pending_reconnect(&self) -> Option<&ReconnectPlan> {
        self.pending_reconnect.as_ref()
    }

    pub async fn connect(
        &mut self,
        server_url: impl Into<String>,
        access_token: impl Into<String>,
    ) -> Result<Url, WsServiceError> {
        let access_token = access_token.into();
        self.server_url = server_url.into();
        self.connection_state = WsConnectionState::Connecting;
        self.manual_disconnect = false;
        self.connected = false;
        self.pending_reconnect = None;
        self.access_token = Some(access_token.clone());

        self.transport.close().await?;

        let uri = self.build_ws_uri(&self.server_url, &access_token)?;
        self.transport.connect(uri.clone()).await?;
        self.transport
            .send(ClientMessage::Connect(ConnectMsg {
                v: PROTOCOL_VERSION,
                token: access_token,
            }))
            .await?;
        Ok(uri)
    }

    pub async fn connect_with_stored_token(&mut self) -> Result<Option<Url>, WsServiceError> {
        let Some(access_token) = self.access_token.clone() else {
            return Ok(None);
        };
        let server_url = self.server_url.clone();
        self.connect(server_url, access_token).await.map(Some)
    }

    pub async fn connect_with_access_token(
        &mut self,
        access_token: impl Into<String>,
    ) -> Result<Url, WsServiceError> {
        let server_url = self.server_url.clone();
        self.connect(server_url, access_token).await
    }

    pub async fn handle_server_message(
        &mut self,
        msg: &ServerMessage,
    ) -> Result<(), WsServiceError> {
        match msg {
            ServerMessage::HelloOk(hello_ok) => self.on_hello_ok(hello_ok),
            ServerMessage::HelloError(hello_error) => self.on_hello_error(hello_error),
            _ => Ok(()),
        }
    }

    pub fn on_transport_error(&mut self) {
        self.connected = false;
        self.connection_state = WsConnectionState::Disconnected;
        self.schedule_reconnect();
    }

    pub fn on_transport_closed(&mut self) {
        self.connected = false;
        self.connection_state = WsConnectionState::Disconnected;
        self.schedule_reconnect();
    }

    pub async fn disconnect(&mut self) -> Result<(), WsServiceError> {
        self.manual_disconnect = true;
        self.connected = false;
        self.pending_reconnect = None;
        self.reconnection_manager.reset();
        self.transport.close().await?;
        self.connection_state = WsConnectionState::Disconnected;
        Ok(())
    }

    pub async fn send_typing_start(
        &self,
        conversation_id: uuid::Uuid,
    ) -> Result<bool, WsServiceError> {
        self.send_if_connected(ClientMessage::TypingStart(TypingMsg {
            v: PROTOCOL_VERSION,
            conversation_id,
            user_id: None,
        }))
        .await
    }

    pub async fn send_typing_stop(
        &self,
        conversation_id: uuid::Uuid,
    ) -> Result<bool, WsServiceError> {
        self.send_if_connected(ClientMessage::TypingStop(TypingMsg {
            v: PROTOCOL_VERSION,
            conversation_id,
            user_id: None,
        }))
        .await
    }

    pub async fn send_ack(
        &self,
        conversation_id: uuid::Uuid,
        last_seq: i64,
    ) -> Result<bool, WsServiceError> {
        self.send_if_connected(ClientMessage::MessageAck(MessageAckMsg {
            v: PROTOCOL_VERSION,
            conversation_id,
            last_seq,
        }))
        .await
    }

    pub async fn request_sync(
        &self,
        conversation_id: uuid::Uuid,
        last_seq: i64,
    ) -> Result<bool, WsServiceError> {
        self.send_if_connected(ClientMessage::Sync(SyncMsg {
            v: PROTOCOL_VERSION,
            conversation_id,
            last_seq,
        }))
        .await
    }

    fn build_ws_uri(&self, raw_server_url: &str, token: &str) -> Result<Url, WsServiceError> {
        let mut url = Url::parse(raw_server_url)
            .map_err(|error| WsServiceError::InvalidUrl(error.to_string()))?;

        let scheme = match url.scheme() {
            "https" | "wss" => "wss",
            "http" | "ws" => "ws",
            other => {
                return Err(WsServiceError::InvalidUrl(format!(
                    "unsupported base scheme: {other}"
                )))
            }
        };

        url.set_scheme(scheme)
            .map_err(|_| WsServiceError::InvalidUrl("failed to set websocket scheme".into()))?;
        url.set_path("/ws");
        url.set_query(Some(&format!("token={token}")));

        Ok(url)
    }

    fn on_hello_ok(&mut self, _msg: &HelloOkMsg) -> Result<(), WsServiceError> {
        self.connected = true;
        self.connection_state = WsConnectionState::Connected;
        self.pending_reconnect = None;
        self.reconnection_manager.on_connected();

        if let Some(sync_all) = &self.sync_all {
            sync_all()?;
        }

        Ok(())
    }

    fn on_hello_error(&mut self, _msg: &HelloErrorMsg) -> Result<(), WsServiceError> {
        self.connected = false;
        self.connection_state = WsConnectionState::Disconnected;
        self.schedule_reconnect();
        Ok(())
    }

    fn schedule_reconnect(&mut self) {
        if self.manual_disconnect {
            self.pending_reconnect = None;
            self.connection_state = WsConnectionState::Disconnected;
            return;
        }

        let Some(token) = self.access_token.as_deref() else {
            self.pending_reconnect = None;
            self.connection_state = WsConnectionState::Disconnected;
            return;
        };

        let Some(delay) = self.reconnection_manager.next_delay() else {
            self.pending_reconnect = None;
            self.connection_state = WsConnectionState::Disconnected;
            return;
        };

        match self.build_ws_uri(&self.server_url, token) {
            Ok(uri) => {
                self.connection_state = WsConnectionState::Retrying;
                self.pending_reconnect = Some(ReconnectPlan {
                    delay,
                    uri,
                    attempt: self.reconnection_manager.attempts(),
                });
            }
            Err(_) => {
                self.pending_reconnect = None;
                self.connection_state = WsConnectionState::Disconnected;
            }
        }
    }

    async fn send_if_connected(&self, message: ClientMessage) -> Result<bool, WsServiceError> {
        if !self.connected {
            return Ok(false);
        }

        self.transport.send(message).await?;
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use chrono::Utc;
    use paw_proto::{HelloOkMsg, PROTOCOL_VERSION};
    use uuid::Uuid;

    use super::*;

    #[derive(Default)]
    struct RecordingTransport {
        connections: Mutex<Vec<Url>>,
        sent: Mutex<Vec<ClientMessage>>,
        closes: Mutex<usize>,
    }

    #[async_trait]
    impl WsTransport for RecordingTransport {
        async fn connect(&self, uri: Url) -> Result<(), WsServiceError> {
            self.connections.lock().unwrap().push(uri);
            Ok(())
        }

        async fn send(&self, message: ClientMessage) -> Result<(), WsServiceError> {
            self.sent.lock().unwrap().push(message);
            Ok(())
        }

        async fn close(&self) -> Result<(), WsServiceError> {
            *self.closes.lock().unwrap() += 1;
            Ok(())
        }
    }

    #[tokio::test]
    async fn connect_builds_ws_uri_and_sends_connect_frame() {
        let transport = Arc::new(RecordingTransport::default());
        let mut service = WsService::new(
            "https://paw.example/api",
            transport.clone(),
            ReconnectionManager::default(),
        );

        let uri = service
            .connect("https://paw.example/api", "token-123")
            .await
            .unwrap();

        assert_eq!(service.connection_state(), WsConnectionState::Connecting);
        assert_eq!(uri.as_str(), "wss://paw.example/ws?token=token-123");
        assert_eq!(
            transport.connections.lock().unwrap()[0].as_str(),
            "wss://paw.example/ws?token=token-123"
        );
        assert!(matches!(
            transport.sent.lock().unwrap()[0],
            ClientMessage::Connect(_)
        ));
    }

    #[tokio::test]
    async fn hello_ok_marks_connected_and_runs_sync_hook() {
        let transport = Arc::new(RecordingTransport::default());
        let sync_calls = Arc::new(Mutex::new(0usize));
        let sync_calls_clone = sync_calls.clone();
        let mut service = WsService::new(
            "https://paw.example",
            transport,
            ReconnectionManager::default(),
        );
        service.set_sync_all(move || {
            *sync_calls_clone.lock().unwrap() += 1;
            Ok(())
        });

        service
            .connect("https://paw.example", "token-123")
            .await
            .unwrap();
        service
            .handle_server_message(&ServerMessage::HelloOk(HelloOkMsg {
                v: PROTOCOL_VERSION,
                user_id: Uuid::new_v4(),
                server_time: Utc::now(),
            }))
            .await
            .unwrap();

        assert!(service.is_connected());
        assert_eq!(service.connection_state(), WsConnectionState::Connected);
        assert_eq!(*sync_calls.lock().unwrap(), 1);
    }

    #[tokio::test]
    async fn transport_failures_schedule_backoff() {
        let transport = Arc::new(RecordingTransport::default());
        let mut service = WsService::new(
            "http://localhost:38173",
            transport,
            ReconnectionManager::new(3, vec![Duration::from_secs(1), Duration::from_secs(5)]),
        );
        service
            .connect("http://localhost:38173", "token-123")
            .await
            .unwrap();

        service.on_transport_closed();
        let plan = service.pending_reconnect().unwrap();

        assert_eq!(service.connection_state(), WsConnectionState::Retrying);
        assert_eq!(plan.delay, Duration::from_secs(1));
        assert_eq!(plan.attempt, 1);
        assert_eq!(plan.uri.as_str(), "ws://localhost:38173/ws?token=token-123");
    }
}
