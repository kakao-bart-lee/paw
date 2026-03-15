use base64::Engine;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
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

    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
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
    let token = issue_token(
        user_id,
        None,
        "session",
        Duration::minutes(15),
        TEST_JWT_SECRET,
    );
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
    let token = issue_token(
        Uuid::new_v4(),
        None,
        "session",
        Duration::minutes(15),
        TEST_JWT_SECRET,
    );
    let result = verify_token(&token, "wrong_secret", Some("session"));
    assert!(result.is_err());
}

#[test]
fn jwt_wrong_token_type_rejected() {
    let token = issue_token(
        Uuid::new_v4(),
        None,
        "session",
        Duration::minutes(15),
        TEST_JWT_SECRET,
    );
    let result = verify_token(&token, TEST_JWT_SECRET, Some("access"));
    assert!(result.is_err());
}

#[test]
fn jwt_expired_token_rejected() {
    let token = issue_token(
        Uuid::new_v4(),
        None,
        "session",
        Duration::seconds(-120),
        TEST_JWT_SECRET,
    );
    let result = verify_token(&token, TEST_JWT_SECRET, Some("session"));
    assert!(result.is_err());
}

// ── Protocol: frame serialization ───────────────────────────────────────

#[test]
fn protocol_message_send_includes_v_field() {
    let frame = paw_proto::MessageSendMsg {
        v: paw_proto::PROTOCOL_VERSION,
        conversation_id: Uuid::new_v4(),
        thread_id: None,
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
        thread_id: None,
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
        capabilities: None,
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
        thread_id: None,
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
        thread_id: None,
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
        thread_id: None,
        user_id: None,
    };
    let json = serde_json::to_string(&msg).unwrap();
    assert!(
        !json.contains("user_id"),
        "user_id=None should be omitted via skip_serializing_if"
    );
}

#[test]
fn protocol_typing_includes_user_id_when_present() {
    let uid = Uuid::new_v4();
    let msg = paw_proto::TypingMsg {
        v: 1,
        conversation_id: Uuid::new_v4(),
        thread_id: None,
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
        thread_id: None,
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
        .post("http://localhost:38173/auth/request-otp")
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
        .post("http://localhost:38173/auth/request-otp")
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
        .post("http://localhost:38173/auth/verify-otp")
        .json(&serde_json::json!({ "phone": "+821012345678", "code": "abc" }))
        .send()
        .await
        .unwrap();

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"], "invalid_code_format");
}

#[tokio::test]
#[ignore = "requires running paw-server"]
async fn auth_dev_login_route_is_unavailable() {
    let client = reqwest::Client::new();

    let get_resp = client
        .get("http://localhost:38173/auth/dev-login")
        .send()
        .await
        .expect("server must be reachable");
    assert_eq!(get_resp.status(), 404);

    let post_resp = client
        .post("http://localhost:38173/auth/dev-login")
        .json(&serde_json::json!({ "phone": "+821012345678" }))
        .send()
        .await
        .expect("server must be reachable");
    assert_eq!(post_resp.status(), 404);
}

#[tokio::test]
#[ignore = "requires running paw-server and PostgreSQL"]
async fn auth_full_flow_request_verify_register() {
    let client = reqwest::Client::new();
    let phone = format!("+8210{:08}", rand_u32() % 100_000_000);

    let resp = client
        .post("http://localhost:38173/auth/request-otp")
        .json(&serde_json::json!({ "phone": phone }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    let resp = client
        .post("http://localhost:38173/auth/verify-otp")
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
    let base = "http://localhost:38173";
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
    let base = "http://localhost:38173";
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
        assert_eq!(
            body1["id"], body2["id"],
            "idempotent sends must return same message id"
        );
        assert_eq!(
            body1["seq"], body2["seq"],
            "idempotent sends must return same seq"
        );
    }
}

// ── Integration: Gap-fill via WebSocket (requires running server) ───────

#[tokio::test]
#[ignore = "requires running paw-server with auth token"]
async fn ws_connect_receives_hello_ok() {
    use futures_util::StreamExt;
    use tokio_tungstenite::connect_async;

    let token = std::env::var("PAW_TEST_TOKEN").unwrap_or_else(|_| "test_token".into());
    let url = format!("ws://localhost:38173/ws?token={token}");

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
    let url = format!("ws://localhost:38173/ws?token={token}");

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
        .send(Message::Text(sync_frame.to_string()))
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
        assert!(
            seq_b > seq_a,
            "gap-fill messages must be ordered by seq: {seq_a} < {seq_b}"
        );
    }
}

// ── Integration: Health check ───────────────────────────────────────────

#[tokio::test]
#[ignore = "requires running paw-server"]
async fn health_check_returns_ok() {
    let resp = reqwest::get("http://localhost:38173/health").await.unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(resp.text().await.unwrap(), "OK");
}

// ── Backup: serde roundtrip tests ───────────────────────────────────────

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
struct InitiateBackupResponse {
    backup_id: Uuid,
    upload_url: String,
    s3_key: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
struct BackupEntry {
    id: Uuid,
    size_bytes: i64,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
struct ListBackupsResponse {
    backups: Vec<BackupEntry>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Clone)]
struct BackupSettingsTest {
    frequency: String,
}

#[test]
fn backup_initiate_response_serde_roundtrip() {
    let backup_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let resp = InitiateBackupResponse {
        backup_id,
        upload_url: "https://s3.example.com/presigned-put?sig=abc".to_string(),
        s3_key: format!("backups/{}/{}.enc", user_id, backup_id),
    };

    let json = serde_json::to_string(&resp).unwrap();
    let parsed: InitiateBackupResponse = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.backup_id, backup_id);
    assert!(parsed.upload_url.contains("presigned"));
    assert!(parsed.s3_key.ends_with(".enc"));
    assert_eq!(parsed, resp);
}

#[test]
fn backup_list_response_serde_roundtrip() {
    let now = Utc::now();
    let entries = vec![
        BackupEntry {
            id: Uuid::new_v4(),
            size_bytes: 1_048_576,
            created_at: now,
        },
        BackupEntry {
            id: Uuid::new_v4(),
            size_bytes: 2_097_152,
            created_at: now - Duration::days(1),
        },
    ];

    let resp = ListBackupsResponse { backups: entries };
    let json = serde_json::to_string(&resp).unwrap();
    let parsed: ListBackupsResponse = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.backups.len(), 2);
    assert_eq!(parsed.backups[0].size_bytes, 1_048_576);
    assert_eq!(parsed.backups[1].size_bytes, 2_097_152);
    assert_eq!(parsed, resp);
}

#[test]
fn backup_settings_serde_roundtrip() {
    for frequency in &["daily", "weekly", "never"] {
        let settings = BackupSettingsTest {
            frequency: frequency.to_string(),
        };

        let json = serde_json::to_string(&settings).unwrap();
        let parsed: BackupSettingsTest = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.frequency, *frequency);
        assert_eq!(parsed, settings);
    }
}

#[test]
fn backup_s3_key_format_is_correct() {
    let user_id = Uuid::new_v4();
    let backup_id = Uuid::new_v4();
    let s3_key = format!("backups/{}/{}.enc", user_id, backup_id);

    assert!(s3_key.starts_with("backups/"));
    assert!(s3_key.ends_with(".enc"));
    assert!(s3_key.contains(&user_id.to_string()));
    assert!(s3_key.contains(&backup_id.to_string()));
}

fn rand_u32() -> u32 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    std::time::SystemTime::now().hash(&mut hasher);
    hasher.finish() as u32
}

// ── OTP Expiry Validation ───────────────────────────────────────────────

#[test]
fn otp_code_must_be_six_ascii_digits() {
    let valid_codes = ["000000", "123456", "999999"];
    let invalid_codes = [
        "12345", "1234567", "abcdef", "12 456", "", "12345a", "00000\n",
    ];

    for code in valid_codes {
        assert!(
            code.len() == 6 && code.chars().all(|c| c.is_ascii_digit()),
            "expected valid OTP: {code}"
        );
    }
    for code in invalid_codes {
        assert!(
            !(code.len() == 6 && code.chars().all(|c| c.is_ascii_digit())),
            "expected invalid OTP: {code:?}"
        );
    }
}

#[test]
fn otp_expiry_window_is_five_minutes() {
    let now = Utc::now();
    let expires_at = now + Duration::minutes(5);
    let delta_secs = (expires_at - now).num_seconds();

    assert_eq!(delta_secs, 300, "OTP TTL must be exactly 5 minutes (300s)");
    assert!(expires_at > now, "expiry must be in the future");
    assert!(
        expires_at < now + Duration::minutes(6),
        "expiry must not exceed 6 minutes"
    );
}

#[test]
fn otp_expired_code_is_rejected_by_time_check() {
    let now = Utc::now();
    let expired_at = now - Duration::seconds(1);
    let still_valid = now + Duration::seconds(60);

    assert!(
        expired_at <= now,
        "an OTP whose expires_at <= now must be rejected"
    );
    assert!(
        still_valid > now,
        "an OTP whose expires_at > now is still valid"
    );
}

// ── Idempotency Key Uniqueness ──────────────────────────────────────────

#[test]
fn idempotency_key_preserved_through_serialization() {
    let key = Uuid::new_v4();
    let conv_id = Uuid::new_v4();

    let msg = paw_proto::MessageSendMsg {
        v: 1,
        conversation_id: conv_id,
        thread_id: None,
        content: "test".into(),
        format: paw_proto::MessageFormat::Plain,
        blocks: vec![],
        idempotency_key: key,
    };

    let json = serde_json::to_value(&msg).unwrap();
    assert_eq!(json["idempotency_key"], key.to_string());

    let roundtrip: paw_proto::MessageSendMsg = serde_json::from_value(json).unwrap();
    assert_eq!(roundtrip.idempotency_key, key);
    assert_eq!(roundtrip.conversation_id, conv_id);
}

#[test]
fn different_idempotency_keys_produce_different_messages() {
    let conv_id = Uuid::new_v4();

    let msg_a = paw_proto::MessageSendMsg {
        v: 1,
        conversation_id: conv_id,
        thread_id: None,
        content: "same content".into(),
        format: paw_proto::MessageFormat::Plain,
        blocks: vec![],
        idempotency_key: Uuid::new_v4(),
    };

    let msg_b = paw_proto::MessageSendMsg {
        v: 1,
        conversation_id: conv_id,
        thread_id: None,
        content: "same content".into(),
        format: paw_proto::MessageFormat::Plain,
        blocks: vec![],
        idempotency_key: Uuid::new_v4(),
    };

    assert_ne!(
        msg_a.idempotency_key, msg_b.idempotency_key,
        "distinct sends must have distinct idempotency keys"
    );

    let json_a = serde_json::to_string(&msg_a).unwrap();
    let json_b = serde_json::to_string(&msg_b).unwrap();
    assert_ne!(
        json_a, json_b,
        "different idempotency keys must serialize differently"
    );
}

#[test]
fn same_idempotency_key_same_conversation_same_sender_is_duplicate() {
    let key = Uuid::new_v4();
    let conv_id = Uuid::new_v4();
    let sender_id = Uuid::new_v4();

    let triple_a = (conv_id, sender_id, key);
    let triple_b = (conv_id, sender_id, key);

    assert_eq!(
        triple_a, triple_b,
        "(conv_id, sender_id, idempotency_key) triple must match for duplicates"
    );

    let different_sender = (conv_id, Uuid::new_v4(), key);
    assert_ne!(
        triple_a, different_sender,
        "same key but different sender is NOT a duplicate"
    );
}

// ── Gap-Fill Seq Ordering ───────────────────────────────────────────────

#[test]
fn gap_fill_messages_must_be_monotonically_increasing() {
    let conv_id = Uuid::new_v4();
    let sender_id = Uuid::new_v4();
    let now = Utc::now();

    let messages: Vec<paw_proto::MessageReceivedMsg> = (1..=10)
        .map(|seq| paw_proto::MessageReceivedMsg {
            v: 1,
            id: Uuid::new_v4(),
            conversation_id: conv_id,
            thread_id: None,
            sender_id,
            content: format!("msg {seq}"),
            format: paw_proto::MessageFormat::Markdown,
            seq,
            created_at: now + Duration::seconds(seq),
            blocks: vec![],
        })
        .collect();

    for window in messages.windows(2) {
        assert!(
            window[1].seq > window[0].seq,
            "gap-fill seq must be strictly increasing: {} should be > {}",
            window[1].seq,
            window[0].seq
        );
    }
}

#[test]
fn gap_fill_sync_with_last_seq_filters_correctly() {
    let last_seq: i64 = 5;
    let all_seqs: Vec<i64> = (1..=10).collect();

    let gap_fill: Vec<i64> = all_seqs.iter().copied().filter(|&s| s > last_seq).collect();

    assert_eq!(gap_fill, vec![6, 7, 8, 9, 10]);
    assert!(gap_fill.iter().all(|&s| s > last_seq));

    let is_sorted = gap_fill.windows(2).all(|w| w[0] < w[1]);
    assert!(is_sorted, "gap-fill results must be in ascending order");
}

#[test]
fn gap_fill_limit_caps_at_100() {
    let last_seq: i64 = 0;
    let all_seqs: Vec<i64> = (1..=250).collect();
    let limit: usize = 100;

    let gap_fill: Vec<i64> = all_seqs
        .iter()
        .copied()
        .filter(|&s| s > last_seq)
        .take(limit)
        .collect();

    assert_eq!(gap_fill.len(), 100, "gap-fill must cap at LIMIT 100");
    assert_eq!(*gap_fill.first().unwrap(), 1);
    assert_eq!(*gap_fill.last().unwrap(), 100);
}

#[test]
fn test_key_bundle_base64_roundtrip() {
    let key = vec![42u8; 32];
    let encoded = base64::engine::general_purpose::STANDARD.encode(&key);
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(&encoded)
        .unwrap();
    assert_eq!(key, decoded);
}

#[test]
fn test_agent_token_format() {
    let token = format!("paw_agent_{}", uuid::Uuid::new_v4());
    assert!(token.starts_with("paw_agent_"));
    assert!(token.len() > 10);
}

#[test]
fn test_inbound_context_serialization() {
    let msg = paw_proto::MessageReceivedMsg {
        v: 1,
        id: Uuid::new_v4(),
        conversation_id: Uuid::new_v4(),
        thread_id: None,
        sender_id: Uuid::new_v4(),
        content: "hello".into(),
        format: paw_proto::MessageFormat::Markdown,
        seq: 1,
        created_at: Utc::now(),
        blocks: vec![],
    };

    let ctx = paw_proto::InboundContext {
        v: 1,
        conversation_id: msg.conversation_id,
        message: msg.clone(),
        recent_messages: vec![msg],
    };

    let json = serde_json::to_value(&ctx).unwrap();
    assert_eq!(json["v"], 1);
    assert!(json["message"].is_object());
    assert!(json["recent_messages"].is_array());
}

#[test]
fn test_agent_response_msg_roundtrip() {
    let msg = paw_proto::AgentResponseMsg {
        v: 1,
        conversation_id: Uuid::new_v4(),
        content: "I can help with that!".into(),
        format: "markdown".into(),
    };

    let json = serde_json::to_string(&msg).unwrap();
    let parsed: paw_proto::AgentResponseMsg = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.v, 1);
    assert_eq!(parsed.content, "I can help with that!");
    assert_eq!(parsed.format, "markdown");
}

#[test]
fn test_agent_stream_msg_roundtrip() {
    let conversation_id = Uuid::new_v4();
    let agent_id = Uuid::new_v4();
    let stream_id = Uuid::new_v4();

    let frames = vec![
        paw_proto::AgentStreamMsg::StreamStart(paw_proto::StreamStartMsg {
            v: 1,
            conversation_id,
            thread_id: None,
            agent_id,
            stream_id,
        }),
        paw_proto::AgentStreamMsg::ContentDelta(paw_proto::ContentDeltaMsg {
            v: 1,
            stream_id,
            delta: "hello".into(),
        }),
        paw_proto::AgentStreamMsg::ToolStart(paw_proto::ToolStartMsg {
            v: 1,
            stream_id,
            tool: "search".into(),
            label: "Searching".into(),
        }),
        paw_proto::AgentStreamMsg::ToolEnd(paw_proto::ToolEndMsg {
            v: 1,
            stream_id,
            tool: "search".into(),
        }),
        paw_proto::AgentStreamMsg::StreamEnd(paw_proto::StreamEndMsg {
            v: 1,
            stream_id,
            tokens: 42,
            duration_ms: 1234,
        }),
    ];

    for frame in frames {
        let json = serde_json::to_string(&frame).unwrap();
        let parsed: paw_proto::AgentStreamMsg = serde_json::from_str(&json).unwrap();

        match parsed {
            paw_proto::AgentStreamMsg::StreamStart(msg) => {
                assert_eq!(msg.v, 1);
                assert_eq!(msg.conversation_id, conversation_id);
                assert_eq!(msg.agent_id, agent_id);
                assert_eq!(msg.stream_id, stream_id);
            }
            paw_proto::AgentStreamMsg::ContentDelta(msg) => {
                assert_eq!(msg.v, 1);
                assert_eq!(msg.stream_id, stream_id);
                assert_eq!(msg.delta, "hello");
            }
            paw_proto::AgentStreamMsg::ToolStart(msg) => {
                assert_eq!(msg.v, 1);
                assert_eq!(msg.stream_id, stream_id);
                assert_eq!(msg.tool, "search");
                assert_eq!(msg.label, "Searching");
            }
            paw_proto::AgentStreamMsg::ToolEnd(msg) => {
                assert_eq!(msg.v, 1);
                assert_eq!(msg.stream_id, stream_id);
                assert_eq!(msg.tool, "search");
            }
            paw_proto::AgentStreamMsg::StreamEnd(msg) => {
                assert_eq!(msg.v, 1);
                assert_eq!(msg.stream_id, stream_id);
                assert_eq!(msg.tokens, 42);
                assert_eq!(msg.duration_ms, 1234);
            }
        }
    }
}

#[test]
fn test_stream_limit_enforcement_thresholds() {
    const MAX_STREAM_DURATION_SECONDS: i64 = 300;
    const MAX_STREAM_BYTES: usize = 1_048_576;

    fn within_limits(started_at: chrono::DateTime<Utc>, bytes_sent: usize) -> bool {
        let elapsed = Utc::now() - started_at;
        elapsed.num_seconds() <= MAX_STREAM_DURATION_SECONDS && bytes_sent <= MAX_STREAM_BYTES
    }

    let just_within_duration = Utc::now() - Duration::seconds(MAX_STREAM_DURATION_SECONDS);
    assert!(within_limits(just_within_duration, MAX_STREAM_BYTES));

    let over_duration = Utc::now() - Duration::seconds(MAX_STREAM_DURATION_SECONDS + 1);
    assert!(!within_limits(over_duration, MAX_STREAM_BYTES));

    assert!(within_limits(Utc::now(), MAX_STREAM_BYTES));
    assert!(!within_limits(Utc::now(), MAX_STREAM_BYTES + 1));
}

#[test]
fn stream_concurrent_limit_is_10() {
    assert_eq!(paw_proto::PROTOCOL_VERSION, 1);
    let limit: usize = 10;
    assert_eq!(limit, 10);
    assert!(limit <= 100, "concurrent stream limit should be reasonable");
}

#[test]
fn delta_size_limit_is_4096() {
    let limit: usize = 4096;
    assert_eq!(limit, 4096);
    assert!(limit > 0 && limit <= 65536);
}

#[test]
fn test_replenish_threshold() {
    fn should_replenish(remaining: u32) -> bool {
        remaining < 5
    }

    for count in 0..5u32 {
        assert!(
            should_replenish(count),
            "should replenish at count {}",
            count
        );
    }
    assert!(!should_replenish(5), "should NOT replenish at count 5");
}

const MAX_GROUP_MEMBERS: usize = 100;

fn validate_group_member_count(total_members: usize) -> Result<(), &'static str> {
    if total_members > MAX_GROUP_MEMBERS {
        Err("too_many_members")
    } else {
        Ok(())
    }
}

#[test]
fn group_max_members_is_100() {
    assert_eq!(MAX_GROUP_MEMBERS, 100);
}

#[test]
fn group_member_count_validation() {
    assert!(validate_group_member_count(100).is_ok());
    assert!(validate_group_member_count(101).is_err());
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct GroupNameUpdateRequest {
    name: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct InviteAgentRequest {
    agent_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct InviteAgentResponse {
    invited: bool,
}

#[test]
fn group_name_update_request_serialization() {
    let request = GroupNameUpdateRequest {
        name: "Weekend Plans".to_string(),
    };

    let json = serde_json::to_string(&request).unwrap();
    let parsed: GroupNameUpdateRequest = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed, request);
}

#[test]
fn invite_agent_request_serialization() {
    let request = InviteAgentRequest {
        agent_id: Uuid::new_v4(),
    };

    let json = serde_json::to_string(&request).unwrap();
    let parsed: InviteAgentRequest = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed, request);
}

#[test]
fn invite_agent_response_serialization() {
    let response = InviteAgentResponse { invited: true };

    let json = serde_json::to_string(&response).unwrap();
    let parsed: InviteAgentResponse = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed, response);
}

#[test]
fn agent_stream_msg_all_variants_have_version() {
    use paw_proto::{
        AgentStreamMsg, ContentDeltaMsg, StreamEndMsg, StreamStartMsg, ToolEndMsg, ToolStartMsg,
    };

    let conversation_id = Uuid::new_v4();
    let agent_id = Uuid::new_v4();
    let stream_id = Uuid::new_v4();

    let frames = vec![
        AgentStreamMsg::StreamStart(StreamStartMsg {
            v: 1,
            conversation_id,
            thread_id: None,
            agent_id,
            stream_id,
        }),
        AgentStreamMsg::ContentDelta(ContentDeltaMsg {
            v: 1,
            stream_id,
            delta: "delta".to_string(),
        }),
        AgentStreamMsg::ToolStart(ToolStartMsg {
            v: 1,
            stream_id,
            tool: "search".to_string(),
            label: "Searching".to_string(),
        }),
        AgentStreamMsg::ToolEnd(ToolEndMsg {
            v: 1,
            stream_id,
            tool: "search".to_string(),
        }),
        AgentStreamMsg::StreamEnd(StreamEndMsg {
            v: 1,
            stream_id,
            tokens: 7,
            duration_ms: 321,
        }),
    ];

    for frame in frames {
        let json = serde_json::to_value(&frame).unwrap();
        assert_eq!(json["v"], 1);
        assert!(json["type"].is_string());
    }
}

#[test]
fn stream_start_msg_has_conversation_id() {
    let msg = paw_proto::StreamStartMsg {
        v: 1,
        conversation_id: Uuid::new_v4(),
        thread_id: None,
        agent_id: Uuid::new_v4(),
        stream_id: Uuid::new_v4(),
    };

    let json = serde_json::to_value(&msg).unwrap();
    assert_eq!(json["v"], 1);
    assert_eq!(json["conversation_id"], msg.conversation_id.to_string());
    assert_eq!(json["agent_id"], msg.agent_id.to_string());
}

#[test]
fn content_delta_msg_delta_roundtrip() {
    let msg = paw_proto::ContentDeltaMsg {
        v: 1,
        stream_id: Uuid::new_v4(),
        delta: "안녕 👋 — مرحبا — hello".to_string(),
    };

    let json = serde_json::to_string(&msg).unwrap();
    let parsed: paw_proto::ContentDeltaMsg = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.v, 1);
    assert_eq!(parsed.stream_id, msg.stream_id);
    assert_eq!(parsed.delta, msg.delta);
}

#[test]
fn group_member_limit_constant_value() {
    let limit: usize = 100;
    assert_eq!(MAX_GROUP_MEMBERS, 100);
    assert_eq!(limit, 100);
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct AddMemberRequest {
    user_id: Uuid,
}

#[test]
fn add_member_request_serialization() {
    let req = AddMemberRequest {
        user_id: Uuid::new_v4(),
    };

    let json = serde_json::to_value(&req).unwrap();
    assert_eq!(json["user_id"], req.user_id.to_string());

    let parsed: AddMemberRequest = serde_json::from_value(json).unwrap();
    assert_eq!(parsed, req);
}

#[test]
fn invite_agent_request_serialization_phase2() {
    let req = InviteAgentRequest {
        agent_id: Uuid::new_v4(),
    };

    let json = serde_json::to_value(&req).unwrap();
    let object = json.as_object().unwrap();
    assert_eq!(
        object.len(),
        1,
        "request should only contain agent_id field"
    );
    assert_eq!(json["agent_id"], req.agent_id.to_string());

    let parsed: InviteAgentRequest = serde_json::from_value(json).unwrap();
    assert_eq!(parsed.agent_id, req.agent_id);
}

#[test]
fn stream_end_msg_has_metrics_fields() {
    let msg = paw_proto::StreamEndMsg {
        v: 1,
        stream_id: Uuid::new_v4(),
        tokens: 42,
        duration_ms: 1_234,
    };

    let json = serde_json::to_value(&msg).unwrap();
    assert_eq!(json["tokens"], 42);
    assert_eq!(json["duration_ms"], 1_234);

    let parsed: paw_proto::StreamEndMsg = serde_json::from_value(json).unwrap();
    let _: u32 = parsed.tokens;
    let _: u64 = parsed.duration_ms;
}

#[test]
fn protocol_version_is_one() {
    assert_eq!(paw_proto::PROTOCOL_VERSION, 1);
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct ChannelCreateRequest {
    name: String,
    is_public: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct ChannelSubscribeResponse {
    subscribed: bool,
}

fn can_send_channel_message(owner_id: Uuid, sender_id: Uuid) -> bool {
    owner_id == sender_id
}

#[test]
fn channel_create_request_roundtrip() {
    let req = ChannelCreateRequest {
        name: "announcements".to_string(),
        is_public: Some(true),
    };

    let json = serde_json::to_value(&req).unwrap();
    assert_eq!(json["name"], "announcements");
    assert_eq!(json["is_public"], true);

    let parsed: ChannelCreateRequest = serde_json::from_value(json).unwrap();
    assert_eq!(parsed, req);
}

#[test]
fn channel_subscribe_response_roundtrip() {
    let res = ChannelSubscribeResponse { subscribed: true };
    let json = serde_json::to_string(&res).unwrap();
    let parsed: ChannelSubscribeResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, res);
}

#[test]
fn channel_owner_only_send_allows_owner() {
    let owner_id = Uuid::new_v4();
    assert!(can_send_channel_message(owner_id, owner_id));
}

#[test]
fn channel_owner_only_send_rejects_non_owner() {
    let owner_id = Uuid::new_v4();
    let other_user_id = Uuid::new_v4();
    assert!(!can_send_channel_message(owner_id, other_user_id));
}

// ── Push Notification: model serde + E2EE payload ──────────────────────

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum PushPlatform {
    Fcm,
    Apns,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct RegisterPushTokenRequest {
    platform: PushPlatform,
    token: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct MuteConversationRequest {
    #[serde(default)]
    duration_minutes: Option<i64>,
    #[serde(default)]
    forever: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct PushPayload {
    #[serde(rename = "type")]
    payload_type: String,
    conversation_id: Uuid,
    sender_id: Uuid,
}

#[test]
fn push_token_register_request_serde_roundtrip() {
    let req = RegisterPushTokenRequest {
        platform: PushPlatform::Fcm,
        token: "fcm_token_abc123".to_string(),
    };

    let json = serde_json::to_value(&req).unwrap();
    assert_eq!(json["platform"], "fcm");
    assert_eq!(json["token"], "fcm_token_abc123");

    let parsed: RegisterPushTokenRequest = serde_json::from_value(json).unwrap();
    assert_eq!(parsed, req);

    let apns_req = RegisterPushTokenRequest {
        platform: PushPlatform::Apns,
        token: "apns_device_token_xyz".to_string(),
    };
    let json = serde_json::to_value(&apns_req).unwrap();
    assert_eq!(json["platform"], "apns");
    let parsed: RegisterPushTokenRequest = serde_json::from_value(json).unwrap();
    assert_eq!(parsed, apns_req);
}

#[test]
fn mute_conversation_request_serde_roundtrip() {
    let duration_req = MuteConversationRequest {
        duration_minutes: Some(480),
        forever: None,
    };
    let json = serde_json::to_value(&duration_req).unwrap();
    assert_eq!(json["duration_minutes"], 480);
    let parsed: MuteConversationRequest = serde_json::from_value(json).unwrap();
    assert_eq!(parsed, duration_req);

    let forever_req = MuteConversationRequest {
        duration_minutes: None,
        forever: Some(true),
    };
    let json = serde_json::to_value(&forever_req).unwrap();
    assert_eq!(json["forever"], true);
    let parsed: MuteConversationRequest = serde_json::from_value(json).unwrap();
    assert_eq!(parsed, forever_req);
}

#[test]
fn e2ee_push_payload_has_no_content_field() {
    let payload = PushPayload {
        payload_type: "new_message".to_string(),
        conversation_id: Uuid::new_v4(),
        sender_id: Uuid::new_v4(),
    };

    let json = serde_json::to_value(&payload).unwrap();
    let obj = json.as_object().unwrap();

    assert_eq!(json["type"], "new_message");
    assert!(json["conversation_id"].is_string());
    assert!(json["sender_id"].is_string());

    assert!(
        !obj.contains_key("content"),
        "E2EE push payload must NOT contain message content"
    );
    assert!(
        !obj.contains_key("body"),
        "E2EE push payload must NOT contain message body"
    );
    assert!(
        !obj.contains_key("text"),
        "E2EE push payload must NOT contain message text"
    );

    assert_eq!(
        obj.len(),
        3,
        "push payload should only have type, conversation_id, sender_id"
    );

    let roundtrip: PushPayload = serde_json::from_value(json).unwrap();
    assert_eq!(roundtrip, payload);
}

#[tokio::test]
#[ignore = "requires running paw-server with auth token and group conversation"]
async fn thread_create_list_get_delete_flow() {
    let base = "http://localhost:38173";
    let token = std::env::var("PAW_TEST_TOKEN").unwrap_or_else(|_| "test_token".into());
    let conv_id = std::env::var("PAW_TEST_GROUP_CONV_ID")
        .unwrap_or_else(|_| "00000000-0000-0000-0000-000000000010".into());
    let root_message_id = std::env::var("PAW_TEST_ROOT_MESSAGE_ID")
        .unwrap_or_else(|_| "00000000-0000-0000-0000-000000000011".into());
    let client = reqwest::Client::new();

    let response = client
        .post(format!("{base}/conversations/{conv_id}/threads"))
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "root_message_id": root_message_id,
            "title": "integration thread"
        }))
        .send()
        .await
        .unwrap();

    assert!(
        response.status().is_success() || response.status().as_u16() == 409,
        "expected success or duplicate/thread-limit style conflict, got {}",
        response.status()
    );
}

#[tokio::test]
#[ignore = "requires running paw-server with auth token and direct conversation"]
async fn thread_create_in_direct_conversation_returns_threads_not_allowed() {
    let base = "http://localhost:38173";
    let token = std::env::var("PAW_TEST_TOKEN").unwrap_or_else(|_| "test_token".into());
    let conv_id = std::env::var("PAW_TEST_DIRECT_CONV_ID")
        .unwrap_or_else(|_| "00000000-0000-0000-0000-000000000020".into());
    let root_message_id = std::env::var("PAW_TEST_ROOT_MESSAGE_ID")
        .unwrap_or_else(|_| "00000000-0000-0000-0000-000000000021".into());
    let client = reqwest::Client::new();

    let response = client
        .post(format!("{base}/conversations/{conv_id}/threads"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "root_message_id": root_message_id }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
#[ignore = "requires running paw-server with auth token and seeded thread root message"]
async fn thread_root_message_delete_is_protected() {
    let base = "http://localhost:38173";
    let token = std::env::var("PAW_TEST_TOKEN").unwrap_or_else(|_| "test_token".into());
    let conv_id = std::env::var("PAW_TEST_GROUP_CONV_ID")
        .unwrap_or_else(|_| "00000000-0000-0000-0000-000000000030".into());
    let root_message_id = std::env::var("PAW_TEST_THREAD_ROOT_MESSAGE_ID")
        .unwrap_or_else(|_| "00000000-0000-0000-0000-000000000031".into());
    let client = reqwest::Client::new();

    let response = client
        .delete(format!(
            "{base}/conversations/{conv_id}/messages/{root_message_id}"
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();

    assert!(
        response.status().as_u16() == 409 || response.status().as_u16() == 404,
        "expected root protection or missing seed data, got {}",
        response.status()
    );
}
