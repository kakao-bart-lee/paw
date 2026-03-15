---
name: rust-backend
description: Paw server development with Axum handlers, SQLx queries, and WebSocket integration
origin: Paw
---

# Rust Backend Skill

Paw server (`paw-server/`) is an Axum-based REST + WebSocket service backed by
PostgreSQL (via SQLx) and an optional NATS sidecar for agent communication.

## Axum Handler Pattern

Most handlers follow this signature family:

```rust
pub async fn handler_name(
    State(state): State<AppState>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path(resource_id): Path<Uuid>,      // optional
    Json(payload): Json<RequestType>,   // optional
) -> Response {
    // validate -> membership/auth check -> sqlx query -> localized response
}
```

Key conventions in this repo:
- Return types vary by module: `Response`, `(StatusCode, Json<Value>)`, or
  `Result<Json<T>, (StatusCode, Json<Value>)>`.
- `RequestLocale` and `UserId` are extracted via `Extension(...)` on protected
  routes.
- Membership checks are centralized (`service::check_member` /
  `ensure_membership`) before data mutation.
- Logs use structured fields (`conversation_id`, `user_id`, `agent_id`) with
  `tracing::error!`, `warn!`, `info!`.

## SQLx Query Rules

- Prefer `sqlx::query!` / `query_as!` where schema-checked macros are practical.
- Existing code also uses `sqlx::query_as::<_, Model>`, `query_scalar`, and
  `query` for dynamic/partial-row workflows.
- **Never** build SQL by string interpolation or `format!`.
- Bind parameters with `$1`, `$2`, etc.
- Use `fetch_one`, `fetch_optional`, or `fetch_all` -- pick the tightest fit.
- For inserts with returned IDs, use `RETURNING ...` + `fetch_one` / `query_scalar`.

## Error Handling

| Layer | Crate | Example |
|-------|-------|---------|
| Domain errors | `thiserror` | `GroupManagementError`, `AuthError` |
| Infrastructure | `anyhow` | database timeouts, file IO |
| HTTP response | helper `error()` | `error(StatusCode::BAD_REQUEST, "code", &locale, "msg")` |

- Domain errors carry an error code string (`"invalid_content"`, `"not_member"`)
  used by the i18n layer (`crate::i18n::error_response`) to look up localized text.
- Never expose internal details (SQL errors, stack traces) in HTTP responses.

## Router Composition

Route registration is centralized in `paw-server/src/main.rs`:

- Public routes: `/auth/request-otp`, `/auth/verify-otp`, `/auth/register-device`, `/auth/refresh`
- Protected routes: conversations/messages/threads/channels/media/agents/backup/moderation/push
- Protected chain: rate-limit layer + `auth_middleware`
- Global chain: locale middleware -> request-id middleware -> CORS -> metrics middleware

## Response Format

Common shapes in handlers:
- Paginated/list APIs: `{ "messages": [...], "has_more": bool }`,
  `{ "conversations": [...] }`, `{ "agents": [...] }`
- Boolean action responses: `{ "deleted": bool }`, `{ "invited": bool }`
- Errors are localized via `error_response(...)` and usually include
  code/message (plus optional details)

## Authentication & Authorization

All protected routes pass through `auth_middleware` (`auth/middleware.rs`):

1. Extract `Authorization: Bearer <jwt>` header.
2. Validate JWT via `jwt::verify`.
3. Insert `UserId(uuid)` and `DeviceId(Option<uuid>)` as extensions.
4. If invalid, return localized 401 with `error_response_with_request_id`.

When adding a new route:
- Wrap the router layer with `.layer(middleware::from_fn_with_state(state.clone(), auth_middleware))`.
- Extract `Extension(UserId(user_id))` in the handler.

## WebSocket Architecture

```
Client ── WS ──▸ paw-server ──▸ Hub (in-memory fan-out)
                      │
                      ├── pg_notify("new_message")  ← INSERT trigger
                      └── NATS publish (agent gateway, optional)
```

- `ws/hub.rs` maintains `HashMap<Uuid, Vec<WsSender>>` (user_id -> connections).
- `Hub::register` / `Hub::unregister` manage per-connection lifecycle.
- `pg_notify` on the `messages` table fires a trigger that broadcasts new
  messages to all connected Axum instances (multi-process support).
- For agent-originated messages, publish to NATS; gracefully degrade
  (`tracing::warn!`) if NATS is unavailable.

## Agent Gateway (NATS)

- Publish inbound context: `nats.publish("agent.inbound.<agent_id>", payload)`.
- Subscribe: agent workers consume and reply.
- If NATS is unreachable, log a warning and continue -- never block the
  user-facing request path.

## Testing

Server test entry points:

```
paw-server/tests/architecture_test.rs
paw-server/tests/integration_test.rs
paw-server/src/**/handlers.rs (unit tests under #[cfg(test)])
```

Pattern:
1. Fast tests are mostly serde/protocol/domain invariant checks.
2. Real HTTP/WS flows are `#[tokio::test]` + `#[ignore]`, hitting
   `http://localhost:38173` / `ws://localhost:38173` with env-backed tokens.
3. Architecture guard test validates Cargo dependency direction across crates.
4. Handler-local unit tests focus on pure helper functions (e.g. username/locale normalization).

Always run: `cargo test -p paw-server`
Optional full gate: `./scripts/verify.sh`
