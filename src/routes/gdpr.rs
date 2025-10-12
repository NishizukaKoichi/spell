// GDPR/CCPA/Japanese Personal Information Protection Act Compliance
// Implements §30 data deletion and export requirements

use actix_web::{web, HttpMessage, HttpRequest, HttpResponse, Result};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::User;

// ============================================================================
// Data Deletion (GDPR Article 17 - Right to Erasure)
// ============================================================================

/// DELETE /v1/users/me
///
/// Permanently deletes all user data including:
/// - User profile
/// - Sessions (ON DELETE CASCADE)
/// - API keys (ON DELETE CASCADE)
/// - Billing account (ON DELETE CASCADE)
/// - Usage counters (ON DELETE CASCADE)
/// - Budgets (ON DELETE CASCADE)
/// - Cast history user_id set to NULL (audit trail preserved)
///
/// Compliance:
/// - GDPR Article 17 (Right to Erasure)
/// - CCPA §1798.105 (Right to Delete)
/// - Japanese APPI Article 30 (Erasure)
pub async fn delete_user_data(req: HttpRequest, pool: web::Data<PgPool>) -> Result<HttpResponse> {
    // Get authenticated user from request extensions (set by auth middleware)
    let user_id = {
        let ext = req.extensions();
        ext.get::<User>()
            .ok_or_else(|| actix_web::error::ErrorUnauthorized("User not authenticated"))?
            .id
    };

    // Delete user (cascades to all related data)
    let result = sqlx::query!("DELETE FROM users WHERE id = $1 RETURNING id", user_id)
        .fetch_optional(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    if result.is_none() {
        return Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "user_not_found",
            "message": "User not found"
        })));
    }

    Ok(HttpResponse::NoContent().finish())
}

// ============================================================================
// Data Export (GDPR Article 20 - Right to Data Portability)
// ============================================================================

#[derive(Serialize)]
pub struct UserDataExport {
    pub user: UserProfileExport,
    pub api_keys: Vec<ApiKeyExport>,
    pub billing: Option<BillingExport>,
    pub budgets: Option<BudgetExport>,
    pub usage: Vec<UsageExport>,
    pub casts: Vec<CastExport>,
    pub exported_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize)]
pub struct UserProfileExport {
    pub id: Uuid,
    pub github_id: i64,
    pub github_login: String,
    pub github_name: Option<String>,
    pub github_email: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize)]
pub struct ApiKeyExport {
    pub id: Uuid,
    pub name: String,
    pub prefix: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Serialize)]
pub struct BillingExport {
    pub stripe_customer_id: Option<String>,
    pub plan: String,
    pub status: String,
    pub current_period_end: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Serialize)]
pub struct BudgetExport {
    pub period: String,
    pub soft_limit_cents: Option<i32>,
    pub hard_limit_cents: Option<i32>,
    pub notify_thresholds: Vec<i32>,
}

#[derive(Serialize)]
pub struct UsageExport {
    pub window_start: chrono::DateTime<chrono::Utc>,
    pub window_end: chrono::DateTime<chrono::Utc>,
    pub calls: i32,
    pub cost_cents: i32,
}

#[derive(Serialize)]
pub struct CastExport {
    pub id: Uuid,
    pub spell_name: String,
    pub cost_cents: Option<i32>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// GET /v1/users/me/export
///
/// Returns all user data in machine-readable JSON format
///
/// Compliance:
/// - GDPR Article 20 (Right to Data Portability)
/// - CCPA §1798.110 (Right to Know)
/// - Japanese APPI Article 28 (Disclosure)
pub async fn export_user_data(req: HttpRequest, pool: web::Data<PgPool>) -> Result<HttpResponse> {
    // Get authenticated user from request extensions (set by auth middleware)
    let user_id = {
        let ext = req.extensions();
        ext.get::<User>()
            .ok_or_else(|| actix_web::error::ErrorUnauthorized("User not authenticated"))?
            .id
    };

    // Fetch user profile
    let user = sqlx::query_as!(
        UserProfileExport,
        r#"
        SELECT
            id,
            github_id,
            github_login,
            github_name,
            github_email,
            created_at
        FROM users
        WHERE id = $1
        "#,
        user_id
    )
    .fetch_optional(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?
    .ok_or_else(|| actix_web::error::ErrorNotFound("User not found"))?;

    // Fetch API keys (without hash for security)
    let api_keys = sqlx::query_as!(
        ApiKeyExport,
        r#"
        SELECT id, name, prefix, created_at, last_used_at
        FROM api_keys
        WHERE user_id = $1
        ORDER BY created_at DESC
        "#,
        user_id
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    // Fetch billing info
    let billing = sqlx::query!(
        r#"
        SELECT stripe_customer_id, plan, status, current_period_end
        FROM billing_accounts
        WHERE user_id = $1
        "#,
        user_id
    )
    .fetch_optional(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?
    .map(|row| BillingExport {
        stripe_customer_id: row.stripe_customer_id,
        plan: row.plan,
        status: row.status,
        current_period_end: row.current_period_end,
    });

    // Fetch budget
    let budgets = sqlx::query!(
        r#"
        SELECT period, soft_limit_cents, hard_limit_cents, notify_thresholds_json
        FROM budgets
        WHERE user_id = $1
        "#,
        user_id
    )
    .fetch_optional(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?
    .map(|row| BudgetExport {
        period: row.period,
        soft_limit_cents: row.soft_limit_cents,
        hard_limit_cents: row.hard_limit_cents,
        notify_thresholds: row
            .notify_thresholds_json
            .as_ref()
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_i64().map(|i| i as i32))
                    .collect()
            })
            .unwrap_or_default(),
    });

    // Fetch usage history
    let usage = sqlx::query_as!(
        UsageExport,
        r#"
        SELECT window_start, window_end, calls, cost_cents
        FROM usage_counters
        WHERE user_id = $1
        ORDER BY window_start DESC
        "#,
        user_id
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    // Fetch casts (limited to last 1000 for performance)
    let casts = sqlx::query_as!(
        CastExport,
        r#"
        SELECT id, spell_name, cost_cents, created_at
        FROM casts
        WHERE user_id = $1
        ORDER BY created_at DESC
        LIMIT 1000
        "#,
        user_id
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    let export = UserDataExport {
        user,
        api_keys,
        billing,
        budgets,
        usage,
        casts,
        exported_at: chrono::Utc::now(),
    };

    Ok(HttpResponse::Ok().json(export))
}

// ============================================================================
// Router Configuration
// ============================================================================

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/v1/users")
            .route("/me", web::delete().to(delete_user_data))
            .route("/me/export", web::get().to(export_user_data)),
    );
}
