.PHONY: dev server test lint fmt clean docker-up docker-down

# Start development server
dev: docker-up server

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
