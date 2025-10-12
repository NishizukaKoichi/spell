use actix_web::{web, HttpResponse};
use parking_lot::Mutex;
use prometheus::{opts, Counter, Encoder, Gauge, Histogram, HistogramOpts, Registry, TextEncoder};
use std::sync::Arc;

pub struct Metrics {
    pub registry: Arc<Registry>,
    pub cast_total: Counter,
    pub cast_failed: Counter,
    pub cast_duration: Histogram,
    pub rate_limited_total: Counter,
    pub budget_blocked_total: Counter,
    pub stripe_webhook_total: Counter,
    pub db_pool_in_use: Gauge,
    pub redis_errors_total: Counter,
}

impl Metrics {
    pub fn new() -> Self {
        let registry = Arc::new(Registry::new());

        let cast_total =
            Counter::with_opts(opts!("spell_cast_total", "Total number of spell casts")).unwrap();
        registry.register(Box::new(cast_total.clone())).unwrap();

        let cast_failed = Counter::with_opts(opts!(
            "spell_cast_failed_total",
            "Total number of failed spell casts"
        ))
        .unwrap();
        registry.register(Box::new(cast_failed.clone())).unwrap();

        let cast_duration = Histogram::with_opts(
            HistogramOpts::new("spell_cast_duration_seconds", "Duration of spell casts")
                .buckets(vec![0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]),
        )
        .unwrap();
        registry.register(Box::new(cast_duration.clone())).unwrap();

        let rate_limited_total = Counter::with_opts(opts!(
            "spell_rate_limited_total",
            "Total number of rate limited requests"
        ))
        .unwrap();
        registry
            .register(Box::new(rate_limited_total.clone()))
            .unwrap();

        let budget_blocked_total = Counter::with_opts(opts!(
            "spell_budget_blocked_total",
            "Total number of budget blocked requests"
        ))
        .unwrap();
        registry
            .register(Box::new(budget_blocked_total.clone()))
            .unwrap();

        let stripe_webhook_total = Counter::with_opts(opts!(
            "spell_stripe_webhook_total",
            "Total number of Stripe webhook events"
        ))
        .unwrap();
        registry
            .register(Box::new(stripe_webhook_total.clone()))
            .unwrap();

        let db_pool_in_use = Gauge::with_opts(opts!(
            "spell_db_pool_in_use",
            "Number of database connections in use"
        ))
        .unwrap();
        registry.register(Box::new(db_pool_in_use.clone())).unwrap();

        let redis_errors_total = Counter::with_opts(opts!(
            "spell_redis_errors_total",
            "Total number of Redis errors"
        ))
        .unwrap();
        registry
            .register(Box::new(redis_errors_total.clone()))
            .unwrap();

        Self {
            registry,
            cast_total,
            cast_failed,
            cast_duration,
            rate_limited_total,
            budget_blocked_total,
            stripe_webhook_total,
            db_pool_in_use,
            redis_errors_total,
        }
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/metrics", web::get().to(get_metrics));
}

async fn get_metrics(
    metrics: web::Data<Arc<Mutex<Metrics>>>,
) -> Result<HttpResponse, actix_web::Error> {
    let metrics = metrics.lock();
    let encoder = TextEncoder::new();
    let metric_families = metrics.registry.gather();

    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).map_err(|e| {
        log::error!("Failed to encode metrics: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to encode metrics")
    })?;

    Ok(HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4")
        .body(buffer))
}
