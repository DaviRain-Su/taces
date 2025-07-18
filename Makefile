.PHONY: help db-up db-down db-reset db-seed test test-unit test-integration run dev

help:
	@echo "Available commands:"
	@echo "  make db-up          - Start MySQL databases"
	@echo "  make db-down        - Stop MySQL databases"
	@echo "  make db-reset       - Reset databases and run migrations"
	@echo "  make db-seed        - Seed database with test data"
	@echo "  make test           - Run all tests"
	@echo "  make test-unit      - Run unit tests"
	@echo "  make test-integration - Run integration tests"
	@echo "  make run            - Run the backend server"
	@echo "  make dev            - Start databases and run server"

# Database commands
db-up:
	docker-compose up -d mysql mysql-test adminer
	@echo "Waiting for MySQL to be ready..."
	@sleep 10

db-down:
	docker-compose down

db-reset: db-down db-up
	@echo "Databases reset successfully"

db-seed:
	cd backend && cargo run --bin seed

# Test commands
test: test-unit test-integration

test-unit:
	cd backend && cargo test --lib

test-integration:
	cd backend && cargo test --test '*' -- --test-threads=1

# Run commands
run:
	cd backend && cargo run

dev: db-up
	cd backend && cargo run

# Build commands
build:
	cd backend && cargo build --release

clean:
	cd backend && cargo clean
	docker-compose down -v