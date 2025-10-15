use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use stripe::{Account, Client, Customer, CustomerId};
use uuid::Uuid;

use crate::middleware::auth::authenticate_from_cookie;
use crate::services::stripe_service::SetupIntentFailure;
use crate::AppState;

#[derive(Deserialize)]
pub struct StripeSelfQuery {
    pub user_id: Option<Uuid>,
    #[serde(default)]
    pub skip_user_lookup: bool,
}

#[derive(Deserialize)]
pub struct StripeDoctorQuery {
    pub user_id: Option<Uuid>,
}

#[derive(Serialize)]
struct StripeSelfResponse {
    account_id: String,
    livemode: bool,
    customer_in_db: Option<String>,
    customer_exists: Option<bool>,
}

#[derive(Serialize)]
struct StripeDoctorResponse {
    account_id: String,
    livemode: bool,
    user_id: Option<Uuid>,
    customer_before: Option<String>,
    customer_after: Option<String>,
    customer_verified: Option<bool>,
    customer_recreated: bool,
    setup_intent: DoctorSetupIntentOutcome,
}

#[derive(Serialize)]
struct DoctorSetupIntentOutcome {
    created: bool,
    setup_intent_id: Option<String>,
    setup_intent_status: Option<String>,
    client_secret_present: bool,
    failure: Option<SetupIntentFailure>,
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/debug/stripe")
            .route("/self", web::get().to(get_self))
            .route("/doctor", web::get().to(get_doctor)),
    );
}

async fn get_self(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<StripeSelfQuery>,
) -> Result<HttpResponse, actix_web::Error> {
    let _user = authenticate_from_cookie(&req, &state.db)
        .await
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("Not authenticated"))?;

    let stripe_service = state
        .stripe
        .as_ref()
        .ok_or_else(|| actix_web::error::ErrorServiceUnavailable("Billing not configured"))?;

    let client = stripe_service
        .clone_client()
        .ok_or_else(|| actix_web::error::ErrorServiceUnavailable("Stripe client unavailable"))?;

    let account = retrieve_current_account(&client).await.map_err(|err| {
        log::error!("stripe_self_account_failed source={err}");
        actix_web::error::ErrorInternalServerError("Failed to fetch Stripe account details")
    })?;

    let mut customer_in_db = None;
    let mut customer_exists = None;

    if !query.skip_user_lookup {
        if let Some(user_id) = query.user_id {
            let stored_customer = fetch_customer_id(&state, user_id).await?;
            if let Some(cid) = stored_customer.clone() {
                customer_in_db = Some(cid.clone());
                customer_exists = Some(check_customer_exists(&client, &cid).await);
            }
        }
    }

    let response = StripeSelfResponse {
        account_id: account.id.to_string(),
        livemode: infer_livemode().unwrap_or(false),
        customer_in_db,
        customer_exists,
    };

    Ok(HttpResponse::Ok().json(response))
}

async fn get_doctor(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<StripeDoctorQuery>,
) -> Result<HttpResponse, actix_web::Error> {
    let auth_user = authenticate_from_cookie(&req, &state.db)
        .await
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("Not authenticated"))?;

    let target_user = query.user_id.unwrap_or(auth_user.id);
    if target_user != auth_user.id {
        return Err(actix_web::error::ErrorForbidden(
            "Cannot inspect other user's billing state",
        ));
    }

    let stripe_service = state
        .stripe
        .as_ref()
        .ok_or_else(|| actix_web::error::ErrorServiceUnavailable("Billing not configured"))?;

    let client = stripe_service
        .clone_client()
        .ok_or_else(|| actix_web::error::ErrorServiceUnavailable("Stripe client unavailable"))?;

    let account = retrieve_current_account(&client).await.map_err(|err| {
        log::error!("stripe_doctor_account_failed source={err}");
        actix_web::error::ErrorInternalServerError("Failed to fetch Stripe account details")
    })?;

    let before_customer = fetch_customer_id(&state, target_user).await?;

    let current_customer_id = stripe_service
        .get_or_create_customer(&state.db, target_user)
        .await
        .map_err(|err| {
            log::error!("stripe_doctor_customer_sync_failed user_id={target_user} source={err:?}");
            actix_web::error::ErrorInternalServerError("Failed to synchronize Stripe customer")
        })?;

    let customer_verified = check_customer_exists(&client, &current_customer_id).await;

    let setup_intent = match stripe_service
        .create_setup_intent(&current_customer_id)
        .await
    {
        Ok(intent) => DoctorSetupIntentOutcome {
            created: true,
            setup_intent_id: Some(intent.id.to_string()),
            setup_intent_status: Some(intent.status.to_string()),
            client_secret_present: intent.client_secret.is_some(),
            failure: None,
        },
        Err(failure) => DoctorSetupIntentOutcome {
            created: false,
            setup_intent_id: None,
            setup_intent_status: None,
            client_secret_present: false,
            failure: Some(failure),
        },
    };

    let response = StripeDoctorResponse {
        account_id: account.id.to_string(),
        livemode: infer_livemode().unwrap_or(false),
        user_id: Some(target_user),
        customer_before: before_customer.clone(),
        customer_after: Some(current_customer_id.clone()),
        customer_verified: Some(customer_verified),
        customer_recreated: before_customer
            .as_ref()
            .map(|before| before != &current_customer_id)
            .unwrap_or(false),
        setup_intent,
    };

    Ok(HttpResponse::Ok().json(response))
}

async fn fetch_customer_id(
    state: &web::Data<AppState>,
    user_id: Uuid,
) -> Result<Option<String>, actix_web::Error> {
    let record: Option<(String,)> = sqlx::query_as(
        r#"
        SELECT stripe_customer_id
        FROM billing_accounts
        WHERE user_id = $1 AND stripe_customer_id IS NOT NULL
        "#,
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|err| {
        log::error!("stripe_debug_fetch_customer_failed user_id={user_id} source={err:?}");
        actix_web::error::ErrorInternalServerError("Failed to fetch billing account")
    })?;

    Ok(record.map(|(cid,)| cid))
}

async fn check_customer_exists(client: &Client, customer_id: &str) -> bool {
    match customer_id.parse::<CustomerId>() {
        Ok(parsed) => match Customer::retrieve(client, &parsed, &[]).await {
            Ok(customer) => {
                log::info!(
                    "stripe_customer_verified id={} livemode={}",
                    customer.id,
                    customer.livemode.unwrap_or(false)
                );
                true
            }
            Err(err) => {
                log::warn!("stripe_customer_missing id={customer_id} err={err}");
                false
            }
        },
        Err(_) => {
            log::warn!("stripe_customer_invalid_id id={customer_id}");
            false
        }
    }
}

async fn retrieve_current_account(client: &Client) -> Result<Account, stripe::StripeError> {
    client.get::<Account>("/account").await
}

fn infer_livemode() -> Option<bool> {
    std::env::var("STRIPE_SECRET_KEY")
        .ok()
        .map(|secret| secret.starts_with("sk_live"))
}
