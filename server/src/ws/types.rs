use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Сообщения от сервера к клиенту
#[derive(Serialize, Clone, Debug)]
#[serde(tag = "op", content = "d", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ServerEvent {
    /// Первое сообщение после подключения
    Hello {
        heartbeat_interval: u64,
    },
    /// После успешной аутентификации — полный дамп состояния
    Ready {
        user_id: Uuid,
        username: String,
        guilds: Vec<GuildSummary>,
    },
    /// Новое сообщение в текстовом канале
    MessageCreate {
        message: MessageDto,
    },
    /// Обновление голосового состояния участника
    VoiceStateUpdate {
        user_id: Uuid,
        guild_id: Uuid,
        channel_id: Option<Uuid>,
        is_muted: bool,
        is_deafened: bool,
    },
    /// Адрес и токен LiveKit для подключения к голосу
    VoiceServerUpdate {
        guild_id: Uuid,
        channel_id: Uuid,
        livekit_url: String,
        token: String,
    },
    /// Смена статуса/присутствия участника
    PresenceUpdate {
        user_id: Uuid,
        guild_id: Uuid,
        status: UserStatus,
    },
    /// Редактирование сообщения
    MessageUpdate {
        message_id: Uuid,
        channel_id: Uuid,
        content: String,
        edited_at: chrono::DateTime<chrono::Utc>,
    },
    /// Удаление сообщения
    MessageDelete {
        message_id: Uuid,
        channel_id: Uuid,
    },
    /// Создание канала
    ChannelCreate {
        channel: WsChannel,
    },
    /// Удаление канала
    ChannelDelete {
        channel_id: Uuid,
        guild_id: Uuid,
    },
    /// Ответ на heartbeat
    HeartbeatAck,
    /// Ошибка
    Error {
        message: String,
    },
}

/// Сообщения от клиента к серверу
#[derive(Deserialize, Debug)]
#[serde(tag = "op", content = "d", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ClientEvent {
    /// Аутентификация JWT токеном
    Identify {
        token: String,
    },
    /// Keep-alive
    Heartbeat {
        seq: u64,
    },
    /// Вход/выход из голосового канала
    VoiceStateUpdate {
        guild_id: Uuid,
        channel_id: Option<Uuid>, // None = выход
        is_muted: bool,
        is_deafened: bool,
    },
    /// Запрос токена LiveKit для уже занятого канала
    RequestVoiceToken {
        guild_id: Uuid,
        channel_id: Uuid,
    },
}

#[derive(Serialize, Clone, Debug)]
pub struct WsChannel {
    pub id: Uuid,
    pub guild_id: Uuid,
    pub name: String,
    pub r#type: String,
    pub position: i32,
    pub user_limit: Option<i32>,
}

#[derive(Serialize, Clone, Debug)]
pub struct GuildSummary {
    pub id: Uuid,
    pub name: String,
    pub icon_url: Option<String>,
}

#[derive(Serialize, Clone, Debug)]
pub struct MessageDto {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub author_id: Uuid,
    pub author_username: String,
    pub content: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub edited_at: Option<chrono::DateTime<chrono::Utc>>,
    pub reply_to: Option<Uuid>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum UserStatus {
    Online,
    Away,
    DoNotDisturb,
    Invisible,
}
