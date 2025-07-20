#\!/bin/bash
# Run tests with proper setup

echo "Setting up test database..."
export DATABASE_URL="mysql://tcm_user:tcm_pass123@localhost:3307/tcm_telemedicine_test"

echo "Running migrations..."
sqlx migrate run

echo "Seeding test data..."
cargo run --bin seed

echo "Running tests..."
cargo test "$@"

