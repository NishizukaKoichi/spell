-- Create casts table
CREATE TABLE IF NOT EXISTS casts (
    id UUID PRIMARY KEY,
    spell_name TEXT NOT NULL,
    payload JSONB NOT NULL,
    status TEXT NOT NULL,
    result JSONB,
    error_code TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes
CREATE INDEX idx_casts_spell_name ON casts(spell_name);
CREATE INDEX idx_casts_status ON casts(status);
CREATE INDEX idx_casts_created_at ON casts(created_at DESC);
