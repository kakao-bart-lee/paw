#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SERVER_LOG="/tmp/paw_core_phase3_live.log"
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
source ./scripts/local-env.sh
normalize_legacy_local_env

export DATABASE_URL="${DATABASE_URL:-postgres://postgres:postgres@127.0.0.1:35432/paw}"
export PAW_SERVER_URL="${PAW_SERVER_URL:-http://127.0.0.1:38173}"
export PAW_TEST_PHONE="${PAW_TEST_PHONE:-+8210$(python3 - <<'PY'
import random
print(''.join(str(random.randint(0,9)) for _ in range(8)))
PY
)}"

ensure_sqlx_cli() {
  if cargo sqlx --help >/dev/null 2>&1; then
    return
  fi

  echo "[phase3-live] sqlx-cli not found; installing via cargo..."
  cargo install sqlx-cli --no-default-features --features rustls,postgres
}

echo "[phase3-live] starting docker dependencies"
docker compose up -d

ensure_sqlx_cli

echo "[phase3-live] running migrations"
(
  cd paw-server
  cargo sqlx migrate run
)

if lsof -ti tcp:38173 >/dev/null 2>&1; then
  echo "[phase3-live] stopping existing process on :38173"
  lsof -ti tcp:38173 | xargs kill -9 >/dev/null 2>&1 || true
fi

echo "[phase3-live] starting paw-server with OTP debug exposure"
PAW_EXPOSE_OTP_FOR_E2E=true cargo run -p paw-server >"$SERVER_LOG" 2>&1 &
SERVER_PID=$!

for _ in {1..60}; do
  if curl -sf "$PAW_SERVER_URL/health" >/dev/null 2>&1; then
    break
  fi
  sleep 1
done

if ! curl -sf "$PAW_SERVER_URL/health" >/dev/null 2>&1; then
  echo "[phase3-live] server failed to start"
  exit 1
fi

echo "[phase3-live] provisioning auth tokens + conversation"
OTP_RESPONSE="$(curl -sf -X POST "$PAW_SERVER_URL/auth/request-otp" \
  -H 'Content-Type: application/json' \
  -d "{\"phone\":\"$PAW_TEST_PHONE\"}")"
PAW_TEST_OTP_CODE="$(python3 - <<'PY' "$OTP_RESPONSE"
import json, sys
payload = json.loads(sys.argv[1])
print(payload.get("debug_code", ""))
PY
)"
if [[ -z "$PAW_TEST_OTP_CODE" ]]; then
  echo "[phase3-live] missing debug_code in request-otp response: $OTP_RESPONSE"
  exit 1
fi
export PAW_TEST_OTP_CODE

VERIFY_RESPONSE="$(curl -sf -X POST "$PAW_SERVER_URL/auth/verify-otp" \
  -H 'Content-Type: application/json' \
  -d "{\"phone\":\"$PAW_TEST_PHONE\",\"code\":\"$PAW_TEST_OTP_CODE\"}")"
PAW_TEST_SESSION_TOKEN="$(python3 - <<'PY' "$VERIFY_RESPONSE"
import json, sys
payload = json.loads(sys.argv[1])
print(payload["session_token"])
PY
)"
export PAW_TEST_SESSION_TOKEN

REGISTER_RESPONSE="$(curl -sf -X POST "$PAW_SERVER_URL/auth/register-device" \
  -H 'Content-Type: application/json' \
  -d "{\"session_token\":\"$PAW_TEST_SESSION_TOKEN\",\"device_name\":\"phase3-live-smoke\",\"ed25519_public_key\":\"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=\"}")"
PAW_TEST_TOKEN="$(python3 - <<'PY' "$REGISTER_RESPONSE"
import json, sys
payload = json.loads(sys.argv[1])
print(payload["access_token"])
PY
)"
export PAW_TEST_TOKEN

CONVERSATION_RESPONSE="$(curl -sf -X POST "$PAW_SERVER_URL/conversations" \
  -H 'Content-Type: application/json' \
  -H "Authorization: Bearer $PAW_TEST_TOKEN" \
  -d '{"member_ids":[],"name":"phase3 live"}')"
PAW_TEST_CONV_ID="$(python3 - <<'PY' "$CONVERSATION_RESPONSE"
import json, sys
payload = json.loads(sys.argv[1])
print(payload["id"])
PY
)"
export PAW_TEST_CONV_ID

for i in 1 2; do
  curl -sf -X POST "$PAW_SERVER_URL/conversations/$PAW_TEST_CONV_ID/messages" \
    -H 'Content-Type: application/json' \
    -H "Authorization: Bearer $PAW_TEST_TOKEN" \
    -d "{\"content\":\"phase3 smoke message $i\",\"format\":\"plain\",\"idempotency_key\":\"$(python3 - <<'PY'
import uuid
print(uuid.uuid4())
PY
)\"}" >/dev/null
done

echo "[phase3-live] running paw-core live smoke"
PAW_SERVER_URL="$PAW_SERVER_URL" PAW_TEST_TOKEN="$PAW_TEST_TOKEN" PAW_TEST_CONV_ID="$PAW_TEST_CONV_ID" cargo test -p paw-core --test phase3_live_smoke -- --ignored --nocapture

echo "[phase3-live] completed"
