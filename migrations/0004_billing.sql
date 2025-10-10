-- Phase 2: Billing, Budgets, Usage Tracking

-- billing_accounts: Stripe customer and subscription info
CREATE TABLE IF NOT EXISTS billing_accounts (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    stripe_customer_id TEXT UNIQUE,
    plan TEXT NOT NULL DEFAULT 'free' CHECK (plan IN ('free', 'pro')),
    status TEXT NOT NULL DEFAULT 'inactive' CHECK (status IN ('active', 'inactive', 'canceled', 'past_due')),
    current_period_end TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_billing_accounts_stripe_customer ON billing_accounts(stripe_customer_id);
CREATE INDEX idx_billing_accounts_status ON billing_accounts(status);

-- usage_counters: Track usage per time window for billing
CREATE TABLE IF NOT EXISTS usage_counters (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    window_start TIMESTAMPTZ NOT NULL,
    window_end TIMESTAMPTZ NOT NULL,
    calls INTEGER NOT NULL DEFAULT 0,
    cost_cents INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, window_start),
    CONSTRAINT valid_window CHECK (window_end > window_start)
);

CREATE INDEX idx_usage_counters_window ON usage_counters(window_start, window_end);

-- budgets: User-defined spending limits
CREATE TABLE IF NOT EXISTS budgets (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    period TEXT NOT NULL DEFAULT 'monthly' CHECK (period IN ('daily', 'monthly')),
    soft_limit_cents INTEGER,
    hard_limit_cents INTEGER,
    notify_thresholds_json JSONB DEFAULT '[]'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT valid_limits CHECK (
        (soft_limit_cents IS NULL OR soft_limit_cents >= 0) AND
        (hard_limit_cents IS NULL OR hard_limit_cents >= 0) AND
        (soft_limit_cents IS NULL OR hard_limit_cents IS NULL OR soft_limit_cents <= hard_limit_cents)
    )
);

-- Add cost tracking to casts
ALTER TABLE casts ADD COLUMN IF NOT EXISTS cost_cents INTEGER;

-- Create index for cost queries
CREATE INDEX IF NOT EXISTS idx_casts_user_created ON casts(user_id, created_at);

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Triggers for updated_at
CREATE TRIGGER update_billing_accounts_updated_at BEFORE UPDATE ON billing_accounts
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_usage_counters_updated_at BEFORE UPDATE ON usage_counters
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_budgets_updated_at BEFORE UPDATE ON budgets
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
