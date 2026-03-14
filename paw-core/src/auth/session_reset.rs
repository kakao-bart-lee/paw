use std::future::Future;

use super::TokenStore;

pub async fn run_session_reset<T, F, Fut>(token_store: &dyn TokenStore, after_clear: F) -> T
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = T>,
{
    token_store.clear().await;
    after_clear().await
}

#[cfg(test)]
mod tests {
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };

    use crate::auth::{InMemoryTokenStore, StoredTokens};

    use super::*;

    #[tokio::test]
    async fn clears_token_store_before_running_follow_up_action() {
        let token_store = InMemoryTokenStore::new();
        token_store.write(StoredTokens::new("access", "refresh")).await;
        let observed_empty = Arc::new(AtomicBool::new(false));
        let observed_empty_check = observed_empty.clone();

        run_session_reset(&token_store, || async {
            observed_empty_check.store(token_store.snapshot().await.is_none(), Ordering::SeqCst);
        })
        .await;

        assert!(observed_empty.load(Ordering::SeqCst));
    }
}
