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
    pub email_verified: bool,
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
    let email_verified = !state.config.email_verification_required;

    let row = sqlx::query(
        "INSERT INTO users (username, email, password_hash, email_verified) VALUES ($1, $2, $3, $4) RETURNING id, username",
    )
    .bind(&body.username)
    .bind(body.email.to_lowercase())
    .bind(password_hash)
    .bind(email_verified)
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

    if state.config.email_verification_required {
        let token = generate_token();
        let expires_at = Utc::now() + chrono::Duration::hours(24);
        sqlx::query(
            "INSERT INTO email_verification_tokens (token, user_id, expires_at) VALUES ($1, $2, $3)",
        )
        .bind(&token)
        .bind(user_id)
        .bind(expires_at)
        .execute(&state.db)
        .await?;

        let smtp_host = state.config.smtp_host.clone();
        let smtp_port = state.config.smtp_port;
        let smtp_username = state.config.smtp_username.clone();
        let smtp_password = state.config.smtp_password.clone();
        let smtp_from = state.config.smtp_from.clone();
        let public_url = state.config.central_public_url.clone();
        let email_addr = body.email.to_lowercase();
        let username_str = username.clone();

        tokio::spawn(async move {
            use lettre::{
                transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport,
            };

            let link = format!("{public_url}/auth/verify?token={token}");
            let html_body = format!(
                r#"<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <style>
    body {{
      font-family: 'Helvetica Neue', Helvetica, Arial, sans-serif;
      background-color: #0f1013;
      color: #dbdee1;
      margin: 0;
      padding: 0;
      -webkit-text-size-adjust: none;
      -ms-text-size-adjust: none;
    }}
    .container {{
      max-width: 550px;
      margin: 40px auto;
      background-color: #15161a;
      border: 1px solid #2a2c33;
      border-radius: 12px;
      overflow: hidden;
      box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
    }}
    .header {{
      background: linear-gradient(135deg, #5b7cf6, #7c6cf7);
      padding: 30px;
      text-align: center;
    }}
    .header h1 {{
      color: #ffffff;
      font-size: 28px;
      font-weight: 800;
      margin: 0;
      letter-spacing: -0.5px;
    }}
    .content {{
      padding: 40px 30px;
      line-height: 1.6;
    }}
    .greeting {{
      font-size: 20px;
      font-weight: 700;
      color: #ffffff;
      margin-bottom: 15px;
    }}
    .text {{
      font-size: 15px;
      color: #b5bac1;
      margin-bottom: 30px;
    }}
    .button-wrapper {{
      text-align: center;
      margin-bottom: 30px;
    }}
    .btn {{
      display: inline-block;
      background-color: #5b7cf6;
      color: #ffffff !important;
      text-decoration: none;
      padding: 14px 28px;
      font-weight: 600;
      font-size: 15px;
      border-radius: 8px;
    }}
    .footer {{
      padding: 20px 30px;
      background-color: #0f1013;
      border-top: 1px solid #2a2c33;
      text-align: center;
      font-size: 12px;
      color: #949ba4;
    }}
    .link-fallback {{
      font-size: 12px;
      color: #72767d;
      word-break: break-all;
      margin-top: 25px;
    }}
    .link-fallback a {{
      color: #5b7cf6;
      text-decoration: none;
    }}
  </style>
</head>
<body>
  <div class="container">
    <div class="header">
      <h1>BeyVox</h1>
    </div>
    <div class="content">
      <div class="greeting">Привет, {}!</div>
      <div class="text">
        Спасибо за регистрацию в голосовом мессенджере BeyVox. Чтобы подтвердить свой адрес электронной почты и получить полный доступ к системе, нажмите кнопку ниже:
      </div>
      <div class="button-wrapper">
        <a href="{}" class="btn" style="color: #ffffff;">Подтвердить почту</a>
      </div>
      <div class="link-fallback">
        Если кнопка не работает, скопируйте и вставьте эту ссылку в браузер:<br>
        <a href="{}">{}</a>
      </div>
    </div>
    <div class="footer">
      Это автоматическое письмо, пожалуйста, не отвечайте на него.
    </div>
  </div>
</body>
</html>"#,
                username_str, link, link, link
            );

            let email = Message::builder()
                .from(smtp_from.parse().unwrap())
                .to(email_addr.parse().unwrap())
                .subject("Подтверждение почты BeyVox")
                .header(lettre::message::header::ContentType::parse("text/html; charset=utf-8").unwrap())
                .body(html_body);

            match email {
                Ok(msg) => {
                    let creds = Credentials::new(smtp_username, smtp_password);
                    let transport = SmtpTransport::relay(&smtp_host);
                    match transport {
                        Ok(t_builder) => {
                            let t = t_builder.port(smtp_port).credentials(creds).build();
                            if let Err(e) = t.send(&msg) {
                                tracing::error!("Failed to send verification email: {:?}", e);
                            } else {
                                tracing::info!(
                                    "Verification email sent successfully to {}",
                                    email_addr
                                );
                            }
                        }
                        Err(e) => {
                            tracing::error!("Failed to build SMTP relay: {:?}", e);
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to construct verification email: {:?}", e);
                }
            }
        });
    }

    let (access_token, refresh_token) = issue_tokens(&state, user_id, &username).await?;

    Ok(Json(AuthResponse {
        access_token,
        refresh_token,
        user_id,
        username,
        email_verified,
    }))
}

pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> AppResult<Json<AuthResponse>> {
    let row = sqlx::query(
        "SELECT id, username, password_hash, email_verified FROM users WHERE username = $1 OR email = $1",
    )
    .bind(body.login.to_lowercase())
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::Unauthorized)?;

    use sqlx::Row;
    let user_id: Uuid = row.try_get("id")?;
    let username: String = row.try_get("username")?;
    let password_hash: String = row.try_get("password_hash")?;
    let email_verified: bool = row.try_get("email_verified")?;

    if !verify_password(&body.password, &password_hash)? {
        return Err(AppError::Unauthorized);
    }

    let (access_token, refresh_token) = issue_tokens(&state, user_id, &username).await?;

    Ok(Json(AuthResponse {
        access_token,
        refresh_token,
        user_id,
        username,
        email_verified,
    }))
}

pub async fn refresh(
    State(state): State<AppState>,
    Json(body): Json<RefreshRequest>,
) -> AppResult<Json<AuthResponse>> {
    let token_hash = hash_token(&body.refresh_token);

    let row = sqlx::query(
        "SELECT rt.id, rt.user_id, rt.expires_at, u.username, u.email_verified
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
    let email_verified: bool = row.try_get("email_verified")?;

    if expires_at < Utc::now() {
        return Err(AppError::Unauthorized);
    }

    sqlx::query("DELETE FROM refresh_tokens WHERE id = $1")
        .bind(token_id)
        .execute(&state.db)
        .await?;

    let (access_token, refresh_token) = issue_tokens(&state, user_id, &username).await?;

    Ok(Json(AuthResponse {
        access_token,
        refresh_token,
        user_id,
        username,
        email_verified,
    }))
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

#[derive(Deserialize)]
pub struct VerifyQuery {
    pub token: String,
}

pub async fn verify_email(
    State(state): State<AppState>,
    axum::extract::Query(q): axum::extract::Query<VerifyQuery>,
) -> AppResult<axum::response::Html<String>> {
    use sqlx::Row;

    let row = sqlx::query(
        "SELECT user_id, expires_at FROM email_verification_tokens WHERE token = $1"
    )
    .bind(&q.token)
    .fetch_optional(&state.db)
    .await?;

    let (user_id, expires_at) = match row {
        Some(r) => {
            let uid: Uuid = r.try_get("user_id")?;
            let exp: chrono::DateTime<chrono::Utc> = r.try_get("expires_at")?;
            (uid, exp)
        }
        None => {
            return Ok(axum::response::Html(
                "<html><body style='font-family:sans-serif; text-align:center; padding-top:50px; background:#1e1f22; color:#dbdee1;'>\
                 <h2 style='color:#f23f43;'>Ошибка верификации</h2>\
                 <p>Токен верификации не найден или неверен.</p>\
                 </body></html>".to_string()
            ));
        }
    };

    if expires_at < Utc::now() {
        sqlx::query("DELETE FROM email_verification_tokens WHERE token = $1")
            .bind(&q.token)
            .execute(&state.db)
            .await?;

        return Ok(axum::response::Html(
            "<html><body style='font-family:sans-serif; text-align:center; padding-top:50px; background:#1e1f22; color:#dbdee1;'>\
             <h2 style='color:#f23f43;'>Ссылка истекла</h2>\
             <p>Срок действия ссылки подтверждения почты истек.</p>\
             </body></html>".to_string()
        ));
    }

    sqlx::query("UPDATE users SET email_verified = true WHERE id = $1")
        .bind(user_id)
        .execute(&state.db)
        .await?;

    sqlx::query("DELETE FROM email_verification_tokens WHERE token = $1")
        .bind(&q.token)
        .execute(&state.db)
        .await?;

    Ok(axum::response::Html(
        "<html><body style='font-family:sans-serif; text-align:center; padding-top:50px; background:#1e1f22; color:#dbdee1;'>\
         <h2 style='color:#23a55a;'>Почта успешно подтверждена!</h2>\
         <p>Спасибо! Ваш адрес электронной почты подтвержден. Теперь вы можете вернуться в приложение BeyVox.</p>\
         </body></html>".to_string()
    ))
}

pub async fn status(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> AppResult<Json<serde_json::Value>> {
    let auth_header = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(AppError::Unauthorized)?;

    let claims = state.jwt.verify(auth_header).map_err(|_| AppError::Unauthorized)?;
    let user_id = claims.sub.parse::<Uuid>().map_err(|_| AppError::Unauthorized)?;

    let row = sqlx::query("SELECT email_verified FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::Unauthorized)?;

    use sqlx::Row;
    let email_verified: bool = row.try_get("email_verified")?;

    Ok(Json(serde_json::json!({ "email_verified": email_verified })))
}
