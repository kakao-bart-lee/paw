use std::{collections::HashMap, sync::Arc, time::Duration};

use chrono::{DateTime, Utc};
use reqwest::{Method, RequestBuilder, StatusCode, Url};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use uuid::Uuid;

use super::error::{ErrorPayload, HttpClientError};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Clone)]
pub struct ApiClient {
    base_url: Url,
    client: reqwest::Client,
    access_token: Option<String>,
    on_unauthorized: Option<Arc<dyn Fn() + Send + Sync>>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestOtpResponse {
    pub ok: Option<bool>,
    pub debug_code: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifyOtpResponse {
    pub session_token: String,
    pub user_id: Option<Uuid>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegisterDeviceRequest {
    pub session_token: String,
    pub device_name: String,
    pub ed25519_public_key: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthTokens {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefreshTokenResponse {
    pub access_token: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversationListItem {
    pub id: Uuid,
    pub name: Option<String>,
    pub last_message: Option<String>,
    pub unread_count: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateConversationResponse {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AddMemberResponse {
    pub added: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoveMemberResponse {
    pub removed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SendMessageRequest {
    pub content: String,
    pub format: String,
    pub idempotency_key: Uuid,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SendMessageResponse {
    pub id: Uuid,
    pub seq: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageRecord {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub sender_id: Uuid,
    pub content: String,
    pub format: String,
    pub seq: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetMessagesResponse {
    pub messages: Vec<MessageRecord>,
    pub has_more: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: Uuid,
    pub phone: Option<String>,
    pub username: Option<String>,
    pub discoverable_by_phone: bool,
    pub phone_verified_at: Option<DateTime<Utc>>,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateMeRequest {
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub username: Option<String>,
    pub discoverable_by_phone: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OneTimeKey {
    pub key_id: i64,
    pub key: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyBundle {
    pub user_id: Option<Uuid>,
    pub identity_key: String,
    pub signed_prekey: String,
    #[serde(alias = "signed_prekey_sig", alias = "signature")]
    pub signed_prekey_sig: String,
    #[serde(default, alias = "one_time_prekey", alias = "one_time_key")]
    pub one_time_prekey: Option<OneTimeKey>,
    #[serde(default)]
    pub replenish_prekeys: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UploadKeysRequest {
    pub identity_key: String,
    pub signed_prekey: String,
    pub signed_prekey_sig: String,
    pub one_time_prekeys: Vec<OneTimeKey>,
}

#[derive(Debug, Deserialize)]
struct ConversationsEnvelope {
    conversations: Vec<ConversationListItem>,
}

impl ApiClient {
    pub fn new(base_url: impl AsRef<str>) -> Result<Self, HttpClientError> {
        Self::with_timeout(base_url, DEFAULT_TIMEOUT)
    }

    pub fn with_timeout(
        base_url: impl AsRef<str>,
        timeout: Duration,
    ) -> Result<Self, HttpClientError> {
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .map_err(HttpClientError::from)?;

        let base_url = Url::parse(base_url.as_ref())
            .map_err(|error| HttpClientError::invalid_response(error.to_string()))?;

        Ok(Self {
            base_url,
            client,
            access_token: None,
            on_unauthorized: None,
        })
    }

    pub fn with_unauthorized_handler(mut self, handler: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_unauthorized = Some(Arc::new(handler));
        self
    }

    pub fn set_access_token(&mut self, token: impl Into<String>) {
        self.access_token = Some(token.into());
    }

    pub fn clear_access_token(&mut self) {
        self.access_token = None;
    }

    pub fn access_token(&self) -> Option<&str> {
        self.access_token.as_deref()
    }

    pub async fn request_otp(&self, phone: &str) -> Result<RequestOtpResponse, HttpClientError> {
        self.post_json("/auth/request-otp", &serde_json::json!({ "phone": phone }))
            .await
    }

    pub async fn verify_otp(
        &self,
        phone: &str,
        code: &str,
    ) -> Result<VerifyOtpResponse, HttpClientError> {
        self.post_json(
            "/auth/verify-otp",
            &serde_json::json!({ "phone": phone, "code": code }),
        )
        .await
    }

    pub async fn register_device(
        &self,
        request: &RegisterDeviceRequest,
    ) -> Result<AuthTokens, HttpClientError> {
        self.post_json("/auth/register-device", request).await
    }

    pub async fn refresh_token(
        &self,
        refresh_token: &str,
    ) -> Result<RefreshTokenResponse, HttpClientError> {
        self.post_json(
            "/auth/refresh",
            &serde_json::json!({ "refresh_token": refresh_token }),
        )
        .await
    }

    pub async fn get_conversations(&self) -> Result<Vec<ConversationListItem>, HttpClientError> {
        let response: ConversationsEnvelope = self.get_json("/conversations", None).await?;
        Ok(response.conversations)
    }

    pub async fn create_conversation(
        &self,
        member_ids: Vec<Uuid>,
        name: Option<String>,
    ) -> Result<CreateConversationResponse, HttpClientError> {
        self.post_json(
            "/conversations",
            &serde_json::json!({
                "member_ids": member_ids,
                "name": name.filter(|value| !value.trim().is_empty()),
            }),
        )
        .await
    }

    pub async fn add_member(
        &self,
        conversation_id: Uuid,
        user_id: Uuid,
    ) -> Result<AddMemberResponse, HttpClientError> {
        self.post_json(
            &format!("/conversations/{conversation_id}/members"),
            &serde_json::json!({ "user_id": user_id }),
        )
        .await
    }

    pub async fn remove_member(
        &self,
        conversation_id: Uuid,
        user_id: Uuid,
    ) -> Result<RemoveMemberResponse, HttpClientError> {
        self.delete_json_without_body(&format!(
            "/conversations/{conversation_id}/members/{user_id}"
        ))
        .await
    }

    pub async fn send_message(
        &self,
        conversation_id: Uuid,
        request: &SendMessageRequest,
    ) -> Result<SendMessageResponse, HttpClientError> {
        self.post_json(
            &format!("/conversations/{conversation_id}/messages"),
            request,
        )
        .await
    }

    pub async fn get_messages(
        &self,
        conversation_id: Uuid,
        after_seq: i64,
        limit: i64,
    ) -> Result<GetMessagesResponse, HttpClientError> {
        let mut query = HashMap::new();
        query.insert("after_seq".to_string(), after_seq.to_string());
        query.insert("limit".to_string(), limit.to_string());
        self.get_json(
            &format!("/conversations/{conversation_id}/messages"),
            Some(&query),
        )
        .await
    }

    pub async fn get_me(&self) -> Result<UserProfile, HttpClientError> {
        self.get_json("/users/me", None).await
    }

    pub async fn update_me(
        &self,
        request: &UpdateMeRequest,
    ) -> Result<UserProfile, HttpClientError> {
        self.patch_json("/users/me", request).await
    }

    pub async fn get_user_by_id(&self, user_id: Uuid) -> Result<UserProfile, HttpClientError> {
        self.get_json(&format!("/users/{user_id}"), None).await
    }

    pub async fn search_user(
        &self,
        phone: Option<&str>,
        username: Option<&str>,
    ) -> Result<Option<UserProfile>, HttpClientError> {
        let mut query = HashMap::new();
        if let Some(phone) = phone.filter(|value| !value.is_empty()) {
            query.insert("phone".to_string(), phone.to_string());
        }
        if let Some(username) = username.filter(|value| !value.is_empty()) {
            query.insert("username".to_string(), username.to_string());
        }

        match self
            .get_json::<UserProfile>("/users/search", Some(&query))
            .await
        {
            Ok(user) => Ok(Some(user)),
            Err(error) if error.status_code() == Some(StatusCode::NOT_FOUND.as_u16()) => Ok(None),
            Err(error) => Err(error),
        }
    }

    pub async fn upload_keys(&self, bundle: &UploadKeysRequest) -> Result<(), HttpClientError> {
        self.post_empty("/api/v1/keys/upload", bundle).await
    }

    pub async fn get_key_bundle(
        &self,
        user_id: Uuid,
    ) -> Result<Option<KeyBundle>, HttpClientError> {
        match self
            .get_json::<KeyBundle>(&format!("/api/v1/keys/{user_id}"), None)
            .await
        {
            Ok(bundle) => Ok(Some(bundle)),
            Err(error) if error.status_code() == Some(StatusCode::NOT_FOUND.as_u16()) => Ok(None),
            Err(error) => Err(error),
        }
    }

    async fn get_json<T: DeserializeOwned>(
        &self,
        path: &str,
        query: Option<&HashMap<String, String>>,
    ) -> Result<T, HttpClientError> {
        let mut builder = self.request(Method::GET, path)?;
        if let Some(query) = query {
            builder = builder.query(query);
        }
        self.send_json(builder).await
    }

    async fn post_json<T: DeserializeOwned>(
        &self,
        path: &str,
        body: &impl Serialize,
    ) -> Result<T, HttpClientError> {
        let builder = self.request(Method::POST, path)?.json(body);
        self.send_json(builder).await
    }

    async fn patch_json<T: DeserializeOwned>(
        &self,
        path: &str,
        body: &impl Serialize,
    ) -> Result<T, HttpClientError> {
        let builder = self.request(Method::PATCH, path)?.json(body);
        self.send_json(builder).await
    }

    async fn delete_json_without_body<T: DeserializeOwned>(
        &self,
        path: &str,
    ) -> Result<T, HttpClientError> {
        let builder = self.request(Method::DELETE, path)?;
        self.send_json(builder).await
    }

    async fn post_empty(&self, path: &str, body: &impl Serialize) -> Result<(), HttpClientError> {
        let response = self.request(Method::POST, path)?.json(body).send().await?;
        self.ensure_success(response).await.map(|_| ())
    }

    fn request(&self, method: Method, path: &str) -> Result<RequestBuilder, HttpClientError> {
        let normalized = if path.starts_with('/') {
            path.to_string()
        } else {
            format!("/{path}")
        };
        let url = self
            .base_url
            .join(normalized.trim_start_matches('/'))
            .map_err(|error| HttpClientError::invalid_response(error.to_string()))?;

        let builder = self
            .client
            .request(method, url)
            .header("content-type", "application/json");
        Ok(match &self.access_token {
            Some(token) => builder.bearer_auth(token),
            None => builder,
        })
    }

    async fn send_json<T: DeserializeOwned>(
        &self,
        builder: RequestBuilder,
    ) -> Result<T, HttpClientError> {
        let response = builder.send().await?;
        let response = self.ensure_success(response).await?;
        response.json::<T>().await.map_err(HttpClientError::from)
    }

    async fn ensure_success(
        &self,
        response: reqwest::Response,
    ) -> Result<reqwest::Response, HttpClientError> {
        let status = response.status();
        if status.is_success() {
            return Ok(response);
        }

        let body = response.text().await.unwrap_or_default();
        let payload = serde_json::from_str::<ErrorPayload>(&body).ok();
        let code = payload.as_ref().and_then(|payload| payload.error.clone());
        let fallback = if body.is_empty() {
            format!("HTTP {}", status.as_u16())
        } else {
            body
        };
        let message = payload
            .as_ref()
            .and_then(|payload| payload.message.clone())
            .or_else(|| payload.as_ref().and_then(|payload| payload.error.clone()))
            .unwrap_or(fallback);

        let error = match status {
            StatusCode::UNAUTHORIZED => {
                if let Some(handler) = &self.on_unauthorized {
                    handler();
                }
                HttpClientError::unauthorized(message, code)
            }
            StatusCode::FORBIDDEN => HttpClientError::forbidden(message, code),
            StatusCode::NOT_FOUND => HttpClientError::not_found(message, code),
            status if status.is_server_error() => HttpClientError::server(status, message, code),
            status => HttpClientError::client(status, message, code),
        };

        Err(error)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        convert::Infallible,
        net::SocketAddr,
        sync::atomic::{AtomicUsize, Ordering},
    };

    use axum::{
        body::Body,
        extract::{Path, Query, State},
        http::{Request, StatusCode},
        response::IntoResponse,
        routing::{delete, get, post},
        Json, Router,
    };
    use serde_json::{json, Value};
    use tokio::{net::TcpListener, sync::Mutex, task::JoinHandle};

    use super::*;

    #[derive(Clone, Default)]
    struct TestState {
        auth_header: Arc<Mutex<Option<String>>>,
        message_query: Arc<Mutex<Option<HashMap<String, String>>>>,
        request_bodies: Arc<Mutex<Vec<Value>>>,
    }

    struct TestServer {
        address: SocketAddr,
        shutdown: Option<tokio::sync::oneshot::Sender<()>>,
        task: JoinHandle<()>,
        state: TestState,
    }

    impl TestServer {
        async fn start() -> Self {
            let state = TestState::default();
            let unauthorized_hits = Arc::new(AtomicUsize::new(0));
            let app = router(state.clone(), unauthorized_hits);
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let address = listener.local_addr().unwrap();
            let (tx, rx) = tokio::sync::oneshot::channel();
            let task = tokio::spawn(async move {
                axum::serve(listener, app)
                    .with_graceful_shutdown(async {
                        let _ = rx.await;
                    })
                    .await
                    .unwrap();
            });

            Self {
                address,
                shutdown: Some(tx),
                task,
                state,
            }
        }

        fn base_url(&self) -> String {
            format!("http://{}", self.address)
        }
    }

    impl Drop for TestServer {
        fn drop(&mut self) {
            if let Some(shutdown) = self.shutdown.take() {
                let _ = shutdown.send(());
            }
            self.task.abort();
        }
    }

    fn router(state: TestState, unauthorized_hits: Arc<AtomicUsize>) -> Router {
        Router::new()
            .route("/auth/request-otp", post(request_otp))
            .route("/auth/verify-otp", post(verify_otp))
            .route("/auth/register-device", post(register_device))
            .route("/auth/refresh", post(refresh))
            .route(
                "/conversations",
                get(list_conversations).post(create_conversation),
            )
            .route("/conversations/{id}/members", post(add_member))
            .route(
                "/conversations/{id}/members/{user_id}",
                delete(remove_member),
            )
            .route(
                "/conversations/{id}/messages",
                post(send_message).get(get_messages),
            )
            .route("/users/me", get(get_me).patch(update_me))
            .route("/users/search", get(search_user))
            .route("/users/{user_id}", get(get_user))
            .route("/api/v1/keys/upload", post(upload_keys))
            .route("/api/v1/keys/{user_id}", get(get_key_bundle))
            .route(
                "/errors/unauthorized",
                get(move || unauthorized(unauthorized_hits.clone())),
            )
            .fallback(record_request)
            .with_state(state)
    }

    async fn read_body(request: Request<Body>) -> Value {
        let bytes = axum::body::to_bytes(request.into_body(), usize::MAX)
            .await
            .unwrap();
        if bytes.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&bytes).unwrap()
        }
    }

    async fn request_otp(
        State(state): State<TestState>,
        request: Request<Body>,
    ) -> impl IntoResponse {
        let json = read_body(request).await;
        state.request_bodies.lock().await.push(json.clone());
        Json(json!({ "ok": true, "debug_code": "123456" }))
    }

    async fn verify_otp(request: Request<Body>) -> impl IntoResponse {
        let json = read_body(request).await;
        Json(json!({
            "session_token": format!("session-for-{}", json["phone"].as_str().unwrap()),
            "user_id": Uuid::nil(),
        }))
    }

    async fn register_device(request: Request<Body>) -> impl IntoResponse {
        let json = read_body(request).await;
        Json(json!({
            "access_token": format!("access:{}", json["device_name"].as_str().unwrap()),
            "refresh_token": "refresh-1",
        }))
    }

    async fn refresh(request: Request<Body>) -> impl IntoResponse {
        let json = read_body(request).await;
        Json(json!({ "access_token": format!("new-{}", json["refresh_token"].as_str().unwrap()) }))
    }

    async fn list_conversations(
        State(state): State<TestState>,
        request: Request<Body>,
    ) -> impl IntoResponse {
        state.auth_header.lock().await.replace(
            request
                .headers()
                .get("authorization")
                .unwrap()
                .to_str()
                .unwrap()
                .to_string(),
        );
        Json(json!({
            "conversations": [{
                "id": Uuid::nil(),
                "name": "General",
                "last_message": "Hello",
                "unread_count": 3,
            }]
        }))
    }

    async fn create_conversation(request: Request<Body>) -> impl IntoResponse {
        let _json = read_body(request).await;
        (
            StatusCode::CREATED,
            Json(json!({
                "id": Uuid::nil(),
                "created_at": Utc::now(),
            })),
        )
    }

    async fn add_member(request: Request<Body>) -> impl IntoResponse {
        let _json = read_body(request).await;
        Json(json!({ "added": true }))
    }

    async fn remove_member() -> impl IntoResponse {
        Json(json!({ "removed": true }))
    }

    async fn send_message(request: Request<Body>) -> impl IntoResponse {
        let _json = read_body(request).await;
        Json(json!({
            "id": Uuid::nil(),
            "seq": 7,
            "created_at": Utc::now(),
        }))
    }

    async fn get_messages(
        State(state): State<TestState>,
        Query(query): Query<HashMap<String, String>>,
    ) -> impl IntoResponse {
        state.message_query.lock().await.replace(query);
        Json(json!({
            "messages": [{
                "id": Uuid::nil(),
                "conversation_id": Uuid::nil(),
                "sender_id": Uuid::nil(),
                "content": "hello",
                "format": "plain",
                "seq": 7,
                "created_at": Utc::now(),
            }],
            "has_more": false,
        }))
    }

    async fn get_me() -> impl IntoResponse {
        Json(sample_user("me"))
    }

    async fn update_me(request: Request<Body>) -> impl IntoResponse {
        let json = read_body(request).await;
        Json(json!({
            "id": Uuid::nil(),
            "phone": "+821012345678",
            "username": json["username"],
            "discoverable_by_phone": json["discoverable_by_phone"],
            "phone_verified_at": Utc::now(),
            "display_name": json["display_name"],
            "avatar_url": json["avatar_url"],
            "created_at": Utc::now(),
        }))
    }

    async fn search_user(Query(query): Query<HashMap<String, String>>) -> impl IntoResponse {
        if query.get("username") == Some(&"missing".to_string()) {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "user_not_found", "message": "User not found" })),
            )
                .into_response();
        }
        Json(sample_user(
            query.get("username").map(String::as_str).unwrap_or("found"),
        ))
        .into_response()
    }

    async fn get_user(Path(user_id): Path<Uuid>) -> impl IntoResponse {
        Json(json!({
            "id": user_id,
            "username": "other_user",
            "display_name": "Other",
            "avatar_url": null,
            "discoverable_by_phone": false,
            "phone_verified_at": null,
            "phone": null,
            "created_at": Utc::now(),
        }))
    }

    async fn upload_keys(request: Request<Body>) -> impl IntoResponse {
        let json = read_body(request).await;
        assert_eq!(json["signed_prekey_sig"], "sig");
        StatusCode::NO_CONTENT
    }

    async fn get_key_bundle(Path(user_id): Path<Uuid>) -> impl IntoResponse {
        if user_id == Uuid::max() {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "bundle_not_found",
                    "message": "Prekey bundle not found"
                })),
            )
                .into_response();
        }

        Json(json!({
            "user_id": user_id,
            "identity_key": "identity",
            "signed_prekey": "signed",
            "signed_prekey_sig": "sig",
            "one_time_prekey": {
                "key_id": 1,
                "key": "otk",
            },
            "replenish_prekeys": false,
        }))
        .into_response()
    }

    async fn unauthorized(counter: Arc<AtomicUsize>) -> impl IntoResponse {
        counter.fetch_add(1, Ordering::SeqCst);
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "unauthorized", "message": "expired" })),
        )
    }

    async fn record_request(
        State(_state): State<TestState>,
        _request: Request<Body>,
    ) -> Result<impl IntoResponse, Infallible> {
        Ok((
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "missing", "message": "missing" })),
        ))
    }

    fn sample_user(username: &str) -> Value {
        json!({
            "id": Uuid::nil(),
            "phone": "+821012345678",
            "username": username,
            "discoverable_by_phone": true,
            "phone_verified_at": Utc::now(),
            "display_name": "Paw Friend",
            "avatar_url": "https://example.com/avatar.png",
            "created_at": Utc::now(),
        })
    }

    #[tokio::test]
    async fn auth_endpoints_round_trip() {
        let server = TestServer::start().await;
        let client = ApiClient::new(server.base_url()).unwrap();

        let otp = client.request_otp("+821012345678").await.unwrap();
        assert_eq!(otp.debug_code.as_deref(), Some("123456"));

        let verified = client.verify_otp("+821012345678", "123456").await.unwrap();
        assert_eq!(verified.session_token, "session-for-+821012345678");

        let tokens = client
            .register_device(&RegisterDeviceRequest {
                session_token: verified.session_token,
                device_name: "Haruna's iPhone".to_string(),
                ed25519_public_key: "pubkey".to_string(),
            })
            .await
            .unwrap();
        assert_eq!(tokens.access_token, "access:Haruna's iPhone");

        let refreshed = client.refresh_token("refresh-1").await.unwrap();
        assert_eq!(refreshed.access_token, "new-refresh-1");
    }

    #[tokio::test]
    async fn conversation_and_message_endpoints_use_auth_and_query_params() {
        let server = TestServer::start().await;
        let mut client = ApiClient::new(server.base_url()).unwrap();
        client.set_access_token("token-123");

        let conversations = client.get_conversations().await.unwrap();
        assert_eq!(conversations.len(), 1);
        assert_eq!(
            server.state.auth_header.lock().await.as_deref(),
            Some("Bearer token-123")
        );

        let created = client
            .create_conversation(vec![Uuid::nil()], Some("Group".to_string()))
            .await
            .unwrap();
        assert_eq!(created.id, Uuid::nil());

        let added = client.add_member(Uuid::nil(), Uuid::nil()).await.unwrap();
        assert!(added.added);

        let removed = client
            .remove_member(Uuid::nil(), Uuid::nil())
            .await
            .unwrap();
        assert!(removed.removed);

        let sent = client
            .send_message(
                Uuid::nil(),
                &SendMessageRequest {
                    content: "hello".to_string(),
                    format: "plain".to_string(),
                    idempotency_key: Uuid::nil(),
                },
            )
            .await
            .unwrap();
        assert_eq!(sent.seq, 7);

        let history = client.get_messages(Uuid::nil(), 9, 20).await.unwrap();
        assert_eq!(history.messages.len(), 1);
        assert_eq!(
            server.state.message_query.lock().await.clone(),
            Some(HashMap::from([
                ("after_seq".to_string(), "9".to_string()),
                ("limit".to_string(), "20".to_string()),
            ]))
        );
    }

    #[tokio::test]
    async fn user_and_key_endpoints_return_typed_payloads_and_optional_not_found() {
        let server = TestServer::start().await;
        let mut client = ApiClient::new(server.base_url()).unwrap();
        client.set_access_token("token-123");

        let me = client.get_me().await.unwrap();
        assert_eq!(me.username.as_deref(), Some("me"));

        let updated = client
            .update_me(&UpdateMeRequest {
                display_name: Some("Haruna".to_string()),
                avatar_url: Some("https://example.com/me.png".to_string()),
                username: Some("haruna".to_string()),
                discoverable_by_phone: Some(true),
            })
            .await
            .unwrap();
        assert_eq!(updated.username.as_deref(), Some("haruna"));

        let other = client.get_user_by_id(Uuid::new_v4()).await.unwrap();
        assert_eq!(other.username.as_deref(), Some("other_user"));

        let found = client.search_user(None, Some("friend")).await.unwrap();
        assert_eq!(
            found.and_then(|user| user.username),
            Some("friend".to_string())
        );
        assert_eq!(
            client.search_user(None, Some("missing")).await.unwrap(),
            None
        );

        client
            .upload_keys(&UploadKeysRequest {
                identity_key: "identity".to_string(),
                signed_prekey: "signed".to_string(),
                signed_prekey_sig: "sig".to_string(),
                one_time_prekeys: vec![OneTimeKey {
                    key_id: 1,
                    key: "otk".to_string(),
                }],
            })
            .await
            .unwrap();

        let bundle = client.get_key_bundle(Uuid::nil()).await.unwrap().unwrap();
        assert_eq!(bundle.signed_prekey_sig, "sig");
        assert_eq!(client.get_key_bundle(Uuid::max()).await.unwrap(), None);
    }

    #[tokio::test]
    async fn unauthorized_responses_are_typed_and_trigger_hook() {
        let server = TestServer::start().await;
        let hook_hits = Arc::new(AtomicUsize::new(0));
        let hook_counter = hook_hits.clone();
        let client = ApiClient::new(server.base_url())
            .unwrap()
            .with_unauthorized_handler(move || {
                hook_counter.fetch_add(1, Ordering::SeqCst);
            });

        let error = client
            .get_json::<Value>("/errors/unauthorized", None)
            .await
            .unwrap_err();

        assert!(error.is_unauthorized());
        assert_eq!(error.code(), Some("unauthorized"));
        assert_eq!(error.message(), "expired");
        assert_eq!(hook_hits.load(Ordering::SeqCst), 1);
    }
}
