# paw-android

Minimal native Android shell for the Phase 1 Kotlin + Compose migration target.

## What is scaffolded

- Gradle Kotlin DSL project layout (`settings.gradle.kts`, root `build.gradle.kts`, `gradle.properties`)
- `app/` Android application module with Jetpack Compose + Material 3
- `MainActivity` and a small Compose home screen
- `PawCoreBridge` that attempts a `uniffi.paw_core.ping()` call and surfaces success/failure in the UI
- Reserved `app/src/main/jniLibs/` path for `scripts/build-core-android.sh` output
- Generated Kotlin bindings wired in as an additional source directory from `../paw-core/generated/kotlin`

## Current build/tooling blockers

- Gradle wrapper files are now present, but `./gradlew :app:assembleDebug` is still blocked until Android SDK location is configured (`ANDROID_HOME` or `local.properties` with `sdk.dir=...`).
- `cargo-ndk` is available, but `./scripts/build-core-android.sh` is still blocked until the Android NDK is installed and `ANDROID_NDK_HOME` points to it.

## Expected next commands

```bash
./scripts/gen-ffi-bindings.sh
./scripts/build-core-android.sh
# then, after Android SDK + NDK are configured:
./gradlew :app:assembleDebug
```
