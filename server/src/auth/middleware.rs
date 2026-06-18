use std::sync::Arc;

use anyhow::{anyhow, Result};
use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::AppState;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub username: String,
    pub exp: i64,
}

#[derive(Clone, Debug)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub username: String,
}

pub struct JwksCache {
    pub decoding_key: Option<DecodingKey>,
}

impl JwksCache {
    pub fn new() -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self { decoding_key: None }))
    }
}

pub async fn fetch_jwks(central_url: &str) -> Result<DecodingKey> {
    let url = format!("{central_url}/.well-known/jwks.json");
    let resp: serde_json::Value = reqwest::get(&url).await?.json().await?;

    let key = resp["keys"]
        .as_array()
        .and_then(|keys| keys.first())
        .ok_or_else(|| anyhow!("empty JWKS"))?;

    let n = key["n"].as_str().ok_or_else(|| anyhow!("missing n"))?;
    let e = key["e"].as_str().ok_or_else(|| anyhow!("missing e"))?;

    Ok(DecodingKey::from_rsa_components(n, e)?)
}

#[axum::async_trait]
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let app_state = state.clone();

        let token = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or_else(|| {
                (StatusCode::UNAUTHORIZED, "missing token").into_response()
            })?;

        let decoding_key = {
            let cache = app_state.jwks_cache.read().await;
            cache.decoding_key.clone()
        };

        let key = if let Some(k) = decoding_key {
            k
        } else {
            let key = fetch_jwks(&app_state.config.central_url)
                .await
                .map_err(|e| {
                    tracing::error!("failed to fetch JWKS: {e}");
                    (StatusCode::SERVICE_UNAVAILABLE, "cannot verify token").into_response()
                })?;
            let mut cache = app_state.jwks_cache.write().await;
            cache.decoding_key = Some(key.clone());
            key
        };

        let mut validation = Validation::new(Algorithm::RS256);
        validation.validate_exp = true;

        let claims = decode::<Claims>(token, &key, &validation)
            .map_err(|_| (StatusCode::UNAUTHORIZED, "invalid token").into_response())?
            .claims;

        let user_id = claims
            .sub
            .parse::<Uuid>()
            .map_err(|_| (StatusCode::UNAUTHORIZED, "invalid sub").into_response())?;

        // Проверяем глобальный бан пользователя
        let is_banned: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM banned_users WHERE user_id = $1)")
            .bind(user_id)
            .fetch_one(&app_state.db)
            .await
            .map_err(|e| {
                tracing::error!("failed to check global ban: {e}");
                (StatusCode::INTERNAL_SERVER_ERROR, "database error").into_response()
            })?;

        if is_banned {
            return Err((StatusCode::FORBIDDEN, "user is globally banned").into_response());
        }

        Ok(AuthUser { user_id, username: claims.username })
    }
}

