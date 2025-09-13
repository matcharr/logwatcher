#!/bin/bash

# LogWatcher Basic Usage Examples

echo "Creating sample log files..."

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

echo "Sample files created: app.log, nginx.log"
echo ""
echo "Example 1: Basic error monitoring"
echo "logwatcher -f app.log"
echo ""

echo "Example 2: Multiple files with custom patterns"
echo "logwatcher -f app.log -f nginx.log -p \"ERROR,404,500\" --color-map \"404:yellow,500:red\""
echo ""

echo "Example 3: Dry-run mode"
echo "logwatcher -f app.log --dry-run -p \"ERROR,WARN,CRITICAL\""
echo ""

echo "Example 4: Regex patterns"
echo "logwatcher -f app.log -r -p \"user_id=\\\\d+|session_\\\\w+\""
echo ""

echo "Example 5: Quiet mode with notifications"
echo "logwatcher -f app.log -q -p \"ERROR,CRITICAL\" --notify"
echo ""

echo "Run any of these examples to see LogWatcher in action!"
echo "Press Ctrl+C to stop monitoring."
