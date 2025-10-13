use chrono::{Datelike, Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::services::stripe_service::StripeService;

pub struct BillingService;

impl BillingService {
    /// Run monthly billing for all users with usage
    pub async fn process_monthly_billing(
        stripe: &StripeService,
        db: &PgPool,
    ) -> Result<(), anyhow::Error> {
        log::info!("Starting monthly billing process");

        // Get last month's date range
        let now = Utc::now();
        let last_month_end = now.date_naive().with_day(1).unwrap() - Duration::days(1);
        let last_month_start = last_month_end.with_day(1).unwrap();

        log::info!("Processing billing for period: {last_month_start} to {last_month_end}");

        // Get all users with usage in the last month
        let users_with_usage: Vec<(Uuid, i64, i32)> = sqlx::query_as(
            r#"
            SELECT
                user_id,
                COUNT(*) as total_calls,
                COALESCE(SUM(cost_cents), 0) as total_cost
            FROM usage_counters
            WHERE DATE(created_at) >= $1 AND DATE(created_at) <= $2
            GROUP BY user_id
            HAVING SUM(cost_cents) > 0
            "#,
        )
        .bind(last_month_start)
        .bind(last_month_end)
        .fetch_all(db)
        .await?;

        log::info!("Found {} users with usage", users_with_usage.len());

        let mut success_count = 0;
        let mut error_count = 0;

        for (user_id, total_calls, total_cost) in users_with_usage {
            match Self::bill_user(
                stripe,
                db,
                &user_id,
                total_calls,
                total_cost,
                &last_month_start,
            )
            .await
            {
                Ok(invoice_id) => {
                    log::info!(
                        "Created invoice {} for user {} (${:.2}, {} calls)",
                        invoice_id,
                        user_id,
                        total_cost as f64 / 100.0,
                        total_calls
                    );
                    success_count += 1;
                }
                Err(e) => {
                    log::error!("Failed to bill user {user_id}: {e}");
                    error_count += 1;
                }
            }
        }

        log::info!("Monthly billing completed: {success_count} successful, {error_count} errors");

        Ok(())
    }

    /// Bill a single user for their monthly usage
    async fn bill_user(
        stripe: &StripeService,
        db: &PgPool,
        user_id: &Uuid,
        total_calls: i64,
        total_cost: i32,
        period_start: &chrono::NaiveDate,
    ) -> Result<String, anyhow::Error> {
        // Get user's Stripe customer ID
        let customer_id: Option<(String,)> = sqlx::query_as(
            r#"
            SELECT stripe_customer_id FROM billing_accounts WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(db)
        .await?;

        let customer_id = customer_id
            .ok_or_else(|| anyhow::anyhow!("No billing account found for user {user_id}"))?
            .0;

        // Create invoice description
        let description = format!(
            "Spell Platform API Usage - {} ({} calls, ${:.2})",
            period_start.format("%B %Y"),
            total_calls,
            total_cost as f64 / 100.0
        );

        // Create invoice via Stripe
        let invoice = stripe
            .create_monthly_invoice(&customer_id, total_cost, &description)
            .await?;

        Ok(invoice.id.to_string())
    }
}
