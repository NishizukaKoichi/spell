mod db;
mod errors;
mod middleware;
mod models;
mod routes;
mod services;
mod utils;
mod wasm;

use actix_cors::Cors;
use actix_web::{http::header, middleware::DefaultHeaders, web, App, HttpResponse, HttpServer};
use dotenvy::dotenv;
use parking_lot::Mutex;
use std::env;
use std::sync::Arc;

use routes::metrics::Metrics;
use services::stripe_service::StripeService;

fn cors() -> Cors {
    Cors::default()
        .allowed_origin("https://magicspell.io")
        .allowed_origin("https://studio.magicspell.dev")
        .allowed_methods(vec!["GET", "POST", "PUT", "PATCH", "DELETE"])
        .allowed_headers(vec![
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
        ])
        .allowed_header("Stripe-Signature")
        .allowed_header("X-CSRF-Token")
        .expose_headers(vec![
            "RateLimit-Limit",
            "RateLimit-Remaining",
            "RateLimit-Reset",
        ])
        .supports_credentials()
        .max_age(3600)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL must be set");
    let wasm_path = env::var("WASM_MODULE_PATH").unwrap_or_else(|_| "./modules".to_string());

    log::info!("Connecting to database...");
    let pool = db::create_pool(&database_url)
        .await
        .expect("Failed to create database pool");

    log::info!("Connecting to Redis...");
    let redis_config = deadpool_redis::Config::from_url(&redis_url);
    let redis_pool = redis_config
        .create_pool(Some(deadpool_redis::Runtime::Tokio1))
        .expect("Failed to create Redis pool");

    log::info!("Initializing WASM runtime...");
    let wasm_runtime = wasm::WasmRuntime::new(&wasm_path);

    log::info!("Initializing Stripe service...");
    let stripe_service = StripeService::new();
    let stripe_enabled = stripe_service.is_enabled();
    let stripe_data = if stripe_enabled {
        Some(stripe_service)
    } else {
        log::warn!("Stripe service disabled - billing features unavailable");
        None
    };

    log::info!("Initializing metrics...");
    let metrics = Arc::new(Mutex::new(Metrics::new()));

    let app_data = web::Data::new(AppState {
        db: pool,
        wasm: wasm_runtime,
        redis: redis_pool.clone(),
        stripe: stripe_data,
    });

    let metrics_data = web::Data::new(metrics.clone());

    log::info!("Starting server on 0.0.0.0:8080");

    HttpServer::new(move || {
        App::new()
            .wrap(cors())
            .wrap(
                DefaultHeaders::new()
                    .add(("Strict-Transport-Security", "max-age=31536000; includeSubDomains"))
                    .add(("Referrer-Policy", "strict-origin-when-cross-origin"))
                    .add(("X-Frame-Options", "DENY"))
                    .add(("Permissions-Policy", "geolocation=(), microphone=(), camera=()"))
                    .add(("Vary", "Origin"))
            )
            .wrap(middleware::rate_limit::RateLimit::new(redis_pool.clone()))
            .app_data(app_data.clone())
            .app_data(metrics_data.clone())
            .route("/healthz", web::get().to(healthz))
            .configure(routes::metrics::configure)
            .configure(routes::auth::configure)
            .configure(routes::gdpr::config)
            .configure(routes::admin::configure)
            .service(
                web::scope("/v1")
                    .configure(routes::cast::configure)
                    .configure(routes::keys::configure)
                    .configure(routes::billing::configure)
                    .configure(routes::budgets::configure),
            )
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}

async fn healthz() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

pub struct AppState {
    pub db: sqlx::PgPool,
    pub wasm: wasm::WasmRuntime,
    pub redis: deadpool_redis::Pool,
    pub stripe: Option<StripeService>,
}
