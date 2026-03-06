use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const TEST_JWT_SECRET: &str = "integration_test_secret_key_do_not_use_in_prod";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Claims {
    sub: Uuid,
    device_id: Option<Uuid>,
    token_type: String,
    exp: i64,
    iat: i64,
}

fn issue_token(
    user_id: Uuid,
    device_id: Option<Uuid>,
    token_type: &str,
    ttl: Duration,
    secret: &str,
) -> String {
    let now = Utc::now();
    let claims = Claims {
        sub: user_id,
        device_id,
        token_type: token_type.to_string(),
        iat: now.timestamp(),
        exp: (now + ttl).timestamp(),
    };
    encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .expect("token encoding must succeed")
}

fn verify_token(token: &str, secret: &str, expected_type: Option<&str>) -> Result<Claims, String> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;

    let data = decode::<Claims>(token, &DecodingKey::from_secret(secret.as_bytes()), &validation)
        .map_err(|e| format!("invalid token: {e}"))?;

    if let Some(expected) = expected_type {
        if data.claims.token_type != expected {
            return Err("invalid token type".to_string());
        }
    }
    Ok(data.claims)
}

// ── JWT: session token round-trip ───────────────────────────────────────

#[test]
fn jwt_session_token_roundtrip() {
    let user_id = Uuid::new_v4();
    let token = issue_token(user_id, None, "session", Duration::minutes(15), TEST_JWT_SECRET);
    let claims = verify_token(&token, TEST_JWT_SECRET, Some("session")).unwrap();

    assert_eq!(claims.sub, user_id);
    assert_eq!(claims.token_type, "session");
    assert!(claims.device_id.is_none());
}

#[test]
fn jwt_access_token_contains_device_id() {
    let user_id = Uuid::new_v4();
    let device_id = Uuid::new_v4();
    let token = issue_token(
        user_id,
        Some(device_id),
        "access",
        Duration::days(7),
        TEST_JWT_SECRET,
    );
    let claims = verify_token(&token, TEST_JWT_SECRET, Some("access")).unwrap();

    assert_eq!(claims.sub, user_id);
    assert_eq!(claims.device_id, Some(device_id));
    assert_eq!(claims.token_type, "access");
}

#[test]
fn jwt_refresh_token_thirty_day_ttl() {
    let user_id = Uuid::new_v4();
    let device_id = Uuid::new_v4();
    let token = issue_token(
        user_id,
        Some(device_id),
        "refresh",
        Duration::days(30),
        TEST_JWT_SECRET,
    );
    let claims = verify_token(&token, TEST_JWT_SECRET, Some("refresh")).unwrap();

    let expected_exp = Utc::now() + Duration::days(30);
    let exp_delta = (claims.exp - expected_exp.timestamp()).abs();
    assert!(exp_delta < 5, "refresh token TTL should be ~30 days");
}

#[test]
fn jwt_wrong_secret_rejected() {
    let token = issue_token(Uuid::new_v4(), None, "session", Duration::minutes(15), TEST_JWT_SECRET);
    let result = verify_token(&token, "wrong_secret", Some("session"));
    assert!(result.is_err());
}

#[test]
fn jwt_wrong_token_type_rejected() {
    let token = issue_token(Uuid::new_v4(), None, "session", Duration::minutes(15), TEST_JWT_SECRET);
    let result = verify_token(&token, TEST_JWT_SECRET, Some("access"));
    assert!(result.is_err());
}

#[test]
fn jwt_expired_token_rejected() {
    let token = issue_token(Uuid::new_v4(), None, "session", Duration::seconds(-120), TEST_JWT_SECRET);
    let result = verify_token(&token, TEST_JWT_SECRET, Some("session"));
    assert!(result.is_err());
}

// ── Protocol: frame serialization ───────────────────────────────────────

#[test]
fn protocol_message_send_includes_v_field() {
    let frame = paw_proto::MessageSendMsg {
        v: paw_proto::PROTOCOL_VERSION,
        conversation_id: Uuid::new_v4(),
        content: "hello".into(),
        format: paw_proto::MessageFormat::Markdown,
        blocks: vec![],
        idempotency_key: Uuid::new_v4(),
    };
    let json = serde_json::to_value(&frame).unwrap();
    assert_eq!(json["v"], 1);
}

#[test]
fn protocol_client_message_tagged_serialization() {
    let msg = paw_proto::ClientMessage::MessageSend(paw_proto::MessageSendMsg {
        v: 1,
        conversation_id: Uuid::new_v4(),
        content: "test".into(),
        format: paw_proto::MessageFormat::Plain,
        blocks: vec![],
        idempotency_key: Uuid::new_v4(),
    });
    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains(r#""type":"message_send""#));
    assert!(json.contains(r#""v":1"#));
}

#[test]
fn protocol_server_hello_ok_roundtrip() {
    let user_id = Uuid::new_v4();
    let now = Utc::now();
    let msg = paw_proto::ServerMessage::HelloOk(paw_proto::HelloOkMsg {
        v: 1,
        user_id,
        server_time: now,
    });
    let json = serde_json::to_string(&msg).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed["type"], "hello_ok");
    assert_eq!(parsed["v"], 1);
    assert_eq!(parsed["user_id"], user_id.to_string());
}

#[test]
fn protocol_message_received_all_fields() {
    let msg = paw_proto::MessageReceivedMsg {
        v: 1,
        id: Uuid::new_v4(),
        conversation_id: Uuid::new_v4(),
        sender_id: Uuid::new_v4(),
        content: "Hello!".into(),
        format: paw_proto::MessageFormat::Markdown,
        seq: 42,
        created_at: Utc::now(),
        blocks: vec![],
    };
    let json = serde_json::to_value(&msg).unwrap();

    assert_eq!(json["v"], 1);
    assert_eq!(json["seq"], 42);
    assert_eq!(json["content"], "Hello!");
    assert_eq!(json["format"], "markdown");
    assert!(json["id"].is_string());
    assert!(json["created_at"].is_string());
}

#[test]
fn protocol_sync_frame_roundtrip() {
    let conv_id = Uuid::new_v4();
    let msg = paw_proto::ClientMessage::Sync(paw_proto::SyncMsg {
        v: 1,
        conversation_id: conv_id,
        last_seq: 99,
    });
    let json = serde_json::to_string(&msg).unwrap();
    let parsed: paw_proto::ClientMessage = serde_json::from_str(&json).unwrap();
    match parsed {
        paw_proto::ClientMessage::Sync(sync) => {
            assert_eq!(sync.conversation_id, conv_id);
            assert_eq!(sync.last_seq, 99);
            assert_eq!(sync.v, 1);
        }
        _ => panic!("expected Sync variant"),
    }
}

#[test]
fn protocol_typing_omits_user_id_when_none() {
    let msg = paw_proto::TypingMsg {
        v: 1,
        conversation_id: Uuid::new_v4(),
        user_id: None,
    };
    let json = serde_json::to_string(&msg).unwrap();
    assert!(!json.contains("user_id"), "user_id=None should be omitted via skip_serializing_if");
}

#[test]
fn protocol_typing_includes_user_id_when_present() {
    let uid = Uuid::new_v4();
    let msg = paw_proto::TypingMsg {
        v: 1,
        conversation_id: Uuid::new_v4(),
        user_id: Some(uid),
    };
    let json = serde_json::to_value(&msg).unwrap();
    assert_eq!(json["user_id"], uid.to_string());
}

#[test]
fn protocol_message_ack_roundtrip() {
    let conv_id = Uuid::new_v4();
    let msg = paw_proto::ClientMessage::MessageAck(paw_proto::MessageAckMsg {
        v: 1,
        conversation_id: conv_id,
        last_seq: 7,
    });
    let serialized = serde_json::to_string(&msg).unwrap();
    let parsed: paw_proto::ClientMessage = serde_json::from_str(&serialized).unwrap();
    match parsed {
        paw_proto::ClientMessage::MessageAck(ack) => {
            assert_eq!(ack.last_seq, 7);
            assert_eq!(ack.conversation_id, conv_id);
        }
        _ => panic!("expected MessageAck"),
    }
}

// ── Integration: Auth flow (requires running server + DB) ───────────────

#[tokio::test]
#[ignore = "requires running paw-server and PostgreSQL"]
async fn auth_request_otp_valid_phone() {
    let client = reqwest::Client::new();
    let resp = client
        .post("http://localhost:3000/auth/request-otp")
        .json(&serde_json::json!({ "phone": "+821012345678" }))
        .send()
        .await
        .expect("server must be reachable");

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["ok"], true);
}

#[tokio::test]
#[ignore = "requires running paw-server and PostgreSQL"]
async fn auth_request_otp_invalid_phone_rejected() {
    let client = reqwest::Client::new();
    let resp = client
        .post("http://localhost:3000/auth/request-otp")
        .json(&serde_json::json!({ "phone": "123" }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"], "invalid_phone");
}

#[tokio::test]
#[ignore = "requires running paw-server and PostgreSQL"]
async fn auth_verify_otp_invalid_code_format() {
    let client = reqwest::Client::new();
    let resp = client
        .post("http://localhost:3000/auth/verify-otp")
        .json(&serde_json::json!({ "phone": "+821012345678", "code": "abc" }))
        .send()
        .await
        .unwrap();

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"], "invalid_code_format");
}

#[tokio::test]
#[ignore = "requires running paw-server and PostgreSQL"]
async fn auth_full_flow_request_verify_register() {
    let client = reqwest::Client::new();
    let phone = format!("+8210{:08}", rand_u32() % 100_000_000);

    let resp = client
        .post("http://localhost:3000/auth/request-otp")
        .json(&serde_json::json!({ "phone": phone }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    let resp = client
        .post("http://localhost:3000/auth/verify-otp")
        .json(&serde_json::json!({ "phone": phone, "code": "000000" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
}

// ── Integration: Message relay via HTTP (requires running server) ───────

#[tokio::test]
#[ignore = "requires running paw-server with auth token"]
async fn message_send_and_retrieve() {
    let base = "http://localhost:3000";
    let token = std::env::var("PAW_TEST_TOKEN").unwrap_or_else(|_| "test_token".into());
    let conv_id = std::env::var("PAW_TEST_CONV_ID")
        .unwrap_or_else(|_| "00000000-0000-0000-0000-000000000001".into());

    let client = reqwest::Client::new();
    let idempotency_key = Uuid::new_v4();

    let send_resp = client
        .post(format!("{base}/conversations/{conv_id}/messages"))
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "content": "integration test message",
            "format": "plain",
            "idempotency_key": idempotency_key,
        }))
        .send()
        .await
        .unwrap();

    assert!(
        send_resp.status() == 200 || send_resp.status() == 403,
        "expected 200 or 403 (not a member), got {}",
        send_resp.status()
    );

    if send_resp.status() == 200 {
        let body: serde_json::Value = send_resp.json().await.unwrap();
        assert!(body["id"].is_string());
        assert!(body["seq"].is_number());

        let get_resp = client
            .get(format!("{base}/conversations/{conv_id}/messages?limit=5"))
            .bearer_auth(&token)
            .send()
            .await
            .unwrap();
        assert_eq!(get_resp.status(), 200);
        let get_body: serde_json::Value = get_resp.json().await.unwrap();
        assert!(get_body["messages"].is_array());
    }
}

#[tokio::test]
#[ignore = "requires running paw-server with auth token"]
async fn message_idempotency_returns_same_result() {
    let base = "http://localhost:3000";
    let token = std::env::var("PAW_TEST_TOKEN").unwrap_or_else(|_| "test_token".into());
    let conv_id = std::env::var("PAW_TEST_CONV_ID")
        .unwrap_or_else(|_| "00000000-0000-0000-0000-000000000001".into());

    let client = reqwest::Client::new();
    let idempotency_key = Uuid::new_v4();
    let payload = serde_json::json!({
        "content": "idempotency test",
        "format": "plain",
        "idempotency_key": idempotency_key,
    });

    let resp1 = client
        .post(format!("{base}/conversations/{conv_id}/messages"))
        .bearer_auth(&token)
        .json(&payload)
        .send()
        .await
        .unwrap();

    let resp2 = client
        .post(format!("{base}/conversations/{conv_id}/messages"))
        .bearer_auth(&token)
        .json(&payload)
        .send()
        .await
        .unwrap();

    if resp1.status() == 200 && resp2.status() == 200 {
        let body1: serde_json::Value = resp1.json().await.unwrap();
        let body2: serde_json::Value = resp2.json().await.unwrap();
        assert_eq!(body1["id"], body2["id"], "idempotent sends must return same message id");
        assert_eq!(body1["seq"], body2["seq"], "idempotent sends must return same seq");
    }
}

// ── Integration: Gap-fill via WebSocket (requires running server) ───────

#[tokio::test]
#[ignore = "requires running paw-server with auth token"]
async fn ws_connect_receives_hello_ok() {
    use futures_util::StreamExt;
    use tokio_tungstenite::connect_async;

    let token = std::env::var("PAW_TEST_TOKEN").unwrap_or_else(|_| "test_token".into());
    let url = format!("ws://localhost:3000/ws?token={token}");

    let (mut ws_stream, _) = connect_async(&url).await.expect("WS connect failed");

    if let Some(Ok(msg)) = ws_stream.next().await {
        let text = msg.to_text().unwrap();
        let frame: serde_json::Value = serde_json::from_str(text).unwrap();
        assert_eq!(frame["type"], "hello_ok");
        assert_eq!(frame["v"], 1);
        assert!(frame["user_id"].is_string());
    } else {
        panic!("expected hello_ok frame");
    }
}

#[tokio::test]
#[ignore = "requires running paw-server with auth token"]
async fn ws_sync_gap_fill() {
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::connect_async;
    use tokio_tungstenite::tungstenite::Message;

    let token = std::env::var("PAW_TEST_TOKEN").unwrap_or_else(|_| "test_token".into());
    let conv_id = std::env::var("PAW_TEST_CONV_ID")
        .unwrap_or_else(|_| "00000000-0000-0000-0000-000000000001".into());
    let url = format!("ws://localhost:3000/ws?token={token}");

    let (mut ws_stream, _) = connect_async(&url).await.expect("WS connect failed");

    // Consume hello_ok
    let _ = ws_stream.next().await;

    let sync_frame = serde_json::json!({
        "v": 1,
        "type": "sync",
        "conversation_id": conv_id,
        "last_seq": 0,
    });
    ws_stream
        .send(Message::Text(sync_frame.to_string().into()))
        .await
        .unwrap();

    // Collect message_received frames (with timeout)
    let mut received = Vec::new();
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(3);
    loop {
        tokio::select! {
            msg = ws_stream.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        let frame: serde_json::Value = serde_json::from_str(&text).unwrap_or_default();
                        if frame["type"] == "message_received" {
                            received.push(frame);
                        }
                    }
                    _ => break,
                }
            }
            _ = tokio::time::sleep_until(deadline) => break,
        }
    }

    // Verify sequential ordering
    for window in received.windows(2) {
        let seq_a = window[0]["seq"].as_i64().unwrap();
        let seq_b = window[1]["seq"].as_i64().unwrap();
        assert!(seq_b > seq_a, "gap-fill messages must be ordered by seq: {seq_a} < {seq_b}");
    }
}

// ── Integration: Health check ───────────────────────────────────────────

#[tokio::test]
#[ignore = "requires running paw-server"]
async fn health_check_returns_ok() {
    let resp = reqwest::get("http://localhost:3000/health").await.unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(resp.text().await.unwrap(), "OK");
}

fn rand_u32() -> u32 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    std::time::SystemTime::now().hash(&mut hasher);
    hasher.finish() as u32
}
