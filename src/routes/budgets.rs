use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use actix_web_httpauth::middleware::HttpAuthentication;

use crate::middleware::auth::authenticate_from_cookie;
use crate::models::billing::{Budget, BudgetResponse, CreateBudgetRequest};
use crate::models::User;
use crate::services::budget_service::BudgetService;
use crate::AppState;

// Minimum and maximum budget limits in cents ($10 - $500)
const MIN_BUDGET_CENTS: i32 = 1000;  // $10
const MAX_BUDGET_CENTS: i32 = 50000; // $500

pub fn configure(cfg: &mut web::ServiceConfig) {
    let auth = HttpAuthentication::bearer(crate::middleware::auth::validator);

    cfg.service(
        web::scope("/budgets")
            .wrap(auth)
            .route("", web::get().to(get_budget))
            .route("", web::post().to(create_budget))
            .route("", web::put().to(update_budget))
            .route("", web::delete().to(delete_budget))
            .route("/usage", web::get().to(get_usage)),
    )
    .service(
        web::resource("/budget")
            .route(web::get().to(get_budget_cookie))
            .route(web::put().to(update_budget_cookie))
    );
}

async fn get_budget(
    state: web::Data<AppState>,
    http_req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = {
        let ext = http_req.extensions();
        ext.get::<User>()
            .ok_or_else(|| actix_web::error::ErrorUnauthorized("User not authenticated"))?
            .id
    };

    let budget: Option<Budget> = sqlx::query_as(
        r#"
        SELECT * FROM budgets WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        log::error!("Failed to fetch budget: {e}");
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    match budget {
        Some(b) => {
            let thresholds: Vec<i32> =
                serde_json::from_value(b.notify_thresholds_json.clone()).unwrap_or_default();

            Ok(HttpResponse::Ok().json(BudgetResponse {
                user_id: b.user_id,
                period: b.period,
                soft_limit_cents: b.soft_limit_cents,
                hard_limit_cents: b.hard_limit_cents,
                notify_thresholds: thresholds,
                created_at: b.created_at,
                updated_at: b.updated_at,
            }))
        }
        None => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "No budget configured"
        }))),
    }
}

async fn create_budget(
    state: web::Data<AppState>,
    http_req: HttpRequest,
    req: web::Json<CreateBudgetRequest>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = {
        let ext = http_req.extensions();
        ext.get::<User>()
            .ok_or_else(|| actix_web::error::ErrorUnauthorized("User not authenticated"))?
            .id
    };

    let period = req.period.as_deref().unwrap_or("monthly");
    let thresholds_json = serde_json::to_value(req.notify_thresholds.clone().unwrap_or_default())
        .unwrap_or(serde_json::json!([]));

    // Validate limits
    if let Some(hard) = req.hard_limit_cents {
        if hard < MIN_BUDGET_CENTS || hard > MAX_BUDGET_CENTS {
            return Err(actix_web::error::ErrorBadRequest(format!(
                "Hard limit must be between ${} and ${}",
                MIN_BUDGET_CENTS / 100,
                MAX_BUDGET_CENTS / 100
            )));
        }
    }

    if let Some(soft) = req.soft_limit_cents {
        if soft < MIN_BUDGET_CENTS || soft > MAX_BUDGET_CENTS {
            return Err(actix_web::error::ErrorBadRequest(format!(
                "Soft limit must be between ${} and ${}",
                MIN_BUDGET_CENTS / 100,
                MAX_BUDGET_CENTS / 100
            )));
        }
    }

    if let (Some(soft), Some(hard)) = (req.soft_limit_cents, req.hard_limit_cents) {
        if soft > hard {
            return Err(actix_web::error::ErrorBadRequest(
                "Soft limit cannot exceed hard limit",
            ));
        }
    }

    let budget: Budget = sqlx::query_as(
        r#"
        INSERT INTO budgets (user_id, period, soft_limit_cents, hard_limit_cents, notify_thresholds_json, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, NOW(), NOW())
        ON CONFLICT (user_id) DO UPDATE
        SET period = EXCLUDED.period,
            soft_limit_cents = EXCLUDED.soft_limit_cents,
            hard_limit_cents = EXCLUDED.hard_limit_cents,
            notify_thresholds_json = EXCLUDED.notify_thresholds_json,
            updated_at = NOW()
        RETURNING *
        "#,
    )
    .bind(user_id)
    .bind(period)
    .bind(req.soft_limit_cents)
    .bind(req.hard_limit_cents)
    .bind(&thresholds_json)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        log::error!("Failed to create budget: {e}");
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    let thresholds: Vec<i32> =
        serde_json::from_value(budget.notify_thresholds_json.clone()).unwrap_or_default();

    Ok(HttpResponse::Created().json(BudgetResponse {
        user_id: budget.user_id,
        period: budget.period,
        soft_limit_cents: budget.soft_limit_cents,
        hard_limit_cents: budget.hard_limit_cents,
        notify_thresholds: thresholds,
        created_at: budget.created_at,
        updated_at: budget.updated_at,
    }))
}

async fn update_budget(
    state: web::Data<AppState>,
    http_req: HttpRequest,
    req: web::Json<CreateBudgetRequest>,
) -> Result<HttpResponse, actix_web::Error> {
    // Same as create_budget (uses ON CONFLICT DO UPDATE)
    create_budget(state, http_req, req).await
}

async fn delete_budget(
    state: web::Data<AppState>,
    http_req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = {
        let ext = http_req.extensions();
        ext.get::<User>()
            .ok_or_else(|| actix_web::error::ErrorUnauthorized("User not authenticated"))?
            .id
    };

    sqlx::query(
        r#"
        DELETE FROM budgets WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .execute(&state.db)
    .await
    .map_err(|e| {
        log::error!("Failed to delete budget: {e}");
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    Ok(HttpResponse::NoContent().finish())
}

async fn get_usage(
    state: web::Data<AppState>,
    http_req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = {
        let ext = http_req.extensions();
        ext.get::<User>()
            .ok_or_else(|| actix_web::error::ErrorUnauthorized("User not authenticated"))?
            .id
    };

    // Get budget to determine period
    let budget: Option<Budget> = sqlx::query_as(
        r#"
        SELECT * FROM budgets WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        log::error!("Failed to fetch budget: {e}");
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    let period = budget
        .as_ref()
        .map(|b| b.period.as_str())
        .unwrap_or("monthly");

    let (total_calls, total_cost) = BudgetService::get_current_usage(&user_id, period, &state.db)
        .await
        .map_err(|e| {
            log::error!("Failed to get usage: {e}");
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "user_id": user_id,
        "period": period,
        "total_calls": total_calls,
        "total_cost_cents": total_cost,
        "hard_limit_cents": budget.as_ref().and_then(|b| b.hard_limit_cents),
        "soft_limit_cents": budget.as_ref().and_then(|b| b.soft_limit_cents),
    })))
}

// Cookie-based authentication endpoints

async fn get_budget_cookie(
    state: web::Data<AppState>,
    http_req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let user = authenticate_from_cookie(&http_req, &state.db)
        .await
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("Not authenticated"))?;

    let budget: Option<Budget> = sqlx::query_as(
        r#"
        SELECT * FROM budgets WHERE user_id = $1 AND period = 'monthly'
        "#,
    )
    .bind(user.id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        log::error!("Failed to fetch budget: {e}");
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    match budget {
        Some(b) => {
            let thresholds: Vec<i32> =
                serde_json::from_value(b.notify_thresholds_json.clone()).unwrap_or_default();

            Ok(HttpResponse::Ok().json(BudgetResponse {
                user_id: b.user_id,
                period: b.period,
                soft_limit_cents: b.soft_limit_cents,
                hard_limit_cents: b.hard_limit_cents,
                notify_thresholds: thresholds,
                created_at: b.created_at,
                updated_at: b.updated_at,
            }))
        }
        None => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "No budget configured"
        }))),
    }
}

async fn update_budget_cookie(
    state: web::Data<AppState>,
    http_req: HttpRequest,
    req: web::Json<CreateBudgetRequest>,
) -> Result<HttpResponse, actix_web::Error> {
    let user = authenticate_from_cookie(&http_req, &state.db)
        .await
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("Not authenticated"))?;

    let period = req.period.as_deref().unwrap_or("monthly");
    let thresholds_json = serde_json::to_value(req.notify_thresholds.clone().unwrap_or_default())
        .unwrap_or(serde_json::json!([]));

    // Validate limits
    if let Some(hard) = req.hard_limit_cents {
        if hard < MIN_BUDGET_CENTS || hard > MAX_BUDGET_CENTS {
            return Err(actix_web::error::ErrorBadRequest(format!(
                "Hard limit must be between ${} and ${}",
                MIN_BUDGET_CENTS / 100,
                MAX_BUDGET_CENTS / 100
            )));
        }
    }

    if let Some(soft) = req.soft_limit_cents {
        if soft < MIN_BUDGET_CENTS || soft > MAX_BUDGET_CENTS {
            return Err(actix_web::error::ErrorBadRequest(format!(
                "Soft limit must be between ${} and ${}",
                MIN_BUDGET_CENTS / 100,
                MAX_BUDGET_CENTS / 100
            )));
        }
    }

    if let (Some(soft), Some(hard)) = (req.soft_limit_cents, req.hard_limit_cents) {
        if soft > hard {
            return Err(actix_web::error::ErrorBadRequest(
                "Soft limit cannot exceed hard limit",
            ));
        }
    }

    let budget: Budget = sqlx::query_as(
        r#"
        INSERT INTO budgets (user_id, period, soft_limit_cents, hard_limit_cents, notify_thresholds_json, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, NOW(), NOW())
        ON CONFLICT (user_id, period) DO UPDATE
        SET soft_limit_cents = EXCLUDED.soft_limit_cents,
            hard_limit_cents = EXCLUDED.hard_limit_cents,
            notify_thresholds_json = EXCLUDED.notify_thresholds_json,
            updated_at = NOW()
        RETURNING *
        "#,
    )
    .bind(user.id)
    .bind(period)
    .bind(req.soft_limit_cents)
    .bind(req.hard_limit_cents)
    .bind(&thresholds_json)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        log::error!("Failed to update budget: {e}");
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    let thresholds: Vec<i32> =
        serde_json::from_value(budget.notify_thresholds_json.clone()).unwrap_or_default();

    log::info!("Budget updated for user {} to ${}", user.github_login, budget.hard_limit_cents.unwrap_or(0) / 100);

    Ok(HttpResponse::Ok().json(BudgetResponse {
        user_id: budget.user_id,
        period: budget.period,
        soft_limit_cents: budget.soft_limit_cents,
        hard_limit_cents: budget.hard_limit_cents,
        notify_thresholds: thresholds,
        created_at: budget.created_at,
        updated_at: budget.updated_at,
    }))
}
