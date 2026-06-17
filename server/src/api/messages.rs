use std::collections::HashMap;

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
use super::permissions::{ensure_channel_permission, SEND_MESSAGES};

#[derive(Serialize, Clone)]
pub struct ReactionSummary {
    pub emoji: String,
    pub count: i64,
    pub me: bool,
}

#[derive(Serialize, Clone)]
pub struct MessageDto {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub author_id: Uuid,
    pub author_username: String,
    pub content: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub edited_at: Option<chrono::DateTime<chrono::Utc>>,
    pub reply_to: Option<Uuid>,
    pub mention_user_ids: Vec<Uuid>,
    pub reactions: Vec<ReactionSummary>,
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
            "SELECT m.id, m.channel_id, m.author_id,
                    COALESCE(mem.username, '') as author_username,
                    m.content, m.created_at, m.edited_at, m.reply_to, m.mention_user_ids
             FROM messages m
             LEFT JOIN members mem ON mem.user_id = m.author_id AND mem.guild_id = $4
             WHERE m.channel_id = $1
               AND m.created_at < (SELECT created_at FROM messages WHERE id = $2)
             ORDER BY m.created_at DESC LIMIT $3",
        )
        .bind(channel_id)
        .bind(before)
        .bind(limit)
        .bind(guild_id)
        .fetch_all(&state.db)
        .await?
    } else {
        sqlx::query(
            "SELECT m.id, m.channel_id, m.author_id,
                    COALESCE(mem.username, '') as author_username,
                    m.content, m.created_at, m.edited_at, m.reply_to, m.mention_user_ids
             FROM messages m
             LEFT JOIN members mem ON mem.user_id = m.author_id AND mem.guild_id = $3
             WHERE m.channel_id = $1
             ORDER BY m.created_at DESC LIMIT $2",
        )
        .bind(channel_id)
        .bind(limit)
        .bind(guild_id)
        .fetch_all(&state.db)
        .await?
    };

    let mut msgs: Vec<MessageDto> = rows
        .iter()
        .map(|r| MessageDto {
            id: r.get("id"),
            channel_id: r.get("channel_id"),
            author_id: r.get("author_id"),
            author_username: r.get("author_username"),
            content: r.get("content"),
            created_at: r.get("created_at"),
            edited_at: r.get("edited_at"),
            reply_to: r.get("reply_to"),
            mention_user_ids: r.get::<Vec<Uuid>, _>("mention_user_ids"),
            reactions: vec![],
        })
        .collect();

    msgs.reverse();

    // Загружаем реакции для всех сообщений одним запросом
    if !msgs.is_empty() {
        let msg_ids: Vec<Uuid> = msgs.iter().map(|m| m.id).collect();

        let reaction_rows = sqlx::query(
            "SELECT message_id, emoji, COUNT(*) as cnt,
                    bool_or(user_id = $2) as me
             FROM message_reactions
             WHERE message_id = ANY($1)
             GROUP BY message_id, emoji
             ORDER BY emoji",
        )
        .bind(&msg_ids)
        .bind(user.user_id)
        .fetch_all(&state.db)
        .await?;

        // Группируем реакции по message_id
        let mut reactions_map: HashMap<Uuid, Vec<ReactionSummary>> = HashMap::new();
        for row in &reaction_rows {
            let mid: Uuid = row.get("message_id");
            reactions_map.entry(mid).or_default().push(ReactionSummary {
                emoji: row.get("emoji"),
                count: row.get("cnt"),
                me: row.get("me"),
            });
        }

        for msg in &mut msgs {
            if let Some(rxns) = reactions_map.remove(&msg.id) {
                msg.reactions = rxns;
            }
        }
    }

    Ok(Json(msgs))
}

pub async fn send_message(
    State(state): State<AppState>,
    user: AuthUser,
    Path((guild_id, channel_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<SendMessageRequest>,
) -> AppResult<Json<MessageDto>> {
    ensure_member(&state, user.user_id, guild_id).await?;
    ensure_channel_permission(&state, user.user_id, guild_id, channel_id, SEND_MESSAGES).await?;

    // Проверка таймаута
    let timeout_until: Option<chrono::DateTime<chrono::Utc>> = sqlx::query_scalar(
        "SELECT timeout_until FROM members WHERE user_id = $1 AND guild_id = $2",
    )
    .bind(user.user_id)
    .bind(guild_id)
    .fetch_optional(&state.db)
    .await?
    .flatten();

    if let Some(until) = timeout_until {
        if until > chrono::Utc::now() {
            return Err(AppError::Forbidden);
        }
    }

    if body.content.is_empty() || body.content.len() > 4000 {
        return Err(AppError::BadRequest("content must be 1-4000 chars".into()));
    }

    // Парсим @упоминания и резолвим в user_ids
    let mention_user_ids = resolve_mentions(&state, guild_id, &body.content).await;

    let id = Uuid::new_v4();
    let now = chrono::Utc::now();

    sqlx::query(
        "INSERT INTO messages (id, channel_id, author_id, content, reply_to, created_at, mention_user_ids)
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(id)
    .bind(channel_id)
    .bind(user.user_id)
    .bind(&body.content)
    .bind(body.reply_to)
    .bind(now)
    .bind(&mention_user_ids)
    .execute(&state.db)
    .await?;

    let msg = MessageDto {
        id,
        channel_id,
        author_id: user.user_id,
        author_username: user.username.clone(),
        content: body.content,
        created_at: now,
        edited_at: None,
        reply_to: body.reply_to,
        mention_user_ids: mention_user_ids.clone(),
        reactions: vec![],
    };

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
                mention_user_ids,
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

    let mention_user_ids = resolve_mentions(&state, guild_id, &body.content).await;

    let row = sqlx::query(
        "UPDATE messages SET content = $1, edited_at = NOW(), mention_user_ids = $4
         WHERE id = $2 AND author_id = $3
         RETURNING edited_at",
    )
    .bind(&body.content)
    .bind(message_id)
    .bind(user.user_id)
    .bind(&mention_user_ids)
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

pub async fn add_reaction(
    State(state): State<AppState>,
    user: AuthUser,
    Path((guild_id, channel_id, message_id, emoji)): Path<(Uuid, Uuid, Uuid, String)>,
) -> AppResult<Json<serde_json::Value>> {
    ensure_member(&state, user.user_id, guild_id).await?;

    // Не более 20 символов в эмодзи (безопасность)
    if emoji.chars().count() > 20 {
        return Err(AppError::BadRequest("emoji too long".into()));
    }

    sqlx::query(
        "INSERT INTO message_reactions (message_id, user_id, emoji) VALUES ($1, $2, $3)
         ON CONFLICT DO NOTHING",
    )
    .bind(message_id)
    .bind(user.user_id)
    .bind(&emoji)
    .execute(&state.db)
    .await?;

    broadcast_to_guild(
        &state,
        guild_id,
        ServerEvent::ReactionAdd {
            message_id,
            channel_id,
            guild_id,
            user_id: user.user_id,
            emoji,
        },
    )
    .await;

    Ok(Json(serde_json::json!({ "ok": true })))
}

pub async fn remove_reaction(
    State(state): State<AppState>,
    user: AuthUser,
    Path((guild_id, channel_id, message_id, emoji)): Path<(Uuid, Uuid, Uuid, String)>,
) -> AppResult<Json<serde_json::Value>> {
    ensure_member(&state, user.user_id, guild_id).await?;

    sqlx::query(
        "DELETE FROM message_reactions WHERE message_id = $1 AND user_id = $2 AND emoji = $3",
    )
    .bind(message_id)
    .bind(user.user_id)
    .bind(&emoji)
    .execute(&state.db)
    .await?;

    broadcast_to_guild(
        &state,
        guild_id,
        ServerEvent::ReactionRemove {
            message_id,
            channel_id,
            guild_id,
            user_id: user.user_id,
            emoji,
        },
    )
    .await;

    Ok(Json(serde_json::json!({ "ok": true })))
}

/// Парсит @username из текста и возвращает user_ids участников гильдии
async fn resolve_mentions(state: &AppState, guild_id: Uuid, content: &str) -> Vec<Uuid> {
    let names = parse_mention_names(content);
    if names.is_empty() {
        return vec![];
    }

    sqlx::query("SELECT user_id FROM members WHERE guild_id = $1 AND username = ANY($2)")
        .bind(guild_id)
        .bind(&names)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default()
        .iter()
        .map(|r| r.get("user_id"))
        .collect()
}

fn parse_mention_names(content: &str) -> Vec<String> {
    let mut names = std::collections::HashSet::new();
    let bytes = content.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'@' {
            i += 1;
            let start = i;
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1;
            }
            let len = i - start;
            if len > 0 && len <= 50 {
                names.insert(content[start..i].to_string());
            }
        } else {
            i += 1;
        }
    }
    names.into_iter().collect()
}
