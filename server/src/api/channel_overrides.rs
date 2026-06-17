use axum::{extract::{Path, State}, Json};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use uuid::Uuid;

use crate::{auth::AuthUser, error::AppResult, AppState};
use super::guilds::ensure_member;
use super::permissions::{ensure_permission, MANAGE_CHANNELS};

#[derive(Serialize)]
pub struct OverrideDto {
    pub channel_id: Uuid,
    pub role_id: Uuid,
    pub allow: i64,
    pub deny: i64,
}

#[derive(Deserialize)]
pub struct SetOverrideRequest {
    pub allow: i64,
    pub deny: i64,
}

pub async fn get_channel_permissions(
    State(state): State<AppState>,
    user: AuthUser,
    Path((guild_id, channel_id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<Vec<OverrideDto>>> {
    ensure_member(&state, user.user_id, guild_id).await?;

    let rows = sqlx::query(
        "SELECT channel_id, role_id, allow, deny FROM channel_permission_overrides
         WHERE channel_id = $1",
    )
    .bind(channel_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(
        rows.iter()
            .map(|r| OverrideDto {
                channel_id: r.get("channel_id"),
                role_id: r.get("role_id"),
                allow: r.get("allow"),
                deny: r.get("deny"),
            })
            .collect(),
    ))
}

pub async fn set_channel_permission(
    State(state): State<AppState>,
    user: AuthUser,
    Path((guild_id, channel_id, role_id)): Path<(Uuid, Uuid, Uuid)>,
    Json(body): Json<SetOverrideRequest>,
) -> AppResult<Json<OverrideDto>> {
    ensure_member(&state, user.user_id, guild_id).await?;
    ensure_permission(&state, user.user_id, guild_id, MANAGE_CHANNELS).await?;

    sqlx::query(
        "INSERT INTO channel_permission_overrides (channel_id, role_id, allow, deny)
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (channel_id, role_id) DO UPDATE SET allow = $3, deny = $4",
    )
    .bind(channel_id)
    .bind(role_id)
    .bind(body.allow)
    .bind(body.deny)
    .execute(&state.db)
    .await?;

    Ok(Json(OverrideDto { channel_id, role_id, allow: body.allow, deny: body.deny }))
}

pub async fn delete_channel_permission(
    State(state): State<AppState>,
    user: AuthUser,
    Path((guild_id, channel_id, role_id)): Path<(Uuid, Uuid, Uuid)>,
) -> AppResult<Json<serde_json::Value>> {
    ensure_member(&state, user.user_id, guild_id).await?;
    ensure_permission(&state, user.user_id, guild_id, MANAGE_CHANNELS).await?;

    sqlx::query(
        "DELETE FROM channel_permission_overrides WHERE channel_id = $1 AND role_id = $2",
    )
    .bind(channel_id)
    .bind(role_id)
    .execute(&state.db)
    .await?;

    Ok(Json(serde_json::json!({ "ok": true })))
}
