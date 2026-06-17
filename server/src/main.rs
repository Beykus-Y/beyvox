mod admin;
mod api;
mod auth;
mod config;
mod error;
mod voice;
mod ws;

use std::sync::Arc;

use axum::{
    routing::{delete, get, patch, post, put},
    Router,
};
use dashmap::DashMap;
use sqlx::postgres::PgPoolOptions;
use tokio::sync::RwLock;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use uuid::Uuid;

use auth::middleware::JwksCache;
use ws::handler::ClientSender;
use config::Config;
use voice::VoiceService;

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub config: Arc<Config>,
    pub jwks_cache: Arc<RwLock<JwksCache>>,
    pub voice: Arc<VoiceService>,
    /// Активные WebSocket соединения: user_id → sender
    pub connections: Arc<DashMap<Uuid, ClientSender>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("beyvox_server=debug".parse()?),
        )
        .init();

    let config = Config::from_env()?;

    let db = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&db).await?;

    let voice = VoiceService::new(
        config.livekit_api_key.clone(),
        config.livekit_api_secret.clone(),
    );

    let state = AppState {
        db,
        config: Arc::new(config.clone()),
        jwks_cache: JwksCache::new(),
        voice: Arc::new(voice),
        connections: Arc::new(DashMap::new()),
    };

    tracing::info!("══════════════════════════════════════");
    tracing::info!("  Server name: {}", config.server_name);
    if config.owner_username.is_empty() {
        tracing::info!("  OWNER: (any user can create guilds)");
    } else {
        tracing::info!("  OWNER: @{}", config.owner_username);
    }
    tracing::info!("══════════════════════════════════════");

    let info_name = config.server_name.clone();

    let app = Router::new()
        .route("/", get(|| async { axum::Json(serde_json::json!({ "status": "ok" })) }))
        .route("/info", get({
            let name = info_name.clone();
            move || {
                let n = name.clone();
                async move {
                    tracing::info!("GET /info");
                    axum::Json(serde_json::json!({
                        "name": n,
                        "version": env!("CARGO_PKG_VERSION"),
                    }))
                }
            }
        }))
        // Setup (только при первом запуске)
        .route("/setup", get(admin::setup_page).post(admin::setup_submit))
        // Admin panel
        .route("/admin/login", get(admin::login_page).post(admin::login_submit))
        .route("/admin/logout", post(admin::logout))
        .route("/admin", get(admin::dashboard_page))
        .route("/admin/api/guilds", get(admin::admin_list_guilds))
        .route("/admin/api/guilds/:id/set-default", post(admin::admin_set_default))
        .route("/admin/api/guilds/:id", delete(admin::admin_delete_guild))
        .route("/admin/api/guilds/:id/members", get(admin::admin_list_members))
        .route("/admin/api/guilds/:gid/members/:uid/kick", post(admin::admin_kick_member))
        .route("/admin/api/guilds/:gid/members/:uid/ban", post(admin::admin_ban_member))
        // Discovery (для central pull)
        .route("/api/discovery", get(admin::discovery))
        // WebSocket
        .route("/ws", get(ws::handler::ws_handler))
        // Guilds
        .route("/guilds", post(api::guilds::create_guild))
        .route("/guilds/:id", get(api::guilds::get_guild).delete(api::guilds::delete_guild))
        .route("/guilds/:id/set-default", post(api::guilds::set_default_guild))
        .route("/guilds/:id/invites", post(api::guilds::create_invite))
        .route("/invites/:code/join", post(api::guilds::join_by_invite))
        // Channels
        .route("/guilds/:id/channels", get(api::channels::list_channels))
        .route("/guilds/:id/channels", post(api::channels::create_channel))
        .route("/guilds/:gid/channels/:cid", delete(api::channels::delete_channel))
        .route("/guilds/:gid/channels/:cid/voice-state", get(api::channels::get_voice_state))
        // Messages
        .route("/guilds/:gid/channels/:cid/messages", get(api::messages::get_messages))
        .route("/guilds/:gid/channels/:cid/messages", post(api::messages::send_message))
        .route("/guilds/:gid/channels/:cid/messages/:mid", patch(api::messages::edit_message))
        .route("/guilds/:gid/channels/:cid/messages/:mid", delete(api::messages::delete_message))
        // Reactions
        .route("/guilds/:gid/channels/:cid/messages/:mid/reactions/:emoji",
            put(api::messages::add_reaction).delete(api::messages::remove_reaction))
        // Members
        .route("/guilds/:id/members", get(api::members::list_members))
        .route("/guilds/:id/members/me", get(api::members::get_me))
        .route("/guilds/:gid/members/:uid/kick", post(api::members::kick_member))
        .route("/guilds/:gid/members/:uid/ban", post(api::members::ban_member))
        .route("/guilds/:gid/members/:uid/mute", post(api::members::mute_member))
        .route("/guilds/:gid/members/:uid/timeout", post(api::members::timeout_member))
        // Roles
        .route("/guilds/:id/roles", get(api::roles::list_roles).post(api::roles::create_role))
        .route("/guilds/:gid/roles/:rid", patch(api::roles::update_role).delete(api::roles::delete_role))
        .route("/guilds/:gid/members/:uid/roles/:rid",
            put(api::roles::assign_role).delete(api::roles::remove_role))
        // Invites management
        .route("/guilds/:id/invites", get(api::guilds::list_invites))
        .route("/guilds/:id/invites/:code", delete(api::guilds::delete_invite))
        // Channel permission overrides
        .route("/guilds/:gid/channels/:cid/permissions",
            get(api::channel_overrides::get_channel_permissions))
        .route("/guilds/:gid/channels/:cid/permissions/:rid",
            put(api::channel_overrides::set_channel_permission)
            .delete(api::channel_overrides::delete_channel_permission))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("beyvox-server listening on {addr}");
    axum::serve(listener, app).await?;

    Ok(())
}
