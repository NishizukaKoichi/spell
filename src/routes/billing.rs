use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use actix_web_httpauth::middleware::HttpAuthentication;

use crate::models::User;
use crate::AppState;

pub fn configure(cfg: &mut web::ServiceConfig) {
    let auth = HttpAuthentication::bearer(crate::middleware::auth::validator);

    cfg.service(
        web::scope("/billing")
            .wrap(auth)
            .route("/checkout", web::post().to(create_checkout_session)),
    )
    .service(web::resource("/webhooks/stripe").route(web::post().to(stripe_webhook)));
}

async fn create_checkout_session(
    state: web::Data<AppState>,
    http_req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    // Get authenticated user
    let user_id = {
        let ext = http_req.extensions();
        ext.get::<User>()
            .ok_or_else(|| actix_web::error::ErrorUnauthorized("User not authenticated"))?
            .id
    };

    let stripe_service = state
        .stripe
        .as_ref()
        .ok_or_else(|| actix_web::error::ErrorServiceUnavailable("Billing not configured"))?;

    if !stripe_service.is_enabled() {
        return Err(actix_web::error::ErrorServiceUnavailable(
            "Billing features are currently disabled",
        ));
    }

    // TODO: Make these configurable or get from request
    let success_url = "https://spell-platform.fly.dev/billing/success".to_string();
    let cancel_url = "https://spell-platform.fly.dev/billing/cancel".to_string();

    let session = stripe_service
        .create_checkout_session(user_id, success_url, cancel_url)
        .await
        .map_err(|e| {
            log::error!("Failed to create checkout session: {e}");
            actix_web::error::ErrorInternalServerError("Failed to create checkout session")
        })?;

    Ok(HttpResponse::Ok().json(session))
}

async fn stripe_webhook(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Bytes,
) -> Result<HttpResponse, actix_web::Error> {
    let stripe_service = state
        .stripe
        .as_ref()
        .ok_or_else(|| actix_web::error::ErrorServiceUnavailable("Billing not configured"))?;

    // Get Stripe signature from header
    let signature = req
        .headers()
        .get("stripe-signature")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| actix_web::error::ErrorBadRequest("Missing stripe-signature header"))?;

    // Verify webhook signature
    let event = stripe_service
        .verify_webhook_signature(&body, signature)
        .map_err(|e| {
            log::error!("Webhook signature verification failed: {e}");
            actix_web::error::ErrorUnauthorized("Invalid webhook signature")
        })?;

    log::info!("Received Stripe webhook event: {:?}", event.type_);

    // Handle event
    stripe_service
        .handle_webhook_event(event, &state.db)
        .await
        .map_err(|e| {
            log::error!("Failed to handle webhook event: {e}");
            actix_web::error::ErrorInternalServerError("Failed to process webhook")
        })?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "success"
    })))
}
