# Paw WebSocket Protocol v1

## Overview
Paw uses WebSocket as the primary transport with REST as fallback.
All messages are JSON with a mandatory `v` field (currently `1`).

## Version Field (MANDATORY)
Every single message frame MUST include `"v": 1`.
Messages without `v` are rejected by the server.

## Connection Lifecycle
1. Client connects: `ws://host/ws?token=JWT`
2. Server validates JWT → sends `hello_ok` or `hello_error`
3. Client can now send/receive messages
4. Server sends heartbeat ping every 30s
5. Client must respond with pong (WebSocket native)
6. 90s without pong → server closes with code 1000

## Message Types (Client → Server)

### connect
Sent immediately after WebSocket upgrade.
```json
{"v":1,"type":"connect","token":"<JWT>"}
```

### message_send
Send a message to a conversation.
```json
{
  "v": 1,
  "type": "message_send",
  "conversation_id": "<UUID>",
  "content": "Hello!",
  "format": "markdown",
  "blocks": [],
  "idempotency_key": "<UUID>"
}
```

### typing_start / typing_stop
```json
{"v":1,"type":"typing_start","conversation_id":"<UUID>"}
{"v":1,"type":"typing_stop","conversation_id":"<UUID>"}
```

### thread_subscribe / thread_unsubscribe
Subscribe or unsubscribe the current socket from a specific thread feed.
```json
{"v":1,"type":"thread_subscribe","conversation_id":"<UUID>","thread_id":"<UUID>"}
{"v":1,"type":"thread_unsubscribe","conversation_id":"<UUID>","thread_id":"<UUID>"}
```

### send_thread_message
Send a message into a thread-scoped feed.
```json
{
  "v": 1,
  "type": "send_thread_message",
  "conversation_id": "<UUID>",
  "thread_id": "<UUID>",
  "content": "Reply inside the thread",
  "format": "markdown",
  "blocks": [],
  "idempotency_key": "<UUID>"
}
```

### typing_thread_start / typing_thread_end
Thread-scoped typing indicators are only delivered to sockets subscribed to that thread.
```json
{"v":1,"type":"typing_thread_start","conversation_id":"<UUID>","thread_id":"<UUID>"}
{"v":1,"type":"typing_thread_end","conversation_id":"<UUID>","thread_id":"<UUID>"}
```

### message_ack
Acknowledge messages read up to seq.
```json
{"v":1,"type":"message_ack","conversation_id":"<UUID>","last_seq":42}
```

### device_sync (Phase 3)
Sync multiple conversations in a single request.
```json
{
  "v": 1,
  "type": "device_sync",
  "conversations": [
    {"conversation_id": "<UUID>", "last_seq": 42},
    {"conversation_id": "<UUID>", "last_seq": 10}
  ]
}
```

### sync
Request messages after last_seq (gap-fill on reconnect).
```json
{"v":1,"type":"sync","conversation_id":"<UUID>","last_seq":42}
```

## Message Types (Server → Client)

### hello_ok
```json
{"v":1,"type":"hello_ok","user_id":"<UUID>","server_time":"<ISO8601>"}
```

### hello_error
```json
{"v":1,"type":"hello_error","code":"invalid_token","message":"JWT expired"}
```

### message_received
```json
{
  "v": 1,
  "type": "message_received",
  "id": "<UUID>",
  "conversation_id": "<UUID>",
  "sender_id": "<UUID>",
  "content": "Hello!",
  "format": "markdown",
  "seq": 1,
  "created_at": "<ISO8601>",
  "blocks": [],
  "attachments": []
}
```

### thread_message_received
Delivered only to sockets subscribed to the target thread.
```json
{
  "v": 1,
  "type": "thread_message_received",
  "id": "<UUID>",
  "conversation_id": "<UUID>",
  "thread_id": "<UUID>",
  "sender_id": "<UUID>",
  "content": "Reply inside the thread",
  "format": "markdown",
  "seq": 3,
  "conversation_seq": 17,
  "created_at": "<ISO8601>",
  "blocks": [],
  "attachments": []
}
```

### typing_thread_start / typing_thread_end
```json
{"v":1,"type":"typing_thread_start","conversation_id":"<UUID>","thread_id":"<UUID>","user_id":"<UUID>"}
{"v":1,"type":"typing_thread_end","conversation_id":"<UUID>","thread_id":"<UUID>","user_id":"<UUID>"}
```

### typing_start / typing_stop
```json
{"v":1,"type":"typing_start","conversation_id":"<UUID>","user_id":"<UUID>"}
```

### presence_update
```json
{"v":1,"type":"presence_update","user_id":"<UUID>","online":true}
```

### device_sync_response (Phase 3)
```json
{
  "v": 1,
  "type": "device_sync_response",
  "messages": [
    {
      "v": 1,
      "type": "message_received",
      "id": "<UUID>",
      "conversation_id": "<UUID>",
      "sender_id": "<UUID>",
      "content": "Hello!",
      "format": "markdown",
      "seq": 43,
      "created_at": "<ISO8601>",
      "blocks": [],
      "attachments": []
    }
  ]
}
```

## Phase 2 Streaming Types (Reserved, Not Implemented)
These types are defined but not sent in Phase 1.

### stream_start
```json
{"v":1,"type":"stream_start","conversation_id":"<UUID>","agent_id":"<UUID>","stream_id":"<UUID>"}
```

### content_delta
```json
{"v":1,"type":"content_delta","stream_id":"<UUID>","delta":"Hello, "}
```

### tool_start
```json
{"v":1,"type":"tool_start","stream_id":"<UUID>","tool":"web_search","label":"날씨 검색 중..."}
```

### tool_end
```json
{"v":1,"type":"tool_end","stream_id":"<UUID>","tool":"web_search"}
```

### stream_end
```json
{"v":1,"type":"stream_end","stream_id":"<UUID>","tokens":156,"duration_ms":1200}
```

## Message Format
- `format`: `"markdown"` or `"plain"`
- Markdown is CommonMark + code highlighting
- NO LaTeX/Mermaid in Phase 1
- NO character limit (AI responses can be long)
- Graceful degradation: parse errors → plaintext fallback

## Rich Blocks (Phase 2)
Reserved field in Phase 1. Will contain card/button blocks.

## Error Codes
| Code | Meaning |
|------|---------|
| `invalid_token` | JWT invalid/expired |
| `unauthorized` | Not a conversation member |
| `rate_limited` | Too many requests |
| `message_too_large` | Content > 64KB |

## Reconnection Protocol
1. Client disconnects
2. Client reconnects with JWT
3. Client sends `sync` messages for each conversation with `last_seq`
4. Server responds with all messages after `last_seq`
5. Client resumes from where it left off

## Gap-fill Algorithm
```
for each conversation in local_db:
  last_seq = local_db.get_last_seq(conversation_id)
  send: {"type":"sync","conversation_id":conv_id,"last_seq":last_seq}

receive: batch of message_received frames with seq > last_seq
```
