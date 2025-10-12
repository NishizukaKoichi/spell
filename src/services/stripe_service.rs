use stripe::{
    CheckoutSession, CheckoutSessionMode, Client, CreateCheckoutSession,
    CreateCheckoutSessionLineItems, CreatePrice, CreatePriceRecurring, Currency, EventObject,
    EventType, Price, Webhook,
};
use uuid::Uuid;

use crate::models::billing::CheckoutSessionResponse;

pub struct StripeService {
    client: Option<Client>,
    webhook_secret: Option<String>,
}

impl StripeService {
    pub fn new() -> Self {
        let client = std::env::var("STRIPE_SECRET_KEY")
            .ok()
            .map(|key| Client::new(key));

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

        log::info!(
            "Checkout completed for user {} with customer {}",
            user_id,
            customer_id
        );

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
        .bind(&user_id)
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

        log::info!(
            "Subscription updated for customer {} to status {}",
            customer_id,
            status
        );

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
}
