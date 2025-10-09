use crate::models::{GitHubAccessTokenResponse, GitHubUser, Session, User};
use crate::AppState;
use actix_web::{web, HttpResponse};
use chrono::{Duration, Utc};
use rand::Rng;
use serde::Deserialize;
use std::env;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct GitHubCallbackQuery {
    code: String,
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/auth/github").route(web::get().to(github_login)))
        .service(web::resource("/auth/github/callback").route(web::get().to(github_callback)));
}

async fn github_login() -> HttpResponse {
    let client_id = env::var("GITHUB_CLIENT_ID").unwrap_or_else(|_| "your_client_id".to_string());
    let redirect_uri =
        env::var("GITHUB_REDIRECT_URI").unwrap_or_else(|_| "http://localhost:8080/auth/github/callback".to_string());

    let url = format!(
        "https://github.com/login/oauth/authorize?client_id={}&redirect_uri={}&scope=user:email",
        client_id, redirect_uri
    );

    HttpResponse::Found()
        .append_header(("Location", url))
        .finish()
}

async fn github_callback(
    state: web::Data<AppState>,
    query: web::Query<GitHubCallbackQuery>,
) -> HttpResponse {
    let client_id = env::var("GITHUB_CLIENT_ID").unwrap_or_else(|_| "your_client_id".to_string());
    let client_secret =
        env::var("GITHUB_CLIENT_SECRET").unwrap_or_else(|_| "your_client_secret".to_string());

    // Exchange code for access token
    let client = reqwest::Client::new();
    let token_response = match client
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .json(&serde_json::json!({
            "client_id": client_id,
            "client_secret": client_secret,
            "code": query.code,
        }))
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(e) => {
            log::error!("Failed to exchange code for token: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to authenticate with GitHub"
            }));
        }
    };

    let token_data: GitHubAccessTokenResponse = match token_response.json().await {
        Ok(data) => data,
        Err(e) => {
            log::error!("Failed to parse token response: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to parse GitHub response"
            }));
        }
    };

    // Get user info from GitHub
    let user_response = match client
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {}", token_data.access_token))
        .header("User-Agent", "Spell-Platform")
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(e) => {
            log::error!("Failed to get user info: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to get user info from GitHub"
            }));
        }
    };

    let github_user: GitHubUser = match user_response.json().await {
        Ok(data) => data,
        Err(e) => {
            log::error!("Failed to parse user response: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to parse user data"
            }));
        }
    };

    // Upsert user in database
    let user: User = match sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (github_id, github_login, github_name, github_email, github_avatar_url, updated_at)
        VALUES ($1, $2, $3, $4, $5, NOW())
        ON CONFLICT (github_id) DO UPDATE SET
            github_login = EXCLUDED.github_login,
            github_name = EXCLUDED.github_name,
            github_email = EXCLUDED.github_email,
            github_avatar_url = EXCLUDED.github_avatar_url,
            updated_at = NOW()
        RETURNING *
        "#,
    )
    .bind(github_user.id)
    .bind(&github_user.login)
    .bind(&github_user.name)
    .bind(&github_user.email)
    .bind(&github_user.avatar_url)
    .fetch_one(&state.db)
    .await
    {
        Ok(user) => user,
        Err(e) => {
            log::error!("Failed to upsert user: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to save user"
            }));
        }
    };

    // Create session
    let session_token = generate_session_token();
    let expires_at = Utc::now() + Duration::days(30);

    match sqlx::query(
        r#"
        INSERT INTO sessions (user_id, token, expires_at)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(&user.id)
    .bind(&session_token)
    .bind(expires_at)
    .execute(&state.db)
    .await
    {
        Ok(_) => {}
        Err(e) => {
            log::error!("Failed to create session: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to create session"
            }));
        }
    };

    log::info!("User {} logged in successfully", user.github_login);

    HttpResponse::Ok().json(serde_json::json!({
        "status": "authenticated",
        "user": {
            "id": user.id,
            "github_login": user.github_login,
            "github_name": user.github_name,
            "github_avatar_url": user.github_avatar_url
        },
        "session_token": session_token,
        "expires_at": expires_at
    }))
}

fn generate_session_token() -> String {
    let mut rng = rand::thread_rng();
    let token: String = (0..64)
        .map(|_| {
            let idx = rng.gen_range(0..62);
            let chars = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
            chars[idx] as char
        })
        .collect();
    token
}
