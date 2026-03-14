use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoredTokens {
    pub access_token: String,
    pub refresh_token: String,
}

impl StoredTokens {
    pub fn new(access_token: impl Into<String>, refresh_token: impl Into<String>) -> Self {
        Self {
            access_token: access_token.into(),
            refresh_token: refresh_token.into(),
        }
    }
}

#[async_trait]
pub trait TokenStore: Send + Sync {
    async fn read(&self) -> Option<StoredTokens>;
    async fn write(&self, tokens: StoredTokens);
    async fn clear(&self);
}

#[derive(Clone, Default)]
pub struct InMemoryTokenStore {
    inner: Arc<Mutex<Option<StoredTokens>>>,
}

impl InMemoryTokenStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn snapshot(&self) -> Option<StoredTokens> {
        self.inner.lock().await.clone()
    }
}

#[async_trait]
impl TokenStore for InMemoryTokenStore {
    async fn read(&self) -> Option<StoredTokens> {
        self.inner.lock().await.clone()
    }

    async fn write(&self, tokens: StoredTokens) {
        *self.inner.lock().await = Some(tokens);
    }

    async fn clear(&self) {
        *self.inner.lock().await = None;
    }
}
