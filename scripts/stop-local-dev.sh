#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

stop_port() {
  local port="$1"
  if lsof -ti tcp:"$port" >/dev/null 2>&1; then
    echo "[stop-local-dev] stopping process on :$port"
    lsof -ti tcp:"$port" | xargs kill >/dev/null 2>&1 || true
    sleep 1
    if lsof -ti tcp:"$port" >/dev/null 2>&1; then
      lsof -ti tcp:"$port" | xargs kill -9 >/dev/null 2>&1 || true
    fi
  fi
}

stop_port 38173
stop_port 38481

echo "[stop-local-dev] stopping docker dependencies"
docker compose down

echo "[stop-local-dev] done"
