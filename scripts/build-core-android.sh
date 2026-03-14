#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

need_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "missing required command: $1" >&2
    exit 1
  fi
}

need_cmd cargo
need_cmd rustup
need_cmd cargo-ndk

ANDROID_OUT="paw-android/app/src/main/jniLibs"
mkdir -p "$ANDROID_OUT"

./scripts/gen-ffi-bindings.sh

rustup target add aarch64-linux-android x86_64-linux-android

cargo ndk -t arm64-v8a -t x86_64 -o "$ANDROID_OUT" build -p paw-core --release

echo "built paw-core Android artifacts into $ANDROID_OUT"
