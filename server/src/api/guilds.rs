use axum::{extract::{Path, State}, Json};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use uuid::Uuid;

use crate::{auth::AuthUser, error::{AppError, AppResult}, AppState};

#[derive(Serialize)]
pub struct GuildDto {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub icon_url: Option<String>,
    pub owner_id: Uuid,
    pub member_count: i64,
}

#[derive(Deserialize)]
pub struct CreateGuildRequest {
    pub name: String,
    pub description: Option<String>,
    pub owner_token: Option<String>,
}

#[derive(Serialize)]
pub struct InviteDto {
    pub code: String,
    pub guild_id: Uuid,
    pub uses: i32,
    pub max_uses: Option<i32>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Deserialize)]
pub struct CreateInviteRequest {
    pub max_uses: Option<i32>,
    pub expires_hours: Option<i64>,
}

pub async fn get_guild(
    State(state): State<AppState>,
    user: AuthUser,
    Path(guild_id): Path<Uuid>,
) -> AppResult<Json<GuildDto>> {
    ensure_member(&state, user.user_id, guild_id).await?;

    let row = sqlx::query(
        "SELECT g.id, g.name, g.description, g.icon_url, g.owner_id,
                COUNT(m.user_id) as member_count
         FROM guilds g
         LEFT JOIN members m ON m.guild_id = g.id AND m.is_banned = false
         WHERE g.id = $1
         GROUP BY g.id",
    )
    .bind(guild_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(Json(GuildDto {
        id: row.get("id"),
        name: row.get("name"),
        description: row.get("description"),
        icon_url: row.get("icon_url"),
        owner_id: row.get("owner_id"),
        member_count: row.get("member_count"),
    }))
}

pub async fn create_guild(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<CreateGuildRequest>,
) -> AppResult<Json<GuildDto>> {
    // Проверяем owner token
    let provided = body.owner_token.as_deref().unwrap_or("").trim().to_string();
    if provided != state.config.owner_token {
        return Err(AppError::Forbidden);
    }

    if body.name.is_empty() || body.name.len() > 100 {
        return Err(AppError::BadRequest("name must be 1-100 chars".into()));
    }

    let guild_id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO guilds (id, name, description, owner_id) VALUES ($1, $2, $3, $4)",
    )
    .bind(guild_id)
    .bind(&body.name)
    .bind(&body.description)
    .bind(user.user_id)
    .execute(&state.db)
    .await?;

    // Создаём роль @everyone с базовыми правами
    let everyone_perms: i64 = 16 | 32 | 64; // SEND_MESSAGES | ATTACH_FILES | CONNECT_VOICE
    sqlx::query(
        "INSERT INTO roles (guild_id, name, position, permissions) VALUES ($1, '@everyone', 0, $2)",
    )
    .bind(guild_id)
    .bind(everyone_perms)
    .execute(&state.db)
    .await?;

    // Добавляем создателя как участника
    sqlx::query("INSERT INTO members (user_id, guild_id) VALUES ($1, $2)")
        .bind(user.user_id)
        .bind(guild_id)
        .execute(&state.db)
        .await?;

    // Создаём канал #general по умолчанию
    sqlx::query(
        "INSERT INTO channels (guild_id, name, type, position) VALUES ($1, 'general', 'text', 0)",
    )
    .bind(guild_id)
    .execute(&state.db)
    .await?;

    Ok(Json(GuildDto {
        id: guild_id,
        name: body.name,
        description: body.description,
        icon_url: None,
        owner_id: user.user_id,
        member_count: 1,
    }))
}

pub async fn create_invite(
    State(state): State<AppState>,
    user: AuthUser,
    Path(guild_id): Path<Uuid>,
    Json(body): Json<CreateInviteRequest>,
) -> AppResult<Json<InviteDto>> {
    ensure_member(&state, user.user_id, guild_id).await?;

    let code = generate_code();
    let expires_at = body.expires_hours.map(|h| {
        chrono::Utc::now() + chrono::Duration::hours(h)
    });

    sqlx::query(
        "INSERT INTO invites (code, guild_id, created_by, expires_at, max_uses)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(&code)
    .bind(guild_id)
    .bind(user.user_id)
    .bind(expires_at)
    .bind(body.max_uses)
    .execute(&state.db)
    .await?;

    Ok(Json(InviteDto { code, guild_id, uses: 0, max_uses: body.max_uses, expires_at }))
}

pub async fn join_by_invite(
    State(state): State<AppState>,
    user: AuthUser,
    Path(code): Path<String>,
) -> AppResult<Json<GuildDto>> {
    let row = sqlx::query(
        "SELECT guild_id, expires_at, max_uses, uses FROM invites WHERE code = $1",
    )
    .bind(&code)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    let guild_id: Uuid = row.get("guild_id");
    let expires_at: Option<chrono::DateTime<chrono::Utc>> = row.get("expires_at");
    let max_uses: Option<i32> = row.get("max_uses");
    let uses: i32 = row.get("uses");

    if let Some(exp) = expires_at {
        if exp < chrono::Utc::now() {
            return Err(AppError::BadRequest("invite expired".into()));
        }
    }
    if let Some(max) = max_uses {
        if uses >= max {
            return Err(AppError::BadRequest("invite max uses reached".into()));
        }
    }

    // Добавляем участника
    sqlx::query(
        "INSERT INTO members (user_id, guild_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
    )
    .bind(user.user_id)
    .bind(guild_id)
    .execute(&state.db)
    .await?;

    // Инкрементируем счётчик
    sqlx::query("UPDATE invites SET uses = uses + 1 WHERE code = $1")
        .bind(&code)
        .execute(&state.db)
        .await?;

    get_guild(State(state), user, Path(guild_id)).await
}

pub async fn ensure_member(state: &AppState, user_id: Uuid, guild_id: Uuid) -> AppResult<()> {
    let row = sqlx::query(
        "SELECT is_banned FROM members WHERE user_id = $1 AND guild_id = $2",
    )
    .bind(user_id)
    .bind(guild_id)
    .fetch_optional(&state.db)
    .await?;

    match row {
        None => Err(AppError::Forbidden),
        Some(r) if r.get::<bool, _>("is_banned") => Err(AppError::Forbidden),
        _ => Ok(()),
    }
}

fn generate_code() -> String {
    use rand::Rng;
    const CHARS: &[u8] = b"abcdefghijkmnpqrstuvwxyz23456789";
    let mut rng = rand::thread_rng();
    (0..8).map(|_| CHARS[rng.gen_range(0..CHARS.len())] as char).collect()
}
