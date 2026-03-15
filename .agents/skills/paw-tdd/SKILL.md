---
name: paw-tdd
description: Test-driven development workflow for Paw Rust server and workspace crates
origin: Paw
---

# Paw TDD Skill

Use this when adding or changing behavior in `paw-server`, `paw-proto`,
`paw-core`, or `paw-crypto`.

## Where Tests Live (Actual Paths)

- `paw-server/tests/architecture_test.rs` — crate dependency-direction guards.
- `paw-server/tests/integration_test.rs` — protocol/auth/http/ws integration and
  contract-like checks.
- `paw-server/src/**/handlers.rs` — handler-adjacent unit tests under
  `#[cfg(test)]` (example: `paw-server/src/users/handlers.rs`).
- `paw-server/src/**/{service,models,metrics,i18n,rate_limit}.rs` — focused unit
  tests for domain and utility behavior.
- `paw-proto/src/lib.rs` — protocol serialization/deserialization tests.
- `paw-core/tests/phase3_live_smoke.rs` — ignored live smoke tests against a
  running server.
- `paw-core/src/**` and `paw-crypto/src/mls.rs` — crate-local unit tests.

## Canonical Commands

```bash
cargo test --workspace
cargo test -p paw-server
cargo test -p paw-server --test architecture_test
cargo test -p paw-proto
```

Project aliases:

```bash
make test
./scripts/verify.sh
```

## TDD Loop for Paw

1. Pick the narrowest target area and test file first.
2. Write the failing test using current project style (`#[test]` or `#[tokio::test]`).
3. Run the smallest command (`cargo test -p <crate> <name_fragment>`).
4. Implement minimal code to pass.
5. Refactor with tests green.
6. Finish with `cargo test --workspace` (or `./scripts/verify.sh` before handoff).

## Handler Test Pattern (Paw-Specific)

Current server pattern is hybrid:

- Pure helper logic is unit-tested in-module (e.g. username/locale normalization
  in `paw-server/src/users/handlers.rs`).
- Endpoint behavior is mostly validated in `paw-server/tests/integration_test.rs`
  with real HTTP calls to `http://localhost:38173`.
- Most networked integration tests are `#[ignore = "requires running ..."]`
  and require env setup (`PAW_TEST_TOKEN`, `PAW_TEST_CONV_ID`, etc.).

When adding a handler:

- Add/extend pure function tests in the handler module if logic can be isolated.
- Add protocol/HTTP assertions in `paw-server/tests/integration_test.rs` for
  request/response contract behavior.
- Prefer asserting status + key JSON fields, not full response snapshots.

## Architecture Test Pattern

`paw-server/tests/architecture_test.rs` enforces dependency direction by reading
crate `Cargo.toml` dependency sections:

- `paw-proto` and `paw-crypto` must remain leaves.
- `paw-core` may depend on `paw-proto` only.
- `paw-server` may depend on `paw-proto` only.
- Circular dependency checks are explicit.

If crate dependencies change, update architecture tests in the same PR.

## Integration Test Patterns Used Here

- JWT round-trip tests validate claim shape/type/expiry.
- Protocol frame tests validate `"type"` tag and mandatory `"v": 1`.
- HTTP tests use `reqwest::Client` with bearer tokens for protected endpoints.
- WS tests use `tokio_tungstenite` and assert ordered `message_received` seq.
- Live tests are intentionally ignored by default and serve as manual gates.

## Mock / Stub Pattern in Paw

No dedicated mocking framework (`mockall`, `wiremock`, `mockito`) is used in the
current Rust workspace tests.

Instead, the project uses:

- Real-serialization tests over protocol structs.
- Deterministic pure-function unit tests.
- Env-driven integration tests against a running local stack.
- Small local helper functions/stubs inside test files when needed.

Prefer this style unless introducing a mock dependency is clearly justified.

## Assertion Patterns to Reuse

- `assert_eq!` for status codes, protocol version, enum tags, IDs, seq.
- `assert!` for shape and invariants (`is_string`, monotonic ordering, limits).
- Pattern-match enum variants (`match parsed { ... }`) after deserialization.
- Validate optional-field behavior (`skip_serializing_if`, `#[serde(default)]`).
- For idempotency, assert stable `(id, seq)` across duplicate send attempts.

## Done Criteria for Paw TDD

- New behavior has at least one failing-then-passing test in the nearest test
  location.
- Relevant crate test command passes.
- `cargo test --workspace` passes before merge.
- If dependency edges changed, `architecture_test` also passes.
