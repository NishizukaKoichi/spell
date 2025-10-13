use actix_web::{web, HttpRequest, HttpResponse};
use std::env;

use crate::services::billing_service::BillingService;
use crate::AppState;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/admin/billing/process-monthly")
            .route(web::post().to(process_monthly_billing)),
    );
}

/// Process monthly billing for all users
/// Requires ADMIN_SECRET environment variable to match X-Admin-Secret header
async fn process_monthly_billing(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    // Check admin secret
    let admin_secret = env::var("ADMIN_SECRET").ok();
    let request_secret = req
        .headers()
        .get("X-Admin-Secret")
        .and_then(|h| h.to_str().ok());

    match (admin_secret, request_secret) {
        (Some(expected), Some(provided)) if expected == provided => {
            // Authorized
        }
        _ => {
            return Err(actix_web::error::ErrorUnauthorized(
                "Invalid or missing admin secret",
            ));
        }
    }

    let stripe = state.stripe.as_ref().ok_or_else(|| {
        actix_web::error::ErrorServiceUnavailable("Billing not configured")
    })?;

    BillingService::process_monthly_billing(stripe, &state.db)
        .await
        .map_err(|e| {
            log::error!("Failed to process monthly billing: {e}");
            actix_web::error::ErrorInternalServerError("Failed to process monthly billing")
        })?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "message": "Monthly billing processed successfully"
    })))
}
