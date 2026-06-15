use anyhow::Result;

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub host: String,
    pub port: u16,
    pub jwt_private_key_pem: String,
    pub jwt_public_key_pem: String,
    pub access_token_ttl_secs: i64,
    pub refresh_token_ttl_secs: i64,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let jwt_private_key_pem = if let Ok(path) = std::env::var("JWT_PRIVATE_KEY_PATH") {
            std::fs::read_to_string(&path)
                .map_err(|e| anyhow::anyhow!("cannot read {path}: {e}"))?
        } else {
            required("JWT_PRIVATE_KEY_PEM")?.replace("\\n", "\n")
        };

        let jwt_public_key_pem = if let Ok(path) = std::env::var("JWT_PUBLIC_KEY_PATH") {
            std::fs::read_to_string(&path)
                .map_err(|e| anyhow::anyhow!("cannot read {path}: {e}"))?
        } else {
            required("JWT_PUBLIC_KEY_PEM")?.replace("\\n", "\n")
        };

        Ok(Self {
            database_url: required("DATABASE_URL")?,
            host: std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "3000".into())
                .parse()?,
            jwt_private_key_pem,
            jwt_public_key_pem,
            access_token_ttl_secs: std::env::var("ACCESS_TOKEN_TTL_SECS")
                .unwrap_or_else(|_| "900".into())
                .parse()?,
            refresh_token_ttl_secs: std::env::var("REFRESH_TOKEN_TTL_SECS")
                .unwrap_or_else(|_| "2592000".into())
                .parse()?,
        })
    }
}

fn required(key: &str) -> Result<String> {
    std::env::var(key).map_err(|_| anyhow::anyhow!("missing env var: {}", key))
}
