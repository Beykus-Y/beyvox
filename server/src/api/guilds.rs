use axum::{extract::{Path, State}, Json};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use uuid::Uuid;

use crate::{auth::AuthUser, error::{AppError, AppResult}, AppState};
use super::permissions::{ensure_permission, MANAGE_MEMBERS};

#[derive(Serialize)]
pub struct GuildDto {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub icon_url: Option<String>,
    pub owner_id: Uuid,
    pub member_count: i64,
    pub is_default: bool,
    pub is_public: bool,
}

#[derive(Deserialize)]
pub struct CreateGuildRequest {
    pub name: String,
    pub description: Option<String>,
    pub is_public: Option<bool>,
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
    fetch_guild_dto(&state, guild_id).await.map(Json)
}

async fn fetch_guild_dto(state: &AppState, guild_id: Uuid) -> AppResult<GuildDto> {
    let row = sqlx::query(
        "SELECT g.id, g.name, g.description, g.icon_url, g.owner_id, g.is_default, g.is_public,
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

    Ok(GuildDto {
        id: row.get("id"),
        name: row.get("name"),
        description: row.get("description"),
        icon_url: row.get("icon_url"),
        owner_id: row.get("owner_id"),
        member_count: row.get("member_count"),
        is_default: row.get("is_default"),
        is_public: row.get("is_public"),
    })
}

pub async fn create_guild(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<CreateGuildRequest>,
) -> AppResult<Json<GuildDto>> {
    if !state.config.owner_username.is_empty() && user.username != state.config.owner_username {
        return Err(AppError::Forbidden);
    }

    if body.name.is_empty() || body.name.len() > 100 {
        return Err(AppError::BadRequest("name must be 1-100 chars".into()));
    }

    let guild_id = Uuid::new_v4();
    let is_public = body.is_public.unwrap_or(true);

    // Первая гильдия на сервере становится дефолтной
    let existing: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM guilds")
        .fetch_one(&state.db)
        .await?;
    let is_default = existing == 0;

    sqlx::query(
        "INSERT INTO guilds (id, name, description, owner_id, is_default, is_public)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(guild_id)
    .bind(&body.name)
    .bind(&body.description)
    .bind(user.user_id)
    .bind(is_default)
    .bind(is_public)
    .execute(&state.db)
    .await?;

    let everyone_perms: i64 = 16 | 32 | 64;
    sqlx::query(
        "INSERT INTO roles (guild_id, name, position, permissions) VALUES ($1, '@everyone', 0, $2)",
    )
    .bind(guild_id)
    .bind(everyone_perms)
    .execute(&state.db)
    .await?;

    sqlx::query("INSERT INTO members (user_id, guild_id, username) VALUES ($1, $2, $3)")
        .bind(user.user_id)
        .bind(guild_id)
        .bind(&user.username)
        .execute(&state.db)
        .await?;

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
        is_default,
        is_public,
    }))
}

pub async fn delete_guild(
    State(state): State<AppState>,
    user: AuthUser,
    Path(guild_id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let row = sqlx::query(
        "SELECT owner_id, is_default FROM guilds WHERE id = $1",
    )
    .bind(guild_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    let owner_id: Uuid = row.get("owner_id");
    let is_default: bool = row.get("is_default");

    if owner_id != user.user_id {
        return Err(AppError::Forbidden);
    }
    if is_default {
        return Err(AppError::BadRequest(
            "cannot delete default guild — set another guild as default first".into(),
        ));
    }

    sqlx::query("DELETE FROM guilds WHERE id = $1")
        .bind(guild_id)
        .execute(&state.db)
        .await?;

    Ok(Json(serde_json::json!({ "ok": true })))
}

pub async fn set_default_guild(
    State(state): State<AppState>,
    user: AuthUser,
    Path(guild_id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let row = sqlx::query("SELECT owner_id, is_public FROM guilds WHERE id = $1")
        .bind(guild_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;

    let owner_id: Uuid = row.get("owner_id");
    if owner_id != user.user_id {
        return Err(AppError::Forbidden);
    }

    // Атомарно: снять is_default со всех, поставить на одну
    sqlx::query("UPDATE guilds SET is_default = (id = $1)")
        .bind(guild_id)
        .execute(&state.db)
        .await?;

    Ok(Json(serde_json::json!({ "ok": true })))
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

    let existing = sqlx::query(
        "SELECT is_banned FROM members WHERE user_id = $1 AND guild_id = $2",
    )
    .bind(user.user_id)
    .bind(guild_id)
    .fetch_optional(&state.db)
    .await?;

    if let Some(row) = existing {
        if row.get::<bool, _>("is_banned") {
            return Err(AppError::Forbidden);
        }
        return fetch_guild_dto(&state, guild_id).await.map(Json);
    }

    sqlx::query(
        "INSERT INTO members (user_id, guild_id, username) VALUES ($1, $2, $3)",
    )
    .bind(user.user_id)
    .bind(guild_id)
    .bind(&user.username)
    .execute(&state.db)
    .await?;

    sqlx::query("UPDATE invites SET uses = uses + 1 WHERE code = $1")
        .bind(&code)
        .execute(&state.db)
        .await?;

    // Auto-join в дефолтную гильдию если пользователь вступает в другую
    let default_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT id FROM guilds WHERE is_default = true LIMIT 1",
    )
    .fetch_optional(&state.db)
    .await?;

    if let Some(default_id) = default_id {
        if default_id != guild_id {
            let already = sqlx::query(
                "SELECT 1 FROM members WHERE user_id = $1 AND guild_id = $2",
            )
            .bind(user.user_id)
            .bind(default_id)
            .fetch_optional(&state.db)
            .await?;

            if already.is_none() {
                let _ = sqlx::query(
                    "INSERT INTO members (user_id, guild_id, username) VALUES ($1, $2, $3)
                     ON CONFLICT DO NOTHING",
                )
                .bind(user.user_id)
                .bind(default_id)
                .bind(&user.username)
                .execute(&state.db)
                .await;
            }
        }
    }

    fetch_guild_dto(&state, guild_id).await.map(Json)
}

pub async fn list_invites(
    State(state): State<AppState>,
    user: AuthUser,
    Path(guild_id): Path<Uuid>,
) -> AppResult<Json<Vec<InviteDto>>> {
    ensure_member(&state, user.user_id, guild_id).await?;
    ensure_permission(&state, user.user_id, guild_id, MANAGE_MEMBERS).await?;

    let rows = sqlx::query(
        "SELECT code, guild_id, uses, max_uses, expires_at FROM invites
         WHERE guild_id = $1 ORDER BY created_at DESC",
    )
    .bind(guild_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(
        rows.iter()
            .map(|r| InviteDto {
                code: r.get("code"),
                guild_id: r.get("guild_id"),
                uses: r.get("uses"),
                max_uses: r.get("max_uses"),
                expires_at: r.get("expires_at"),
            })
            .collect(),
    ))
}

pub async fn delete_invite(
    State(state): State<AppState>,
    user: AuthUser,
    Path((guild_id, code)): Path<(Uuid, String)>,
) -> AppResult<Json<serde_json::Value>> {
    ensure_member(&state, user.user_id, guild_id).await?;
    ensure_permission(&state, user.user_id, guild_id, MANAGE_MEMBERS).await?;

    let deleted = sqlx::query(
        "DELETE FROM invites WHERE code = $1 AND guild_id = $2",
    )
    .bind(&code)
    .bind(guild_id)
    .execute(&state.db)
    .await?;

    if deleted.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(Json(serde_json::json!({ "ok": true })))
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
        None => {
            // Если пользователя нет в гильдии, проверяем, является ли он владельцем сервера
            if !state.config.owner_username.is_empty() {
                let is_server_owner: bool = sqlx::query_scalar(
                    "SELECT EXISTS(SELECT 1 FROM members WHERE user_id = $1 AND username = $2)"
                )
                .bind(user_id)
                .bind(&state.config.owner_username)
                .fetch_one(&state.db)
                .await
                .unwrap_or(false);

                if is_server_owner {
                    return Ok(());
                }
            }
            Err(AppError::Forbidden)
        }
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
