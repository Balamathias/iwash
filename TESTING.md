# iWash Backend - Testing Guide

## 🧪 Test Suite Overview

The iWash backend has comprehensive integration tests covering all API endpoints.

### Test Coverage

- **Health Check** (1 test)
  - API health endpoint validation

- **Authentication** (6 tests)
  - ✅ Successful user registration
  - ✅ Registration validation (invalid email, short password)
  - ✅ Successful login
  - ✅ Login with invalid credentials
  - ✅ Login with wrong password

- **Users CRUD** (11 tests)
  - ✅ Get current user profile (`/me`)
  - ✅ Unauthorized access prevention
  - ✅ Invalid token rejection
  - ✅ List all users
  - ✅ Create new user
  - ✅ Get user by ID
  - ✅ Update user (PATCH)
  - ✅ Delete user
  - ✅ Delete non-existent user (404)

**Total: 18 integration tests**

## 🚀 Running Tests

### Quick Run

```bash
./run_tests.sh
```

### Manual Run

```bash
JWT_SECRET=test_secret_key cargo test
```

### Run Specific Test Suite

```bash
# Auth tests only
JWT_SECRET=test_secret_key cargo test --test auth_tests

# Users tests only
JWT_SECRET=test_secret_key cargo test --test users_tests

# Health check only
JWT_SECRET=test_secret_key cargo test --test health_check
```

### Run Specific Test

```bash
JWT_SECRET=test_secret_key cargo test test_register_success
```

### Verbose Output

```bash
JWT_SECRET=test_secret_key cargo test -- --nocapture
```

## 📋 Test Environment

Tests use a **separate test database** to ensure isolation from production data.

### Database Setup

1. **Create test database**:
   ```bash
   psql -U postgres -c "CREATE DATABASE iwash_test;"
   ```

2. **Apply test schema**:
   ```bash
   psql -U postgres -d iwash_test -f sql/test_schema.sql
   ```

3. **Set environment variables**:
   ```bash
   export TEST_DATABASE_URL=postgres://postgres:password@localhost/iwash_test
   export JWT_SECRET=test_secret_key
   ```

### Environment Variables

- `TEST_DATABASE_URL` - Connection string for test database (separate from production)
- `JWT_SECRET` - Secret key for JWT token generation in tests

**Important**: Tests will **NOT** use your production `DATABASE_URL`. They use `TEST_DATABASE_URL` to ensure complete isolation.

### Test Database Schema

The test schema (`sql/test_schema.sql`) includes:
- `DROP TABLE IF EXISTS` statements for clean resets
- Complete table definitions matching production
- Indexes for performance

This allows tests to start with a clean slate on each run.

## 🔧 Test Utilities

All test utilities are centralized in `tests/common/mod.rs` for consistency and reusability.

### Helper Functions

Located in `tests/common/mod.rs`:

- `test_database_url()` - Returns TEST_DATABASE_URL from environment
- `create_test_app()` - Creates a test instance of the API router with test database
- `create_test_pool()` - Creates a real database pool for direct DB access
- `parse_json_response()` - Parses JSON response bodies
- `unique_email()` - Generates unique emails for test isolation (timestamp-based)
- `register_and_get_token()` - Registers a user and returns JWT token
- `cleanup_test_data()` - Truncates all tables for cleanup
- `setup_test_database()` - Applies test schema and returns pool
- `begin_test_transaction()` - Starts a transaction for isolated testing

### Test Isolation

- **Separate test database** (`iwash_test`) - Never touches production data
- **Unique email addresses** (nanosecond timestamp) - Prevents conflicts
- **Clean schema on demand** - `test_schema.sql` can reset database
- **Independent tests** - No shared state between tests
- **Parallel execution safe** - Tests can run concurrently

## 📊 Test Output Example

```
running 6 tests
test test_register_invalid_email ... ok
test test_register_short_password ... ok
test test_login_invalid_credentials ... ok
test test_register_success ... ok
test test_login_wrong_password ... ok
test test_login_success ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## 🐛 Debugging Failed Tests

### View detailed error messages

```bash
JWT_SECRET=test_secret_key RUST_BACKTRACE=1 cargo test
```

### Check specific test with output

```bash
JWT_SECRET=test_secret_key cargo test test_name -- --nocapture --test-threads=1
```

### Run tests serially (one at a time)

```bash
JWT_SECRET=test_secret_key cargo test -- --test-threads=1
```

## 📝 Writing New Tests

### Structure

```rust
#[tokio::test]
async fn test_your_feature() {
    let token = register_and_get_token(&unique_email("testuser"), "password123").await;

    let req = Request::builder()
        .uri("/api/v1/your/endpoint")
        .method("GET")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let resp = create_test_app().oneshot(req).await.unwrap();
    
    assert_eq!(resp.status(), StatusCode::OK);
    
    let body = parse_json_response(resp.into_body()).await;
    assert_eq!(body["field"], "expected_value");
}
```

### Best Practices

1. **Use unique emails** - Always use `unique_email()` to avoid conflicts
2. **Test both success and failure** - Cover happy path and error cases
3. **Verify status codes** - Assert expected HTTP status
4. **Check response bodies** - Validate JSON structure and values
5. **Test authentication** - Include both authenticated and unauthorized scenarios
6. **Clean assertions** - One logical assertion per test when possible

## 🔄 Continuous Integration

For CI/CD pipelines, ensure:

```yaml
env:
  TEST_DATABASE_URL: postgres://postgres:password@localhost:5432/iwash_test
  JWT_SECRET: ci_test_secret_key
  RUST_LOG: info

services:
  postgres:
    image: postgres:15
    env:
      POSTGRES_PASSWORD: password
      POSTGRES_DB: iwash_test

script:
  - psql $TEST_DATABASE_URL -f sql/test_schema.sql
  - cargo test --all-features
```

## 📈 Future Test Additions

- [ ] Unit tests for individual handlers
- [ ] Performance/load tests
- [ ] Security tests (SQL injection, XSS, etc.)
- [ ] Integration tests for bookings module
- [ ] API contract tests
- [ ] Test coverage reporting

## 🎯 Coverage Goals

Current: ~100% integration coverage for auth and users  
Target: Maintain >90% coverage as new features are added
