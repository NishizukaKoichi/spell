use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use actix_web_httpauth::middleware::HttpAuthentication;
use serde::Deserialize;

use crate::middleware::auth::authenticate_from_cookie;
use crate::models::{billing::Budget, User};
use crate::services::budget_service::BudgetService;
use crate::AppState;

pub fn configure(cfg: &mut web::ServiceConfig) {
    let auth = HttpAuthentication::bearer(crate::middleware::auth::validator);

    cfg.service(
        web::scope("/billing")
            .service(
                web::resource("/checkout")
                    .route(web::post().to(create_checkout_session))
                    .wrap(auth),
            )
            .route("/setup-intent", web::post().to(create_setup_intent))
            .route("/dev-setup-intent", web::post().to(dev_create_setup_intent))
            .service(
                web::resource("/payment-method")
                    .route(web::post().to(attach_payment_method))
                    .route(web::get().to(get_payment_method)),
            )
            .route("/usage", web::get().to(get_usage_cookie)),
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

/// Dev-only endpoint that bypasses authentication (controlled by DEV_MODE_USER_ID env var)
async fn dev_create_setup_intent(
    state: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    let dev_user_id_str = std::env::var("DEV_MODE_USER_ID")
        .map_err(|_| actix_web::error::ErrorServiceUnavailable("DEV_MODE_USER_ID not set"))?;

    let dev_user_id: uuid::Uuid = dev_user_id_str.parse()
        .map_err(|_| actix_web::error::ErrorInternalServerError("Invalid DEV_MODE_USER_ID (must be UUID)"))?;

    log::warn!("🚧 DEV MODE: Creating setup intent for user_id={}", dev_user_id);

    let user = sqlx::query_as::<_, User>(
        r#"SELECT id, github_id, github_login, github_name, github_email, github_avatar_url, created_at, updated_at FROM users WHERE id = $1"#
    )
    .bind(dev_user_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        log::error!("DEV MODE: Failed to fetch user {}: {}", dev_user_id, e);
        actix_web::error::ErrorInternalServerError("Dev user not found")
    })?;

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
    log::info!("Attempting to get/create customer for user: {}", user.id);
    let customer_id = stripe_service
        .get_or_create_customer(&state.db, user.id)
        .await
        .map_err(|e| {
            log::error!("Failed to get/create customer: {e:?}");
            actix_web::error::ErrorInternalServerError("Failed to create customer")
        })?;
    log::info!("Successfully got/created customer: {customer_id}");

    // Create SetupIntent
    match stripe_service.create_setup_intent(&customer_id).await {
        Ok(setup_intent) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "client_secret": setup_intent.client_secret
        }))),
        Err(failure) => {
            let payload = serde_json::json!({
                "error": "Failed to create setup intent",
                "stripe_status": failure.status,
                "stripe_code": failure.code,
                "stripe_message": failure.message,
                "stripe_request_id": failure.request_id,
            });
            Ok(HttpResponse::InternalServerError().json(payload))
        }
    }
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
    log::info!("Attempting to get/create customer for user: {}", user.id);
    let customer_id = stripe_service
        .get_or_create_customer(&state.db, user.id)
        .await
        .map_err(|e| {
            log::error!("Failed to get/create customer: {e:?}");
            log::error!("Error chain: {:?}", e.chain().collect::<Vec<_>>());
            actix_web::error::ErrorInternalServerError("Failed to create customer")
        })?;
    log::info!("Successfully got/created customer: {customer_id}");

    // Create SetupIntent
    match stripe_service.create_setup_intent(&customer_id).await {
        Ok(setup_intent) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "client_secret": setup_intent.client_secret
        }))),
        Err(failure) => {
            let payload = serde_json::json!({
                "error": "Failed to create setup intent",
                "stripe_status": failure.status,
                "stripe_code": failure.code,
                "stripe_message": failure.message,
                "stripe_request_id": failure.request_id,
            });
            Ok(HttpResponse::InternalServerError().json(payload))
        }
    }
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

/// Get payment method details (cookie authentication)
async fn get_payment_method(
    state: web::Data<AppState>,
    http_req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let user = authenticate_from_cookie(&http_req, &state.db)
        .await
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("Not authenticated"))?;

    // Get billing account with payment method
    let billing_account: Option<(String,)> = sqlx::query_as(
        r#"
        SELECT payment_method_id FROM billing_accounts WHERE user_id = $1
        "#,
    )
    .bind(user.id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        log::error!("Failed to fetch billing account: {e}");
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    let payment_method_id = match billing_account {
        Some((pm_id,)) => pm_id,
        None => {
            return Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "No payment method configured"
            })));
        }
    };

    // Get payment method details from Stripe
    let stripe_service = state
        .stripe
        .as_ref()
        .ok_or_else(|| actix_web::error::ErrorServiceUnavailable("Billing not configured"))?;

    let payment_method = stripe_service
        .get_payment_method(&payment_method_id)
        .await
        .map_err(|e| {
            log::error!("Failed to get payment method from Stripe: {e}");
            actix_web::error::ErrorInternalServerError("Failed to get payment method")
        })?;

    // Extract card details
    let card = payment_method.card.as_ref().ok_or_else(|| {
        actix_web::error::ErrorInternalServerError("Payment method is not a card")
    })?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "brand": card.brand,
        "last4": card.last4,
        "exp_month": card.exp_month,
        "exp_year": card.exp_year,
    })))
}

/// Get usage stats (cookie authentication)
async fn get_usage_cookie(
    state: web::Data<AppState>,
    http_req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let user = authenticate_from_cookie(&http_req, &state.db)
        .await
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("Not authenticated"))?;

    // Get budget to determine period
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

    let period = "monthly";
    let (total_calls, total_cost) = BudgetService::get_current_usage(&user.id, period, &state.db)
        .await
        .map_err(|e| {
            log::error!("Failed to get usage: {e}");
            actix_web::error::ErrorInternalServerError("Database error")
        })?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "user_id": user.id,
        "period": period,
        "total_calls": total_calls,
        "total_cost_cents": total_cost,
        "hard_limit_cents": budget.as_ref().and_then(|b| b.hard_limit_cents),
        "soft_limit_cents": budget.as_ref().and_then(|b| b.soft_limit_cents),
    })))
}
