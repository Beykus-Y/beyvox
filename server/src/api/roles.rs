use axum::{extract::{Path, State}, Json};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use uuid::Uuid;

use crate::{auth::AuthUser, error::{AppError, AppResult}, AppState};
use crate::ws::{handler::broadcast_to_guild, types::{ServerEvent, WsRole}};
use super::guilds::ensure_member;
use super::members::fetch_ws_member;
use super::permissions::{ensure_permission, MANAGE_ROLES};

#[derive(Serialize, Clone)]
pub struct RoleDto {
    pub id: Uuid,
    pub guild_id: Uuid,
    pub name: String,
    pub color: Option<String>,
    pub permissions: i64,
    pub position: i32,
}

#[derive(Deserialize)]
pub struct CreateRoleRequest {
    pub name: String,
    pub color: Option<String>,
    pub permissions: Option<i64>,
}

#[derive(Deserialize)]
pub struct UpdateRoleRequest {
    pub name: Option<String>,
    pub color: Option<String>,
    pub permissions: Option<i64>,
}

fn role_to_ws(r: &RoleDto) -> WsRole {
    WsRole {
        id: r.id,
        guild_id: r.guild_id,
        name: r.name.clone(),
        color: r.color.clone(),
        permissions: r.permissions,
        position: r.position,
    }
}

pub async fn list_roles(
    State(state): State<AppState>,
    user: AuthUser,
    Path(guild_id): Path<Uuid>,
) -> AppResult<Json<Vec<RoleDto>>> {
    ensure_member(&state, user.user_id, guild_id).await?;

    let rows = sqlx::query(
        "SELECT id, guild_id, name, color, permissions, position FROM roles
         WHERE guild_id = $1 ORDER BY position",
    )
    .bind(guild_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(
        rows.iter()
            .map(|r| RoleDto {
                id: r.get("id"),
                guild_id: r.get("guild_id"),
                name: r.get("name"),
                color: r.get("color"),
                permissions: r.get("permissions"),
                position: r.get("position"),
            })
            .collect(),
    ))
}

pub async fn create_role(
    State(state): State<AppState>,
    user: AuthUser,
    Path(guild_id): Path<Uuid>,
    Json(body): Json<CreateRoleRequest>,
) -> AppResult<Json<RoleDto>> {
    ensure_member(&state, user.user_id, guild_id).await?;
    ensure_permission(&state, user.user_id, guild_id, MANAGE_ROLES).await?;

    if body.name.is_empty() || body.name.len() > 100 {
        return Err(AppError::BadRequest("name must be 1-100 chars".into()));
    }
    if body.name == "@everyone" {
        return Err(AppError::BadRequest("cannot create role named @everyone".into()));
    }

    let position: i32 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(position), 0) + 1 FROM roles WHERE guild_id = $1",
    )
    .bind(guild_id)
    .fetch_one(&state.db)
    .await?;

    let id = Uuid::new_v4();
    let permissions = body.permissions.unwrap_or(0);

    sqlx::query(
        "INSERT INTO roles (id, guild_id, name, color, permissions, position)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(id)
    .bind(guild_id)
    .bind(&body.name)
    .bind(&body.color)
    .bind(permissions)
    .bind(position)
    .execute(&state.db)
    .await?;

    let dto = RoleDto { id, guild_id, name: body.name, color: body.color, permissions, position };
    broadcast_to_guild(&state, guild_id, ServerEvent::RoleCreate {
        guild_id,
        role: role_to_ws(&dto),
    })
    .await;

    Ok(Json(dto))
}

pub async fn update_role(
    State(state): State<AppState>,
    user: AuthUser,
    Path((guild_id, role_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<UpdateRoleRequest>,
) -> AppResult<Json<RoleDto>> {
    ensure_member(&state, user.user_id, guild_id).await?;
    ensure_permission(&state, user.user_id, guild_id, MANAGE_ROLES).await?;

    let existing = sqlx::query(
        "SELECT name, color, permissions, position FROM roles WHERE id = $1 AND guild_id = $2",
    )
    .bind(role_id)
    .bind(guild_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    let current_name: String = existing.get("name");
    if current_name == "@everyone" && body.name.as_deref().map(|n| n != "@everyone").unwrap_or(false) {
        return Err(AppError::BadRequest("cannot rename @everyone".into()));
    }

    let name = body.name.unwrap_or(current_name);
    let color: Option<String> = body.color.or_else(|| existing.get("color"));
    let permissions = body.permissions.unwrap_or_else(|| existing.get("permissions"));
    let position: i32 = existing.get("position");

    sqlx::query(
        "UPDATE roles SET name = $1, color = $2, permissions = $3 WHERE id = $4",
    )
    .bind(&name)
    .bind(&color)
    .bind(permissions)
    .bind(role_id)
    .execute(&state.db)
    .await?;

    let dto = RoleDto { id: role_id, guild_id, name, color, permissions, position };
    broadcast_to_guild(&state, guild_id, ServerEvent::RoleUpdate {
        guild_id,
        role: role_to_ws(&dto),
    })
    .await;

    Ok(Json(dto))
}

pub async fn delete_role(
    State(state): State<AppState>,
    user: AuthUser,
    Path((guild_id, role_id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<serde_json::Value>> {
    ensure_member(&state, user.user_id, guild_id).await?;
    ensure_permission(&state, user.user_id, guild_id, MANAGE_ROLES).await?;

    let name: Option<String> = sqlx::query_scalar(
        "SELECT name FROM roles WHERE id = $1 AND guild_id = $2",
    )
    .bind(role_id)
    .bind(guild_id)
    .fetch_optional(&state.db)
    .await?;

    match name.as_deref() {
        None => return Err(AppError::NotFound),
        Some("@everyone") => return Err(AppError::BadRequest("cannot delete @everyone".into())),
        _ => {}
    }

    sqlx::query("DELETE FROM roles WHERE id = $1")
        .bind(role_id)
        .execute(&state.db)
        .await?;

    broadcast_to_guild(&state, guild_id, ServerEvent::RoleDelete { guild_id, role_id }).await;

    Ok(Json(serde_json::json!({ "ok": true })))
}

pub async fn assign_role(
    State(state): State<AppState>,
    user: AuthUser,
    Path((guild_id, target_id, role_id)): Path<(Uuid, Uuid, Uuid)>,
) -> AppResult<Json<serde_json::Value>> {
    ensure_member(&state, user.user_id, guild_id).await?;
    ensure_permission(&state, user.user_id, guild_id, MANAGE_ROLES).await?;

    ensure_member(&state, target_id, guild_id).await?;

    let role_exists: Option<Uuid> = sqlx::query_scalar(
        "SELECT id FROM roles WHERE id = $1 AND guild_id = $2",
    )
    .bind(role_id)
    .bind(guild_id)
    .fetch_optional(&state.db)
    .await?;

    if role_exists.is_none() {
        return Err(AppError::NotFound);
    }

    sqlx::query(
        "INSERT INTO member_roles (user_id, guild_id, role_id) VALUES ($1, $2, $3)
         ON CONFLICT DO NOTHING",
    )
    .bind(target_id)
    .bind(guild_id)
    .bind(role_id)
    .execute(&state.db)
    .await?;

    if let Ok(member) = fetch_ws_member(&state, target_id, guild_id).await {
        broadcast_to_guild(&state, guild_id, ServerEvent::MemberUpdate { guild_id, member }).await;
    }

    Ok(Json(serde_json::json!({ "ok": true })))
}

pub async fn remove_role(
    State(state): State<AppState>,
    user: AuthUser,
    Path((guild_id, target_id, role_id)): Path<(Uuid, Uuid, Uuid)>,
) -> AppResult<Json<serde_json::Value>> {
    ensure_member(&state, user.user_id, guild_id).await?;
    ensure_permission(&state, user.user_id, guild_id, MANAGE_ROLES).await?;

    sqlx::query(
        "DELETE FROM member_roles WHERE user_id = $1 AND guild_id = $2 AND role_id = $3",
    )
    .bind(target_id)
    .bind(guild_id)
    .bind(role_id)
    .execute(&state.db)
    .await?;

    if let Ok(member) = fetch_ws_member(&state, target_id, guild_id).await {
        broadcast_to_guild(&state, guild_id, ServerEvent::MemberUpdate { guild_id, member }).await;
    }

    Ok(Json(serde_json::json!({ "ok": true })))
}
