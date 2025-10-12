-- Phase 5.3: Add payment_method_id to billing_accounts

ALTER TABLE billing_accounts
ADD COLUMN IF NOT EXISTS payment_method_id TEXT;

CREATE INDEX IF NOT EXISTS idx_billing_accounts_payment_method
ON billing_accounts(payment_method_id);

-- Update budgets table to support unique constraint on (user_id, period)
ALTER TABLE budgets DROP CONSTRAINT IF EXISTS budgets_pkey;
ALTER TABLE budgets ADD PRIMARY KEY (user_id, period);
