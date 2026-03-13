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

IOS_OUT="paw-ios/PawCore/Artifacts"
mkdir -p "$IOS_OUT"

./scripts/gen-ffi-bindings.sh

cargo build -p paw-core --target aarch64-apple-ios --release
cargo build -p paw-core --target aarch64-apple-ios-sim --release

cp target/aarch64-apple-ios/release/libpaw_core.a "$IOS_OUT/libpaw_core_ios.a"
cp target/aarch64-apple-ios-sim/release/libpaw_core.a "$IOS_OUT/libpaw_core_ios_sim.a"

echo "built paw-core iOS static libraries into $IOS_OUT"
echo "note: XCFramework packaging can be added once the native Xcode shell exists."
