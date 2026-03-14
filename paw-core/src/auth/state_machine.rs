use std::sync::Arc;

use async_trait::async_trait;
use thiserror::Error;

use super::{
    run_session_reset,
    token_store::{StoredTokens, TokenStore},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AuthStep {
    AuthMethodSelect,
    PhoneInput,
    OtpVerify,
    DeviceName,
    UsernameSetup,
    Authenticated,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthState {
    pub step: AuthStep,
    pub phone: String,
    pub device_name: String,
    pub username: String,
    pub discoverable_by_phone: bool,
    pub session_token: Option<String>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub is_loading: bool,
    pub error: Option<String>,
}

impl AuthState {
    pub fn initial() -> Self {
        Self {
            step: AuthStep::AuthMethodSelect,
            phone: String::new(),
            device_name: String::new(),
            username: String::new(),
            discoverable_by_phone: false,
            session_token: None,
            access_token: None,
            refresh_token: None,
            is_loading: false,
            error: None,
        }
    }
}

impl Default for AuthState {
    fn default() -> Self {
        Self::initial()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerifyOtpResponse {
    pub session_token: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegisterDeviceResponse {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthUserProfile {
    pub username: String,
    pub discoverable_by_phone: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SessionExpiryReason {
    Unauthorized,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionEvent {
    pub reason: SessionExpiryReason,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
#[error("{message}")]
pub struct AuthBackendError {
    message: String,
}

impl AuthBackendError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AuthStateMachineError {
    #[error("{0}")]
    Backend(#[from] AuthBackendError),
    #[error("Missing session token for device registration")]
    MissingSessionToken,
    #[error("Authentication required")]
    AuthenticationRequired,
}

#[async_trait]
pub trait AuthClient: Send + Sync {
    async fn request_otp(&self, phone: &str) -> Result<(), AuthBackendError>;
    async fn verify_otp(
        &self,
        phone: &str,
        code: &str,
    ) -> Result<VerifyOtpResponse, AuthBackendError>;
    async fn register_device(
        &self,
        session_token: &str,
        device_name: &str,
        ed25519_public_key: &str,
    ) -> Result<RegisterDeviceResponse, AuthBackendError>;
    async fn get_me(&self, access_token: &str) -> Result<AuthUserProfile, AuthBackendError>;
    async fn update_me(
        &self,
        access_token: &str,
        username: &str,
        discoverable_by_phone: bool,
    ) -> Result<AuthUserProfile, AuthBackendError>;
}

#[async_trait]
pub trait SessionTransport: Send + Sync {
    async fn connect(&self, access_token: &str) -> Result<(), AuthBackendError>;
    async fn disconnect(&self);
}

#[derive(Default)]
pub struct NoopSessionTransport;

#[async_trait]
impl SessionTransport for NoopSessionTransport {
    async fn connect(&self, _access_token: &str) -> Result<(), AuthBackendError> {
        Ok(())
    }

    async fn disconnect(&self) {}
}

pub struct AuthStateMachine {
    client: Arc<dyn AuthClient>,
    token_store: Arc<dyn TokenStore>,
    transport: Arc<dyn SessionTransport>,
    state: AuthState,
}

impl AuthStateMachine {
    pub fn new(
        client: Arc<dyn AuthClient>,
        token_store: Arc<dyn TokenStore>,
        transport: Arc<dyn SessionTransport>,
    ) -> Self {
        Self {
            client,
            token_store,
            transport,
            state: AuthState::initial(),
        }
    }

    pub fn state(&self) -> &AuthState {
        &self.state
    }

    pub fn is_authenticated(&self) -> bool {
        self.state.step == AuthStep::Authenticated
    }

    pub fn show_phone_otp(&mut self) {
        self.state.step = AuthStep::PhoneInput;
        self.state.error = None;
    }

    pub fn back_to_auth_method_select(&mut self) {
        self.state.step = AuthStep::AuthMethodSelect;
        self.state.phone.clear();
        self.state.error = None;
    }

    pub async fn request_otp(
        &mut self,
        phone: impl Into<String>,
    ) -> Result<(), AuthStateMachineError> {
        let phone = phone.into();
        self.start_loading();

        match self.client.request_otp(&phone).await {
            Ok(()) => {
                self.state.step = AuthStep::OtpVerify;
                self.state.phone = phone;
                self.state.is_loading = false;
                Ok(())
            }
            Err(error) => {
                self.fail(error.clone());
                Err(error.into())
            }
        }
    }

    pub async fn verify_otp(&mut self, code: &str) -> Result<(), AuthStateMachineError> {
        self.start_loading();

        match self.client.verify_otp(&self.state.phone, code).await {
            Ok(response) if !response.session_token.is_empty() => {
                self.state.step = AuthStep::DeviceName;
                self.state.session_token = Some(response.session_token);
                self.state.is_loading = false;
                Ok(())
            }
            Ok(_) => {
                let error = AuthBackendError::new("Missing session token");
                self.fail(error.clone());
                Err(error.into())
            }
            Err(error) => {
                self.fail(error.clone());
                Err(error.into())
            }
        }
    }

    pub async fn register_device(
        &mut self,
        device_name: impl Into<String>,
        ed25519_public_key: &str,
    ) -> Result<(), AuthStateMachineError> {
        self.start_loading();
        let device_name = device_name.into();
        let Some(session_token) = self.state.session_token.clone() else {
            self.state.is_loading = false;
            self.state.error = Some("Missing session token for device registration".to_string());
            return Err(AuthStateMachineError::MissingSessionToken);
        };

        let result = self
            .client
            .register_device(&session_token, &device_name, ed25519_public_key)
            .await;

        match result {
            Ok(response)
                if !response.access_token.is_empty() && !response.refresh_token.is_empty() =>
            {
                let tokens = StoredTokens::new(response.access_token, response.refresh_token);
                let profile = match self.client.get_me(&tokens.access_token).await {
                    Ok(profile) => profile,
                    Err(error) => {
                        self.cleanup_persisted_session().await;
                        self.fail(error.clone());
                        return Err(error.into());
                    }
                };

                if let Err(error) = self.transport.connect(&tokens.access_token).await {
                    self.cleanup_persisted_session().await;
                    self.state = AuthState::initial();
                    return Err(error.into());
                }

                self.token_store.write(tokens.clone()).await;
                self.state.step = if profile.username.is_empty() {
                    AuthStep::UsernameSetup
                } else {
                    AuthStep::Authenticated
                };
                self.state.device_name = device_name;
                self.state.access_token = Some(tokens.access_token);
                self.state.refresh_token = Some(tokens.refresh_token);
                self.state.username = profile.username;
                self.state.discoverable_by_phone = profile.discoverable_by_phone;
                self.state.is_loading = false;
                self.state.error = None;
                Ok(())
            }
            Ok(_) => {
                self.cleanup_persisted_session().await;
                let error = AuthBackendError::new("Missing tokens from register-device response");
                self.fail(error.clone());
                Err(error.into())
            }
            Err(error) => {
                self.cleanup_persisted_session().await;
                self.fail(error.clone());
                Err(error.into())
            }
        }
    }

    pub async fn restore_session(&mut self) -> Result<(), AuthStateMachineError> {
        let Some(tokens) = self.token_store.read().await else {
            return Ok(());
        };

        match self.client.get_me(&tokens.access_token).await {
            Ok(profile) => {
                if let Err(error) = self.transport.connect(&tokens.access_token).await {
                    self.cleanup_persisted_session().await;
                    self.fail(error.clone());
                    return Err(error.into());
                }

                self.state.step = AuthStep::Authenticated;
                self.state.access_token = Some(tokens.access_token);
                self.state.refresh_token = Some(tokens.refresh_token);
                self.state.username = profile.username;
                self.state.discoverable_by_phone = profile.discoverable_by_phone;
                self.state.is_loading = false;
                self.state.error = None;
                Ok(())
            }
            Err(error) => {
                self.cleanup_persisted_session().await;
                self.state = AuthState::initial();
                Err(error.into())
            }
        }
    }

    pub async fn complete_username_setup(
        &mut self,
        username: &str,
        discoverable_by_phone: bool,
    ) -> Result<(), AuthStateMachineError> {
        self.start_loading();
        let Some(access_token) = self.state.access_token.clone() else {
            self.state.is_loading = false;
            self.state.error = Some("Authentication required".to_string());
            return Err(AuthStateMachineError::AuthenticationRequired);
        };

        match self
            .client
            .update_me(&access_token, username, discoverable_by_phone)
            .await
        {
            Ok(profile) => {
                self.state.step = AuthStep::Authenticated;
                self.state.username = if profile.username.is_empty() {
                    username.to_string()
                } else {
                    profile.username
                };
                self.state.discoverable_by_phone = profile.discoverable_by_phone;
                self.state.is_loading = false;
                self.state.error = None;
                Ok(())
            }
            Err(error) => {
                self.fail(error.clone());
                Err(error.into())
            }
        }
    }

    pub fn skip_username_setup(&mut self) {
        self.state.step = AuthStep::Authenticated;
        self.state.is_loading = false;
        self.state.error = None;
    }

    pub async fn logout(&mut self) {
        self.clear_session().await;
    }

    pub async fn handle_session_event(&mut self, event: SessionEvent) {
        if matches!(event.reason, SessionExpiryReason::Unauthorized) {
            self.state = AuthState::initial();
        }
    }

    fn start_loading(&mut self) {
        self.state.is_loading = true;
        self.state.error = None;
    }

    fn fail(&mut self, error: AuthBackendError) {
        self.state.is_loading = false;
        self.state.error = Some(error.message().to_string());
    }

    async fn clear_session(&mut self) {
        self.cleanup_persisted_session().await;
        self.state = AuthState::initial();
    }

    async fn cleanup_persisted_session(&self) {
        run_session_reset(self.token_store.as_ref(), || async {
            self.transport.disconnect().await;
        })
        .await;
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::sync::Arc;

    use tokio::sync::Mutex;

    use super::*;
    use crate::auth::token_store::InMemoryTokenStore;

    #[derive(Default)]
    struct StubAuthClient {
        request_otp_result: Mutex<Option<Result<(), AuthBackendError>>>,
        verify_otp_results: Mutex<VecDeque<Result<VerifyOtpResponse, AuthBackendError>>>,
        register_device_results: Mutex<VecDeque<Result<RegisterDeviceResponse, AuthBackendError>>>,
        get_me_results: Mutex<VecDeque<Result<AuthUserProfile, AuthBackendError>>>,
        update_me_results: Mutex<VecDeque<Result<AuthUserProfile, AuthBackendError>>>,
    }

    impl StubAuthClient {
        fn with_request_otp(result: Result<(), AuthBackendError>) -> Self {
            Self {
                request_otp_result: Mutex::new(Some(result)),
                ..Self::default()
            }
        }
    }

    #[async_trait]
    impl AuthClient for StubAuthClient {
        async fn request_otp(&self, _phone: &str) -> Result<(), AuthBackendError> {
            self.request_otp_result
                .lock()
                .await
                .take()
                .unwrap_or(Ok(()))
        }

        async fn verify_otp(
            &self,
            _phone: &str,
            _code: &str,
        ) -> Result<VerifyOtpResponse, AuthBackendError> {
            self.verify_otp_results
                .lock()
                .await
                .pop_front()
                .unwrap_or_else(|| {
                    Ok(VerifyOtpResponse {
                        session_token: "session".to_string(),
                    })
                })
        }

        async fn register_device(
            &self,
            _session_token: &str,
            _device_name: &str,
            _ed25519_public_key: &str,
        ) -> Result<RegisterDeviceResponse, AuthBackendError> {
            self.register_device_results
                .lock()
                .await
                .pop_front()
                .unwrap_or_else(|| {
                    Ok(RegisterDeviceResponse {
                        access_token: "access".to_string(),
                        refresh_token: "refresh".to_string(),
                    })
                })
        }

        async fn get_me(&self, _access_token: &str) -> Result<AuthUserProfile, AuthBackendError> {
            self.get_me_results
                .lock()
                .await
                .pop_front()
                .unwrap_or_else(|| {
                    Ok(AuthUserProfile {
                        username: "paw_friend".to_string(),
                        discoverable_by_phone: true,
                    })
                })
        }

        async fn update_me(
            &self,
            _access_token: &str,
            username: &str,
            discoverable_by_phone: bool,
        ) -> Result<AuthUserProfile, AuthBackendError> {
            self.update_me_results
                .lock()
                .await
                .pop_front()
                .unwrap_or_else(|| {
                    Ok(AuthUserProfile {
                        username: username.to_string(),
                        discoverable_by_phone,
                    })
                })
        }
    }

    #[derive(Default)]
    struct SpyTransport {
        connects: Mutex<Vec<String>>,
        disconnects: Mutex<usize>,
        connect_result: Mutex<Option<Result<(), AuthBackendError>>>,
    }

    #[async_trait]
    impl SessionTransport for SpyTransport {
        async fn connect(&self, access_token: &str) -> Result<(), AuthBackendError> {
            self.connects.lock().await.push(access_token.to_string());
            self.connect_result.lock().await.take().unwrap_or(Ok(()))
        }

        async fn disconnect(&self) {
            let mut disconnects = self.disconnects.lock().await;
            *disconnects += 1;
        }
    }

    #[tokio::test]
    async fn request_otp_advances_to_verify_step() {
        let client = Arc::new(StubAuthClient::with_request_otp(Ok(())));
        let store = Arc::new(InMemoryTokenStore::new());
        let transport = Arc::new(SpyTransport::default());
        let mut machine = AuthStateMachine::new(client, store, transport);

        machine.show_phone_otp();
        machine.request_otp("+821012345678").await.unwrap();

        assert_eq!(machine.state().step, AuthStep::OtpVerify);
        assert_eq!(machine.state().phone, "+821012345678");
        assert!(!machine.state().is_loading);
        assert_eq!(machine.state().error, None);
    }

    #[tokio::test]
    async fn register_device_persists_tokens_and_routes_to_username_setup() {
        let client = Arc::new(StubAuthClient {
            verify_otp_results: Mutex::new(VecDeque::from([Ok(VerifyOtpResponse {
                session_token: "session-1".to_string(),
            })])),
            get_me_results: Mutex::new(VecDeque::from([Ok(AuthUserProfile {
                username: String::new(),
                discoverable_by_phone: false,
            })])),
            ..StubAuthClient::default()
        });
        let store = Arc::new(InMemoryTokenStore::new());
        let transport = Arc::new(SpyTransport::default());
        let mut machine = AuthStateMachine::new(client, store.clone(), transport.clone());

        machine.show_phone_otp();
        machine.request_otp("+821012345678").await.unwrap();
        machine.verify_otp("123456").await.unwrap();
        machine
            .register_device("Haruna's iPhone", "base64pub")
            .await
            .unwrap();

        assert_eq!(machine.state().step, AuthStep::UsernameSetup);
        assert_eq!(machine.state().device_name, "Haruna's iPhone");
        assert_eq!(
            store.snapshot().await,
            Some(StoredTokens::new("access", "refresh"))
        );
        assert_eq!(transport.connects.lock().await.as_slice(), &["access"]);
    }

    #[tokio::test]
    async fn restore_session_failure_clears_tokens_and_state() {
        let client = Arc::new(StubAuthClient {
            get_me_results: Mutex::new(VecDeque::from([Err(AuthBackendError::new(
                "unauthorized",
            ))])),
            ..StubAuthClient::default()
        });
        let store = Arc::new(InMemoryTokenStore::new());
        store
            .write(StoredTokens::new("stale-access", "stale-refresh"))
            .await;
        let transport = Arc::new(SpyTransport::default());
        let mut machine = AuthStateMachine::new(client, store.clone(), transport.clone());

        let result = machine.restore_session().await;

        assert_eq!(
            result,
            Err(AuthStateMachineError::Backend(AuthBackendError::new(
                "unauthorized"
            )))
        );
        assert_eq!(machine.state(), &AuthState::initial());
        assert_eq!(store.snapshot().await, None);
        assert_eq!(*transport.disconnects.lock().await, 1);
    }

    #[tokio::test]
    async fn complete_username_setup_updates_profile_and_authenticates() {
        let client = Arc::new(StubAuthClient {
            update_me_results: Mutex::new(VecDeque::from([Ok(AuthUserProfile {
                username: "paw_friend19".to_string(),
                discoverable_by_phone: true,
            })])),
            ..StubAuthClient::default()
        });
        let store = Arc::new(InMemoryTokenStore::new());
        let transport = Arc::new(SpyTransport::default());
        let mut machine = AuthStateMachine::new(client, store, transport);
        machine.state = AuthState {
            step: AuthStep::UsernameSetup,
            access_token: Some("access".to_string()),
            ..AuthState::initial()
        };

        machine
            .complete_username_setup("paw_friend19", true)
            .await
            .unwrap();

        assert_eq!(machine.state().step, AuthStep::Authenticated);
        assert_eq!(machine.state().username, "paw_friend19");
        assert!(machine.state().discoverable_by_phone);
    }

    #[tokio::test]
    async fn unauthorized_session_events_clear_existing_session() {
        let client = Arc::new(StubAuthClient::default());
        let store = Arc::new(InMemoryTokenStore::new());
        store.write(StoredTokens::new("access", "refresh")).await;
        let transport = Arc::new(SpyTransport::default());
        let mut machine = AuthStateMachine::new(client, store.clone(), transport.clone());
        machine.state = AuthState {
            step: AuthStep::Authenticated,
            access_token: Some("access".to_string()),
            refresh_token: Some("refresh".to_string()),
            ..AuthState::initial()
        };

        machine
            .handle_session_event(SessionEvent {
                reason: SessionExpiryReason::Unauthorized,
            })
            .await;

        assert_eq!(machine.state(), &AuthState::initial());
        assert_eq!(
            store.snapshot().await,
            Some(StoredTokens::new("access", "refresh"))
        );
        assert_eq!(*transport.disconnects.lock().await, 0);
    }

    #[tokio::test]
    async fn logout_clears_persisted_session_and_disconnects_transport() {
        let client = Arc::new(StubAuthClient::default());
        let store = Arc::new(InMemoryTokenStore::new());
        store.write(StoredTokens::new("access", "refresh")).await;
        let transport = Arc::new(SpyTransport::default());
        let mut machine = AuthStateMachine::new(client, store.clone(), transport.clone());
        machine.state = AuthState {
            step: AuthStep::Authenticated,
            access_token: Some("access".to_string()),
            refresh_token: Some("refresh".to_string()),
            ..AuthState::initial()
        };

        machine.logout().await;

        assert_eq!(machine.state(), &AuthState::initial());
        assert_eq!(store.snapshot().await, None);
        assert_eq!(*transport.disconnects.lock().await, 1);
    }
}
