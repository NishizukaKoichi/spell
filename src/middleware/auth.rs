use crate::models::{Session, User};
use actix_web::{dev::ServiceRequest, Error, HttpMessage};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use chrono::Utc;
use sqlx::PgPool;

pub async fn validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let token = credentials.token();

    let pool = req
        .app_data::<actix_web::web::Data<crate::AppState>>()
        .map(|data| &data.db)
        .ok_or_else(|| {
            (
                actix_web::error::ErrorInternalServerError("Database pool not found"),
                req,
            )
        })?;

    match authenticate_user(pool, token).await {
        Ok(user) => {
            req.extensions_mut().insert(user);
            Ok(req)
        }
        Err(_) => Err((
            actix_web::error::ErrorUnauthorized("Invalid or expired session"),
            req,
        )),
    }
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
