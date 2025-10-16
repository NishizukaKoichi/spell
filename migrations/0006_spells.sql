-- Phase 3: Spells metadata and pricing

-- spells: Store spell metadata including pricing
CREATE TABLE IF NOT EXISTS spells (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,
    creator_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    description TEXT,
    price_cents INTEGER NOT NULL DEFAULT 0 CHECK (price_cents >= 0),
    wasm_path TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_spells_name ON spells(name);
CREATE INDEX idx_spells_creator ON spells(creator_id);
CREATE INDEX idx_spells_active ON spells(is_active) WHERE is_active = true;

-- Trigger for updated_at
CREATE TRIGGER update_spells_updated_at BEFORE UPDATE ON spells
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Add spell_id reference to casts table
ALTER TABLE casts ADD COLUMN IF NOT EXISTS spell_id UUID REFERENCES spells(id) ON DELETE SET NULL;
CREATE INDEX IF NOT EXISTS idx_casts_spell ON casts(spell_id);
