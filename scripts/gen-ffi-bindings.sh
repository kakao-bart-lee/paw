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

OUT_KOTLIN="paw-core/generated/kotlin"
OUT_SWIFT="paw-core/generated/swift"

MODE="${1:-all}"

case "$MODE" in
  all)
    mkdir -p "$OUT_KOTLIN" "$OUT_SWIFT"
    ;;
  kotlin)
    mkdir -p "$OUT_KOTLIN"
    ;;
  swift)
    mkdir -p "$OUT_SWIFT"
    ;;
  *)
    echo "usage: $0 [all|kotlin|swift]" >&2
    exit 1
    ;;
esac

cargo build -p paw-core --lib

if [[ "$MODE" == "all" || "$MODE" == "kotlin" ]]; then
  cargo run -p paw-core --bin gen-bindings -- kotlin "$OUT_KOTLIN"
  echo "generated UniFFI kotlin bindings -> $OUT_KOTLIN"
fi

if [[ "$MODE" == "all" || "$MODE" == "swift" ]]; then
  cargo run -p paw-core --bin gen-bindings -- swift "$OUT_SWIFT"
  echo "generated UniFFI swift bindings -> $OUT_SWIFT"
fi

echo "note: copy/generated-placement can move to paw-android/ and paw-ios/ once native shells are scaffolded."
