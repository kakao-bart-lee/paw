#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SERVER_LOG="/tmp/paw_server_real_flutter_e2e.log"
SERVER_PID=""

cleanup() {
  if [[ -n "$SERVER_PID" ]]; then
    kill "$SERVER_PID" >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

cd "$ROOT_DIR"

if [[ ! -f .env ]]; then
  cp .env.example .env
fi

set -a
# shellcheck disable=SC1091
source .env
set +a

export DATABASE_URL="${DATABASE_URL:-postgres://postgres:postgres@127.0.0.1:5432/paw}"
export SERVER_URL="${SERVER_URL:-http://127.0.0.1:3000}"

echo "[real-flutter-e2e] starting docker dependencies"
docker compose up -d

echo "[real-flutter-e2e] running migrations"
(
  cd paw-server
  cargo sqlx migrate run
)

if lsof -ti tcp:3000 >/dev/null 2>&1; then
  echo "[real-flutter-e2e] stopping existing process on :3000"
  lsof -ti tcp:3000 | xargs kill -9 >/dev/null 2>&1 || true
fi

echo "[real-flutter-e2e] starting paw-server with OTP debug exposure"
PAW_EXPOSE_OTP_FOR_E2E=true cargo run -p paw-server >"$SERVER_LOG" 2>&1 &
SERVER_PID=$!

for _ in {1..60}; do
  if curl -sf "$SERVER_URL/health" >/dev/null 2>&1; then
    break
  fi
  sleep 1
done

if ! curl -sf "$SERVER_URL/health" >/dev/null 2>&1; then
  echo "[real-flutter-e2e] server failed to start"
  exit 1
fi

echo "[real-flutter-e2e] running integration_test/real_server_loop_test.dart on macOS"
(
  cd paw-client
  flutter test integration_test/real_server_loop_test.dart -d macos --dart-define=SERVER_URL="$SERVER_URL"
)

echo "[real-flutter-e2e] completed"
