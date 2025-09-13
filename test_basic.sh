#!/bin/bash

# Basic test script for LogWatcher
# This script creates test files and runs basic functionality tests

set -e

echo "🧪 LogWatcher Basic Tests"
echo "========================="

# Create test directory
TEST_DIR="test_output"
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

echo "📁 Creating test files..."

# Create sample log files
cat > app.log << EOF
[2025-01-07 15:00:01] Starting application server...
[2025-01-07 15:00:02] Database connection established
[2025-01-07 15:00:03] INFO User login successful
[2025-01-07 15:00:04] WARN Memory usage at 85%
[2025-01-07 15:00:05] ERROR Failed to bind to port 8080
[2025-01-07 15:00:06] CRITICAL Database connection lost
[2025-01-07 15:00:07] INFO Recovery process started
EOF

cat > nginx.log << EOF
[2025-01-07 15:00:01] 200 GET /api/users
[2025-01-07 15:00:02] 404 GET /api/nonexistent
[2025-01-07 15:00:03] 500 POST /api/orders
[2025-01-07 15:00:04] 200 GET /api/products
[2025-01-07 15:00:05] 403 GET /api/admin
EOF

echo "✅ Test files created"
echo ""

# Test 1: Basic dry-run
echo "🔍 Test 1: Basic dry-run with ERROR pattern"
echo "Command: logwatcher -f app.log --dry-run -p ERROR --no-color"
echo "Expected: 2 ERROR lines should be highlighted"
echo ""

# Test 2: Multiple patterns
echo "🔍 Test 2: Multiple patterns (ERROR, WARN)"
echo "Command: logwatcher -f app.log --dry-run -p ERROR,WARN --no-color"
echo "Expected: 2 ERROR lines + 1 WARN line should be highlighted"
echo ""

# Test 3: Case insensitive
echo "🔍 Test 3: Case insensitive matching"
echo "Command: logwatcher -f app.log --dry-run -p error --case-insensitive --no-color"
echo "Expected: 2 ERROR lines should be highlighted"
echo ""

# Test 4: Multiple files
echo "🔍 Test 4: Multiple files"
echo "Command: logwatcher -f app.log -f nginx.log --dry-run -p ERROR,404 --no-color"
echo "Expected: ERROR lines from app.log + 404 line from nginx.log"
echo ""

# Test 5: Quiet mode
echo "🔍 Test 5: Quiet mode"
echo "Command: logwatcher -f app.log --dry-run -p ERROR --quiet --no-color"
echo "Expected: Only ERROR lines, no other lines"
echo ""

# Test 6: Regex pattern
echo "🔍 Test 6: Regex pattern"
echo "Command: logwatcher -f nginx.log --dry-run -r -p '\\d{3}' --no-color"
echo "Expected: All lines with 3-digit numbers (200, 404, 500, 403)"
echo ""

echo "📋 Test Summary:"
echo "- Created test files: app.log, nginx.log"
echo "- Ready to run LogWatcher tests"
echo "- Use 'cd $TEST_DIR' to access test files"
echo ""
echo "🚀 To run tests, first build LogWatcher:"
echo "   cargo build --release"
echo ""
echo "Then run individual tests:"
echo "   ../target/release/logwatcher -f app.log --dry-run -p ERROR --no-color"
echo ""
echo "🧹 Cleanup:"
echo "   cd .. && rm -rf $TEST_DIR"
