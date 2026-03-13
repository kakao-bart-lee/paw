#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

export PATH="/opt/homebrew/bin:$HOME/.cargo/bin:/opt/homebrew/opt/rustup/bin:$PATH"

source ./scripts/local-env.sh

need_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "missing required command: $1" >&2
    exit 1
  fi
}

need_cmd cargo
need_cmd docker
need_cmd node
need_cmd npm
need_cmd python3
FLUTTER_BIN="$(resolve_flutter_bin)"

if [ ! -f .env ]; then
  cp .env.example .env
  echo "created .env from .env.example"
fi

if ! cargo sqlx --help >/dev/null 2>&1; then
  echo "installing sqlx-cli"
  cargo install sqlx-cli --no-default-features --features rustls,postgres
fi

echo "fetching Rust dependencies"
cargo fetch

echo "installing Flutter dependencies"
(cd paw-client && "$FLUTTER_BIN" pub get)

echo "installing TypeScript SDK dependencies"
(cd adapters/paw-sdk-ts && npm install)

echo "installing OpenClaw adapter dependencies"
(cd adapters/openclaw-adapter && npm install)

if [ ! -d agents/paw-agent-sdk/.venv ]; then
  echo "creating Python virtualenv"
  python3 -m venv agents/paw-agent-sdk/.venv
fi

echo "installing Python SDK dependencies"
(
  cd agents/paw-agent-sdk
  source .venv/bin/activate
  pip install --upgrade pip
  pip install -e '.[dev]'
)

echo "local bootstrap complete"
echo "next:"
echo "  1. ./scripts/run-local-dev.sh          # server + client together"
echo "  2. ./scripts/run-local-stack.sh        # server only"
echo "  3. cd paw-client && $FLUTTER_BIN run  # client only"
echo "  4. ./scripts/stop-local-dev.sh        # stop everything"
echo "  note: if you still have an old .env, scripts auto-normalize legacy local ports"
