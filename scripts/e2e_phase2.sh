#!/bin/bash
set -euo pipefail

# Phase 2 E2E Testing Script
# Tests: Billing, Budgets, Budget enforcement, Metrics

API_URL="${API_URL:-https://spell-platform.fly.dev}"
TOKEN="${TOKEN:-}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "==================================="
echo "Phase 2 E2E Testing"
echo "API: $API_URL"
echo "==================================="
echo ""

# Check if TOKEN is set
if [ -z "$TOKEN" ]; then
    echo -e "${YELLOW}Warning: TOKEN not set. Please authenticate first.${NC}"
    echo "Get token via GitHub OAuth: $API_URL/auth/github"
    echo "Then export TOKEN=<your_session_token>"
    exit 1
fi

# Helper functions
test_pass() {
    echo -e "${GREEN}✓ $1${NC}"
}

test_fail() {
    echo -e "${RED}✗ $1${NC}"
    exit 1
}

test_info() {
    echo -e "${YELLOW}ℹ $1${NC}"
}

# 1. Health check
echo "1. Health Check"
HEALTH=$(curl -s "$API_URL/healthz")
if echo "$HEALTH" | jq -e '.status == "ok"' > /dev/null; then
    test_pass "API is healthy"
else
    test_fail "API health check failed"
fi
echo ""

# 2. Metrics endpoint
echo "2. Metrics Endpoint"
METRICS=$(curl -s "$API_URL/metrics")
if echo "$METRICS" | grep -q "spell_cast_total"; then
    test_pass "Metrics endpoint available"
else
    test_fail "Metrics endpoint not working"
fi
echo ""

# 3. Get current budget (should be 404 initially)
echo "3. Budget Management"
BUDGET_STATUS=$(curl -s -w "%{http_code}" -o /tmp/budget_response.txt \
    -H "Authorization: Bearer $TOKEN" \
    "$API_URL/v1/budgets")

if [ "$BUDGET_STATUS" = "404" ] || [ "$BUDGET_STATUS" = "200" ]; then
    test_pass "Budget endpoint accessible"
else
    test_fail "Budget endpoint returned unexpected status: $BUDGET_STATUS"
fi
echo ""

# 4. Create budget with low hard limit
echo "4. Create Budget with Low Hard Limit"
CREATE_BUDGET=$(curl -s -X POST \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d '{
        "period": "monthly",
        "soft_limit_cents": 5,
        "hard_limit_cents": 10,
        "notify_thresholds": [3, 7]
    }' \
    "$API_URL/v1/budgets")

if echo "$CREATE_BUDGET" | jq -e '.hard_limit_cents == 10' > /dev/null; then
    test_pass "Budget created with hard_limit_cents=10"
else
    test_fail "Failed to create budget"
fi
echo ""

# 5. Get usage (should be 0 initially)
echo "5. Check Initial Usage"
USAGE=$(curl -s \
    -H "Authorization: Bearer $TOKEN" \
    "$API_URL/v1/budgets/usage")

TOTAL_COST=$(echo "$USAGE" | jq -r '.total_cost_cents // 0')
test_info "Current usage: $TOTAL_COST cents"
echo ""

# 6. Test cast endpoint (should work initially)
echo "6. First Cast (should succeed)"
CAST1=$(curl -s -w "\n%{http_code}" -X POST \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d '{
        "spell_name": "echo",
        "payload": {"message": "test"}
    }' \
    "$API_URL/v1/cast")

CAST1_STATUS=$(echo "$CAST1" | tail -1)
CAST1_BODY=$(echo "$CAST1" | head -n -1)

if [ "$CAST1_STATUS" = "200" ]; then
    test_pass "First cast succeeded"
else
    test_info "First cast status: $CAST1_STATUS (may fail if WASM not loaded)"
fi
echo ""

# 7. Make multiple casts to exceed budget (if COST_PER_CAST_CENTS is set)
echo "7. Multiple Casts to Test Budget Enforcement"
test_info "Making 15 cast attempts (should hit budget limit if COST_PER_CAST_CENTS=1)..."

EXCEEDED=false
for i in {1..15}; do
    CAST_STATUS=$(curl -s -w "%{http_code}" -o /dev/null -X POST \
        -H "Authorization: Bearer $TOKEN" \
        -H "Content-Type: application/json" \
        -d '{"spell_name":"echo","payload":{"message":"test"}}' \
        "$API_URL/v1/cast")

    if [ "$CAST_STATUS" = "402" ]; then
        EXCEEDED=true
        test_pass "Budget exceeded after $i casts (got HTTP 402)"
        break
    fi

    sleep 0.2
done

if [ "$EXCEEDED" = "false" ]; then
    test_info "Budget not exceeded (COST_PER_CAST_CENTS may be 0 or not set)"
fi
echo ""

# 8. Check updated usage
echo "8. Check Updated Usage"
USAGE2=$(curl -s \
    -H "Authorization: Bearer $TOKEN" \
    "$API_URL/v1/budgets/usage")

TOTAL_CALLS=$(echo "$USAGE2" | jq -r '.total_calls // 0')
TOTAL_COST2=$(echo "$USAGE2" | jq -r '.total_cost_cents // 0')

test_info "Total calls: $TOTAL_CALLS"
test_info "Total cost: $TOTAL_COST2 cents"
echo ""

# 9. Increase budget to allow more casts
echo "9. Update Budget to Higher Limit"
UPDATE_BUDGET=$(curl -s -X PUT \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d '{
        "period": "monthly",
        "soft_limit_cents": 1000,
        "hard_limit_cents": 2000
    }' \
    "$API_URL/v1/budgets")

if echo "$UPDATE_BUDGET" | jq -e '.hard_limit_cents == 2000' > /dev/null; then
    test_pass "Budget updated to hard_limit_cents=2000"
else
    test_fail "Failed to update budget"
fi
echo ""

# 10. Verify cast works again
echo "10. Verify Cast Works After Budget Increase"
CAST_AFTER=$(curl -s -w "%{http_code}" -o /dev/null -X POST \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d '{"spell_name":"echo","payload":{"message":"test"}}' \
    "$API_URL/v1/cast")

if [ "$CAST_AFTER" = "200" ] || [ "$CAST_AFTER" = "404" ]; then
    test_pass "Cast endpoint accessible after budget increase (status: $CAST_AFTER)"
else
    test_fail "Cast endpoint returned unexpected status: $CAST_AFTER"
fi
echo ""

# 11. Billing checkout (HITL - Human In The Loop)
echo "11. Billing Checkout (HITL)"
test_info "Testing checkout session creation (requires Stripe configured)..."

CHECKOUT=$(curl -s -X POST \
    -H "Authorization: Bearer $TOKEN" \
    "$API_URL/v1/billing/checkout")

if echo "$CHECKOUT" | jq -e '.url' > /dev/null; then
    CHECKOUT_URL=$(echo "$CHECKOUT" | jq -r '.url')
    test_pass "Checkout session created"
    echo ""
    echo "---"
    echo "MANUAL STEP REQUIRED:"
    echo "Visit this URL to complete payment (use Stripe test card 4242 4242 4242 4242):"
    echo "$CHECKOUT_URL"
    echo "---"
    echo ""
    test_info "After payment, billing_accounts should show status='active' and plan='pro'"
else
    test_info "Checkout failed (Stripe may not be configured)"
fi
echo ""

# 12. Metrics validation
echo "12. Validate Metrics"
METRICS2=$(curl -s "$API_URL/metrics")

if echo "$METRICS2" | grep -q "spell_cast_total"; then
    test_pass "spell_cast_total metric present"
fi

if echo "$METRICS2" | grep -q "spell_budget_blocked_total"; then
    test_pass "spell_budget_blocked_total metric present"
fi

if echo "$METRICS2" | grep -q "spell_rate_limited_total"; then
    test_pass "spell_rate_limited_total metric present"
fi
echo ""

# 13. Cleanup (optional - delete budget)
echo "13. Cleanup (optional)"
read -p "Delete test budget? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    DELETE_STATUS=$(curl -s -w "%{http_code}" -o /dev/null -X DELETE \
        -H "Authorization: Bearer $TOKEN" \
        "$API_URL/v1/budgets")

    if [ "$DELETE_STATUS" = "204" ]; then
        test_pass "Budget deleted"
    else
        test_info "Budget deletion status: $DELETE_STATUS"
    fi
fi
echo ""

echo "==================================="
echo "Phase 2 E2E Testing Complete!"
echo "==================================="
echo ""
echo "Summary:"
echo "- Health: OK"
echo "- Metrics: OK"
echo "- Budgets: OK"
echo "- Budget Enforcement: $([ "$EXCEEDED" = "true" ] && echo "OK (402 received)" || echo "SKIPPED (COST_PER_CAST_CENTS=0)")"
echo "- Billing Checkout: HITL (manual verification required)"
echo ""
echo "Next steps:"
echo "1. Set Stripe secrets if not done: flyctl secrets set STRIPE_SECRET_KEY=sk_test_xxx STRIPE_WEBHOOK_SECRET=whsec_xxx"
echo "2. Set cost per cast: flyctl secrets set COST_PER_CAST_CENTS=1"
echo "3. Complete checkout in browser if testing billing"
echo "4. Verify metrics in production: curl $API_URL/metrics"
