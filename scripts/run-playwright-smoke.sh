#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP_DIR="$ROOT_DIR/paw-client"
WEB_LOG="/tmp/paw_playwright_smoke_web.log"
WEB_PID=""
WEB_BASE_URL="${PAW_WEB_BASE_URL:-http://127.0.0.1:38481}"

source "$ROOT_DIR/scripts/local-env.sh"
FLUTTER_BIN="$(resolve_flutter_bin)"

cleanup() {
  if [[ -n "$WEB_PID" ]]; then
    kill "$WEB_PID" >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

cd "$APP_DIR"

if ! curl -sf "$WEB_BASE_URL" >/dev/null 2>&1; then
  echo "[playwright-smoke] building Flutter web bundle"
  "$FLUTTER_BIN" build web --no-wasm-dry-run >"$WEB_LOG" 2>&1

  echo "[playwright-smoke] starting static web server at $WEB_BASE_URL"
  python3 -m http.server 38481 --directory build/web >>"$WEB_LOG" 2>&1 &
  WEB_PID=$!

  for _ in {1..90}; do
    if curl -sf "$WEB_BASE_URL" >/dev/null 2>&1; then
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
  npm ci
  PAW_WEB_BASE_URL="$WEB_BASE_URL" npx playwright test tests/routes-console.spec.ts
)

echo "[playwright-smoke] completed"
