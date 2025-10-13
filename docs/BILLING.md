# Monthly Billing System

## Overview

The Spell Platform implements automated monthly billing using Stripe Invoices. At the end of each month, the system generates invoices for all users based on their API usage.

## How It Works

1. **Usage Tracking**: Every API call is recorded in the `usage_counters` table with its cost in cents
2. **Monthly Processing**: On the 1st of each month, the billing service:
   - Aggregates usage for each user from the previous month
   - Creates Stripe invoice items for each user with usage
   - Generates and finalizes Stripe invoices
   - Stripe automatically charges the user's saved payment method
3. **Notifications**: Users receive email notifications from Stripe about their invoices

## Architecture

### Backend Components

- **`BillingService`** (`src/services/billing_service.rs`):
  - `process_monthly_billing()`: Main entry point for monthly billing
  - Queries all users with usage in the previous month
  - Creates invoices for each user via Stripe

- **`StripeService`** (`src/services/stripe_service.rs`):
  - `create_monthly_invoice()`: Creates invoice items and finalizes invoices
  - Handles all Stripe API interactions

- **Admin Endpoint** (`src/routes/admin.rs`):
  - `POST /admin/billing/process-monthly`: Triggers monthly billing
  - Protected by `X-Admin-Secret` header

### Automation

**GitHub Actions** (`.github/workflows/monthly-billing.yml`):
- Runs on the 1st of each month at 00:00 UTC
- Can be manually triggered via workflow_dispatch
- Calls the admin endpoint to process billing

**Manual Execution** (`scripts/run_monthly_billing.sh`):
```bash
export ADMIN_SECRET="your-secret-here"
export API_URL="https://spell-platform.fly.dev"
./scripts/run_monthly_billing.sh
```

## Setup

### 1. Set Admin Secret

```bash
# Generate a secure random secret
openssl rand -hex 32

# Set as environment variable in production
flyctl secrets set ADMIN_SECRET=<generated-secret>
```

### 2. Configure GitHub Actions Secret

In your GitHub repository settings:
1. Go to Settings → Secrets and variables → Actions
2. Add `ADMIN_SECRET` with the same value as above

### 3. Verify Stripe Configuration

Ensure the following environment variables are set:
- `STRIPE_API_KEY`: Your Stripe secret key
- `STRIPE_WEBHOOK_SECRET`: Stripe webhook signing secret

## Invoice Details

Each invoice includes:
- **Description**: "Spell Platform API Usage - [Month Year] ([N] calls, $[Amount])"
- **Line Items**: Single line item with total usage cost
- **Collection**: Automatic charge to saved payment method
- **Period**: Previous calendar month

## Testing

### Manual Test

```bash
# Set test environment
export ADMIN_SECRET="test-secret"
export API_URL="http://localhost:8080"

# Run billing script
./scripts/run_monthly_billing.sh
```

### Verify in Stripe Dashboard

1. Navigate to Stripe Dashboard → Invoices
2. Check for newly created invoices
3. Verify amounts match usage costs

## Monitoring

### Logs

Check application logs for billing events:
```bash
flyctl logs -a spell-platform | grep "monthly billing"
```

Expected log entries:
- `Starting monthly billing process`
- `Processing billing for period: YYYY-MM-DD to YYYY-MM-DD`
- `Found N users with usage`
- `Created invoice <id> for user <id> ($X.XX, N calls)`
- `Monthly billing completed: N successful, N errors`

### Error Handling

If billing fails for a user:
- Error is logged with user ID and reason
- Processing continues for other users
- Summary shows error count

## Security

- **Admin Secret**: Required for all admin endpoints
- **Header-based Auth**: `X-Admin-Secret` header must match `ADMIN_SECRET` env var
- **No UI**: Admin endpoints are not exposed in the frontend
- **Audit Logging**: All billing operations are logged with user IDs

## Troubleshooting

### Invoice Not Created

Check:
1. User has usage in the previous month
2. User has a Stripe customer ID in `billing_accounts`
3. Stripe API key is valid
4. Check logs for specific error messages

### Payment Failed

- Stripe will automatically retry failed payments
- User receives email notification from Stripe
- Check Stripe Dashboard for payment status

### Duplicate Billing

The system does not prevent duplicate runs. To avoid:
- Only run once per month
- Check Stripe Dashboard before manual execution
- Review logs for recent billing runs

## Future Enhancements

- **Invoice History API**: Endpoint to retrieve past invoices
- **Dashboard Display**: Show invoice history in user dashboard
- **Email Notifications**: Custom email notifications from the platform
- **Retry Logic**: Automatic retry for failed payments
- **Prorated Billing**: Support for mid-month plan changes
