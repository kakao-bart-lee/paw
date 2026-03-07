#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP_DIR="$ROOT_DIR/paw-client"
TEST_TARGET="integration_test/app_flow_test.dart"
DRIVER_TARGET="integration_test/test_driver/integration_test.dart"
WEB_BASE_URL="${PAW_WEB_BASE_URL:-http://127.0.0.1:8080}"

DEVICE="${1:-auto}"

cd "$APP_DIR"

if [[ "$DEVICE" == "auto" ]]; then
  DEVICES="$(flutter devices --machine)"
  if echo "$DEVICES" | grep -q '"id"[[:space:]]*:[[:space:]]*"macos"'; then
    DEVICE="macos"
  elif echo "$DEVICES" | grep -q '"id"[[:space:]]*:[[:space:]]*"android"'; then
    DEVICE="android"
  elif echo "$DEVICES" | grep -q '"id"[[:space:]]*:[[:space:]]*"ios"'; then
    DEVICE="ios"
  elif echo "$DEVICES" | grep -q '"id"[[:space:]]*:[[:space:]]*"chrome"'; then
    DEVICE="chrome"
  else
    echo "No supported device found (macos/android/ios/chrome)."
    exit 1
  fi
fi

echo "[e2e] device=$DEVICE"

if [[ "$DEVICE" == "chrome" || "$DEVICE" == "web-server" ]]; then
  run_playwright_with_server() {
    local server_pid=""
    if ! curl -sf "$WEB_BASE_URL" >/dev/null 2>&1; then
      echo "[e2e] starting Flutter web-server at $WEB_BASE_URL"
      flutter run -d web-server --web-port 8080 >/tmp/paw_web_server.log 2>&1 &
      server_pid=$!
      for _ in {1..60}; do
        if curl -sf "$WEB_BASE_URL" >/dev/null 2>&1; then
          break
        fi
        sleep 1
      done
    fi

    (cd "$APP_DIR/e2e/playwright" && npm install && PAW_WEB_BASE_URL="$WEB_BASE_URL" npx playwright test)
    local code=$?

    if [[ -n "$server_pid" ]]; then
      kill "$server_pid" >/dev/null 2>&1 || true
    fi

    return $code
  }

  echo "[e2e] running web integration via flutter drive"
  if ! command -v chromedriver >/dev/null 2>&1; then
    echo "[e2e] chromedriver not found."
    echo "[e2e] install example (macOS): brew install --cask chromedriver"
    echo "[e2e] fallback: run Playwright smoke test"
    run_playwright_with_server
    exit $?
  fi
  flutter drive -d "$DEVICE" --driver "$DRIVER_TARGET" --target "$TEST_TARGET"
else
  echo "[e2e] running native/desktop integration via flutter test integration_test"
  flutter test "$TEST_TARGET" -d "$DEVICE"
fi
