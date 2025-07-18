#!/bin/bash

# TCM Telemedicine Backend Test Runner

set -e

echo "🏥 TCM Telemedicine Backend Test Suite"
echo "======================================"

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Check if MySQL is running
if ! docker-compose ps | grep -q "tcm_mysql_test.*Up"; then
    echo "Starting test database..."
    docker-compose up -d mysql-test
    echo "Waiting for MySQL to be ready..."
    sleep 10
fi

# Run database migrations
echo -e "\n${GREEN}Running database migrations...${NC}"
mysql -h 127.0.0.1 -P 3307 -u tcm_user -ptcm_pass123 tcm_telemedicine_test < migrations/20240101000001_create_initial_tables.sql 2>/dev/null || true

# Set test environment variables
export TEST_DATABASE_URL="mysql://tcm_user:tcm_pass123@localhost:3307/tcm_telemedicine_test"
export JWT_SECRET="test_jwt_secret"
export JWT_EXPIRATION="3600"

# Run unit tests
echo -e "\n${GREEN}Running unit tests...${NC}"
cargo test --lib

# Run integration tests
echo -e "\n${GREEN}Running integration tests...${NC}"
cargo test --test '*' -- --test-threads=1

# Run linting
echo -e "\n${GREEN}Running cargo fmt check...${NC}"
cargo fmt -- --check

echo -e "\n${GREEN}Running cargo clippy...${NC}"
cargo clippy -- -D warnings

# Generate test coverage (optional, requires cargo-tarpaulin)
if command -v cargo-tarpaulin &> /dev/null; then
    echo -e "\n${GREEN}Generating test coverage...${NC}"
    cargo tarpaulin --out Html --output-dir coverage
    echo "Coverage report generated in coverage/tarpaulin-report.html"
fi

echo -e "\n${GREEN}✅ All tests passed!${NC}"