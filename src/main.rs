mod db;
mod errors;
mod middleware;
mod models;
mod routes;
mod wasm;

use actix_web::{web, App, HttpResponse, HttpServer};
use dotenv::dotenv;
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let wasm_path = env::var("WASM_MODULE_PATH").unwrap_or_else(|_| "./modules".to_string());

    log::info!("Connecting to database...");
    let pool = db::create_pool(&database_url)
        .await
        .expect("Failed to create database pool");

    log::info!("Initializing WASM runtime...");
    let wasm_runtime = wasm::WasmRuntime::new(&wasm_path);

    let app_data = web::Data::new(AppState {
        db: pool,
        wasm: wasm_runtime,
    });

    log::info!("Starting server on 0.0.0.0:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(app_data.clone())
            .route("/healthz", web::get().to(healthz))
            .configure(routes::auth::configure)
            .service(web::scope("/v1").configure(routes::cast::configure))
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
}
