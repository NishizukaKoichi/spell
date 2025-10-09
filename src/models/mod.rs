use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Cast {
    pub id: Uuid,
    pub spell_name: String,
    pub payload: serde_json::Value,
    pub status: String,
    pub result: Option<serde_json::Value>,
    pub error_code: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CastRequest {
    pub spell_name: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct CastResponse {
    pub id: Uuid,
    pub status: String,
    pub result: Option<serde_json::Value>,
    pub error_code: Option<String>,
    pub created_at: DateTime<Utc>,
}
