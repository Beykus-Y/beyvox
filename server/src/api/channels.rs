use axum::{extract::{Path, State}, Json};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use uuid::Uuid;

use crate::{auth::AuthUser, error::{AppError, AppResult}, AppState};
use super::guilds::ensure_member;

#[derive(Serialize)]
pub struct ChannelDto {
    pub id: Uuid,
    pub guild_id: Uuid,
    pub name: String,
    pub r#type: String,
    pub position: i32,
    pub user_limit: Option<i32>,
}

#[derive(Deserialize)]
pub struct CreateChannelRequest {
    pub name: String,
    pub r#type: Option<String>,
    pub user_limit: Option<i32>,
}

pub async fn list_channels(
    State(state): State<AppState>,
    user: AuthUser,
    Path(guild_id): Path<Uuid>,
) -> AppResult<Json<Vec<ChannelDto>>> {
    ensure_member(&state, user.user_id, guild_id).await?;

    let rows = sqlx::query(
        "SELECT id, guild_id, name, type::text, position, user_limit
         FROM channels WHERE guild_id = $1 ORDER BY position",
    )
    .bind(guild_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(
        rows.iter()
            .map(|r| ChannelDto {
                id: r.get("id"),
                guild_id: r.get("guild_id"),
                name: r.get("name"),
                r#type: r.get("type"),
                position: r.get("position"),
                user_limit: r.get("user_limit"),
            })
            .collect(),
    ))
}

pub async fn create_channel(
    State(state): State<AppState>,
    user: AuthUser,
    Path(guild_id): Path<Uuid>,
    Json(body): Json<CreateChannelRequest>,
) -> AppResult<Json<ChannelDto>> {
    ensure_member(&state, user.user_id, guild_id).await?;
    // TODO: проверить право MANAGE_CHANNELS

    if body.name.is_empty() || body.name.len() > 100 {
        return Err(AppError::BadRequest("name must be 1-100 chars".into()));
    }

    let ch_type = body.r#type.as_deref().unwrap_or("text");
    if ch_type != "text" && ch_type != "voice" {
        return Err(AppError::BadRequest("type must be text or voice".into()));
    }

    let row = sqlx::query(
        "SELECT COALESCE(MAX(position), -1) + 1 as next_pos FROM channels WHERE guild_id = $1",
    )
    .bind(guild_id)
    .fetch_one(&state.db)
    .await?;
    let position: i32 = row.get("next_pos");

    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO channels (id, guild_id, name, type, position, user_limit)
         VALUES ($1, $2, $3, $4::channel_type, $5, $6)",
    )
    .bind(id)
    .bind(guild_id)
    .bind(&body.name)
    .bind(ch_type)
    .bind(position)
    .bind(body.user_limit)
    .execute(&state.db)
    .await?;

    Ok(Json(ChannelDto {
        id,
        guild_id,
        name: body.name,
        r#type: ch_type.into(),
        position,
        user_limit: body.user_limit,
    }))
}

pub async fn delete_channel(
    State(state): State<AppState>,
    user: AuthUser,
    Path((_guild_id, channel_id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<serde_json::Value>> {
    // TODO: проверить право MANAGE_CHANNELS
    let _ = user;
    sqlx::query("DELETE FROM channels WHERE id = $1")
        .bind(channel_id)
        .execute(&state.db)
        .await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

pub async fn get_voice_state(
    State(state): State<AppState>,
    user: AuthUser,
    Path((guild_id, channel_id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<Vec<serde_json::Value>>> {
    ensure_member(&state, user.user_id, guild_id).await?;

    let rows = sqlx::query(
        "SELECT user_id, is_muted, is_deafened FROM voice_states
         WHERE channel_id = $1",
    )
    .bind(channel_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(
        rows.iter()
            .map(|r| {
                serde_json::json!({
                    "user_id": r.get::<Uuid, _>("user_id"),
                    "is_muted": r.get::<bool, _>("is_muted"),
                    "is_deafened": r.get::<bool, _>("is_deafened"),
                })
            })
            .collect(),
    ))
}
