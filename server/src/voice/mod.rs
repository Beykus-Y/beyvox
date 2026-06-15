use anyhow::Result;
use chrono::Utc;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::Serialize;
use uuid::Uuid;

#[derive(Clone)]
pub struct VoiceService {
    api_key: String,
    api_secret: String,
}

#[derive(Serialize)]
struct VideoGrant {
    room: String,
    #[serde(rename = "roomJoin")]
    room_join: bool,
    #[serde(rename = "canPublish")]
    can_publish: bool,
    #[serde(rename = "canSubscribe")]
    can_subscribe: bool,
    #[serde(rename = "canPublishData")]
    can_publish_data: bool,
}

#[derive(Serialize)]
struct LiveKitClaims {
    sub: String,
    iss: String,
    exp: i64,
    nbf: i64,
    video: VideoGrant,
}

impl VoiceService {
    pub fn new(api_key: String, api_secret: String) -> Self {
        Self { api_key, api_secret }
    }

    /// Генерирует LiveKit JWT токен.
    /// Room name = "{guild_id}:{channel_id}"
    pub async fn token(&self, user_id: Uuid, guild_id: Uuid, channel_id: Uuid) -> Result<String> {
        let now = Utc::now().timestamp();
        let room = format!("{guild_id}:{channel_id}");

        let claims = LiveKitClaims {
            sub: user_id.to_string(),
            iss: self.api_key.clone(),
            exp: now + 3600 * 6, // 6 часов
            nbf: now,
            video: VideoGrant {
                room,
                room_join: true,
                can_publish: true,
                can_subscribe: true,
                can_publish_data: true,
            },
        };

        let mut header = Header::new(Algorithm::HS256);
        header.kid = None;

        let token = encode(
            &header,
            &claims,
            &EncodingKey::from_secret(self.api_secret.as_bytes()),
        )?;

        Ok(token)
    }
}
