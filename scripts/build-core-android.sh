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

if [[ -z "${ANDROID_NDK_HOME:-}" ]]; then
  if [[ -n "${ANDROID_NDK_ROOT:-}" ]]; then
    export ANDROID_NDK_HOME="${ANDROID_NDK_ROOT}"
  elif [[ -n "${ANDROID_HOME:-}" ]] && [[ -d "${ANDROID_HOME}/ndk" ]]; then
    latest_ndk="$(find "${ANDROID_HOME}/ndk" -mindepth 1 -maxdepth 1 -type d | sort | tail -n 1)"
    if [[ -n "${latest_ndk:-}" ]]; then
      export ANDROID_NDK_HOME="${latest_ndk}"
    fi
  fi
fi

if [[ -z "${ANDROID_NDK_HOME:-}" || ! -d "${ANDROID_NDK_HOME}" ]]; then
  echo "missing required Android NDK. Set ANDROID_NDK_HOME or ANDROID_NDK_ROOT." >&2
  exit 1
fi

cargo ndk -t arm64-v8a -t x86_64 -o "$ANDROID_OUT" build -p paw-core --release

echo "built paw-core Android artifacts into $ANDROID_OUT"
