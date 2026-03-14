.DEFAULT_GOAL := help

.PHONY: help bootstrap-local local-stack dev stop server test lint fmt clean docker-up docker-down migrate migrate-add bindings core-android core-ios verify e2e-core-phase3

help:
	@printf "%s\n" \
	"Available targets:" \
	"  make bootstrap-local  # install local dependencies and prepare the repo" \
	"  make dev              # start server + client together" \
	"  make stop             # stop local server/client processes and docker services" \
	"  make local-stack      # start postgres/minio, run migrations, start server only" \
	"  make server           # run paw-server only" \
	"  make test             # run Rust workspace tests" \
	"  make lint             # run clippy" \
	"  make fmt              # run rustfmt" \
	"  make docker-up        # start docker services" \
	"  make docker-down      # stop docker services" \
	"  make migrate          # run SQLx migrations" \
	"  make migrate-add name=example  # create a new migration" \
	"  make bindings         # generate UniFFI Kotlin/Swift bindings for paw-core" \
	"  make core-android     # build paw-core Android artifacts" \
	"  make core-ios         # build paw-core iOS artifacts" \
	"  make verify           # run pre-completion checks (fmt, clippy, build, test, arch)" \
	"  make e2e-core-phase3  # run paw-core live Phase 3 smoke against local server"

# Start full local development environment
# usage: make dev [device=chrome]
dev:
	./scripts/run-local-dev.sh $(if $(device),$(device),chrome)

# Stop full local development environment
stop:
	./scripts/stop-local-dev.sh

# Bootstrap local development environment
bootstrap-local:
	./scripts/bootstrap-local.sh

# Start local dependency stack, run migrations, then start server
local-stack:
	./scripts/run-local-stack.sh

# Start Rust server only
server:
	@if [ -f .env ]; then \
	  set -a; . ./.env; set +a; \
	fi; \
	cargo run -p paw-server

# Run all tests
test:
	cargo test --workspace

# Lint
lint:
	cargo clippy --workspace -- -D warnings

# Format
fmt:
	cargo fmt --all

# Clean
clean:
	cargo clean

# Docker
docker-up:
	docker compose up -d

docker-down:
	docker compose down

# Database migrations
migrate:
	cd paw-server && cargo sqlx migrate run

migrate-add:
	cd paw-server && cargo sqlx migrate add $(name)

bindings:
	./scripts/gen-ffi-bindings.sh

core-android:
	./scripts/build-core-android.sh

core-ios:
	./scripts/build-core-ios.sh

verify:
	./scripts/verify.sh

e2e-core-phase3:
	./scripts/run-paw-core-phase3-live-smoke.sh
