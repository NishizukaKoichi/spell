use crate::models::{ApiKey, Session, User};
use crate::utils::apikey::{extract_prefix, verify_api_key};
use actix_web::{dev::ServiceRequest, Error, HttpMessage};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use chrono::Utc;
use sqlx::PgPool;

pub async fn validator(
    mut req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let token = credentials.token();

    let pool = match req.app_data::<actix_web::web::Data<crate::AppState>>() {
        Some(data) => &data.db,
        None => {
            return Err((
                actix_web::error::ErrorInternalServerError("Database pool not found"),
                req,
            ));
        }
    };

    // Check if token is an API key (starts with "sk_")
    if token.starts_with("sk_") {
        match authenticate_api_key(pool, token).await {
            Ok(user) => {
                req.extensions_mut().insert(user);
                return Ok(req);
            }
            Err(e) => {
                log::debug!("API key authentication failed: {}", e);
                // Fall through to session authentication
            }
        }
    }

    // Try session authentication
    match authenticate_user(pool, token).await {
        Ok(user) => {
            req.extensions_mut().insert(user);
            Ok(req)
        }
        Err(_) => Err((
            actix_web::error::ErrorUnauthorized("Invalid or expired credentials"),
            req,
        )),
    }
}

async fn authenticate_api_key(pool: &PgPool, token: &str) -> Result<User, anyhow::Error> {
    // Extract prefix from API key
    let prefix = extract_prefix(token).ok_or_else(|| anyhow::anyhow!("Invalid API key format"))?;

    // Fetch all API keys with matching prefix
    let api_keys: Vec<ApiKey> = sqlx::query_as::<_, ApiKey>(
        r#"
        SELECT id, user_id, name, prefix, hash, created_at, last_used_at
        FROM api_keys
        WHERE prefix = $1
        "#,
    )
    .bind(&prefix)
    .fetch_all(pool)
    .await?;

    // Try to verify against each hash
    for api_key in api_keys {
        if verify_api_key(token, &api_key.hash)? {
            // Update last_used_at
            sqlx::query(
                r#"
                UPDATE api_keys
                SET last_used_at = NOW()
                WHERE id = $1
                "#,
            )
            .bind(&api_key.id)
            .execute(pool)
            .await?;

            // Fetch and return user
            let user: User = sqlx::query_as::<_, User>(
                r#"
                SELECT * FROM users WHERE id = $1
                "#,
            )
            .bind(api_key.user_id)
            .fetch_one(pool)
            .await?;

            return Ok(user);
        }
    }

    Err(anyhow::anyhow!("API key not found or invalid"))
}

async fn authenticate_user(pool: &PgPool, token: &str) -> Result<User, sqlx::Error> {
    let session: Session = sqlx::query_as::<_, Session>(
        r#"
        SELECT * FROM sessions
        WHERE token = $1 AND expires_at > $2
        "#,
    )
    .bind(token)
    .bind(Utc::now())
    .fetch_one(pool)
    .await?;

    let user: User = sqlx::query_as::<_, User>(
        r#"
        SELECT * FROM users WHERE id = $1
        "#,
    )
    .bind(session.user_id)
    .fetch_one(pool)
    .await?;

    Ok(user)
}
