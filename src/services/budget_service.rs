use chrono::{DateTime, Datelike, Duration, Utc};
use uuid::Uuid;

use crate::models::billing::{Budget, BudgetExceededError};

pub struct BudgetService;

impl BudgetService {
    /// Check if user has exceeded their hard budget limit
    /// Returns Ok(()) if within limit, Err with details if exceeded
    pub async fn check_hard_limit(
        user_id: &Uuid,
        db: &sqlx::PgPool,
    ) -> Result<(), BudgetExceededError> {
        // Get user's budget
        let budget: Option<Budget> = sqlx::query_as(
            r#"
            SELECT * FROM budgets WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(db)
        .await
        .map_err(|e| {
            log::error!("Failed to fetch budget: {e}");
            BudgetExceededError::new("monthly".to_string(), 0, 0)
        })?;

        // If no budget set, allow
        let budget = match budget {
            Some(b) => b,
            None => return Ok(()),
        };

        // If no hard limit set, allow
        let hard_limit = match budget.hard_limit_cents {
            Some(limit) => limit,
            None => return Ok(()),
        };

        // Calculate current period window
        let (window_start, window_end) = match budget.period.as_str() {
            "daily" => Self::daily_window(),
            "monthly" => Self::monthly_window(),
            _ => Self::monthly_window(),
        };

        // Get current spending in this period
        let spent: Option<(i32,)> = sqlx::query_as(
            r#"
            SELECT COALESCE(SUM(cost_cents), 0) as total
            FROM usage_counters
            WHERE user_id = $1
              AND window_start >= $2
              AND window_end <= $3
            "#,
        )
        .bind(user_id)
        .bind(window_start)
        .bind(window_end)
        .fetch_optional(db)
        .await
        .map_err(|e| {
            log::error!("Failed to fetch usage: {e}");
            BudgetExceededError::new(budget.period.clone(), hard_limit, 0)
        })?;

        let spent_cents = spent.map(|(total,)| total).unwrap_or(0);

        if spent_cents >= hard_limit {
            log::warn!(
                "User {} exceeded hard limit: {} >= {} (period: {})",
                user_id,
                spent_cents,
                hard_limit,
                budget.period
            );
            return Err(BudgetExceededError::new(
                budget.period,
                hard_limit,
                spent_cents,
            ));
        }

        Ok(())
    }

    /// Record usage after a successful cast
    pub async fn record_usage(
        user_id: &Uuid,
        cost_cents: i32,
        cast_id: &Uuid,
        db: &sqlx::PgPool,
    ) -> Result<(), anyhow::Error> {
        let (window_start, window_end) = Self::monthly_window();

        // Insert or update usage counter
        sqlx::query(
            r#"
            INSERT INTO usage_counters (user_id, window_start, window_end, calls, cost_cents, created_at, updated_at)
            VALUES ($1, $2, $3, 1, $4, NOW(), NOW())
            ON CONFLICT (user_id, window_start) DO UPDATE
            SET calls = usage_counters.calls + 1,
                cost_cents = usage_counters.cost_cents + EXCLUDED.cost_cents,
                updated_at = NOW()
            "#,
        )
        .bind(user_id)
        .bind(window_start)
        .bind(window_end)
        .bind(cost_cents)
        .execute(db)
        .await?;

        // Update cost in casts table
        sqlx::query(
            r#"
            UPDATE casts
            SET cost_cents = $2
            WHERE id = $1
            "#,
        )
        .bind(cast_id)
        .bind(cost_cents)
        .execute(db)
        .await?;

        log::debug!("Recorded usage for user {user_id}: {cost_cents} cents for cast {cast_id}");

        Ok(())
    }

    /// Get current usage for a user in their budget period
    pub async fn get_current_usage(
        user_id: &Uuid,
        period: &str,
        db: &sqlx::PgPool,
    ) -> Result<(i32, i32), anyhow::Error> {
        let (window_start, window_end) = match period {
            "daily" => Self::daily_window(),
            "monthly" => Self::monthly_window(),
            _ => Self::monthly_window(),
        };

        let result: Option<(i32, i32)> = sqlx::query_as(
            r#"
            SELECT COALESCE(SUM(calls), 0) as total_calls,
                   COALESCE(SUM(cost_cents), 0) as total_cost
            FROM usage_counters
            WHERE user_id = $1
              AND window_start >= $2
              AND window_end <= $3
            "#,
        )
        .bind(user_id)
        .bind(window_start)
        .bind(window_end)
        .fetch_optional(db)
        .await?;

        Ok(result.unwrap_or((0, 0)))
    }

    fn daily_window() -> (DateTime<Utc>, DateTime<Utc>) {
        let now = Utc::now();
        let start = now.date_naive().and_hms_opt(0, 0, 0).unwrap();
        let end = start + Duration::days(1);
        (
            DateTime::from_naive_utc_and_offset(start, Utc),
            DateTime::from_naive_utc_and_offset(end, Utc),
        )
    }

    fn monthly_window() -> (DateTime<Utc>, DateTime<Utc>) {
        let now = Utc::now();
        let year = now.year();
        let month = now.month();

        let start = chrono::NaiveDate::from_ymd_opt(year, month, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();

        let next_month = if month == 12 {
            chrono::NaiveDate::from_ymd_opt(year + 1, 1, 1)
        } else {
            chrono::NaiveDate::from_ymd_opt(year, month + 1, 1)
        }
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap();

        (
            DateTime::from_naive_utc_and_offset(start, Utc),
            DateTime::from_naive_utc_and_offset(next_month, Utc),
        )
    }
}
