use serde::Serialize;
use stripe::{
    CheckoutSession, CheckoutSessionMode, Client, CreateCheckoutSession,
    CreateCheckoutSessionLineItems, CreateInvoice, CreateInvoiceItem, CreatePrice,
    CreatePriceRecurring, CreateSetupIntent, CreateSetupIntentAutomaticPaymentMethods, Currency,
    Customer, ErrorCode, EventObject, EventType, Invoice, InvoiceItem, PaymentMethod, Price,
    SetupIntent, Webhook,
};
use uuid::Uuid;

use crate::models::billing::CheckoutSessionResponse;

pub struct StripeService {
    client: Option<Client>,
    webhook_secret: Option<String>,
}

impl Default for StripeService {
    fn default() -> Self {
        Self::new()
    }
}

impl StripeService {
    pub fn new() -> Self {
        let client = std::env::var("STRIPE_SECRET_KEY").ok().map(Client::new);

        let webhook_secret = std::env::var("STRIPE_WEBHOOK_SECRET").ok();

        if client.is_none() {
            log::warn!("STRIPE_SECRET_KEY not set - billing features will be disabled");
        }
        if webhook_secret.is_none() {
            log::warn!("STRIPE_WEBHOOK_SECRET not set - webhook verification will be disabled");
        }

        Self {
            client,
            webhook_secret,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.client.is_some()
    }

    pub fn clone_client(&self) -> Option<Client> {
        self.client.clone()
    }

    pub async fn create_checkout_session(
        &self,
        user_id: Uuid,
        success_url: String,
        cancel_url: String,
    ) -> Result<CheckoutSessionResponse, anyhow::Error> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Stripe not configured"))?;

        // Create a price for the Pro plan (monthly)
        let mut create_price = CreatePrice::new(Currency::USD);
        create_price.unit_amount = Some(1000); // $10.00
        create_price.recurring = Some(CreatePriceRecurring {
            interval: stripe::CreatePriceRecurringInterval::Month,
            interval_count: Some(1),
            ..Default::default()
        });
        create_price.product_data = Some(stripe::CreatePriceProductData {
            name: "Spell Platform Pro".to_string(),
            metadata: Some(
                vec![("plan".to_string(), "pro".to_string())]
                    .into_iter()
                    .collect(),
            ),
            ..Default::default()
        });

        let price = Price::create(client, create_price).await?;

        let user_id_str = user_id.to_string();

        let mut params = CreateCheckoutSession::new();
        params.mode = Some(CheckoutSessionMode::Subscription);
        params.line_items = Some(vec![CreateCheckoutSessionLineItems {
            price: Some(price.id.to_string()),
            quantity: Some(1),
            ..Default::default()
        }]);
        params.success_url = Some(&success_url);
        params.cancel_url = Some(&cancel_url);
        params.client_reference_id = Some(&user_id_str);
        params.metadata = Some(
            vec![("user_id".to_string(), user_id_str.clone())]
                .into_iter()
                .collect(),
        );

        let session = CheckoutSession::create(client, params).await?;

        Ok(CheckoutSessionResponse {
            url: session
                .url
                .ok_or_else(|| anyhow::anyhow!("No URL in session"))?,
            session_id: session.id.to_string(),
        })
    }

    pub fn verify_webhook_signature(
        &self,
        payload: &[u8],
        signature: &str,
    ) -> Result<stripe::Event, anyhow::Error> {
        let webhook_secret = self
            .webhook_secret
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Webhook secret not configured"))?;

        let payload_str = std::str::from_utf8(payload)?;
        let event = Webhook::construct_event(payload_str, signature, webhook_secret)?;
        Ok(event)
    }

    pub async fn handle_webhook_event(
        &self,
        event: stripe::Event,
        db: &sqlx::PgPool,
    ) -> Result<(), anyhow::Error> {
        match event.type_ {
            EventType::CheckoutSessionCompleted => {
                if let EventObject::CheckoutSession(session) = event.data.object {
                    self.handle_checkout_completed(session, db).await?;
                }
            }
            EventType::CustomerSubscriptionUpdated | EventType::CustomerSubscriptionDeleted => {
                if let EventObject::Subscription(subscription) = event.data.object {
                    self.handle_subscription_updated(subscription, db).await?;
                }
            }
            EventType::SetupIntentSucceeded => {
                if let EventObject::SetupIntent(setup_intent) = event.data.object {
                    self.handle_setup_intent_succeeded(setup_intent, db).await?;
                }
            }
            _ => {
                log::info!("Unhandled webhook event type: {:?}", event.type_);
            }
        }
        Ok(())
    }

    async fn handle_checkout_completed(
        &self,
        session: CheckoutSession,
        db: &sqlx::PgPool,
    ) -> Result<(), anyhow::Error> {
        let user_id_str = session
            .client_reference_id
            .ok_or_else(|| anyhow::anyhow!("No client_reference_id"))?;
        let user_id = Uuid::parse_str(&user_id_str)?;

        let customer_id = session
            .customer
            .ok_or_else(|| anyhow::anyhow!("No customer ID"))?
            .id()
            .to_string();

        log::info!("Checkout completed for user {user_id} with customer {customer_id}");

        // Insert or update billing account
        sqlx::query(
            r#"
            INSERT INTO billing_accounts (user_id, stripe_customer_id, plan, status, created_at, updated_at)
            VALUES ($1, $2, 'pro', 'active', NOW(), NOW())
            ON CONFLICT (user_id) DO UPDATE
            SET stripe_customer_id = EXCLUDED.stripe_customer_id,
                plan = EXCLUDED.plan,
                status = EXCLUDED.status,
                updated_at = NOW()
            "#,
        )
        .bind(user_id)
        .bind(&customer_id)
        .execute(db)
        .await?;

        Ok(())
    }

    async fn handle_subscription_updated(
        &self,
        subscription: stripe::Subscription,
        db: &sqlx::PgPool,
    ) -> Result<(), anyhow::Error> {
        let customer_id = subscription.customer.id().to_string();
        let status = match subscription.status {
            stripe::SubscriptionStatus::Active => "active",
            stripe::SubscriptionStatus::PastDue => "past_due",
            stripe::SubscriptionStatus::Canceled => "canceled",
            _ => "inactive",
        };

        log::info!("Subscription updated for customer {customer_id} to status {status}");

        sqlx::query(
            r#"
            UPDATE billing_accounts
            SET status = $2, updated_at = NOW()
            WHERE stripe_customer_id = $1
            "#,
        )
        .bind(&customer_id)
        .bind(status)
        .execute(db)
        .await?;

        Ok(())
    }

    async fn handle_setup_intent_succeeded(
        &self,
        setup_intent: SetupIntent,
        db: &sqlx::PgPool,
    ) -> Result<(), anyhow::Error> {
        let customer_id = setup_intent
            .customer
            .ok_or_else(|| anyhow::anyhow!("No customer in SetupIntent"))?
            .id()
            .to_string();

        let payment_method_id = setup_intent
            .payment_method
            .ok_or_else(|| anyhow::anyhow!("No payment method in SetupIntent"))?
            .id()
            .to_string();

        log::info!(
            "SetupIntent succeeded: customer={customer_id} payment_method={payment_method_id}"
        );

        // Update billing account with payment method
        sqlx::query(
            r#"
            UPDATE billing_accounts
            SET payment_method_id = $2, updated_at = NOW()
            WHERE stripe_customer_id = $1
            "#,
        )
        .bind(&customer_id)
        .bind(&payment_method_id)
        .execute(db)
        .await?;

        log::info!("Payment method {payment_method_id} saved for customer {customer_id}");

        Ok(())
    }

    /// Get or create a Stripe customer for the user
    pub async fn get_or_create_customer(
        &self,
        db: &sqlx::PgPool,
        user_id: Uuid,
    ) -> Result<String, anyhow::Error> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Stripe not configured"))?;

        // Detect current Stripe mode from secret key
        let secret_key = std::env::var("STRIPE_SECRET_KEY")
            .map_err(|_| anyhow::anyhow!("STRIPE_SECRET_KEY not set"))?;
        let is_live_mode = secret_key.starts_with("sk_live_");

        log::debug!(
            "Stripe mode: {}",
            if is_live_mode { "LIVE" } else { "TEST" }
        );

        // Check mode-specific customer ID first, fallback to legacy column
        let existing: Option<(Option<String>, Option<String>, Option<String>)> = sqlx::query_as(
            r#"
            SELECT stripe_customer_id, stripe_customer_id_live, stripe_customer_id_test
            FROM billing_accounts
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(db)
        .await?;

        let customer_id_option = if let Some((legacy, live_col, test_col)) = existing {
            if is_live_mode {
                live_col.or(legacy.clone()) // Prefer live column, fallback to legacy
            } else {
                test_col.or(legacy) // Prefer test column, fallback to legacy
            }
        } else {
            None
        };

        if let Some(customer_id) = customer_id_option {
            match customer_id.parse::<stripe::CustomerId>() {
                Ok(parsed_id) => match Customer::retrieve(client, &parsed_id, &[]).await {
                    Ok(customer) => {
                        let livemode = customer.livemode.unwrap_or(false);

                        // Verify mode consistency
                        if livemode != is_live_mode {
                            log::warn!(
                                "Mode mismatch: customer {} is in {} mode but API key is {} mode; creating new customer",
                                customer_id,
                                if livemode { "LIVE" } else { "TEST" },
                                if is_live_mode { "LIVE" } else { "TEST" }
                            );
                        } else {
                            log::info!(
                                "Reusing existing Stripe customer {customer_id} for user {user_id} (livemode: {livemode})"
                            );
                            return Ok(customer_id);
                        }
                    }
                    Err(err) if is_missing_customer_error(&err) => {
                        log::warn!(
                            "Stored Stripe customer {customer_id} for user {user_id} not found in current Stripe environment; creating new customer"
                        );
                    }
                    Err(err) => {
                        return Err(err.into());
                    }
                },
                Err(_) => {
                    log::warn!(
                        "Invalid Stripe customer id '{customer_id}' stored for user {user_id}; creating new customer"
                    );
                }
            }
        }

        // Get user info for customer creation
        let user: crate::models::User = sqlx::query_as(
            r#"
            SELECT * FROM users WHERE id = $1
            "#,
        )
        .bind(user_id)
        .fetch_one(db)
        .await?;

        // Create new customer in Stripe
        let mut params = stripe::CreateCustomer::new();
        params.email = user.github_email.as_deref();
        params.name = user.github_name.as_deref();
        params.metadata = Some(
            vec![
                ("user_id".to_string(), user_id.to_string()),
                ("github_login".to_string(), user.github_login.clone()),
            ]
            .into_iter()
            .collect(),
        );

        let customer = Customer::create(client, params).await?;
        let customer_id = customer.id.to_string();

        // Dual-write: Save to both mode-specific column AND legacy column (for backward compatibility)
        if is_live_mode {
            sqlx::query(
                r#"
                INSERT INTO billing_accounts (user_id, stripe_customer_id, stripe_customer_id_live, plan, status)
                VALUES ($1, $2, $2, 'free', 'active')
                ON CONFLICT (user_id) DO UPDATE SET
                    stripe_customer_id = EXCLUDED.stripe_customer_id,
                    stripe_customer_id_live = EXCLUDED.stripe_customer_id_live,
                    updated_at = NOW()
                "#,
            )
            .bind(user_id)
            .bind(&customer_id)
            .execute(db)
            .await?;
        } else {
            sqlx::query(
                r#"
                INSERT INTO billing_accounts (user_id, stripe_customer_id, stripe_customer_id_test, plan, status)
                VALUES ($1, $2, $2, 'free', 'active')
                ON CONFLICT (user_id) DO UPDATE SET
                    stripe_customer_id = EXCLUDED.stripe_customer_id,
                    stripe_customer_id_test = EXCLUDED.stripe_customer_id_test,
                    updated_at = NOW()
                "#,
            )
            .bind(user_id)
            .bind(&customer_id)
            .execute(db)
            .await?;
        }

        log::info!(
            "Created Stripe customer {} for user {} (mode: {})",
            customer_id,
            user.github_login,
            if is_live_mode { "LIVE" } else { "TEST" }
        );

        Ok(customer_id)
    }

    /// Create a SetupIntent for card registration
    pub async fn create_setup_intent(
        &self,
        customer_id: &str,
    ) -> Result<SetupIntent, SetupIntentFailure> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(SetupIntentFailure::configuration)?;

        let parsed_customer = customer_id
            .parse::<stripe::CustomerId>()
            .map_err(|_| SetupIntentFailure::invalid_customer_id(customer_id))?;

        let mut params = CreateSetupIntent::new();
        params.customer = Some(parsed_customer);
        params.automatic_payment_methods = Some(CreateSetupIntentAutomaticPaymentMethods {
            enabled: true,
            allow_redirects: None,
        });

        match SetupIntent::create(client, params).await {
            Ok(setup_intent) => Ok(setup_intent),
            Err(err) => {
                let failure = SetupIntentFailure::from_stripe_error(&err);
                log_stripe_error("setup_intent_create_failed", &failure, &err);
                Err(failure)
            }
        }
    }

    /// Attach a payment method to a customer
    pub async fn attach_payment_method(
        &self,
        customer_id: &str,
        payment_method_id: &str,
    ) -> Result<(), anyhow::Error> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Stripe not configured"))?;

        let payment_method =
            PaymentMethod::retrieve(client, &payment_method_id.parse()?, &[]).await?;

        // Attach payment method to customer
        PaymentMethod::attach(
            client,
            &payment_method.id,
            stripe::AttachPaymentMethod {
                customer: customer_id.parse()?,
            },
        )
        .await?;

        // Set as default payment method
        let mut update_params = stripe::UpdateCustomer::new();
        update_params.invoice_settings = Some(stripe::CustomerInvoiceSettings {
            default_payment_method: Some(payment_method.id.to_string()),
            ..Default::default()
        });

        Customer::update(client, &customer_id.parse()?, update_params).await?;

        Ok(())
    }

    /// Get payment method details (card last 4, brand, etc.)
    pub async fn get_payment_method(
        &self,
        payment_method_id: &str,
    ) -> Result<stripe::PaymentMethod, anyhow::Error> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Stripe not configured"))?;

        let payment_method =
            PaymentMethod::retrieve(client, &payment_method_id.parse()?, &[]).await?;

        Ok(payment_method)
    }

    /// Detach payment method from customer
    pub async fn detach_payment_method(
        &self,
        payment_method_id: &str,
    ) -> Result<(), anyhow::Error> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Stripe not configured"))?;

        PaymentMethod::detach(client, &payment_method_id.parse()?).await?;

        Ok(())
    }

    /// Create monthly invoice for a customer
    pub async fn create_monthly_invoice(
        &self,
        customer_id: &str,
        amount_cents: i32,
        description: &str,
    ) -> Result<Invoice, anyhow::Error> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Stripe not configured"))?;

        // Create invoice item
        let mut item_params = CreateInvoiceItem::new(customer_id.parse()?);
        item_params.amount = Some(amount_cents.into());
        item_params.currency = Some(Currency::USD);
        item_params.description = Some(description);

        InvoiceItem::create(client, item_params).await?;

        // Create invoice
        let mut invoice_params = CreateInvoice::new();
        invoice_params.customer = Some(customer_id.parse()?);
        invoice_params.auto_advance = Some(true); // Automatically finalize and attempt payment
        invoice_params.collection_method = Some(stripe::CollectionMethod::ChargeAutomatically);
        invoice_params.description = Some(description);

        let invoice = Invoice::create(client, invoice_params).await?;

        // Finalize invoice
        let finalized = Invoice::finalize(client, &invoice.id, Default::default()).await?;

        Ok(finalized)
    }
}

fn is_missing_customer_error(err: &stripe::StripeError) -> bool {
    match err {
        stripe::StripeError::Stripe(request_error) => {
            if request_error.http_status == 404 {
                return true;
            }

            matches!(
                request_error.code,
                Some(ErrorCode::ResourceMissing)
                    | Some(ErrorCode::LivemodeMismatch)
                    | Some(ErrorCode::TestmodeChargesOnly)
            ) || request_error
                .message
                .as_ref()
                .map(|msg| msg.to_lowercase().contains("no such customer"))
                .unwrap_or(false)
        }
        _ => false,
    }
}

fn log_stripe_error(context: &str, failure: &SetupIntentFailure, err: &stripe::StripeError) {
    log::error!(
        "{context} status={:?} code={:?} msg={:?} request_id={:?} source={}",
        failure.status,
        failure.code,
        failure.message,
        failure.request_id,
        err
    );
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SetupIntentFailure {
    pub status: Option<u16>,
    pub code: Option<String>,
    pub message: Option<String>,
    pub request_id: Option<String>,
}

impl SetupIntentFailure {
    fn configuration() -> Self {
        Self {
            status: None,
            code: Some("configuration_error".to_string()),
            message: Some("Stripe not configured".to_string()),
            request_id: None,
        }
    }

    fn invalid_customer_id(customer_id: &str) -> Self {
        Self {
            status: None,
            code: Some("invalid_customer_id".to_string()),
            message: Some(format!("Invalid Stripe customer id: {customer_id}")),
            request_id: None,
        }
    }

    fn from_stripe_error(err: &stripe::StripeError) -> Self {
        match err {
            stripe::StripeError::Stripe(request_error) => Self {
                status: Some(request_error.http_status),
                code: request_error.code.as_ref().map(|c| c.to_string()),
                message: request_error.message.clone(),
                request_id: None,
            },
            _ => Self {
                status: None,
                code: Some("client_error".to_string()),
                message: Some(err.to_string()),
                request_id: None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_stripe_error(
        http_status: u16,
        error_type: stripe::ErrorType,
        code: Option<ErrorCode>,
        message: Option<&str>,
    ) -> stripe::StripeError {
        stripe::StripeError::Stripe(stripe::RequestError {
            http_status,
            error_type,
            message: message.map(ToString::to_string),
            code,
            decline_code: None,
            charge: None,
        })
    }

    #[test]
    fn detects_missing_customer_via_status_and_code() {
        let err = make_stripe_error(
            404,
            stripe::ErrorType::InvalidRequest,
            Some(ErrorCode::ResourceMissing),
            Some("No such customer: 'cus_test123'"),
        );

        assert!(is_missing_customer_error(&err));
    }

    #[test]
    fn ignores_unrelated_stripe_errors() {
        let err = make_stripe_error(
            400,
            stripe::ErrorType::InvalidRequest,
            Some(ErrorCode::ParameterMissing),
            Some("Missing required param: customer"),
        );

        assert!(!is_missing_customer_error(&err));
    }
}
