#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SERVER_LOG="/tmp/paw_server_e2e.log"
WEB_LOG="/tmp/paw_web_e2e.log"
SERVER_PID=""
WEB_PID=""

source "$ROOT_DIR/scripts/local-env.sh"
FLUTTER_BIN="$(resolve_flutter_bin)"

cleanup() {
  if [[ -n "$WEB_PID" ]]; then
    kill "$WEB_PID" >/dev/null 2>&1 || true
  fi
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

export DATABASE_URL="${DATABASE_URL:-postgres://postgres:postgres@127.0.0.1:35432/paw}"
export PAW_API_BASE_URL="${PAW_API_BASE_URL:-http://127.0.0.1:38173}"
export PAW_WEB_BASE_URL="${PAW_WEB_BASE_URL:-http://127.0.0.1:38481}"

echo "[real-e2e] starting docker dependencies"
docker compose up -d

echo "[real-e2e] running migrations"
(
  cd paw-server
  cargo sqlx migrate run
)

if lsof -ti tcp:38173 >/dev/null 2>&1; then
  echo "[real-e2e] stopping existing process on :38173"
  lsof -ti tcp:38173 | xargs kill -9 >/dev/null 2>&1 || true
fi

if lsof -ti tcp:38481 >/dev/null 2>&1; then
  echo "[real-e2e] stopping existing process on :38481"
  lsof -ti tcp:38481 | xargs kill -9 >/dev/null 2>&1 || true
fi

echo "[real-e2e] starting paw-server"
cargo run -p paw-server >"$SERVER_LOG" 2>&1 &
SERVER_PID=$!

for _ in {1..60}; do
  if curl -sf "$PAW_API_BASE_URL/health" >/dev/null 2>&1; then
    break
  fi
  sleep 1
done

if ! curl -sf "$PAW_API_BASE_URL/health" >/dev/null 2>&1; then
  echo "[real-e2e] server failed to start"
  exit 1
fi

echo "[real-e2e] starting flutter web-server"
(
  cd paw-client
  "$FLUTTER_BIN" build web --no-wasm-dry-run --dart-define=SERVER_URL="$PAW_API_BASE_URL" >"$WEB_LOG" 2>&1
) &
WEB_PID=$!

for _ in {1..90}; do
  if ! kill -0 "$WEB_PID" >/dev/null 2>&1; then
    break
  fi
  sleep 1
done

wait "$WEB_PID"

if [[ ! -f paw-client/build/web/index.html ]]; then
  echo "[real-e2e] web build failed"
  exit 1
fi

echo "[real-e2e] starting static web-server"
cd paw-client
python3 -m http.server 38481 --directory build/web >>"$WEB_LOG" 2>&1 &
WEB_PID=$!
cd "$ROOT_DIR"

for _ in {1..90}; do
  if curl -sf "$PAW_WEB_BASE_URL" >/dev/null 2>&1; then
    break
  fi
  sleep 1
done

if ! curl -sf "$PAW_WEB_BASE_URL" >/dev/null 2>&1; then
  echo "[real-e2e] web app failed to start"
  exit 1
fi

echo "[real-e2e] running Playwright real full-loop"
(
  cd paw-client/e2e/playwright
  npm ci
  npm run test:real
)

echo "[real-e2e] completed"
