use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::auth::{StoredTokens, TokenStore};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, uniffi::Enum)]
pub enum SecureStorageProvider {
    Keychain,
    Keystore,
    MemoryFallback,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, uniffi::Enum)]
pub enum SecureStorageAvailability {
    Available,
    Degraded,
    Unavailable,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct SecureStorageCapabilities {
    pub provider: SecureStorageProvider,
    pub availability: SecureStorageAvailability,
    pub supports_tokens: bool,
    pub supports_device_keys: bool,
    pub supports_biometric_gate: bool,
}

impl SecureStorageCapabilities {
    pub fn memory_fallback() -> Self {
        Self {
            provider: SecureStorageProvider::MemoryFallback,
            availability: SecureStorageAvailability::Degraded,
            supports_tokens: true,
            supports_device_keys: true,
            supports_biometric_gate: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct DeviceKeyMaterial {
    pub identity_key: Vec<u8>,
    pub x25519_private_key: Vec<u8>,
    pub x25519_public_key: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, uniffi::Enum)]
pub enum SecureStorageErrorCode {
    Unavailable,
    PermissionDenied,
    CorruptedData,
    Serialization,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct SecureStorageError {
    pub code: SecureStorageErrorCode,
    pub message: String,
    pub retryable: bool,
}

#[async_trait]
pub trait SecureTokenVault: Send + Sync {
    async fn capabilities(&self) -> SecureStorageCapabilities;
    async fn read_tokens(&self) -> Result<Option<StoredTokens>, SecureStorageError>;
    async fn write_tokens(&self, tokens: StoredTokens) -> Result<(), SecureStorageError>;
    async fn clear_tokens(&self) -> Result<(), SecureStorageError>;
}

#[async_trait]
pub trait DeviceKeyStore: Send + Sync {
    async fn capabilities(&self) -> SecureStorageCapabilities;
    async fn load_device_keys(&self) -> Result<Option<DeviceKeyMaterial>, SecureStorageError>;
    async fn save_device_keys(&self, keys: DeviceKeyMaterial) -> Result<(), SecureStorageError>;
    async fn clear_device_keys(&self) -> Result<(), SecureStorageError>;
}

#[derive(Clone)]
pub struct InMemorySecureTokenVault {
    tokens: Arc<Mutex<Option<StoredTokens>>>,
    capabilities: SecureStorageCapabilities,
}

impl Default for InMemorySecureTokenVault {
    fn default() -> Self {
        Self {
            tokens: Arc::new(Mutex::new(None)),
            capabilities: SecureStorageCapabilities::memory_fallback(),
        }
    }
}

impl InMemorySecureTokenVault {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl SecureTokenVault for InMemorySecureTokenVault {
    async fn capabilities(&self) -> SecureStorageCapabilities {
        self.capabilities.clone()
    }

    async fn read_tokens(&self) -> Result<Option<StoredTokens>, SecureStorageError> {
        Ok(self.tokens.lock().await.clone())
    }

    async fn write_tokens(&self, tokens: StoredTokens) -> Result<(), SecureStorageError> {
        *self.tokens.lock().await = Some(tokens);
        Ok(())
    }

    async fn clear_tokens(&self) -> Result<(), SecureStorageError> {
        *self.tokens.lock().await = None;
        Ok(())
    }
}

#[async_trait]
impl TokenStore for InMemorySecureTokenVault {
    async fn read(&self) -> Option<StoredTokens> {
        self.read_tokens().await.ok().flatten()
    }

    async fn write(&self, tokens: StoredTokens) {
        let _ = self.write_tokens(tokens).await;
    }

    async fn clear(&self) {
        let _ = self.clear_tokens().await;
    }
}

#[derive(Clone)]
pub struct InMemoryDeviceKeyStore {
    keys: Arc<Mutex<Option<DeviceKeyMaterial>>>,
    capabilities: SecureStorageCapabilities,
}

impl Default for InMemoryDeviceKeyStore {
    fn default() -> Self {
        Self {
            keys: Arc::new(Mutex::new(None)),
            capabilities: SecureStorageCapabilities::memory_fallback(),
        }
    }
}

impl InMemoryDeviceKeyStore {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl DeviceKeyStore for InMemoryDeviceKeyStore {
    async fn capabilities(&self) -> SecureStorageCapabilities {
        self.capabilities.clone()
    }

    async fn load_device_keys(&self) -> Result<Option<DeviceKeyMaterial>, SecureStorageError> {
        Ok(self.keys.lock().await.clone())
    }

    async fn save_device_keys(&self, keys: DeviceKeyMaterial) -> Result<(), SecureStorageError> {
        *self.keys.lock().await = Some(keys);
        Ok(())
    }

    async fn clear_device_keys(&self) -> Result<(), SecureStorageError> {
        *self.keys.lock().await = None;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, uniffi::Enum)]
pub enum PushPlatform {
    Fcm,
    Apns,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, uniffi::Enum)]
pub enum PushRegistrationStatus {
    Unregistered,
    Registering,
    Registered,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct PushTokenRegistration {
    pub token: String,
    pub platform: PushPlatform,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct PushRegistrationState {
    pub status: PushRegistrationStatus,
    pub token: Option<String>,
    pub platform: Option<PushPlatform>,
    pub last_error: Option<String>,
    pub last_updated_ms: i64,
}

impl PushRegistrationState {
    pub fn unregistered() -> Self {
        Self {
            status: PushRegistrationStatus::Unregistered,
            token: None,
            platform: None,
            last_error: None,
            last_updated_ms: Utc::now().timestamp_millis(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, uniffi::Enum)]
pub enum PushRegistrationErrorCode {
    EmptyToken,
    Unauthorized,
    Transport,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct PushRegistrationError {
    pub code: PushRegistrationErrorCode,
    pub message: String,
    pub retryable: bool,
}

#[async_trait]
pub trait PushRegistrar: Send + Sync {
    async fn register(
        &self,
        registration: PushTokenRegistration,
    ) -> Result<PushRegistrationState, PushRegistrationError>;
    async fn unregister(&self) -> Result<PushRegistrationState, PushRegistrationError>;
    async fn current_state(&self) -> PushRegistrationState;
}

#[derive(Clone)]
pub struct NoopPushRegistrar {
    state: Arc<Mutex<PushRegistrationState>>,
}

impl NoopPushRegistrar {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(PushRegistrationState::unregistered())),
        }
    }
}

impl Default for NoopPushRegistrar {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PushRegistrar for NoopPushRegistrar {
    async fn register(
        &self,
        registration: PushTokenRegistration,
    ) -> Result<PushRegistrationState, PushRegistrationError> {
        if registration.token.trim().is_empty() {
            return Err(PushRegistrationError {
                code: PushRegistrationErrorCode::EmptyToken,
                message: "Push token must not be empty".into(),
                retryable: false,
            });
        }

        let next = PushRegistrationState {
            status: PushRegistrationStatus::Registered,
            token: Some(registration.token),
            platform: Some(registration.platform),
            last_error: None,
            last_updated_ms: Utc::now().timestamp_millis(),
        };
        *self.state.lock().await = next.clone();
        Ok(next)
    }

    async fn unregister(&self) -> Result<PushRegistrationState, PushRegistrationError> {
        let next = PushRegistrationState::unregistered();
        *self.state.lock().await = next.clone();
        Ok(next)
    }

    async fn current_state(&self) -> PushRegistrationState {
        self.state.lock().await.clone()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, uniffi::Enum)]
pub enum LifecycleState {
    Launching,
    Active,
    Inactive,
    Background,
    Terminated,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, uniffi::Enum)]
pub enum LifecycleHint {
    RefreshPushToken,
    ReconnectSocket,
    PauseRealtime,
    FlushAcks,
    PersistDrafts,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct LifecycleEvent {
    pub state: LifecycleState,
    pub timestamp_ms: i64,
    pub user_initiated: bool,
}

#[async_trait]
pub trait LifecycleBridge: Send + Sync {
    async fn handle_event(&self, event: LifecycleEvent) -> Vec<LifecycleHint>;
    async fn current_state(&self) -> LifecycleState;
}

#[derive(Clone)]
pub struct InMemoryLifecycleBridge {
    state: Arc<Mutex<LifecycleState>>,
}

impl Default for InMemoryLifecycleBridge {
    fn default() -> Self {
        Self {
            state: Arc::new(Mutex::new(LifecycleState::Launching)),
        }
    }
}

impl InMemoryLifecycleBridge {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl LifecycleBridge for InMemoryLifecycleBridge {
    async fn handle_event(&self, event: LifecycleEvent) -> Vec<LifecycleHint> {
        *self.state.lock().await = event.state.clone();
        lifecycle_hints_for(&event)
    }

    async fn current_state(&self) -> LifecycleState {
        self.state.lock().await.clone()
    }
}

pub fn lifecycle_hints_for(event: &LifecycleEvent) -> Vec<LifecycleHint> {
    match event.state {
        LifecycleState::Launching | LifecycleState::Active => {
            vec![
                LifecycleHint::ReconnectSocket,
                LifecycleHint::RefreshPushToken,
            ]
        }
        LifecycleState::Inactive => vec![LifecycleHint::FlushAcks],
        LifecycleState::Background => {
            vec![
                LifecycleHint::PauseRealtime,
                LifecycleHint::FlushAcks,
                LifecycleHint::PersistDrafts,
            ]
        }
        LifecycleState::Terminated => {
            vec![LifecycleHint::FlushAcks, LifecycleHint::PersistDrafts]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn memory_secure_vault_adapts_to_token_store_behavior() {
        let vault = InMemorySecureTokenVault::new();
        vault
            .write_tokens(StoredTokens::new("access", "refresh"))
            .await
            .unwrap();

        assert_eq!(
            vault.read().await,
            Some(StoredTokens::new("access", "refresh"))
        );

        vault.clear().await;
        assert_eq!(vault.read().await, None);
    }

    #[tokio::test]
    async fn device_key_store_round_trips_material() {
        let store = InMemoryDeviceKeyStore::new();
        let material = DeviceKeyMaterial {
            identity_key: vec![1, 2],
            x25519_private_key: vec![3, 4],
            x25519_public_key: vec![5, 6],
        };

        store.save_device_keys(material.clone()).await.unwrap();
        assert_eq!(store.load_device_keys().await.unwrap(), Some(material));
    }

    #[tokio::test]
    async fn noop_push_registrar_tracks_latest_state() {
        let registrar = NoopPushRegistrar::new();
        let state = registrar
            .register(PushTokenRegistration {
                token: "abc".into(),
                platform: PushPlatform::Fcm,
            })
            .await
            .unwrap();

        assert_eq!(state.status, PushRegistrationStatus::Registered);
        assert_eq!(
            registrar.current_state().await.platform,
            Some(PushPlatform::Fcm)
        );
    }

    #[test]
    fn lifecycle_hints_match_runtime_expectations() {
        let hints = lifecycle_hints_for(&LifecycleEvent {
            state: LifecycleState::Background,
            timestamp_ms: 1,
            user_initiated: false,
        });

        assert_eq!(
            hints,
            vec![
                LifecycleHint::PauseRealtime,
                LifecycleHint::FlushAcks,
                LifecycleHint::PersistDrafts,
            ]
        );
    }
}
