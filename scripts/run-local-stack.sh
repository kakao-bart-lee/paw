#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

export PATH="/opt/homebrew/bin:$HOME/.cargo/bin:/opt/homebrew/opt/rustup/bin:$PATH"

if [ ! -f .env ]; then
  cp .env.example .env
  echo "created .env from .env.example"
fi

docker compose up -d

(
  cd paw-server
  sqlx migrate run
)

cargo run -p paw-server
