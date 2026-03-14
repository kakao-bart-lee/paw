use futures_util::{SinkExt, StreamExt};
use paw_core::{ConversationSyncCursor, MessageSyncOutcome, SyncEngine};
use paw_proto::MessageReceivedMsg;
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

async fn connect_and_expect_hello_ok(
    token: &str,
) -> tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>> {
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

    ws_stream
}

async fn request_sync_and_collect(
    ws_stream: &mut tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    conversation_id: Uuid,
    last_seq: i64,
    engine: &mut SyncEngine,
    wait_for: std::time::Duration,
) -> usize {
    let sync_frame = serde_json::json!({
        "v": 1,
        "type": "sync",
        "conversation_id": conversation_id,
        "last_seq": last_seq,
    });
    ws_stream
        .send(Message::Text(sync_frame.to_string()))
        .await
        .expect("send sync");

    let mut applied = 0;
    let deadline = tokio::time::Instant::now() + wait_for;
    loop {
        tokio::select! {
            msg = ws_stream.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        let frame: Value = serde_json::from_str(&text).expect("frame json");
                        if frame["type"] == "message_received" {
                            let parsed: MessageReceivedMsg =
                                serde_json::from_value(frame).expect("message_received");
                            match engine.ingest_message(&parsed) {
                                MessageSyncOutcome::Applied { .. } => applied += 1,
                                MessageSyncOutcome::DuplicateOrStale { .. } => {}
                                other => panic!("unexpected sync outcome: {other:?}"),
                            }
                        }
                    }
                    Some(Ok(_)) => {}
                    Some(Err(error)) => panic!("ws frame error: {error}"),
                    None => break,
                }
            }
            _ = tokio::time::sleep_until(deadline) => break,
        }
    }

    applied
}

#[tokio::test]
#[ignore = "requires running paw-server with PAW_EXPOSE_OTP_FOR_E2E=true"]
async fn phase3_ws_hello_ok_and_sync_gap_fill() {
    let token = smoke_token();
    let conversation_id = smoke_conversation_id();
    let mut ws_stream = connect_and_expect_hello_ok(&token).await;

    let mut first_engine = SyncEngine::new([ConversationSyncCursor {
        conversation_id,
        last_seq: 0,
    }]);
    let first_applied = request_sync_and_collect(
        &mut ws_stream,
        conversation_id,
        0,
        &mut first_engine,
        std::time::Duration::from_secs(3),
    )
    .await;
    assert!(
        first_applied >= 2,
        "expected at least two gap-fill messages on initial sync"
    );

    ws_stream.close(None).await.expect("close first ws");

    let last_seq = first_engine.last_seq(conversation_id);
    assert!(
        last_seq >= 2,
        "expected reconnect cursor to advance after initial gap fill"
    );

    let mut reconnect_engine = SyncEngine::new([ConversationSyncCursor {
        conversation_id,
        last_seq,
    }]);
    let mut reconnect_stream = connect_and_expect_hello_ok(&token).await;
    let reconnect_applied = request_sync_and_collect(
        &mut reconnect_stream,
        conversation_id,
        last_seq,
        &mut reconnect_engine,
        std::time::Duration::from_secs(1),
    )
    .await;

    assert_eq!(
        reconnect_applied, 0,
        "reconnect sync should not replay already-acked gap-fill messages"
    );
    assert_eq!(
        reconnect_engine.last_seq(conversation_id),
        last_seq,
        "reconnect sync should preserve the latest cursor when no new messages exist"
    );
}
