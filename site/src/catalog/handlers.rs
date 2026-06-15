use axum::{extract::{Path, State}, Json};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use uuid::Uuid;

use crate::{error::{AppError, AppResult}, AppState};

#[derive(Serialize)]
pub struct ServerEntry {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub icon_url: Option<String>,
    pub address: String,
    pub tags: Vec<String>,
    pub online_count: i32,
    pub last_ping: Option<chrono::DateTime<Utc>>,
}

#[derive(Deserialize)]
pub struct RegisterServerRequest {
    pub name: String,
    pub description: Option<String>,
    pub address: String,
    pub tags: Option<Vec<String>>,
    pub is_public: Option<bool>,
}

#[derive(Deserialize)]
pub struct PingRequest {
    pub online_count: i32,
}

pub async fn list_servers(
    State(state): State<AppState>,
) -> AppResult<Json<Vec<ServerEntry>>> {
    let rows = sqlx::query(
        "SELECT id, name, description, icon_url, address, tags, online_count, last_ping
         FROM servers
         WHERE is_public = true AND last_ping > NOW() - INTERVAL '5 minutes'
         ORDER BY online_count DESC",
    )
    .fetch_all(&state.db)
    .await?;

    let servers = rows
        .iter()
        .map(|r| ServerEntry {
            id: r.get("id"),
            name: r.get("name"),
            description: r.get("description"),
            icon_url: r.get("icon_url"),
            address: r.get("address"),
            tags: r.get("tags"),
            online_count: r.get("online_count"),
            last_ping: r.get("last_ping"),
        })
        .collect();

    Ok(Json(servers))
}

pub async fn register_server(
    State(state): State<AppState>,
    // TODO: заменить на реальный user_id из JWT middleware
    Json(body): Json<RegisterServerRequest>,
) -> AppResult<Json<ServerEntry>> {
    if body.name.is_empty() || body.name.len() > 100 {
        return Err(AppError::BadRequest("name must be 1-100 chars".into()));
    }

    let owner_id = Uuid::nil(); // placeholder до auth middleware
    let tags = body.tags.unwrap_or_default();
    let is_public = body.is_public.unwrap_or(true);

    let row = sqlx::query(
        "INSERT INTO servers (name, description, address, tags, is_public, registered_by)
         VALUES ($1, $2, $3, $4, $5, $6)
         RETURNING id, name, description, icon_url, address, tags, online_count, last_ping",
    )
    .bind(&body.name)
    .bind(&body.description)
    .bind(&body.address)
    .bind(&tags)
    .bind(is_public)
    .bind(owner_id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(ServerEntry {
        id: row.get("id"),
        name: row.get("name"),
        description: row.get("description"),
        icon_url: row.get("icon_url"),
        address: row.get("address"),
        tags: row.get("tags"),
        online_count: row.get("online_count"),
        last_ping: row.get("last_ping"),
    }))
}

pub async fn ping_server(
    State(state): State<AppState>,
    Path(server_id): Path<Uuid>,
    Json(body): Json<PingRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let result = sqlx::query(
        "UPDATE servers SET last_ping = NOW(), online_count = $1 WHERE id = $2",
    )
    .bind(body.online_count)
    .bind(server_id)
    .execute(&state.db)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(Json(serde_json::json!({ "ok": true })))
}
