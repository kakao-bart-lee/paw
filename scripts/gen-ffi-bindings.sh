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

mkdir -p "$OUT_KOTLIN" "$OUT_SWIFT"

cargo build -p paw-core --lib
cargo run -p paw-core --bin gen-bindings -- kotlin "$OUT_KOTLIN"
cargo run -p paw-core --bin gen-bindings -- swift "$OUT_SWIFT"

echo "generated UniFFI bindings:"
echo "  kotlin -> $OUT_KOTLIN"
echo "  swift  -> $OUT_SWIFT"
echo "note: copy/generated-placement can move to paw-android/ and paw-ios/ once native shells are scaffolded."
