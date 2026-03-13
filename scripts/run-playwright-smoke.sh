#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP_DIR="$ROOT_DIR/paw-client"
WEB_LOG="/tmp/paw_playwright_smoke_web.log"
WEB_PID=""
WEB_BASE_URL="${PAW_WEB_BASE_URL:-http://127.0.0.1:38481}"

cleanup() {
  if [[ -n "$WEB_PID" ]]; then
    kill "$WEB_PID" >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

cd "$APP_DIR"

if ! curl -sf "$WEB_BASE_URL" >/dev/null 2>&1; then
  echo "[playwright-smoke] starting Flutter web-server at $WEB_BASE_URL"
  flutter run -d web-server --web-port 38481 >"$WEB_LOG" 2>&1 &
  WEB_PID=$!

  for _ in {1..90}; do
    if grep -q "lib/main.dart is being served at" "$WEB_LOG" 2>/dev/null && \
       curl -sf "$WEB_BASE_URL" >/dev/null 2>&1; then
      break
    fi
    sleep 1
  done
fi

if ! curl -sf "$WEB_BASE_URL" >/dev/null 2>&1; then
  echo "[playwright-smoke] web app failed to start"
  exit 1
fi

echo "[playwright-smoke] running Playwright route/console smoke"
(
  cd "$APP_DIR/e2e/playwright"
  npm install
  PAW_WEB_BASE_URL="$WEB_BASE_URL" npx playwright test tests/routes-console.spec.ts
)

echo "[playwright-smoke] completed"
