# Native Foundation Closeout

Updated: 2026-03-14

## Summary

`feat/native-foundation` was merged into `main` via PR #2.

- PR: https://github.com/talelapse/paw/pull/2
- Merge commit: `14d48dd`

This branch completed the repository-wide foundation work for the Flutter Web/Desktop + native mobile split.

## What Landed

### Shared core / migration
- `paw-core/` shared Rust runtime scaffold and core slices
- auth / crypto / db-search / http / ws-sync / runtime contracts
- UniFFI contract surface and platform bridge contracts
- `paw-ffi` / FRB removal from the Flutter Web/Desktop path

### Native shells
- `paw-android/` Kotlin + Compose shell scaffold
- `paw-ios/` SwiftUI shell scaffold
- native bootstrap/auth/chat shell paths
- Android/iOS automation scaffolds and smoke coverage

### Flutter Web/Desktop
- Web/Desktop-only responsibility clarified
- Playwright smoke + real web E2E stabilization
- macOS build/test gates retained
- warm editorial palette + weaker rounding + mono-led theme pass
- Flutter theme mode control added

### CI / docs / scripts
- workflow split: core / android / ios / flutter / server
- native build scripts and local dev scripts updated
- migration/design/contract/automation docs added and synced

## Verification Snapshot

The branch was verified incrementally during development and again before merge.

Representative successful checks:
- `cargo test --workspace`
- `cargo clippy -p paw-server -p paw-proto --all-targets -- -D warnings`
- `cargo test -p paw-server auth::otp`
- `flutter test test/widget_test.dart test/features/chat/widgets/media_picker_test.dart test/features/settings/screens/settings_screen_test.dart test/desktop_service_test.dart`
- `flutter build macos`
- `./scripts/run-playwright-smoke.sh`
- `./scripts/run-real-web-e2e.sh`
- `./scripts/run-real-flutter-e2e.sh`
- `xcodebuild -project paw-ios/Paw.xcodeproj -scheme Paw -destination 'platform=iOS Simulator,name=iPhone 17 Pro' test`
- `./gradlew :app:compileDebugKotlin :app:testDebugUnitTest`

## Important Decisions Captured

- Flutter remains the Web/Desktop client path.
- Native mobile work is centered on Android/iOS shells over shared contracts/runtime.
- `COLOR_palette.md` is the warm palette reference.
- The intended visual tone is editorial, quiet, sparse, with weaker rounding.
- `PAW_FIXED_OTP` remains dev-only and server-side only; OTP disclosure stays gated behind `PAW_EXPOSE_OTP_FOR_E2E=true`.

## Non-blocking Follow-up Notes

These do not block the foundation closeout, but they are natural next work:

- Capture cleaner foreground screenshots for final visual sign-off on Android/iOS.
- Expand theme mode support from Flutter Web/Desktop into native shells if desired.
- Continue reducing legacy Flutter analyze warnings unrelated to this foundation branch.
- Move from shell-level native UI polish into production-ready app flows.

## Branch Outcome

`native-foundation` should be treated as the completed baseline for the next workstream, not as an active long-running feature branch.
