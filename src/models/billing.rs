use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BillingAccount {
    pub user_id: Uuid,
    pub stripe_customer_id: Option<String>,
    pub plan: String,
    pub status: String,
    pub current_period_end: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UsageCounter {
    pub user_id: Uuid,
    pub window_start: DateTime<Utc>,
    pub window_end: DateTime<Utc>,
    pub calls: i32,
    pub cost_cents: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Budget {
    pub user_id: Uuid,
    pub period: String,
    pub soft_limit_cents: Option<i32>,
    pub hard_limit_cents: Option<i32>,
    pub notify_thresholds_json: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Request/Response types
#[derive(Debug, Deserialize)]
pub struct CreateBudgetRequest {
    pub period: Option<String>,
    pub soft_limit_cents: Option<i32>,
    pub hard_limit_cents: Option<i32>,
    pub notify_thresholds: Option<Vec<i32>>,
}

#[derive(Debug, Serialize)]
pub struct BudgetResponse {
    pub user_id: Uuid,
    pub period: String,
    pub soft_limit_cents: Option<i32>,
    pub hard_limit_cents: Option<i32>,
    pub notify_thresholds: Vec<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct CheckoutSessionResponse {
    pub url: String,
    pub session_id: String,
}

#[derive(Debug, Serialize)]
pub struct BudgetExceededError {
    pub error: String,
    pub period: String,
    pub hard_limit_cents: i32,
    pub spent_cents: i32,
}

impl BudgetExceededError {
    pub fn new(period: String, hard_limit_cents: i32, spent_cents: i32) -> Self {
        Self {
            error: "budget_exceeded".to_string(),
            period,
            hard_limit_cents,
            spent_cents,
        }
    }
}
