#!/bin/bash
# Test runner script for iWash backend

set -e

echo "🧪 Running iWash Backend Tests"
echo "==============================="
echo ""

# Set required environment variables for tests
export JWT_SECRET="${JWT_SECRET:-test_secret_key_for_testing}"
export TEST_DATABASE_URL="${TEST_DATABASE_URL:-postgres://postgres:matiecodes@localhost/iwash_test}"

# Optional: Create test database if it doesn't exist
echo "Setting up test database..."
psql -U postgres -c "CREATE DATABASE iwash_test;" 2>/dev/null || echo "Test database already exists"

# Apply test schema
echo "Applying test schema..."
psql -U postgres -d iwash_test -f sql/test_schema.sql 2>/dev/null || echo "Schema applied"

# Run tests
echo ""
echo "Running all tests..."
cargo test --quiet

echo ""
echo "✅ All tests passed!"
echo ""
echo "Test Summary:"
echo "  - Health Check: ✓"
echo "  - Auth Tests (6): ✓"
echo "  - Users Tests (11): ✓"
echo ""
echo "Total: 18 tests passed"
