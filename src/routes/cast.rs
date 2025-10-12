use crate::errors::CastError;
use crate::models::{CastRequest, CastResponse, User};
use crate::services::budget_service::BudgetService;
use crate::AppState;
use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use actix_web_httpauth::middleware::HttpAuthentication;
use uuid::Uuid;

pub fn configure(cfg: &mut web::ServiceConfig) {
    let auth = HttpAuthentication::bearer(crate::middleware::auth::validator);
    cfg.service(
        web::resource("/cast")
            .wrap(auth)
            .route(web::post().to(cast_spell)),
    );
}

async fn cast_spell(
    state: web::Data<AppState>,
    http_req: HttpRequest,
    req: web::Json<CastRequest>,
) -> Result<HttpResponse, CastError> {
    // Get authenticated user
    let user_id = {
        let ext = http_req.extensions();
        ext.get::<User>()
            .ok_or_else(|| CastError::WasmExecutionFailed("User not authenticated".to_string()))?
            .id
    };

    // Check budget hard limit BEFORE execution
    if let Err(budget_err) = BudgetService::check_hard_limit(&user_id, &state.db).await {
        log::warn!("Budget exceeded for user {}: {:?}", user_id, budget_err);
        return Err(CastError::BudgetExceeded(budget_err));
    }

    let cast_id = Uuid::new_v4();
    let spell_name = &req.spell_name;
    let payload = &req.payload;

    log::info!(
        "Cast {} starting for spell: {} by user {}",
        cast_id,
        spell_name,
        user_id
    );

    // Insert initial record with user_id
    sqlx::query(
        r#"
        INSERT INTO casts (id, spell_name, payload, status, user_id, created_at)
        VALUES ($1, $2, $3, 'QUEUED', $4, NOW())
        "#,
    )
    .bind(&cast_id)
    .bind(spell_name)
    .bind(payload)
    .bind(&user_id)
    .execute(&state.db)
    .await?;

    // Execute WASM
    let result = match state.wasm.execute_spell(spell_name, payload.clone()) {
        Ok(output) => {
            // Update with success
            sqlx::query(
                r#"
                UPDATE casts
                SET status = 'COMPLETED', result = $2
                WHERE id = $1
                "#,
            )
            .bind(&cast_id)
            .bind(&output)
            .execute(&state.db)
            .await?;

            log::info!("Cast {} completed successfully", cast_id);

            // Record usage and cost
            let cost_cents = std::env::var("COST_PER_CAST_CENTS")
                .ok()
                .and_then(|v| v.parse::<i32>().ok())
                .unwrap_or(0);

            if cost_cents > 0 {
                if let Err(e) =
                    BudgetService::record_usage(&user_id, cost_cents, &cast_id, &state.db).await
                {
                    log::error!("Failed to record usage for cast {}: {}", cast_id, e);
                    // Continue anyway - don't fail the cast
                }
            }

            CastResponse {
                id: cast_id,
                status: "COMPLETED".to_string(),
                result: Some(output),
                error_code: None,
                created_at: chrono::Utc::now(),
            }
        }
        Err(e) => {
            // Update with error
            let error_code = e.error_code();
            sqlx::query(
                r#"
                UPDATE casts
                SET status = 'FAILED', error_code = $2
                WHERE id = $1
                "#,
            )
            .bind(&cast_id)
            .bind(error_code)
            .execute(&state.db)
            .await?;

            log::error!("Cast {} failed: {}", cast_id, e);

            return Err(e);
        }
    };

    Ok(HttpResponse::Ok().json(result))
}
