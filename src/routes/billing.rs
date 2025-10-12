use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use actix_web_httpauth::middleware::HttpAuthentication;
use serde::Deserialize;

use crate::middleware::auth::authenticate_from_cookie;
use crate::models::User;
use crate::AppState;

pub fn configure(cfg: &mut web::ServiceConfig) {
    let auth = HttpAuthentication::bearer(crate::middleware::auth::validator);

    cfg.service(
        web::scope("/billing")
            .wrap(auth)
            .route("/checkout", web::post().to(create_checkout_session)),
    )
    .service(
        web::resource("/setup-intent")
            .route(web::post().to(create_setup_intent))
    )
    .service(
        web::resource("/payment-method")
            .route(web::post().to(attach_payment_method))
    )
    .service(web::resource("/webhooks/stripe").route(web::post().to(stripe_webhook)));
}

#[derive(Deserialize)]
struct AttachPaymentMethodRequest {
    payment_method_id: String,
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

async fn create_setup_intent(
    state: web::Data<AppState>,
    http_req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    // Authenticate user from cookie
    let user = authenticate_from_cookie(&http_req, &state.db)
        .await
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("Not authenticated"))?;

    let stripe_service = state
        .stripe
        .as_ref()
        .ok_or_else(|| actix_web::error::ErrorServiceUnavailable("Billing not configured"))?;

    if !stripe_service.is_enabled() {
        return Err(actix_web::error::ErrorServiceUnavailable(
            "Billing features are currently disabled",
        ));
    }

    // Create or get Stripe customer
    let customer_id = stripe_service
        .get_or_create_customer(&state.db, user.id)
        .await
        .map_err(|e| {
            log::error!("Failed to get/create customer: {e}");
            actix_web::error::ErrorInternalServerError("Failed to create customer")
        })?;

    // Create SetupIntent
    let setup_intent = stripe_service
        .create_setup_intent(&customer_id)
        .await
        .map_err(|e| {
            log::error!("Failed to create setup intent: {e}");
            actix_web::error::ErrorInternalServerError("Failed to create setup intent")
        })?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "client_secret": setup_intent.client_secret
    })))
}

async fn attach_payment_method(
    state: web::Data<AppState>,
    http_req: HttpRequest,
    req: web::Json<AttachPaymentMethodRequest>,
) -> Result<HttpResponse, actix_web::Error> {
    // Authenticate user from cookie
    let user = authenticate_from_cookie(&http_req, &state.db)
        .await
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("Not authenticated"))?;

    let stripe_service = state
        .stripe
        .as_ref()
        .ok_or_else(|| actix_web::error::ErrorServiceUnavailable("Billing not configured"))?;

    if !stripe_service.is_enabled() {
        return Err(actix_web::error::ErrorServiceUnavailable(
            "Billing features are currently disabled",
        ));
    }

    // Attach payment method to customer
    let customer_id = stripe_service
        .get_or_create_customer(&state.db, user.id)
        .await
        .map_err(|e| {
            log::error!("Failed to get/create customer: {e}");
            actix_web::error::ErrorInternalServerError("Failed to get customer")
        })?;

    stripe_service
        .attach_payment_method(&customer_id, &req.payment_method_id)
        .await
        .map_err(|e| {
            log::error!("Failed to attach payment method: {e}");
            actix_web::error::ErrorInternalServerError("Failed to attach payment method")
        })?;

    // Save payment method ID in database
    sqlx::query(
        r#"
        INSERT INTO billing_accounts (user_id, stripe_customer_id, payment_method_id, plan, status)
        VALUES ($1, $2, $3, 'free', 'active')
        ON CONFLICT (user_id) DO UPDATE SET
            payment_method_id = EXCLUDED.payment_method_id,
            updated_at = NOW()
        "#,
    )
    .bind(user.id)
    .bind(&customer_id)
    .bind(&req.payment_method_id)
    .execute(&state.db)
    .await
    .map_err(|e| {
        log::error!("Failed to save payment method: {e}");
        actix_web::error::ErrorInternalServerError("Failed to save payment method")
    })?;

    // Set initial budget limit ($50)
    sqlx::query(
        r#"
        INSERT INTO budgets (user_id, period, hard_limit_cents, notify_thresholds_json)
        VALUES ($1, 'monthly', 5000, '[]')
        ON CONFLICT (user_id, period) DO UPDATE SET
            hard_limit_cents = EXCLUDED.hard_limit_cents,
            updated_at = NOW()
        "#,
    )
    .bind(user.id)
    .execute(&state.db)
    .await
    .map_err(|e| {
        log::error!("Failed to set budget: {e}");
        actix_web::error::ErrorInternalServerError("Failed to set budget")
    })?;

    log::info!(
        "Payment method attached for user {} with initial $50 limit",
        user.github_login
    );

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "message": "Payment method attached successfully"
    })))
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
