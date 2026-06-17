use std::sync::Arc;

use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::Response,
};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{
    auth::middleware::{fetch_jwks, Claims},
    api::permissions::{ensure_permission, CONNECT_VOICE},
    ws::types::{ClientEvent, GuildSummary, ServerEvent},
    AppState,
};

/// Канал отправки событий конкретному клиенту
pub type ClientSender = mpsc::UnboundedSender<ServerEvent>;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    tracing::info!("WS connection incoming");
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sink, mut stream) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<ServerEvent>();

    // Шлём HELLO
    let hello = serde_json::to_string(&ServerEvent::Hello {
        heartbeat_interval: 30_000,
    })
    .unwrap();
    if sink.send(Message::Text(hello.into())).await.is_err() {
        return;
    }

    // Задача: пересылает события из канала в WebSocket
    let mut send_task = tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            let text = match serde_json::to_string(&event) {
                Ok(t) => t,
                Err(_) => continue,
            };
            if sink.send(Message::Text(text.into())).await.is_err() {
                break;
            }
        }
    });

    let shared_uid = Arc::new(std::sync::Mutex::new(None::<Uuid>));

    // Основной цикл чтения сообщений от клиента
    let mut recv_task = tokio::spawn({
        let state = state.clone();
        let tx = tx.clone();
        let shared_uid = shared_uid.clone();
        async move {
            let mut user_id: Option<Uuid> = None;
            while let Some(Ok(msg)) = stream.next().await {
                let text = match msg {
                    Message::Text(t) => t,
                    Message::Close(_) => break,
                    _ => continue,
                };

                let event: ClientEvent = match serde_json::from_str(&text) {
                    Ok(e) => e,
                    Err(_) => {
                        let _ = tx.send(ServerEvent::Error {
                            message: "invalid message format".into(),
                        });
                        continue;
                    }
                };

                match event {
                    ClientEvent::Identify { token } => {
                        match identify(&state, &token).await {
                            Ok((uid, username, guilds)) => {
                                user_id = Some(uid);
                                *shared_uid.lock().unwrap() = Some(uid);
                                state.connections.insert(uid, tx.clone());
                                let _ = tx.send(ServerEvent::Ready {
                                    user_id: uid,
                                    username,
                                    guilds: guilds.clone(),
                                });
                                // Сразу шлём актуальные voice states — клиент не знает кто уже в голосе
                                send_existing_voice_states(&state, &tx, &guilds).await;
                            }
                            Err(e) => {
                                tracing::error!("identify failed: {e:#}");
                                let _ = tx.send(ServerEvent::Error {
                                    message: "authentication failed".into(),
                                });
                            }
                        }
                    }

                    ClientEvent::Heartbeat { .. } => {
                        let _ = tx.send(ServerEvent::HeartbeatAck);
                    }

                    ClientEvent::VoiceStateUpdate {
                        guild_id,
                        channel_id,
                        is_muted,
                        is_deafened,
                    } => {
                        if let Some(uid) = user_id {
                            handle_voice_state(
                                &state, &tx, uid, guild_id, channel_id, is_muted, is_deafened,
                            )
                            .await;
                        }
                    }

                    ClientEvent::RequestVoiceToken { guild_id, channel_id } => {
                        if let Some(uid) = user_id {
                            issue_voice_token(&state, &tx, uid, guild_id, channel_id).await;
                        }
                    }

                    ClientEvent::ScreenShareStateUpdate { guild_id, channel_id, is_sharing } => {
                        if let Some(uid) = user_id {
                            handle_screen_share_state(&state, &tx, uid, guild_id, channel_id, is_sharing).await;
                        }
                    }
                }
            }
        }
    });

    // Ждём завершения любой из задач
    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }

    // Очистка в любом случае: убираем из connections и закрываем голосовые состояния
    let uid_opt = *shared_uid.lock().unwrap(); // drop the guard before .await
    if let Some(uid) = uid_opt {
        state.connections.remove(&uid);
        cleanup_on_disconnect(&state, uid).await;
    }
}

async fn send_existing_voice_states(state: &AppState, tx: &ClientSender, guilds: &[GuildSummary]) {
    use sqlx::Row;
    for guild in guilds {
        let rows = sqlx::query(
            "SELECT user_id, guild_id, channel_id, is_muted, is_deafened, is_streaming
             FROM voice_states WHERE guild_id = $1",
        )
        .bind(guild.id)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();

        for row in rows {
            let user_id: Uuid = row.get("user_id");
            let guild_id: Uuid = row.get("guild_id");
            let channel_id: Option<Uuid> = row.get("channel_id");
            let is_streaming: bool = row.get("is_streaming");

            let _ = tx.send(ServerEvent::VoiceStateUpdate {
                user_id,
                guild_id,
                channel_id,
                is_muted: row.get("is_muted"),
                is_deafened: row.get("is_deafened"),
            });

            if is_streaming {
                let _ = tx.send(ServerEvent::ScreenShareStateUpdate {
                    user_id,
                    guild_id,
                    channel_id,
                    is_sharing: true,
                });
            }
        }
    }
}

async fn cleanup_on_disconnect(state: &AppState, user_id: Uuid) {
    use sqlx::Row;
    let rows = sqlx::query(
        "DELETE FROM voice_states WHERE user_id = $1 RETURNING guild_id, channel_id, is_streaming",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    for row in rows {
        let guild_id: Uuid = row.get("guild_id");
        let channel_id: Option<Uuid> = row.get("channel_id");
        let was_streaming: bool = row.get("is_streaming");

        broadcast_to_guild(
            state,
            guild_id,
            ServerEvent::VoiceStateUpdate {
                user_id,
                guild_id,
                channel_id: None,
                is_muted: false,
                is_deafened: false,
            },
        )
        .await;

        if was_streaming {
            broadcast_to_guild(
                state,
                guild_id,
                ServerEvent::ScreenShareStateUpdate {
                    user_id,
                    guild_id,
                    channel_id,
                    is_sharing: false,
                },
            )
            .await;
        }
    }
}

async fn handle_screen_share_state(
    state: &AppState,
    tx: &ClientSender,
    user_id: Uuid,
    guild_id: Uuid,
    channel_id: Uuid,
    is_sharing: bool,
) {
    use crate::api::permissions::{ensure_permission, STREAM_SCREEN};

    if is_sharing && ensure_permission(state, user_id, guild_id, STREAM_SCREEN).await.is_err() {
        let _ = tx.send(ServerEvent::Error { message: "missing STREAM_SCREEN permission".into() });
        return;
    }

    let _ = sqlx::query(
        "UPDATE voice_states SET is_streaming = $1 WHERE user_id = $2 AND guild_id = $3",
    )
    .bind(is_sharing)
    .bind(user_id)
    .bind(guild_id)
    .execute(&state.db)
    .await;

    broadcast_to_guild(
        state,
        guild_id,
        ServerEvent::ScreenShareStateUpdate {
            user_id,
            guild_id,
            channel_id: Some(channel_id),
            is_sharing,
        },
    )
    .await;
}

async fn identify(
    state: &AppState,
    token: &str,
) -> anyhow::Result<(Uuid, String, Vec<GuildSummary>)> {
    use jsonwebtoken::{Algorithm, Validation, decode};

    let key = {
        let cache = state.jwks_cache.read().await;
        cache.decoding_key.clone()
    };

    let key = if let Some(k) = key {
        k
    } else {
        let k = fetch_jwks(&state.config.central_url).await?;
        state.jwks_cache.write().await.decoding_key = Some(k.clone());
        k
    };

    let mut validation = Validation::new(Algorithm::RS256);
    validation.validate_exp = true;

    let claims = decode::<Claims>(token, &key, &validation)?.claims;
    let user_id: Uuid = claims.sub.parse()?;

    // Обновляем username во всех гильдиях (исправляет пустые username от прямых DB-вставок)
    let _ = sqlx::query("UPDATE members SET username = $1 WHERE user_id = $2")
        .bind(&claims.username)
        .bind(user_id)
        .execute(&state.db)
        .await;

    // Получаем гильдии пользователя
    let rows = sqlx::query(
        "SELECT g.id, g.name, g.icon_url FROM guilds g
         JOIN members m ON m.guild_id = g.id
         WHERE m.user_id = $1 AND m.is_banned = false",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await?;

    use sqlx::Row;
    let guilds = rows
        .iter()
        .map(|r| GuildSummary {
            id: r.get("id"),
            name: r.get("name"),
            icon_url: r.get("icon_url"),
        })
        .collect();

    Ok((user_id, claims.username, guilds))
}

async fn handle_voice_state(
    state: &AppState,
    tx: &ClientSender,
    user_id: Uuid,
    guild_id: Uuid,
    channel_id: Option<Uuid>,
    is_muted: bool,
    is_deafened: bool,
) {
    // Проверяем право на подключение к голосу
    if channel_id.is_some() {
        if let Err(_) = ensure_permission(state, user_id, guild_id, CONNECT_VOICE).await {
            let _ = tx.send(ServerEvent::Error { message: "missing CONNECT_VOICE permission".into() });
            return;
        }
    }

    // Обновляем voice_state в БД
    let res = sqlx::query(
        "INSERT INTO voice_states (user_id, guild_id, channel_id, is_muted, is_deafened, updated_at)
         VALUES ($1, $2, $3, $4, $5, NOW())
         ON CONFLICT (user_id, guild_id) DO UPDATE
         SET channel_id = $3, is_muted = $4, is_deafened = $5, updated_at = NOW()",
    )
    .bind(user_id)
    .bind(guild_id)
    .bind(channel_id)
    .bind(is_muted)
    .bind(is_deafened)
    .execute(&state.db)
    .await;

    if res.is_err() {
        return;
    }

    // Рассылаем всем участникам гильдии
    broadcast_to_guild(
        state,
        guild_id,
        ServerEvent::VoiceStateUpdate {
            user_id,
            guild_id,
            channel_id,
            is_muted,
            is_deafened,
        },
    )
    .await;

    // Если вошёл в канал — выдаём LiveKit токен
    if let Some(ch_id) = channel_id {
        issue_voice_token(state, tx, user_id, guild_id, ch_id).await;
    }
}

async fn issue_voice_token(
    state: &AppState,
    tx: &ClientSender,
    user_id: Uuid,
    guild_id: Uuid,
    channel_id: Uuid,
) {
    match state.voice.token(user_id, guild_id, channel_id).await {
        Ok(token) => {
            let _ = tx.send(ServerEvent::VoiceServerUpdate {
                guild_id,
                channel_id,
                livekit_url: state.config.livekit_url.clone(),
                token,
            });
        }
        Err(e) => {
            tracing::error!("livekit token error: {e}");
        }
    }
}

pub async fn broadcast_to_guild(state: &AppState, guild_id: Uuid, event: ServerEvent) {
    // Получаем список участников гильдии из БД
    let rows = sqlx::query("SELECT user_id FROM members WHERE guild_id = $1 AND is_banned = false")
        .bind(guild_id)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();

    use sqlx::Row;
    for row in rows {
        let uid: Uuid = row.get("user_id");
        if let Some(sender) = state.connections.get(&uid) {
            let _ = sender.send(event.clone());
        }
    }
}
