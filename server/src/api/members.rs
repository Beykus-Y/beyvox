use axum::{extract::{Path, State}, Json};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use uuid::Uuid;

use crate::{auth::AuthUser, error::{AppError, AppResult}, AppState};
use super::guilds::ensure_member;

#[derive(Serialize)]
pub struct MemberDto {
    pub user_id: Uuid,
    pub nickname: Option<String>,
    pub joined_at: chrono::DateTime<chrono::Utc>,
    pub is_muted: bool,
}

#[derive(Deserialize)]
pub struct MuteRequest {
    pub muted: bool,
}

pub async fn list_members(
    State(state): State<AppState>,
    user: AuthUser,
    Path(guild_id): Path<Uuid>,
) -> AppResult<Json<Vec<MemberDto>>> {
    ensure_member(&state, user.user_id, guild_id).await?;

    let rows = sqlx::query(
        "SELECT user_id, nickname, joined_at, is_muted FROM members
         WHERE guild_id = $1 AND is_banned = false ORDER BY joined_at",
    )
    .bind(guild_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(
        rows.iter()
            .map(|r| MemberDto {
                user_id: r.get("user_id"),
                nickname: r.get("nickname"),
                joined_at: r.get("joined_at"),
                is_muted: r.get("is_muted"),
            })
            .collect(),
    ))
}

pub async fn kick_member(
    State(state): State<AppState>,
    user: AuthUser,
    Path((guild_id, target_id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<serde_json::Value>> {
    ensure_member(&state, user.user_id, guild_id).await?;
    // TODO: проверить право MANAGE_MEMBERS

    if target_id == user.user_id {
        return Err(AppError::BadRequest("cannot kick yourself".into()));
    }

    sqlx::query("DELETE FROM members WHERE user_id = $1 AND guild_id = $2")
        .bind(target_id)
        .bind(guild_id)
        .execute(&state.db)
        .await?;

    Ok(Json(serde_json::json!({ "ok": true })))
}

pub async fn ban_member(
    State(state): State<AppState>,
    user: AuthUser,
    Path((guild_id, target_id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<serde_json::Value>> {
    ensure_member(&state, user.user_id, guild_id).await?;
    // TODO: проверить право BAN_MEMBERS

    if target_id == user.user_id {
        return Err(AppError::BadRequest("cannot ban yourself".into()));
    }

    sqlx::query(
        "UPDATE members SET is_banned = true WHERE user_id = $1 AND guild_id = $2",
    )
    .bind(target_id)
    .bind(guild_id)
    .execute(&state.db)
    .await?;

    Ok(Json(serde_json::json!({ "ok": true })))
}

pub async fn mute_member(
    State(state): State<AppState>,
    user: AuthUser,
    Path((guild_id, target_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<MuteRequest>,
) -> AppResult<Json<serde_json::Value>> {
    ensure_member(&state, user.user_id, guild_id).await?;
    // TODO: проверить право MUTE_MEMBERS

    sqlx::query(
        "UPDATE members SET is_muted = $1 WHERE user_id = $2 AND guild_id = $3",
    )
    .bind(body.muted)
    .bind(target_id)
    .bind(guild_id)
    .execute(&state.db)
    .await?;

    Ok(Json(serde_json::json!({ "ok": true })))
}
