use sqlx::Row;
use uuid::Uuid;

use crate::{error::{AppError, AppResult}, AppState};

pub const ADMINISTRATOR: i64   = 1 << 0;
pub const MANAGE_CHANNELS: i64 = 1 << 1;
pub const MANAGE_ROLES: i64    = 1 << 2;
pub const MANAGE_MEMBERS: i64  = 1 << 3;
pub const SEND_MESSAGES: i64   = 1 << 4;
pub const ATTACH_FILES: i64    = 1 << 5;
pub const CONNECT_VOICE: i64   = 1 << 6;
pub const STREAM_SCREEN: i64   = 1 << 7;
pub const MUTE_MEMBERS: i64    = 1 << 8;
pub const BAN_MEMBERS: i64     = 1 << 9;

/// OR всех ролей участника (@everyone + назначенные). ADMINISTRATOR → i64::MAX.
pub async fn get_member_permissions(state: &AppState, user_id: Uuid, guild_id: Uuid) -> AppResult<i64> {
    // 1. Проверяем, является ли пользователь владельцем сервера (Server Owner)
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
            return Ok(i64::MAX);
        }
    }

    // 2. Проверяем, является ли пользователь создателем гильдии (Guild Owner)
    let owner_id: Option<Uuid> = sqlx::query_scalar("SELECT owner_id FROM guilds WHERE id = $1")
        .bind(guild_id)
        .fetch_optional(&state.db)
        .await?;

    if owner_id == Some(user_id) {
        return Ok(i64::MAX);
    }

    let everyone: i64 = sqlx::query_scalar(
        "SELECT COALESCE(permissions, 0) FROM roles WHERE guild_id = $1 AND name = '@everyone'",
    )
    .bind(guild_id)
    .fetch_optional(&state.db)
    .await?
    .unwrap_or(0);

    let member: i64 = sqlx::query_scalar(
        "SELECT COALESCE(BIT_OR(r.permissions), 0) FROM roles r
         JOIN member_roles mr ON mr.role_id = r.id
         WHERE mr.user_id = $1 AND mr.guild_id = $2",
    )
    .bind(user_id)
    .bind(guild_id)
    .fetch_one(&state.db)
    .await?;

    let combined = everyone | member;
    if combined & ADMINISTRATOR != 0 {
        return Ok(i64::MAX);
    }
    Ok(combined)
}


/// Базовые права + применение channel_permission_overrides.
pub async fn get_effective_permissions(
    state: &AppState,
    user_id: Uuid,
    guild_id: Uuid,
    channel_id: Uuid,
) -> AppResult<i64> {
    let base = get_member_permissions(state, user_id, guild_id).await?;
    if base == i64::MAX {
        return Ok(i64::MAX);
    }

    let rows = sqlx::query(
        "SELECT cpo.allow, cpo.deny FROM channel_permission_overrides cpo
         WHERE cpo.channel_id = $1 AND cpo.role_id IN (
             SELECT mr.role_id FROM member_roles mr WHERE mr.user_id = $2 AND mr.guild_id = $3
             UNION ALL
             SELECT r.id FROM roles r WHERE r.guild_id = $3 AND r.name = '@everyone'
         )",
    )
    .bind(channel_id)
    .bind(user_id)
    .bind(guild_id)
    .fetch_all(&state.db)
    .await?;

    let mut deny_mask: i64 = 0;
    let mut allow_mask: i64 = 0;
    for row in &rows {
        deny_mask |= row.get::<i64, _>("deny");
        allow_mask |= row.get::<i64, _>("allow");
    }

    Ok((base & !deny_mask) | allow_mask)
}

/// Owner гильдии всегда проходит. ADMINISTRATOR тоже.
pub async fn ensure_permission(
    state: &AppState,
    user_id: Uuid,
    guild_id: Uuid,
    perm: i64,
) -> AppResult<()> {
    let owner_id: Option<Uuid> = sqlx::query_scalar("SELECT owner_id FROM guilds WHERE id = $1")
        .bind(guild_id)
        .fetch_optional(&state.db)
        .await?;

    if owner_id == Some(user_id) {
        return Ok(());
    }

    let perms = get_member_permissions(state, user_id, guild_id).await?;
    if perms == i64::MAX || perms & perm != 0 {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}

/// Аналог ensure_permission, но учитывает channel overrides.
pub async fn ensure_channel_permission(
    state: &AppState,
    user_id: Uuid,
    guild_id: Uuid,
    channel_id: Uuid,
    perm: i64,
) -> AppResult<()> {
    let owner_id: Option<Uuid> = sqlx::query_scalar("SELECT owner_id FROM guilds WHERE id = $1")
        .bind(guild_id)
        .fetch_optional(&state.db)
        .await?;

    if owner_id == Some(user_id) {
        return Ok(());
    }

    let perms = get_effective_permissions(state, user_id, guild_id, channel_id).await?;
    if perms == i64::MAX || perms & perm != 0 {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}
