use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Spell {
    pub id: Uuid,
    pub name: String,
    pub creator_id: Uuid,
    pub description: Option<String>,
    pub price_cents: i32,
    pub wasm_path: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSpellRequest {
    pub name: String,
    pub description: Option<String>,
    pub price_cents: i32,
}

#[derive(Debug, Serialize)]
pub struct SpellResponse {
    pub id: Uuid,
    pub name: String,
    pub creator_id: Uuid,
    pub description: Option<String>,
    pub price_cents: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

impl From<Spell> for SpellResponse {
    fn from(spell: Spell) -> Self {
        Self {
            id: spell.id,
            name: spell.name,
            creator_id: spell.creator_id,
            description: spell.description,
            price_cents: spell.price_cents,
            is_active: spell.is_active,
            created_at: spell.created_at,
        }
    }
}
