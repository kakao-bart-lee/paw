---
name: protocol-designer
description: Design paw-proto WebSocket message types with backward compatibility
origin: Paw
---

# Protocol Designer Skill

The Paw WebSocket protocol is defined in `paw-proto/src/lib.rs`. Every frame
is a JSON object with a `"type"` discriminator and a mandatory `"v": 1`
version field.

## Core Enums

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage { ... }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage { ... }
```

### Client -> Server Messages

| Variant | Struct | Purpose |
|---------|--------|---------|
| `Connect` | `ConnectMsg` | Authenticate WebSocket with JWT |
| `MessageSend` | `MessageSendMsg` | Send a chat message |
| `TypingStart` | `TypingMsg` | Begin typing indicator |
| `TypingStop` | `TypingMsg` | End typing indicator |
| `MessageAck` | `MessageAckMsg` | Acknowledge receipt up to seq |
| `Sync` | `SyncMsg` | Request missed messages after seq |
| `DeviceSync` | `DeviceSyncRequest` | Sync device-specific state |

### Server -> Client Messages

| Variant | Struct | Purpose |
|---------|--------|---------|
| `HelloOk` | `HelloOkMsg` | Auth success |
| `HelloError` | `HelloErrorMsg` | Auth failure |
| `MessageReceived` | `MessageReceivedMsg` | New message delivery |
| `TypingStart` | `TypingMsg` | Typing indicator fan-out |
| `TypingStop` | `TypingMsg` | Typing indicator fan-out |
| `PresenceUpdate` | `PresenceUpdateMsg` | Online/offline status |
| `StreamStart` | `StreamStartMsg` | Agent stream begins (Phase 2) |
| `ContentDelta` | `ContentDeltaMsg` | Streaming text chunk (Phase 2) |
| `ToolStart` | `ToolStartMsg` | Agent tool invocation (Phase 2) |
| `ToolEnd` | `ToolEndMsg` | Agent tool result (Phase 2) |
| `StreamEnd` | `StreamEndMsg` | Agent stream complete (Phase 2) |

## Version Field

Every struct MUST have `pub v: u8`. The constant `PROTOCOL_VERSION` (currently
`1`) is defined at crate root. Clients that send a mismatched version receive
`HelloError`.

```rust
pub const PROTOCOL_VERSION: u8 = 1;
```

## Backward Compatibility Rules

| Change | Allowed? |
|--------|----------|
| Add a new variant to `ClientMessage` or `ServerMessage` | YES |
| Add an optional field with `#[serde(default)]` | YES |
| Remove a variant | NO -- deprecate and stop sending |
| Remove a field | NO |
| Rename a field | NO |
| Change a field's type | NO |
| Change `rename_all` strategy | NO |

When a field is optional, always annotate:

```rust
#[serde(skip_serializing_if = "Option::is_none", default)]
pub field_name: Option<Type>,
```

## Adding a New Message Type

1. **Update the contract first**: edit `paw-hq/contracts/protocol-v1.md`
   with the new message schema, purpose, and examples.
2. **Add the struct** in `paw-proto/src/lib.rs`:
   ```rust
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct NewFeatureMsg {
       pub v: u8,
       pub required_field: Type,
       #[serde(skip_serializing_if = "Option::is_none", default)]
       pub optional_field: Option<Type>,
   }
   ```
3. **Add the variant** to `ClientMessage` or `ServerMessage`:
   ```rust
   NewFeature(NewFeatureMsg),
   ```
4. **Write round-trip tests**:
   ```rust
   #[test]
   fn test_new_feature_round_trip() {
       let msg = ClientMessage::NewFeature(NewFeatureMsg {
           v: PROTOCOL_VERSION,
           required_field: value,
           optional_field: None,
       });
       let json = serde_json::to_string(&msg).unwrap();
       let parsed: ClientMessage = serde_json::from_str(&json).unwrap();
       // assert fields match
   }
   ```
5. **Handle in server**: add a match arm in the WebSocket dispatch
   (`ws/session.rs` or equivalent).
6. **Handle in client**: add parsing in the Swift/Kotlin WebSocket layer.

## InboundContext and MessageFormat

Shared types used across both enums:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageFormat { Markdown, Plain }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboundContext { ... }
```

These are re-exported from the crate root and used by `paw-server` handlers.

## Testing

Run protocol tests with:

```bash
cargo test -p paw-proto
```

Every new message type requires at minimum:
- Serialization round-trip test (struct -> JSON -> struct).
- Deserialization from a known JSON string (contract snapshot).
- Verify `"type"` discriminator value matches `rename_all = "snake_case"`.
