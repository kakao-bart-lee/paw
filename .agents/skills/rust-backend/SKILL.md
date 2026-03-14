---
name: rust-backend
description: Paw server development with Axum handlers, SQLx queries, and WebSocket integration
origin: Paw
---

# Rust Backend Skill

Paw server (`paw-server/`) is an Axum-based REST + WebSocket service backed by
PostgreSQL (via SQLx) and an optional NATS sidecar for agent communication.

## Axum Handler Pattern

Every handler follows the same signature shape:

```rust
pub async fn handler_name(
    State(state): State<AppState>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path(resource_id): Path<Uuid>,       // optional, from URL
    Json(payload): Json<RequestType>,     // optional, from body
) -> Response {
    // 1. Validate input
    if payload.field.trim().is_empty() {
        return error(StatusCode::BAD_REQUEST, "invalid_field", &locale, "...")
            .into_response();
    }

    // 2. Enforce authorization / membership
    match ensure_membership(&state, resource_id, user_id, &locale).await {
        Ok(()) => {}
        Err(resp) => return resp,
    }

    // 3. Query database
    let row = sqlx::query_as!(
        Model,
        r#"SELECT id, name FROM table WHERE id = $1"#,
        resource_id,
    )
    .fetch_one(&state.db)
    .await;

    // 4. Return JSON
    match row {
        Ok(model) => Json(model).into_response(),
        Err(err) => {
            tracing::error!(%err, "query failed");
            error(StatusCode::INTERNAL_SERVER_ERROR, "query_failed", &locale, "...")
                .into_response()
        }
    }
}
```

Key conventions:
- Return type is always `Response` (via `.into_response()`).
- Use `ApiResult<T>` (`Result<Json<T>, (StatusCode, Json<Value>)>`) for simple cases.
- Extract `RequestLocale` and `UserId` from middleware extensions.
- Log with `tracing::error!` / `tracing::info!`, include structured fields.

## SQLx Query Rules

- **Always** use `sqlx::query!` or `sqlx::query_as!` compile-checked macros.
- **Never** build SQL by string interpolation or `format!`.
- Bind parameters with `$1`, `$2`, etc.
- Use `fetch_one`, `fetch_optional`, or `fetch_all` -- pick the tightest fit.
- For inserts that return a row, use `RETURNING *` with `fetch_one`.

## Error Handling

| Layer | Crate | Example |
|-------|-------|---------|
| Domain errors | `thiserror` | `GroupManagementError`, `AuthError` |
| Infrastructure | `anyhow` | database timeouts, file IO |
| HTTP response | helper `error()` | `error(StatusCode::BAD_REQUEST, "code", &locale, "msg")` |

- Domain errors carry an error code string (`"invalid_content"`, `"not_member"`)
  used by the i18n layer (`crate::i18n::error_response`) to look up localized text.
- Never expose internal details (SQL errors, stack traces) in HTTP responses.

## Response Format

All JSON responses use a flat envelope:

```json
{ "messages": [...], "has_more": true }
```

Error responses:

```json
{ "error": { "code": "not_member", "message": "Localized human text" } }
```

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

- Publish: `nats.publish("paw.agent.<agent_id>", payload)`.
- Subscribe: agent workers consume and reply.
- If NATS is unreachable, log a warning and continue -- never block the
  user-facing request path.

## Testing

Integration tests live in `paw-server/tests/`:

```
tests/
  integration_test.rs   -- main test harness
  helpers/              -- shared fixtures, test DB setup
```

Pattern:
1. Spin up a test database with `sqlx::PgPool` from `DATABASE_URL`.
2. Run migrations via `sqlx::migrate!().run(&pool)`.
3. Build `AppState` with the test pool.
4. Use `axum::test::TestClient` or direct handler calls.
5. Assert HTTP status + JSON body.

Always run: `cargo test -p paw-server`
