#!/bin/bash

# Test CI locally
echo "Testing CI configuration locally..."

# Change to backend directory
cd backend

# Build
echo "Building..."
cargo build --verbose

# Run tests
echo "Running tests..."
export DATABASE_URL="mysql://tcm_user:tcm_pass123@localhost:3306/tcm_telemedicine"
export TEST_DATABASE_URL="mysql://tcm_user:tcm_pass123@localhost:3307/tcm_telemedicine_test"
export JWT_SECRET="test_jwt_secret_key"
export JWT_EXPIRATION="3600"

cargo test --all-features -- --test-threads=1

# Check formatting
echo "Checking formatting..."
cargo fmt -- --check

# Run clippy
echo "Running clippy..."
cargo clippy -- -D warnings

echo "CI checks completed!"