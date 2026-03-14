#!/usr/bin/env bash

normalize_legacy_local_env() {
  local changed=0

  if [[ "${PAW_PORT:-}" == "3000" ]]; then
    export PAW_PORT=38173
    changed=1
  fi

  if [[ "${PAW_WEB_DEV_PORT:-}" == "3000" ]]; then
    export PAW_WEB_DEV_PORT=4100
    changed=1
  fi

  if [[ "${DATABASE_URL:-}" == "postgresql://paw:paw_dev_password@localhost:5432/paw_dev" ]]; then
    export DATABASE_URL="postgresql://paw:paw_dev_password@localhost:35432/paw_dev"
    changed=1
  elif [[ "${DATABASE_URL:-}" == "postgres://postgres:postgres@127.0.0.1:5432/paw" ]]; then
    export DATABASE_URL="postgres://postgres:postgres@127.0.0.1:35432/paw"
    changed=1
  elif [[ "${DATABASE_URL:-}" == "postgres://postgres:postgres@localhost:5432/paw" ]]; then
    export DATABASE_URL="postgres://postgres:postgres@localhost:35432/paw"
    changed=1
  fi

  if [[ "${S3_ENDPOINT:-}" == "http://localhost:9000" ]]; then
    export S3_ENDPOINT="http://localhost:39080"
    changed=1
  fi

  if [[ "${NATS_URL:-}" == "nats://localhost:4222" ]]; then
    export NATS_URL="nats://localhost:34223"
    changed=1
  fi

  if [[ "${SERVER_URL:-}" == "http://127.0.0.1:3000" || "${SERVER_URL:-}" == "http://localhost:3000" ]]; then
    export SERVER_URL="http://127.0.0.1:38173"
    changed=1
  fi

  if [[ "${PAW_API_BASE_URL:-}" == "http://127.0.0.1:3000" || "${PAW_API_BASE_URL:-}" == "http://localhost:3000" ]]; then
    export PAW_API_BASE_URL="http://127.0.0.1:38173"
    changed=1
  fi

  if [[ "${PAW_WEB_BASE_URL:-}" == "http://127.0.0.1:8080" || "${PAW_WEB_BASE_URL:-}" == "http://localhost:8080" ]]; then
    export PAW_WEB_BASE_URL="http://127.0.0.1:38481"
    changed=1
  fi

  if [[ $changed -eq 1 ]]; then
    echo "[local-env] detected legacy .env defaults; using updated local ports in this session"
    echo "[local-env] consider refreshing .env from .env.example"
  fi
}

resolve_flutter_bin() {
  if [[ -n "${FLUTTER_BIN:-}" && -x "${FLUTTER_BIN}" ]]; then
    printf '%s\n' "${FLUTTER_BIN}"
    return 0
  fi

  if command -v flutter >/dev/null 2>&1; then
    command -v flutter
    return 0
  fi

  local candidate=""
  local -a candidates=()

  if [[ -n "${FLUTTER_ROOT:-}" ]]; then
    candidates+=("${FLUTTER_ROOT}/bin/flutter")
  fi

  candidates+=(
    "/opt/homebrew/share/flutter/bin/flutter"
    "$HOME/develop/flutter/bin/flutter"
    "$HOME/development/flutter/bin/flutter"
  )

  local old_nullglob
  old_nullglob="$(shopt -p nullglob || true)"
  shopt -s nullglob
  candidates+=("/opt/homebrew/Caskroom/flutter"/*/flutter/bin/flutter)
  if [[ -n "$old_nullglob" ]]; then
    eval "$old_nullglob"
  else
    shopt -u nullglob
  fi

  for candidate in "${candidates[@]}"; do
    if [[ -x "$candidate" ]]; then
      printf '%s\n' "$candidate"
      return 0
    fi
  done

  echo "[local-env] flutter not found. Set FLUTTER_BIN or add flutter to PATH." >&2
  return 1
}
