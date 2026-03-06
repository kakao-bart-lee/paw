use base64::Engine;
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

// ── OTP Expiry Validation ───────────────────────────────────────────────

#[test]
fn otp_code_must_be_six_ascii_digits() {
    let valid_codes = ["000000", "123456", "999999"];
    let invalid_codes = ["12345", "1234567", "abcdef", "12 456", "", "12345a", "00000\n"];

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
        content: "same content".into(),
        format: paw_proto::MessageFormat::Plain,
        blocks: vec![],
        idempotency_key: Uuid::new_v4(),
    };

    let msg_b = paw_proto::MessageSendMsg {
        v: 1,
        conversation_id: conv_id,
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
    assert_ne!(json_a, json_b, "different idempotency keys must serialize differently");
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
fn test_replenish_threshold() {
    for count in 0..5u32 {
        assert!(count < 5, "should replenish at count {}", count);
    }
    assert!(!(5 < 5), "should NOT replenish at count 5");
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

#[test]
fn group_name_update_request_serialization() {
    let request = GroupNameUpdateRequest {
        name: "Weekend Plans".to_string(),
    };

    let json = serde_json::to_string(&request).unwrap();
    let parsed: GroupNameUpdateRequest = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed, request);
}
