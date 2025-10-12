use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::Rng;

pub fn generate_api_key(prefix: &str) -> Result<(String, String), anyhow::Error> {
    // Generate secure random bytes
    let mut rng = rand::thread_rng();
    let raw: [u8; 32] = rng.gen();
    let token = URL_SAFE_NO_PAD.encode(raw);

    // Combine prefix and token
    let api_key = format!("{prefix}{token}");

    // Hash the API key
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(api_key.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("Failed to hash API key: {e}"))?
        .to_string();

    Ok((api_key, hash))
}

pub fn verify_api_key(provided: &str, hash: &str) -> Result<bool, anyhow::Error> {
    let parsed_hash =
        PasswordHash::new(hash).map_err(|e| anyhow::anyhow!("Failed to parse hash: {e}"))?;
    let argon2 = Argon2::default();

    match argon2.verify_password(provided.as_bytes(), &parsed_hash) {
        Ok(()) => Ok(true),
        Err(_) => Ok(false),
    }
}

pub fn extract_prefix(api_key: &str) -> Option<String> {
    // Extract prefix (e.g., "sk_live_")
    if api_key.starts_with("sk_") {
        let parts: Vec<&str> = api_key.splitn(3, '_').collect();
        if parts.len() >= 2 {
            return Some(format!("{}_{}_", parts[0], parts[1]));
        }
    }
    None
}
