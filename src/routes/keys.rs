use crate::middleware::auth::authenticate_from_cookie;
use crate::models::{ApiKey, CreateApiKeyRequest, CreateApiKeyResponse, ListApiKeyResponse, User};
use crate::utils::apikey::{extract_prefix, generate_api_key};
use crate::AppState;
use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use std::env;
use uuid::Uuid;

pub fn configure(cfg: &mut web::ServiceConfig) {
    let auth = actix_web_httpauth::middleware::HttpAuthentication::bearer(
        crate::middleware::auth::validator,
    );
    cfg.service(
        web::resource("/keys")
            .wrap(auth.clone())
            .route(web::post().to(create_api_key))
            .route(web::get().to(list_api_keys)),
    )
    .service(
        web::resource("/keys/{id}")
            .wrap(auth)
            .route(web::delete().to(delete_api_key)),
    )
    .service(
        web::resource("/api-keys")
            .route(web::post().to(create_api_key_cookie))
            .route(web::get().to(list_api_keys_cookie)),
    )
    .service(web::resource("/api-keys/{id}").route(web::delete().to(delete_api_key_cookie)));
}

async fn create_api_key(
    state: web::Data<AppState>,
    req: HttpRequest,
    payload: web::Json<CreateApiKeyRequest>,
) -> Result<HttpResponse, actix_web::Error> {
    // Get user from request extensions (set by auth middleware)
    let user_id = {
        let ext = req.extensions();
        ext.get::<User>()
            .ok_or_else(|| actix_web::error::ErrorUnauthorized("User not authenticated"))?
            .id
    };

    let prefix = env::var("API_KEY_PREFIX").unwrap_or_else(|_| "sk_live_".to_string());

    // Generate API key
    let (api_key, hash) = generate_api_key(&prefix).map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Failed to generate API key: {e}"))
    })?;

    // Extract prefix for storage
    let stored_prefix = extract_prefix(&api_key)
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("Failed to extract prefix"))?;

    // Insert into database
    let api_key_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO api_keys (user_id, name, prefix, hash)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
    )
    .bind(user_id)
    .bind(&payload.name)
    .bind(&stored_prefix)
    .bind(&hash)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Failed to create API key: {e}"))
    })?;

    log::info!("API key {api_key_id} created for user {user_id}");

    Ok(HttpResponse::Ok().json(CreateApiKeyResponse {
        id: api_key_id,
        name: payload.name.clone(),
        api_key,
    }))
}

async fn list_api_keys(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = {
        let ext = req.extensions();
        ext.get::<User>()
            .ok_or_else(|| actix_web::error::ErrorUnauthorized("User not authenticated"))?
            .id
    };

    let keys: Vec<ApiKey> = sqlx::query_as::<_, ApiKey>(
        r#"
        SELECT id, user_id, name, prefix, hash, created_at, last_used_at
        FROM api_keys
        WHERE user_id = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Failed to list API keys: {e}"))
    })?;

    let response: Vec<ListApiKeyResponse> = keys
        .into_iter()
        .map(|k| ListApiKeyResponse {
            id: k.id,
            name: k.name,
            created_at: k.created_at,
            last_used_at: k.last_used_at,
        })
        .collect();

    Ok(HttpResponse::Ok().json(response))
}

async fn delete_api_key(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = {
        let ext = req.extensions();
        ext.get::<User>()
            .ok_or_else(|| actix_web::error::ErrorUnauthorized("User not authenticated"))?
            .id
    };

    let key_id = path.into_inner();

    let result = sqlx::query(
        r#"
        DELETE FROM api_keys
        WHERE id = $1 AND user_id = $2
        "#,
    )
    .bind(key_id)
    .bind(user_id)
    .execute(&state.db)
    .await
    .map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Failed to delete API key: {e}"))
    })?;

    if result.rows_affected() == 0 {
        return Err(actix_web::error::ErrorNotFound("API key not found"));
    }

    log::info!("API key {key_id} deleted by user {user_id}");

    Ok(HttpResponse::NoContent().finish())
}

// Cookie-based authentication endpoints

async fn create_api_key_cookie(
    state: web::Data<AppState>,
    http_req: HttpRequest,
    payload: web::Json<CreateApiKeyRequest>,
) -> Result<HttpResponse, actix_web::Error> {
    let user = authenticate_from_cookie(&http_req, &state.db)
        .await
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("Not authenticated"))?;

    let prefix = env::var("API_KEY_PREFIX").unwrap_or_else(|_| "sk_live_".to_string());

    // Generate API key
    let (api_key, hash) = generate_api_key(&prefix).map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Failed to generate API key: {e}"))
    })?;

    // Extract prefix for storage
    let stored_prefix = extract_prefix(&api_key)
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("Failed to extract prefix"))?;

    // Insert into database
    let api_key_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO api_keys (user_id, name, prefix, hash)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
    )
    .bind(user.id)
    .bind(&payload.name)
    .bind(&stored_prefix)
    .bind(&hash)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Failed to create API key: {e}"))
    })?;

    log::info!(
        "API key {api_key_id} created for user {} ({})",
        user.github_login,
        user.id
    );

    Ok(HttpResponse::Ok().json(CreateApiKeyResponse {
        id: api_key_id,
        name: payload.name.clone(),
        api_key,
    }))
}

async fn list_api_keys_cookie(
    state: web::Data<AppState>,
    http_req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let user = authenticate_from_cookie(&http_req, &state.db)
        .await
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("Not authenticated"))?;

    let keys: Vec<ApiKey> = sqlx::query_as::<_, ApiKey>(
        r#"
        SELECT id, user_id, name, prefix, hash, created_at, last_used_at
        FROM api_keys
        WHERE user_id = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(user.id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Failed to list API keys: {e}"))
    })?;

    let response: Vec<ListApiKeyResponse> = keys
        .into_iter()
        .map(|k| ListApiKeyResponse {
            id: k.id,
            name: k.name,
            created_at: k.created_at,
            last_used_at: k.last_used_at,
        })
        .collect();

    Ok(HttpResponse::Ok().json(response))
}

async fn delete_api_key_cookie(
    state: web::Data<AppState>,
    http_req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, actix_web::Error> {
    let user = authenticate_from_cookie(&http_req, &state.db)
        .await
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("Not authenticated"))?;

    let key_id = path.into_inner();

    let result = sqlx::query(
        r#"
        DELETE FROM api_keys
        WHERE id = $1 AND user_id = $2
        "#,
    )
    .bind(key_id)
    .bind(user.id)
    .execute(&state.db)
    .await
    .map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Failed to delete API key: {e}"))
    })?;

    if result.rows_affected() == 0 {
        return Err(actix_web::error::ErrorNotFound("API key not found"));
    }

    log::info!(
        "API key {key_id} deleted by user {} ({})",
        user.github_login,
        user.id
    );

    Ok(HttpResponse::NoContent().finish())
}
