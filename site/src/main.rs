mod auth;
mod catalog;
mod config;
mod error;

use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};
use sqlx::postgres::PgPoolOptions;
use tower_http::{cors::CorsLayer, services::{ServeDir, ServeFile}, trace::TraceLayer};

use auth::jwt::JwtKeys;
use config::Config;

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub config: Arc<Config>,
    pub jwt: Arc<JwtKeys>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("beyvox_central=debug".parse()?),
        )
        .init();

    let config = Config::from_env()?;
    let jwt = JwtKeys::from_pem(&config.jwt_private_key_pem, &config.jwt_public_key_pem)?;

    let db = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&db).await?;

    let state = AppState {
        db,
        config: Arc::new(config.clone()),
        jwt: Arc::new(jwt),
    };

    catalog::puller::spawn(state.clone());

    let api = Router::new()
        .route("/auth/register", post(auth::handlers::register))
        .route("/auth/login", post(auth::handlers::login))
        .route("/auth/refresh", post(auth::handlers::refresh))
        .route("/auth/verify", get(auth::handlers::verify_email))
        .route("/auth/status", get(auth::handlers::status))
        .route("/.well-known/jwks.json", get(auth::handlers::jwks))
        .route("/servers", get(catalog::handlers::list_servers))
        .route("/servers", post(catalog::handlers::register_server))
        .route("/servers/:id/ping", post(catalog::handlers::ping_server))
        .with_state(state);

    let static_dir = std::env::var("STATIC_DIR").unwrap_or_else(|_| "static".into());
    let app = Router::new()
        .nest("/api", api.clone())
        .nest("/", api)
        .fallback_service(
            ServeDir::new(&static_dir)
                .fallback(ServeFile::new(format!("{static_dir}/index.html")))
        )
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("listening on {addr}");
    axum::serve(listener, app).await?;

    Ok(())
}
