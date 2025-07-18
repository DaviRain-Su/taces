# TCM Telemedicine Backend Testing Guide

## Overview
This guide covers how to run and write tests for the TCM Telemedicine backend.

## Test Structure

```
tests/
├── unit/                # Unit tests for individual functions
│   ├── test_password.rs # Password hashing tests
│   └── test_jwt.rs      # JWT token tests
├── integration/         # Integration tests for API endpoints
│   ├── test_auth.rs     # Authentication endpoint tests
│   ├── test_user.rs     # User management tests
│   └── test_doctor.rs   # Doctor management tests
└── common/             # Shared test utilities
    └── mod.rs          # Test app setup and HTTP helpers
```

## Running Tests

### Prerequisites
1. Docker and Docker Compose installed
2. MySQL database running (via Docker Compose)

### Quick Start

```bash
# Start test database
make db-up

# Run all tests
make test

# Or use the test script
cd backend
./run_tests.sh
```

### Running Specific Tests

```bash
# Run only unit tests
cargo test --lib

# Run only integration tests
cargo test --test '*' -- --test-threads=1

# Run a specific test
cargo test test_register_success

# Run tests with output
cargo test -- --nocapture
```

## Test Database

The test suite uses a separate MySQL instance running on port 3307 to avoid conflicts with the development database.

### Database Configuration
- Host: localhost
- Port: 3307
- Database: tcm_telemedicine_test
- User: tcm_user
- Password: tcm_pass123

### Reset Test Database

```bash
# Reset and re-run migrations
docker-compose down mysql-test
docker-compose up -d mysql-test
```

## Writing Tests

### Unit Tests

Unit tests are located in `tests/unit/` and test individual functions in isolation.

Example:
```rust
#[test]
fn test_hash_password() {
    let password = "test_password123";
    let hashed = hash_password(password).unwrap();
    
    assert_ne!(password, hashed);
    assert!(hashed.len() > 50);
}
```

### Integration Tests

Integration tests are located in `tests/integration/` and test full API endpoints.

Example:
```rust
#[tokio::test]
async fn test_register_success() {
    let mut app = TestApp::new().await;
    
    let user_dto = CreateUserDto {
        account: "test_user".to_string(),
        name: "测试用户".to_string(),
        password: "password123".to_string(),
        // ... other fields
    };
    
    let (status, body) = app.post("/api/v1/auth/register", user_dto).await;
    
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["success"], true);
}
```

### Test Utilities

The `test_helpers` module provides utilities for creating test data:

```rust
// Create a test user
let (user_id, account, password) = create_test_user(&pool, "patient").await;

// Create a test doctor
let doctor_id = create_test_doctor(&pool, user_id).await;
```

## Test Coverage

To generate test coverage reports:

```bash
# Install cargo-tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage

# View report
open coverage/tarpaulin-report.html
```

## CI/CD

Tests are automatically run on GitHub Actions for:
- Every push to main/develop branches
- Every pull request

The CI pipeline:
1. Sets up MySQL test database
2. Runs all tests
3. Checks code formatting (cargo fmt)
4. Runs linting (cargo clippy)
5. Builds release binary

## Troubleshooting

### MySQL Connection Issues
```bash
# Check if MySQL is running
docker-compose ps

# View MySQL logs
docker-compose logs mysql-test

# Test MySQL connection
mysql -h 127.0.0.1 -P 3307 -u tcm_user -ptcm_pass123 tcm_telemedicine_test
```

### Test Failures
```bash
# Run tests with more output
RUST_LOG=debug cargo test -- --nocapture

# Run a single test with backtrace
RUST_BACKTRACE=1 cargo test test_name -- --exact
```

### Clean Test Environment
```bash
# Clean build artifacts
cargo clean

# Reset database
make db-reset

# Full clean
make clean
```

## Test Data

The seed script (`src/bin/seed.rs`) creates test data including:
- Admin user: admin / admin123
- Doctor users: doctor_dong / doctor123, doctor_wang / doctor123
- Patient users: patient1-10 / patient123
- Sample appointments and prescriptions

Run seed data:
```bash
make db-seed
# or
cargo run --bin seed
```

## Best Practices

1. **Isolation**: Each test should be independent and not rely on other tests
2. **Cleanup**: Tests should clean up after themselves
3. **Naming**: Use descriptive test names that explain what is being tested
4. **Assertions**: Include multiple assertions to thoroughly test the behavior
5. **Error Cases**: Test both success and failure scenarios

## Performance Testing

For load testing, consider using tools like:
- [drill](https://github.com/fcsonline/drill)
- [vegeta](https://github.com/tsenart/vegeta)
- [k6](https://k6.io/)

Example with drill:
```yaml
# drill.yml
benchmark:
  - name: Login endpoint
    request:
      method: POST
      url: http://localhost:3000/api/v1/auth/login
      body: '{"account": "patient1", "password": "patient123"}'
      headers:
        Content-Type: application/json
```

Run: `drill --benchmark drill.yml --stats`