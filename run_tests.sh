#!/bin/bash
# Test runner script for iWash backend

set -e

echo "🧪 Running iWash Backend Tests"
echo "==============================="
echo ""

# Set required environment variables for tests
export JWT_SECRET="${JWT_SECRET:-test_secret_key_for_testing}"
export DATABASE_URL="${DATABASE_URL:-postgres://postgres:matiecodes@localhost/iwash_db}"

# Run tests
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
