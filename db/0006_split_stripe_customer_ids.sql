-- Migration 0006: Split stripe_customer_id into mode-specific columns
-- Purpose: Prevent test/live mode mismatch errors by storing customer IDs separately
-- Safe: Backward compatible, dual-write to both old and new columns

BEGIN;

-- Add mode-specific customer ID columns
ALTER TABLE billing_accounts
  ADD COLUMN IF NOT EXISTS stripe_customer_id_live TEXT,
  ADD COLUMN IF NOT EXISTS stripe_customer_id_test TEXT;

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_billing_accounts_customer_live
  ON billing_accounts(stripe_customer_id_live)
  WHERE stripe_customer_id_live IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_billing_accounts_customer_test
  ON billing_accounts(stripe_customer_id_test)
  WHERE stripe_customer_id_test IS NOT NULL;

-- Migrate existing data: Move existing customer IDs to test column
-- Rationale: Cannot determine mode of existing IDs, safer to assume test
-- Live mode customer IDs will be created fresh on next billing access
UPDATE billing_accounts
SET stripe_customer_id_test = stripe_customer_id
WHERE stripe_customer_id IS NOT NULL
  AND stripe_customer_id_test IS NULL
  AND stripe_customer_id_live IS NULL;

-- Add comment for documentation
COMMENT ON COLUMN billing_accounts.stripe_customer_id_live IS
  'Stripe customer ID for live mode (sk_live_*)';
COMMENT ON COLUMN billing_accounts.stripe_customer_id_test IS
  'Stripe customer ID for test mode (sk_test_*)';
COMMENT ON COLUMN billing_accounts.stripe_customer_id IS
  'Legacy column, kept for backward compatibility during migration';

COMMIT;

-- Verification query (run manually to verify)
-- SELECT
--   user_id::text,
--   stripe_customer_id as legacy,
--   stripe_customer_id_test as test_col,
--   stripe_customer_id_live as live_col
-- FROM billing_accounts
-- WHERE stripe_customer_id IS NOT NULL
--    OR stripe_customer_id_test IS NOT NULL
--    OR stripe_customer_id_live IS NOT NULL
-- ORDER BY updated_at DESC
-- LIMIT 10;
