use anyhow::Result;

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub host: String,
    pub port: u16,
    pub central_url: String,
    pub livekit_url: String,
    pub livekit_api_key: String,
    pub livekit_api_secret: String,
    pub catalog_register: bool,
    pub server_name: String,
    pub server_address: String,
    /// Токен владельца — нужен для создания гильдий. Если не задан в .env, генерируется при старте.
    pub owner_token: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let owner_token = std::env::var("OWNER_TOKEN").unwrap_or_else(|_| {
            // Генерируем случайный токен если не задан
            use std::fmt::Write;
            let bytes: Vec<u8> = (0..16).map(|_| rand::random::<u8>()).collect();
            bytes.iter().fold(String::new(), |mut s, b| { write!(s, "{b:02x}").ok(); s })
        });

        Ok(Self {
            database_url: required("DATABASE_URL")?,
            host: std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            port: std::env::var("PORT").unwrap_or_else(|_| "8080".into()).parse()?,
            central_url: std::env::var("CENTRAL_URL")
                .unwrap_or_else(|_| "https://beyvox.beykus.fun".into()),
            livekit_url: required("LIVEKIT_URL")?,
            livekit_api_key: required("LIVEKIT_API_KEY")?,
            livekit_api_secret: required("LIVEKIT_API_SECRET")?,
            catalog_register: std::env::var("CATALOG_REGISTER")
                .unwrap_or_else(|_| "false".into())
                .parse()
                .unwrap_or(false),
            server_name: std::env::var("SERVER_NAME").unwrap_or_else(|_| "BeyVox Server".into()),
            server_address: std::env::var("SERVER_ADDRESS").unwrap_or_else(|_| "localhost:8080".into()),
            owner_token,
        })
    }
}

fn required(key: &str) -> Result<String> {
    std::env::var(key).map_err(|_| anyhow::anyhow!("missing env var: {}", key))
}
