use axum::{extract::State, Json};
use base64::Engine;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{error::{AppError, AppResult}, AppState};

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub login: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user_id: Uuid,
    pub username: String,
}

pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterRequest>,
) -> AppResult<Json<AuthResponse>> {
    if body.username.len() < 3 || body.username.len() > 32 {
        return Err(AppError::BadRequest("username must be 3-32 chars".into()));
    }
    if body.password.len() < 8 {
        return Err(AppError::BadRequest("password must be at least 8 chars".into()));
    }

    let password_hash = hash_password(&body.password)?;

    let row = sqlx::query(
        "INSERT INTO users (username, email, password_hash) VALUES ($1, $2, $3) RETURNING id, username",
    )
    .bind(&body.username)
    .bind(body.email.to_lowercase())
    .bind(password_hash)
    .fetch_one(&state.db)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(ref db_err) if db_err.constraint() == Some("users_username_key") => {
            AppError::Conflict("username already taken".into())
        }
        sqlx::Error::Database(ref db_err) if db_err.constraint() == Some("users_email_key") => {
            AppError::Conflict("email already registered".into())
        }
        other => AppError::Sqlx(other),
    })?;

    use sqlx::Row;
    let user_id: Uuid = row.try_get("id")?;
    let username: String = row.try_get("username")?;

    let (access_token, refresh_token) = issue_tokens(&state, user_id, &username).await?;

    Ok(Json(AuthResponse { access_token, refresh_token, user_id, username }))
}

pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> AppResult<Json<AuthResponse>> {
    let row = sqlx::query(
        "SELECT id, username, password_hash FROM users WHERE username = $1 OR email = $1",
    )
    .bind(body.login.to_lowercase())
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::Unauthorized)?;

    use sqlx::Row;
    let user_id: Uuid = row.try_get("id")?;
    let username: String = row.try_get("username")?;
    let password_hash: String = row.try_get("password_hash")?;

    if !verify_password(&body.password, &password_hash)? {
        return Err(AppError::Unauthorized);
    }

    let (access_token, refresh_token) = issue_tokens(&state, user_id, &username).await?;

    Ok(Json(AuthResponse { access_token, refresh_token, user_id, username }))
}

pub async fn refresh(
    State(state): State<AppState>,
    Json(body): Json<RefreshRequest>,
) -> AppResult<Json<AuthResponse>> {
    let token_hash = hash_token(&body.refresh_token);

    let row = sqlx::query(
        "SELECT rt.id, rt.user_id, rt.expires_at, u.username
         FROM refresh_tokens rt
         JOIN users u ON u.id = rt.user_id
         WHERE rt.token_hash = $1",
    )
    .bind(&token_hash)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::Unauthorized)?;

    use sqlx::Row;
    let token_id: Uuid = row.try_get("id")?;
    let user_id: Uuid = row.try_get("user_id")?;
    let expires_at: chrono::DateTime<Utc> = row.try_get("expires_at")?;
    let username: String = row.try_get("username")?;

    if expires_at < Utc::now() {
        return Err(AppError::Unauthorized);
    }

    sqlx::query("DELETE FROM refresh_tokens WHERE id = $1")
        .bind(token_id)
        .execute(&state.db)
        .await?;

    let (access_token, refresh_token) = issue_tokens(&state, user_id, &username).await?;

    Ok(Json(AuthResponse { access_token, refresh_token, user_id, username }))
}

pub async fn jwks(State(state): State<AppState>) -> String {
    state.jwt.jwks_json.clone()
}

async fn issue_tokens(
    state: &AppState,
    user_id: Uuid,
    username: &str,
) -> AppResult<(String, String)> {
    let access_token = state
        .jwt
        .sign(user_id, username, state.config.access_token_ttl_secs)
        .map_err(anyhow::Error::from)?;

    let refresh_token_raw = generate_token();
    let token_hash = hash_token(&refresh_token_raw);
    let expires_at = Utc::now() + chrono::Duration::seconds(state.config.refresh_token_ttl_secs);

    sqlx::query(
        "INSERT INTO refresh_tokens (user_id, token_hash, expires_at) VALUES ($1, $2, $3)",
    )
    .bind(user_id)
    .bind(token_hash)
    .bind(expires_at)
    .execute(&state.db)
    .await?;

    Ok((access_token, refresh_token_raw))
}

fn hash_password(password: &str) -> AppResult<String> {
    use argon2::{password_hash::{rand_core::OsRng, PasswordHasher, SaltString}, Argon2};
    let salt = SaltString::generate(&mut OsRng);
    Ok(Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("hash error: {e}"))?
        .to_string())
}

fn verify_password(password: &str, hash: &str) -> AppResult<bool> {
    use argon2::{password_hash::{PasswordHash, PasswordVerifier}, Argon2};
    let parsed = PasswordHash::new(hash).map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(Argon2::default().verify_password(password.as_bytes(), &parsed).is_ok())
}

fn generate_token() -> String {
    use rand::Rng;
    let bytes: [u8; 32] = rand::thread_rng().r#gen();
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

fn hash_token(token: &str) -> String {
    use sha2::{Digest, Sha256};
    hex::encode(Sha256::digest(token.as_bytes()))
}
