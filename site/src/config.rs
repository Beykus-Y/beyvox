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
    pub email_verification_required: bool,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub smtp_from: String,
    pub central_public_url: String,
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

        let email_verification_required = std::env::var("EMAIL_VERIFICATION_REQUIRED")
            .unwrap_or_else(|_| "false".into())
            .parse::<bool>()?;

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
            email_verification_required,
            smtp_host: std::env::var("SMTP_HOST").unwrap_or_else(|_| "smtp.beget.com".into()),
            smtp_port: std::env::var("SMTP_PORT").unwrap_or_else(|_| "465".into()).parse()?,
            smtp_username: std::env::var("SMTP_USERNAME").unwrap_or_else(|_| "noreply@beykus.fun".into()),
            smtp_password: std::env::var("SMTP_PASSWORD").unwrap_or_default(),
            smtp_from: std::env::var("SMTP_FROM").unwrap_or_else(|_| "BeyVox <noreply@beykus.fun>".into()),
            central_public_url: std::env::var("CENTRAL_PUBLIC_URL").unwrap_or_else(|_| "https://beyvox.beykus.fun".into()),
        })
    }
}

fn required(key: &str) -> Result<String> {
    std::env::var(key).map_err(|_| anyhow::anyhow!("missing env var: {}", key))
}
