use stripe::{
    CheckoutSession, CheckoutSessionMode, Client, CreateCheckoutSession,
    CreateCheckoutSessionLineItems, CreateInvoice, CreateInvoiceItem, CreatePrice,
    CreatePriceRecurring, CreateSetupIntent, Currency, Customer, EventObject, EventType, Invoice,
    InvoiceItem, PaymentMethod, Price, SetupIntent, Webhook,
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

        // Check if customer already exists in DB
        let existing: Option<(String,)> = sqlx::query_as(
            r#"
            SELECT stripe_customer_id FROM billing_accounts
            WHERE user_id = $1 AND stripe_customer_id IS NOT NULL
            "#,
        )
        .bind(user_id)
        .fetch_optional(db)
        .await?;

        if let Some((customer_id,)) = existing {
            return Ok(customer_id);
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

        // Save to database
        sqlx::query(
            r#"
            INSERT INTO billing_accounts (user_id, stripe_customer_id, plan, status)
            VALUES ($1, $2, 'free', 'active')
            ON CONFLICT (user_id) DO UPDATE SET
                stripe_customer_id = EXCLUDED.stripe_customer_id,
                updated_at = NOW()
            "#,
        )
        .bind(user_id)
        .bind(&customer_id)
        .execute(db)
        .await?;

        log::info!(
            "Created Stripe customer {} for user {}",
            customer_id,
            user.github_login
        );

        Ok(customer_id)
    }

    /// Create a SetupIntent for card registration
    pub async fn create_setup_intent(
        &self,
        customer_id: &str,
    ) -> Result<SetupIntent, anyhow::Error> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Stripe not configured"))?;

        let mut params = CreateSetupIntent::new();
        params.customer = Some(customer_id.parse()?);
        params.payment_method_types = Some(vec!["card".to_string()]);

        let setup_intent = SetupIntent::create(client, params).await?;

        Ok(setup_intent)
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
