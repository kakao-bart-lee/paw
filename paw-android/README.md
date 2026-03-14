# paw-android

Minimal native Android shell for the Phase 1 Kotlin + Compose migration target.

## What is scaffolded

- Gradle Kotlin DSL project layout (`settings.gradle.kts`, root `build.gradle.kts`, `gradle.properties`)
- `app/` Android application module with Jetpack Compose + Material 3
- `MainActivity` and a small Compose home screen
- `PawCoreBridge` that attempts a `uniffi.paw_core.ping()` call and surfaces success/failure in the UI
- Reserved `app/src/main/jniLibs/` path for `scripts/build-core-android.sh` output
- Generated Kotlin bindings wired in as an additional source directory from `../paw-core/generated/kotlin`

## Current shell status

- `PawAndroidApp` now shows a bootstrap/auth preview wired to generated UniFFI contract types
- `PawBootstrapViewModel` provides a minimal native shell state that Android can expand into real auth/bootstrap flow
- `./gradlew :app:assembleDebug` succeeds when the environment variables from `.zshrc` are loaded
- `./scripts/build-core-android.sh` succeeds and copies `libpaw_core.so` into `app/src/main/jniLibs`

## Expected next commands

```bash
source ~/.zshrc
./scripts/gen-ffi-bindings.sh
./scripts/build-core-android.sh
./gradlew :app:assembleDebug
```
