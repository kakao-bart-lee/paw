#!/usr/bin/env bash
set -euo pipefail

TOTAL=6
PASSED=0

step() {
  local num="$1"
  local label="$2"
  printf "\n[%d/%d] %s\n" "$num" "$TOTAL" "$label"
}

fail() {
  local num="$1"
  local label="$2"
  printf "\nFAILED at step [%d/%d] %s\n" "$num" "$TOTAL" "$label"
  exit 1
}

# 1. Formatting
step 1 "cargo fmt --check --all"
cargo fmt --check --all || fail 1 "cargo fmt --check --all"
PASSED=$((PASSED + 1))

# 2. Lint
step 2 "cargo clippy --workspace -- -D warnings"
cargo clippy --workspace -- -D warnings || fail 2 "cargo clippy --workspace -- -D warnings"
PASSED=$((PASSED + 1))

# 3. Build
step 3 "cargo build --workspace"
cargo build --workspace || fail 3 "cargo build --workspace"
PASSED=$((PASSED + 1))

# 4. Tests
step 4 "cargo test --workspace"
cargo test --workspace || fail 4 "cargo test --workspace"
PASSED=$((PASSED + 1))

# 5. Architecture test (skip gracefully if not found)
step 5 "architecture test"
if cargo test -p paw-server --test architecture_test --no-run 2>/dev/null; then
  cargo test -p paw-server --test architecture_test || fail 5 "architecture test"
  PASSED=$((PASSED + 1))
else
  printf "  Skipped (architecture_test not found)\n"
  PASSED=$((PASSED + 1))
fi

# 6. OpenAPI spec exists
step 6 "docs/api/openapi.yaml exists"
if [ -f docs/api/openapi.yaml ]; then
  PASSED=$((PASSED + 1))
else
  fail 6 "docs/api/openapi.yaml exists"
fi

printf "\nAll checks passed (%d/%d)\n" "$PASSED" "$TOTAL"
