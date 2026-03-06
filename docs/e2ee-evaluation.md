# E2EE Protocol Evaluation for Paw

## Options Evaluated
1. vodozemac (Signal Protocol)
2. openmls (MLS Protocol)

## Comparison Matrix
| Criterion | vodozemac | openmls |
|-----------|-----------|---------|
| License | Apache-2.0 | MIT |
| Flutter wrapper | AGPL-3.0 ❌ (flutter_vodozemac) | None required; Rust integration path available |
| Group messaging | Manual composition (Olm + Megolm trade-offs) | Built-in MLS group lifecycle |
| Maturity | Production usage in Matrix ecosystem | RFC 9420-based implementation, actively maturing |
| Dart FFI | Needed | Needed |
| IETF standard | No (protocol family influence, but not MLS standard) | Yes (RFC 9420) |

## Recommendation
Recommend **openmls** for Paw.

Rationale:
- **License compatibility is clean** (MIT) for Apache-2.0 server/client distribution constraints.
- **Group messaging is a first-class concern** in Paw Phase 2, and MLS is designed for efficient, secure group operations.
- **Standards alignment** with RFC 9420 reduces long-term protocol risk and improves interoperability posture.
- While vodozemac is mature, Paw would still need custom architecture for group semantics and should avoid AGPL wrapper dependencies in Flutter.

## Phase 2 Implementation Plan
1. Introduce key server endpoints for publishing/fetching MLS key packages.
2. Add persistence tables (`prekey_bundles`, `e2ee_sessions`) and group state metadata.
3. Add client-side Rust FFI bridge for Flutter and define serialization contract in paw-proto.
4. Implement group lifecycle actions: create, invite/add, remove, rotate/update, leave.
5. Add encrypted payload framing to protocol-v2 message envelope (without changing Phase 1 websocket payloads).
6. Add interoperability and migration tests for one-to-one and group flows.

## Phase 1 PoC
This PoC demonstrates:
- Creating a basic MLS credential and key package.
- Creating an MLS group using OpenMLS.
- Adding a new member via key package and producing a Welcome message.

The PoC is intentionally scoped to cryptographic primitives and group setup only (no full messaging integration yet).
