.DEFAULT_GOAL := help

.PHONY: help bootstrap-local local-stack dev server test lint fmt clean docker-up docker-down migrate migrate-add

help:
	@printf "%s\n" \
	"Available targets:" \
	"  make bootstrap-local  # install local dependencies and prepare the repo" \
	"  make local-stack      # start postgres/minio, run migrations, start server" \
	"  make dev              # docker-up + server" \
	"  make server           # run paw-server" \
	"  make test             # run Rust workspace tests" \
	"  make lint             # run clippy" \
	"  make fmt              # run rustfmt" \
	"  make docker-up        # start docker services" \
	"  make docker-down      # stop docker services" \
	"  make migrate          # run SQLx migrations" \
	"  make migrate-add name=example  # create a new migration"

# Start development server
dev: docker-up server

# Bootstrap local development environment
bootstrap-local:
	./scripts/bootstrap-local.sh

# Start local dependency stack, run migrations, then start server
local-stack:
	./scripts/run-local-stack.sh

# Start Rust server
server:
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
