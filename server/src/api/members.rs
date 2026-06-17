use axum::{extract::{Path, State}, Json};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use uuid::Uuid;

use crate::{auth::AuthUser, error::{AppError, AppResult}, AppState};
use crate::ws::{handler::broadcast_to_guild, types::{ServerEvent, WsMember}};
use super::guilds::ensure_member;
use super::permissions::{ensure_permission, BAN_MEMBERS, MANAGE_MEMBERS, MUTE_MEMBERS};

#[derive(Serialize)]
pub struct MemberDto {
    pub user_id: Uuid,
    pub username: String,
    pub nickname: Option<String>,
    pub joined_at: chrono::DateTime<chrono::Utc>,
    pub is_muted: bool,
    pub timeout_until: Option<chrono::DateTime<chrono::Utc>>,
    pub role_ids: Vec<Uuid>,
}

#[derive(Deserialize)]
pub struct MuteRequest {
    pub muted: bool,
}

#[derive(Deserialize)]
pub struct TimeoutRequest {
    /// 0 = снять таймаут
    pub seconds: i64,
}

pub async fn list_members(
    State(state): State<AppState>,
    user: AuthUser,
    Path(guild_id): Path<Uuid>,
) -> AppResult<Json<Vec<MemberDto>>> {
    ensure_member(&state, user.user_id, guild_id).await?;

    let rows = sqlx::query(
        "SELECT user_id, username, nickname, joined_at, is_muted, timeout_until FROM members
         WHERE guild_id = $1 AND is_banned = false ORDER BY joined_at",
    )
    .bind(guild_id)
    .fetch_all(&state.db)
    .await?;

    let mut members = Vec::with_capacity(rows.len());
    for row in &rows {
        let uid: Uuid = row.get("user_id");
        let role_ids = fetch_role_ids(&state, uid, guild_id).await?;
        members.push(MemberDto {
            user_id: uid,
            username: row.get("username"),
            nickname: row.get("nickname"),
            joined_at: row.get("joined_at"),
            is_muted: row.get("is_muted"),
            timeout_until: row.get("timeout_until"),
            role_ids,
        });
    }

    Ok(Json(members))
}

pub async fn get_me(
    State(state): State<AppState>,
    user: AuthUser,
    Path(guild_id): Path<Uuid>,
) -> AppResult<Json<MemberDto>> {
    ensure_member(&state, user.user_id, guild_id).await?;

    let row = sqlx::query(
        "SELECT user_id, username, nickname, joined_at, is_muted, timeout_until FROM members
         WHERE user_id = $1 AND guild_id = $2",
    )
    .bind(user.user_id)
    .bind(guild_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    let role_ids = fetch_role_ids(&state, user.user_id, guild_id).await?;

    Ok(Json(MemberDto {
        user_id: row.get("user_id"),
        username: row.get("username"),
        nickname: row.get("nickname"),
        joined_at: row.get("joined_at"),
        is_muted: row.get("is_muted"),
        timeout_until: row.get("timeout_until"),
        role_ids,
    }))
}

pub async fn kick_member(
    State(state): State<AppState>,
    user: AuthUser,
    Path((guild_id, target_id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<serde_json::Value>> {
    ensure_member(&state, user.user_id, guild_id).await?;
    ensure_permission(&state, user.user_id, guild_id, MANAGE_MEMBERS).await?;

    if target_id == user.user_id {
        return Err(AppError::BadRequest("cannot kick yourself".into()));
    }

    sqlx::query("DELETE FROM members WHERE user_id = $1 AND guild_id = $2")
        .bind(target_id)
        .bind(guild_id)
        .execute(&state.db)
        .await?;

    broadcast_to_guild(
        &state,
        guild_id,
        ServerEvent::MemberRemove { guild_id, user_id: target_id },
    )
    .await;

    Ok(Json(serde_json::json!({ "ok": true })))
}

pub async fn ban_member(
    State(state): State<AppState>,
    user: AuthUser,
    Path((guild_id, target_id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<serde_json::Value>> {
    ensure_member(&state, user.user_id, guild_id).await?;
    ensure_permission(&state, user.user_id, guild_id, BAN_MEMBERS).await?;

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

    broadcast_to_guild(
        &state,
        guild_id,
        ServerEvent::MemberRemove { guild_id, user_id: target_id },
    )
    .await;

    Ok(Json(serde_json::json!({ "ok": true })))
}

pub async fn mute_member(
    State(state): State<AppState>,
    user: AuthUser,
    Path((guild_id, target_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<MuteRequest>,
) -> AppResult<Json<serde_json::Value>> {
    ensure_member(&state, user.user_id, guild_id).await?;
    ensure_permission(&state, user.user_id, guild_id, MUTE_MEMBERS).await?;

    sqlx::query(
        "UPDATE members SET is_muted = $1 WHERE user_id = $2 AND guild_id = $3",
    )
    .bind(body.muted)
    .bind(target_id)
    .bind(guild_id)
    .execute(&state.db)
    .await?;

    if let Ok(member) = fetch_ws_member(&state, target_id, guild_id).await {
        broadcast_to_guild(&state, guild_id, ServerEvent::MemberUpdate { guild_id, member }).await;
    }

    Ok(Json(serde_json::json!({ "ok": true })))
}

pub async fn timeout_member(
    State(state): State<AppState>,
    user: AuthUser,
    Path((guild_id, target_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<TimeoutRequest>,
) -> AppResult<Json<serde_json::Value>> {
    ensure_member(&state, user.user_id, guild_id).await?;
    ensure_permission(&state, user.user_id, guild_id, MUTE_MEMBERS).await?;

    if target_id == user.user_id {
        return Err(AppError::BadRequest("cannot timeout yourself".into()));
    }

    let timeout_until = if body.seconds > 0 {
        Some(chrono::Utc::now() + chrono::Duration::seconds(body.seconds))
    } else {
        None
    };

    sqlx::query(
        "UPDATE members SET timeout_until = $1 WHERE user_id = $2 AND guild_id = $3",
    )
    .bind(timeout_until)
    .bind(target_id)
    .bind(guild_id)
    .execute(&state.db)
    .await?;

    if let Ok(member) = fetch_ws_member(&state, target_id, guild_id).await {
        broadcast_to_guild(&state, guild_id, ServerEvent::MemberUpdate { guild_id, member }).await;
    }

    Ok(Json(serde_json::json!({ "ok": true })))
}

pub async fn fetch_ws_member(state: &AppState, user_id: Uuid, guild_id: Uuid) -> AppResult<WsMember> {
    let row = sqlx::query(
        "SELECT username, nickname, is_muted, timeout_until FROM members
         WHERE user_id = $1 AND guild_id = $2",
    )
    .bind(user_id)
    .bind(guild_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    let role_ids = fetch_role_ids(state, user_id, guild_id).await?;

    Ok(WsMember {
        user_id,
        username: row.get("username"),
        nickname: row.get("nickname"),
        is_muted: row.get("is_muted"),
        timeout_until: row.get("timeout_until"),
        role_ids,
    })
}

async fn fetch_role_ids(state: &AppState, user_id: Uuid, guild_id: Uuid) -> AppResult<Vec<Uuid>> {
    let rows = sqlx::query(
        "SELECT role_id FROM member_roles WHERE user_id = $1 AND guild_id = $2",
    )
    .bind(user_id)
    .bind(guild_id)
    .fetch_all(&state.db)
    .await?;

    Ok(rows.iter().map(|r| r.get("role_id")).collect())
}
