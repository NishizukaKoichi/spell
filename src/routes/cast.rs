use crate::errors::CastError;
use crate::models::{Cast, CastRequest, CastResponse};
use crate::AppState;
use actix_web::{web, HttpResponse};
use uuid::Uuid;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/cast").route(web::post().to(cast_spell)));
}

async fn cast_spell(
    state: web::Data<AppState>,
    req: web::Json<CastRequest>,
) -> Result<HttpResponse, CastError> {
    let cast_id = Uuid::new_v4();
    let spell_name = &req.spell_name;
    let payload = &req.payload;

    log::info!("Cast {} starting for spell: {}", cast_id, spell_name);

    // Insert initial record
    sqlx::query(
        r#"
        INSERT INTO casts (id, spell_name, payload, status, created_at)
        VALUES ($1, $2, $3, 'QUEUED', NOW())
        "#,
    )
    .bind(&cast_id)
    .bind(spell_name)
    .bind(payload)
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
