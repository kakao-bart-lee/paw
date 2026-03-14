# Paw Messenger

Paw is an AI-native messenger monorepo that is currently migrating from a single Flutter client to a split architecture:

- `paw-core/` — shared Rust runtime for mobile-native clients
- `paw-android/` — Kotlin + Compose native shell
- `paw-ios/` — SwiftUI native shell
- `paw-client/` — Flutter Web/Desktop path kept alive during migration
- `paw-server/` — Rust/Axum backend

## Features

- **Secure Messaging**: End-to-end encryption using the Signal protocol (X3DH + Double Ratchet).
- **Agent-First**: Native support for AI agents with streaming responses and rich UI blocks.
- **Channels**: Broadcast messages to thousands of subscribers.
- **Marketplace**: Discover and install agents from a global marketplace.
- **Shared Core**: Rust `paw-core` foundation for crypto, db/search, auth, http, ws/sync runtime slices.
- **Transitional Client Strategy**: Native mobile shells + Flutter Web/Desktop during migration.
- **Developer Friendly**: Comprehensive SDKs for Python and TypeScript.

## Architecture

```text
                                  +-----------------------+
                                  | PostgreSQL/MinIO/NATS |
                                  +-----------^-----------+
                                              |
+----------------+       +--------------------+--------------------+
+----------------+       | (REST, WS, Auth, Media, Agent Gateway)  |
                         +--------------------+--------------------+
                                              |
                                              v (WS /agent/ws)
                                     +-----------------+
                                     |  Python/TS SDK  |
                                     +--------+--------+
                                              |
                                              v
                                     +-----------------+
                                     | OpenClaw Adapter|
                                     +-----------------+
```

## Quickstart

### Server

1. Clone the repository.
2. Run `docker-compose up -d` to start dependencies (PostgreSQL, MinIO, NATS).
3. Run `cargo run --package paw-server` to start the server.

### Local Dev (server + client)

1. Run `./scripts/run-local-dev.sh` from the repo root.
2. Optionally pass a Flutter device, for example `./scripts/run-local-dev.sh macos`.
3. When using a web device (`chrome`, `edge`, `web-server`), the dev app binds to `http://127.0.0.1:4100` by default.
4. For local-only manual auth testing, uncomment `PAW_FIXED_OTP=137900` in `.env`.
5. Stop everything with `./scripts/stop-local-dev.sh`.

### Client

1. Navigate to `paw-client`.
2. Run `flutter run`.

## Repository status

Migration progress follows `docs/native-migration-plan.md`.

- Phase 0–1 groundwork: in repo
- Phase 2 core slices: crypto / db-search / auth foundation implemented in `paw-core`
- Phase 3 core slices: HTTP + WS/sync/runtime foundation implemented in `paw-core`
- Phase 4–5: native auth/bootstrap/chat shells and platform automation are in repo
- Phase 6: Flutter client has been shifted toward Web/Desktop verification gates
- Phase 7: Flutter Web/Desktop path is independent of `paw-ffi`; final cleanup is reflected in workspace/CI/docs

## Quickstart

### Shared Rust core

```bash
cargo test -p paw-core
make bindings
```

### Flutter Web/Desktop

```bash
cd paw-client
flutter test test/widget_test.dart test/features/chat/widgets/media_picker_test.dart test/features/settings/screens/settings_screen_test.dart test/desktop_service_test.dart
flutter build macos
./scripts/run-playwright-smoke.sh
./scripts/run-real-web-e2e.sh
```

### Android shell scaffold

```bash
make core-android
cd paw-android
./gradlew :app:assembleDebug
```

### iOS shell scaffold

```bash
make core-ios
cd paw-ios
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer \
xcodebuild -project Paw.xcodeproj -scheme Paw -destination 'platform=iOS Simulator,name=iPhone 17 Pro' test
```

> Note: Android SDK/NDK and iOS simulator runtimes must be installed for full native build verification.

## CI layout

The repo is split into dedicated workflows:

- `.github/workflows/core.yml`
- `.github/workflows/android.yml`
- `.github/workflows/ios.yml`
- `.github/workflows/flutter.yml`
- `.github/workflows/server.yml`

## Documentation

- [Native migration plan](docs/native-migration-plan.md)
- [Native core contract](docs/native-core-contract.md)
- [Native platform contract](docs/native-platform-contract.md)
- [Native mobile automation plan](docs/native-mobile-automation-plan.md)
- [API Reference](docs/api/openapi.yaml)
- [WebSocket Protocol](docs/protocol-v1.md)
- [Architecture Deep Dive](docs/ARCHITECTURE.md)
- [Python SDK Quickstart](docs/sdk/python-quickstart.md)
- [TypeScript SDK Quickstart](docs/sdk/typescript-quickstart.md)

## License

MIT License. See [LICENSE](LICENSE) for details.

## License

MIT License. See [LICENSE](LICENSE) for details.
