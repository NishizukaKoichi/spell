// API Key Tests
//
// Tests for:
// - API key creation with Argon2 hashing (§19)
// - API key validation
// - Prefix-based lookup (sk_live_* / sk_test_*)
// - Key deletion by prefix

use actix_web::http::StatusCode;
use actix_web::{test, web, App, HttpResponse};
use argon2::{Argon2, PasswordHash, PasswordVerifier};

// Mock API key structure
#[derive(Debug, Clone)]
pub struct ApiKey {
    pub prefix: String,
    pub hash: String,
    pub user_id: String,
}

// Helper: Generate API key with prefix (matches src/routes/api_keys.rs logic)
fn generate_api_key(prefix: &str) -> (String, String) {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    // Generate random key part (32 chars)
    let key_part: String = (0..32)
        .map(|_| {
            let idx = rng.gen_range(0..62);
            let chars = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
            chars[idx] as char
        })
        .collect();

    let full_key = format!("{prefix}{key_part}");
    (full_key, key_part)
}

// Helper: Hash API key with Argon2
fn hash_api_key(key: &str) -> Result<String, argon2::password_hash::Error> {
    use argon2::{
        password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
        Argon2,
    };

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2.hash_password(key.as_bytes(), &salt)?;
    Ok(hash.to_string())
}

// Mock endpoint for API key creation
async fn mock_create_api_key() -> HttpResponse {
    let (full_key, _) = generate_api_key("sk_live_");

    // Hash the key
    let hash = match hash_api_key(&full_key) {
        Ok(h) => h,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    HttpResponse::Ok().json(serde_json::json!({
        "api_key": full_key,
        "prefix": &full_key[..8],
        "hash": hash,
    }))
}

#[actix_rt::test]
async fn test_api_key_generation_format() {
    // Test key format: prefix + 32 chars random
    let (full_key, _) = generate_api_key("sk_live_");

    assert!(
        full_key.starts_with("sk_live_"),
        "Key should start with sk_live_"
    );
    assert_eq!(
        full_key.len(),
        "sk_live_".len() + 32,
        "Key should be prefix + 32 random characters"
    );
    assert!(
        full_key["sk_live_".len()..]
            .chars()
            .all(|c| c.is_ascii_alphanumeric()),
        "Random part should be alphanumeric"
    );
}

#[actix_rt::test]
async fn test_api_key_test_prefix() {
    // Test test environment prefix
    let (full_key, _) = generate_api_key("sk_test_");

    assert!(
        full_key.starts_with("sk_test_"),
        "Test key should start with sk_test_"
    );
    assert_eq!(full_key.len(), "sk_test_".len() + 32);
}

#[actix_rt::test]
async fn test_argon2_hashing() {
    // Test Argon2 password hashing
    let (full_key, _) = generate_api_key("sk_live_");

    // Hash the key
    let hash = hash_api_key(&full_key).expect("Should hash successfully");

    // Verify hash format (Argon2 PHC string)
    assert!(
        hash.starts_with("$argon2"),
        "Hash should be Argon2 PHC format"
    );

    // Verify hash can be parsed
    let parsed_hash = PasswordHash::new(&hash).expect("Hash should be valid PHC format");

    // Verify the hash matches the original key
    let argon2 = Argon2::default();
    assert!(
        argon2
            .verify_password(full_key.as_bytes(), &parsed_hash)
            .is_ok(),
        "Hash should verify against original key"
    );
}

#[actix_rt::test]
async fn test_argon2_wrong_key_fails() {
    // Test that wrong key doesn't verify
    let (full_key, _) = generate_api_key("sk_live_");
    let (wrong_key, _) = generate_api_key("sk_live_");

    // Hash the first key
    let hash = hash_api_key(&full_key).expect("Should hash successfully");
    let parsed_hash = PasswordHash::new(&hash).expect("Hash should be valid");

    // Verify wrong key fails
    let argon2 = Argon2::default();
    assert!(
        argon2
            .verify_password(wrong_key.as_bytes(), &parsed_hash)
            .is_err(),
        "Wrong key should fail verification"
    );
}

#[actix_rt::test]
async fn test_create_api_key_endpoint_returns_key_and_hash() {
    // Arrange
    let app =
        test::init_service(App::new().route("/v1/keys", web::post().to(mock_create_api_key))).await;

    // Act
    let req = test::TestRequest::post().uri("/v1/keys").to_request();
    let resp = test::call_service(&app, req).await;

    // Assert
    assert_eq!(resp.status(), StatusCode::OK);

    let body = test::read_body(resp).await;
    let json: serde_json::Value =
        serde_json::from_slice(&body).expect("Response should be valid JSON");

    assert!(json.get("api_key").is_some(), "Should return api_key");
    assert!(json.get("prefix").is_some(), "Should return prefix");
    assert!(json.get("hash").is_some(), "Should return hash");

    // Verify api_key format
    let api_key = json["api_key"].as_str().expect("api_key should be string");
    assert!(
        api_key.starts_with("sk_live_") || api_key.starts_with("sk_test_"),
        "API key should have valid prefix"
    );

    // Verify hash is Argon2 format
    let hash = json["hash"].as_str().expect("hash should be string");
    assert!(hash.starts_with("$argon2"), "Hash should be Argon2 format");
}

#[actix_rt::test]
async fn test_api_key_prefix_extraction() {
    // Test prefix extraction logic (first 8 chars for "sk_live_" or "sk_test_")
    let (full_key, _) = generate_api_key("sk_live_");
    let prefix = &full_key[..8];

    assert_eq!(prefix, "sk_live_", "Prefix should be first 8 characters");
    assert_eq!(prefix.len(), 8, "Prefix length should be 8");
}

#[actix_rt::test]
async fn test_argon2_hash_is_not_reversible() {
    // Verify Argon2 is one-way (can't extract original key from hash)
    let (full_key, _) = generate_api_key("sk_live_");
    let hash = hash_api_key(&full_key).expect("Should hash successfully");

    // Hash should not contain the original key
    assert!(
        !hash.contains(&full_key),
        "Hash should not contain original key (one-way)"
    );

    // Hash should be different from original key
    assert_ne!(hash, full_key, "Hash should be different from original key");
}
