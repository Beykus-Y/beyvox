use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    error::{AppError, AppResult},
    ws::{handler::broadcast_to_guild, types::ServerEvent},
    AppState,
};
use super::guilds::ensure_member;

#[derive(Serialize, Clone)]
pub struct MessageDto {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub author_id: Uuid,
    pub content: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub edited_at: Option<chrono::DateTime<chrono::Utc>>,
    pub reply_to: Option<Uuid>,
}

#[derive(Deserialize)]
pub struct SendMessageRequest {
    pub content: String,
    pub reply_to: Option<Uuid>,
}

#[derive(Deserialize)]
pub struct EditMessageRequest {
    pub content: String,
}

#[derive(Deserialize)]
pub struct MessagesQuery {
    pub before: Option<Uuid>,
    pub limit: Option<i64>,
}

pub async fn get_messages(
    State(state): State<AppState>,
    user: AuthUser,
    Path((guild_id, channel_id)): Path<(Uuid, Uuid)>,
    Query(q): Query<MessagesQuery>,
) -> AppResult<Json<Vec<MessageDto>>> {
    ensure_member(&state, user.user_id, guild_id).await?;

    let limit = q.limit.unwrap_or(50).min(100);

    let rows = if let Some(before) = q.before {
        sqlx::query(
            "SELECT id, channel_id, author_id, content, created_at, edited_at, reply_to
             FROM messages
             WHERE channel_id = $1 AND created_at < (SELECT created_at FROM messages WHERE id = $2)
             ORDER BY created_at DESC LIMIT $3",
        )
        .bind(channel_id)
        .bind(before)
        .bind(limit)
        .fetch_all(&state.db)
        .await?
    } else {
        sqlx::query(
            "SELECT id, channel_id, author_id, content, created_at, edited_at, reply_to
             FROM messages WHERE channel_id = $1
             ORDER BY created_at DESC LIMIT $2",
        )
        .bind(channel_id)
        .bind(limit)
        .fetch_all(&state.db)
        .await?
    };

    let mut msgs: Vec<MessageDto> = rows
        .iter()
        .map(|r| MessageDto {
            id: r.get("id"),
            channel_id: r.get("channel_id"),
            author_id: r.get("author_id"),
            content: r.get("content"),
            created_at: r.get("created_at"),
            edited_at: r.get("edited_at"),
            reply_to: r.get("reply_to"),
        })
        .collect();

    msgs.reverse(); // хронологический порядок
    Ok(Json(msgs))
}

pub async fn send_message(
    State(state): State<AppState>,
    user: AuthUser,
    Path((guild_id, channel_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<SendMessageRequest>,
) -> AppResult<Json<MessageDto>> {
    ensure_member(&state, user.user_id, guild_id).await?;

    if body.content.is_empty() || body.content.len() > 4000 {
        return Err(AppError::BadRequest("content must be 1-4000 chars".into()));
    }

    let id = Uuid::new_v4();
    let now = chrono::Utc::now();

    sqlx::query(
        "INSERT INTO messages (id, channel_id, author_id, content, reply_to, created_at)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(id)
    .bind(channel_id)
    .bind(user.user_id)
    .bind(&body.content)
    .bind(body.reply_to)
    .bind(now)
    .execute(&state.db)
    .await?;

    let msg = MessageDto {
        id,
        channel_id,
        author_id: user.user_id,
        content: body.content,
        created_at: now,
        edited_at: None,
        reply_to: body.reply_to,
    };

    // Рассылаем всем участникам гильдии через WebSocket
    broadcast_to_guild(
        &state,
        guild_id,
        ServerEvent::MessageCreate {
            message: crate::ws::types::MessageDto {
                id: msg.id,
                channel_id: msg.channel_id,
                author_id: msg.author_id,
                author_username: user.username,
                content: msg.content.clone(),
                created_at: msg.created_at,
                edited_at: None,
                reply_to: msg.reply_to,
            },
        },
    )
    .await;

    Ok(Json(msg))
}

pub async fn edit_message(
    State(state): State<AppState>,
    user: AuthUser,
    Path((guild_id, channel_id, message_id)): Path<(Uuid, Uuid, Uuid)>,
    Json(body): Json<EditMessageRequest>,
) -> AppResult<Json<serde_json::Value>> {
    if body.content.is_empty() || body.content.len() > 4000 {
        return Err(AppError::BadRequest("content must be 1-4000 chars".into()));
    }

    let row = sqlx::query(
        "UPDATE messages SET content = $1, edited_at = NOW()
         WHERE id = $2 AND author_id = $3
         RETURNING edited_at",
    )
    .bind(&body.content)
    .bind(message_id)
    .bind(user.user_id)
    .fetch_optional(&state.db)
    .await?;

    let row = row.ok_or(AppError::Forbidden)?;
    let edited_at: chrono::DateTime<chrono::Utc> = row.get("edited_at");

    broadcast_to_guild(
        &state,
        guild_id,
        ServerEvent::MessageUpdate {
            message_id,
            channel_id,
            content: body.content,
            edited_at,
        },
    )
    .await;

    Ok(Json(serde_json::json!({ "ok": true })))
}

pub async fn delete_message(
    State(state): State<AppState>,
    user: AuthUser,
    Path((guild_id, channel_id, message_id)): Path<(Uuid, Uuid, Uuid)>,
) -> AppResult<Json<serde_json::Value>> {
    let result = sqlx::query(
        "DELETE FROM messages WHERE id = $1 AND author_id = $2",
    )
    .bind(message_id)
    .bind(user.user_id)
    .execute(&state.db)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::Forbidden);
    }

    broadcast_to_guild(
        &state,
        guild_id,
        ServerEvent::MessageDelete { message_id, channel_id },
    )
    .await;

    Ok(Json(serde_json::json!({ "ok": true })))
}
