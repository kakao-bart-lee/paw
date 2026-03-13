use futures_util::{SinkExt, StreamExt};
use paw_core::{ConversationSyncCursor, MessageSyncOutcome, SyncEngine};
use serde_json::Value;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use uuid::Uuid;

fn server_url() -> String {
    std::env::var("PAW_SERVER_URL").unwrap_or_else(|_| "http://127.0.0.1:38173".to_string())
}

fn smoke_token() -> String {
    std::env::var("PAW_TEST_TOKEN").expect("PAW_TEST_TOKEN")
}

fn smoke_conversation_id() -> Uuid {
    let raw = std::env::var("PAW_TEST_CONV_ID").expect("PAW_TEST_CONV_ID");
    Uuid::parse_str(&raw).expect("valid PAW_TEST_CONV_ID")
}

#[tokio::test]
#[ignore = "requires running paw-server with PAW_EXPOSE_OTP_FOR_E2E=true"]
async fn phase3_ws_hello_ok_and_sync_gap_fill() {
    let token = smoke_token();
    let conversation_id = smoke_conversation_id();
    let ws_url = server_url()
        .replace("https://", "wss://")
        .replace("http://", "ws://");
    let url = format!("{ws_url}/ws?token={token}");

    let (mut ws_stream, _) = connect_async(&url).await.expect("ws connect");

    let hello = ws_stream
        .next()
        .await
        .expect("hello frame")
        .expect("hello ok");
    let hello_text = hello.into_text().expect("hello text");
    let hello_json: Value = serde_json::from_str(&hello_text).expect("hello json");
    assert_eq!(hello_json["type"], "hello_ok");
    assert_eq!(hello_json["v"], 1);

    let sync_frame = serde_json::json!({
        "v": 1,
        "type": "sync",
        "conversation_id": conversation_id,
        "last_seq": 0,
    });
    ws_stream
        .send(Message::Text(sync_frame.to_string()))
        .await
        .expect("send sync");

    let mut engine = SyncEngine::new([ConversationSyncCursor {
        conversation_id,
        last_seq: 0,
    }]);

    let mut applied = 0;
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(3);
    loop {
        tokio::select! {
            msg = ws_stream.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        let frame: Value = serde_json::from_str(&text).expect("frame json");
                        if frame["type"] == "message_received" {
                            let parsed = serde_json::from_value(frame).expect("message_received");
                            match engine.ingest_message(&parsed) {
                                MessageSyncOutcome::Applied { .. } => applied += 1,
                                other => panic!("unexpected sync outcome: {other:?}"),
                            }
                            if applied >= 2 {
                                break;
                            }
                        }
                    }
                    _ => break,
                }
            }
            _ = tokio::time::sleep_until(deadline) => break,
        }
    }

    assert!(applied >= 2, "expected at least two gap-fill messages");
}
