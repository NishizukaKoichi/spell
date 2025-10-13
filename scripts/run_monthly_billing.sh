#!/bin/bash
# Script to trigger monthly billing via the admin endpoint
# This should be run via cron (e.g., on the 1st of each month)

set -e

# Configuration
API_URL="${API_URL:-https://spell-platform.fly.dev}"
ADMIN_SECRET="${ADMIN_SECRET:-}"

if [ -z "$ADMIN_SECRET" ]; then
    echo "Error: ADMIN_SECRET environment variable is not set"
    exit 1
fi

echo "=== Running Monthly Billing ==="
echo "API URL: $API_URL"
echo "Timestamp: $(date -Iseconds)"
echo ""

# Call the admin billing endpoint
response=$(curl -s -w "\n%{http_code}" \
    -X POST \
    -H "X-Admin-Secret: $ADMIN_SECRET" \
    -H "Content-Type: application/json" \
    "$API_URL/admin/billing/process-monthly")

# Extract status code (last line) and body (everything else)
http_code=$(echo "$response" | tail -n1)
body=$(echo "$response" | sed '$d')

echo "Response:"
echo "$body" | jq '.' 2>/dev/null || echo "$body"
echo ""

if [ "$http_code" -eq 200 ]; then
    echo "✓ Monthly billing completed successfully (HTTP $http_code)"
    exit 0
else
    echo "✗ Monthly billing failed (HTTP $http_code)"
    exit 1
fi
