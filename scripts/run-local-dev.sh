#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SERVER_LOG="/tmp/paw_local_stack.log"
SERVER_PID=""
CLIENT_DEVICE="${1:-chrome}"

cleanup() {
  if [[ -n "$SERVER_PID" ]] && kill -0 "$SERVER_PID" >/dev/null 2>&1; then
    echo
    echo "[local-dev] stopping background server (pid=$SERVER_PID)"
    kill "$SERVER_PID" >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT INT TERM

cd "$ROOT_DIR"

export PATH="/opt/homebrew/share/flutter/bin:/opt/homebrew/bin:$HOME/.cargo/bin:/opt/homebrew/opt/rustup/bin:$PATH"

if [[ ! -f .env ]]; then
  cp .env.example .env
  echo "created .env from .env.example"
fi

set -a
# shellcheck disable=SC1091
source .env
set +a
source ./scripts/local-env.sh
normalize_legacy_local_env

PAW_API_BASE_URL="${SERVER_URL:-http://127.0.0.1:38173}"

if curl -sf "$PAW_API_BASE_URL/health" >/dev/null 2>&1; then
  echo "[local-dev] existing server detected at $PAW_API_BASE_URL"
else
  echo "[local-dev] starting server via ./scripts/run-local-stack.sh"
  ./scripts/run-local-stack.sh >"$SERVER_LOG" 2>&1 &
  SERVER_PID=$!

  for _ in {1..180}; do
    if curl -sf "$PAW_API_BASE_URL/health" >/dev/null 2>&1; then
      break
    fi
    if ! kill -0 "$SERVER_PID" >/dev/null 2>&1; then
      echo "[local-dev] server exited unexpectedly; recent log:"
      tail -n 80 "$SERVER_LOG" 2>/dev/null || true
      exit 1
    fi
    sleep 1
  done

  if ! curl -sf "$PAW_API_BASE_URL/health" >/dev/null 2>&1; then
    echo "[local-dev] server did not become healthy in time; recent log:"
    tail -n 80 "$SERVER_LOG" 2>/dev/null || true
    exit 1
  fi
fi

echo "[local-dev] server ready at $PAW_API_BASE_URL"
echo "[local-dev] launching Flutter client on device: $CLIENT_DEVICE"
cd paw-client
flutter run -d "$CLIENT_DEVICE" --dart-define=SERVER_URL="$PAW_API_BASE_URL"
