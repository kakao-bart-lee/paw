#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

if [ ! -f .env ]; then
  cp .env.example .env
  echo "created .env from .env.example"
fi

export PATH="/opt/homebrew/bin:$HOME/.cargo/bin:/opt/homebrew/opt/rustup/bin:$PATH"
set -a
# shellcheck disable=SC1091
source .env
set +a

ensure_sqlx_cli() {
  if cargo sqlx --help >/dev/null 2>&1; then
    return
  fi

  echo "sqlx-cli not found; installing via cargo..."
  cargo install sqlx-cli --no-default-features --features rustls,postgres
}

docker compose up -d

ensure_sqlx_cli

(
  cd paw-server
  cargo sqlx migrate run
)

cargo run -p paw-server
