use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use axum::{
    extract::{Form, Path, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use uuid::Uuid;

use crate::{error::AppResult, AppState};

// ── Session helpers ─────────────────────────────────────────────────────────

fn extract_session_token(headers: &HeaderMap) -> Option<Uuid> {
    let raw = headers.get("cookie")?.to_str().ok()?;
    raw.split(';').find_map(|part| {
        let part = part.trim();
        let val = part.strip_prefix("adm_session=")?;
        Uuid::parse_str(val).ok()
    })
}

async fn resolve_session(state: &AppState, headers: &HeaderMap) -> Option<Uuid> {
    let token = extract_session_token(headers)?;
    let row = sqlx::query(
        "SELECT admin_id FROM admin_sessions WHERE token = $1 AND expires_at > NOW()",
    )
    .bind(token)
    .fetch_optional(&state.db)
    .await
    .ok()??;
    Some(row.get("admin_id"))
}

fn redirect_login() -> Response {
    Redirect::to("/admin/login").into_response()
}

// ── Setup ───────────────────────────────────────────────────────────────────

async fn setup_open(state: &AppState) -> bool {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM server_admins")
        .fetch_one(&state.db)
        .await
        .unwrap_or(1);
    count == 0
}

pub async fn setup_page(State(state): State<AppState>) -> Response {
    if !setup_open(&state).await {
        return StatusCode::NOT_FOUND.into_response();
    }
    Html(SETUP_HTML).into_response()
}

#[derive(Deserialize)]
pub struct SetupForm {
    username: String,
    password: String,
    guild_name: String,
}

pub async fn setup_submit(
    State(state): State<AppState>,
    Form(form): Form<SetupForm>,
) -> Response {
    if !setup_open(&state).await {
        return StatusCode::NOT_FOUND.into_response();
    }

    if form.username.is_empty() || form.password.len() < 8 {
        return Html(setup_error("Username required; password must be at least 8 chars."))
            .into_response();
    }

    let salt = SaltString::generate(&mut OsRng);
    let hash = match Argon2::default().hash_password(form.password.as_bytes(), &salt) {
        Ok(h) => h.to_string(),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    if let Err(e) = sqlx::query(
        "INSERT INTO server_admins (username, password_hash) VALUES ($1, $2)",
    )
    .bind(&form.username)
    .bind(&hash)
    .execute(&state.db)
    .await
    {
        tracing::error!("setup insert admin: {e}");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    // Создаём дефолтную гильдию если сервер пустой
    let guild_name = form.guild_name.trim();
    if !guild_name.is_empty() {
        let existing: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM guilds")
            .fetch_one(&state.db)
            .await
            .unwrap_or(1);

        if existing == 0 {
            let guild_id = Uuid::new_v4();
            let _ = sqlx::query(
                "INSERT INTO guilds (id, name, owner_id, is_default, is_public)
                 VALUES ($1, $2, $3, true, true)",
            )
            .bind(guild_id)
            .bind(guild_name)
            .bind(Uuid::nil())
            .execute(&state.db)
            .await;

            let _ = sqlx::query(
                "INSERT INTO channels (guild_id, name, type, position)
                 VALUES ($1, 'general', 'text', 0)",
            )
            .bind(guild_id)
            .execute(&state.db)
            .await;

            let everyone_perms: i64 = 16 | 32 | 64;
            let _ = sqlx::query(
                "INSERT INTO roles (guild_id, name, position, permissions)
                 VALUES ($1, '@everyone', 0, $2)",
            )
            .bind(guild_id)
            .bind(everyone_perms)
            .execute(&state.db)
            .await;
        }
    }

    Redirect::to("/admin/login").into_response()
}

// ── Login / Logout ──────────────────────────────────────────────────────────

pub async fn login_page() -> Html<&'static str> {
    Html(LOGIN_HTML)
}

#[derive(Deserialize)]
pub struct LoginForm {
    username: String,
    password: String,
}

pub async fn login_submit(
    State(state): State<AppState>,
    Form(form): Form<LoginForm>,
) -> Response {
    let row = sqlx::query(
        "SELECT id, password_hash FROM server_admins WHERE username = $1",
    )
    .bind(&form.username)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    let ok = row.as_ref().and_then(|r| {
        let hash: String = r.get("password_hash");
        let parsed = PasswordHash::new(&hash).ok()?;
        Argon2::default()
            .verify_password(form.password.as_bytes(), &parsed)
            .ok()
    });

    if ok.is_none() {
        return Html(login_error("Invalid username or password.")).into_response();
    }

    let admin_id: Uuid = row.unwrap().get("id");
    let token = Uuid::new_v4();
    let expires_at = chrono::Utc::now() + chrono::Duration::days(7);

    if let Err(e) = sqlx::query(
        "INSERT INTO admin_sessions (token, admin_id, expires_at) VALUES ($1, $2, $3)",
    )
    .bind(token)
    .bind(admin_id)
    .bind(expires_at)
    .execute(&state.db)
    .await
    {
        tracing::error!("login insert session: {e}");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    let cookie = format!(
        "adm_session={}; Path=/admin; HttpOnly; SameSite=Lax; Max-Age=604800",
        token
    );
    (
        [(axum::http::header::SET_COOKIE, cookie)],
        Redirect::to("/admin"),
    )
        .into_response()
}

pub async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Response {
    if let Some(token) = extract_session_token(&headers) {
        let _ = sqlx::query("DELETE FROM admin_sessions WHERE token = $1")
            .bind(token)
            .execute(&state.db)
            .await;
    }
    let clear = "adm_session=; Path=/admin; HttpOnly; Max-Age=0";
    (
        [(axum::http::header::SET_COOKIE, clear)],
        Redirect::to("/admin/login"),
    )
        .into_response()
}

// ── Dashboard page ──────────────────────────────────────────────────────────

pub async fn dashboard_page(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Response {
    if resolve_session(&state, &headers).await.is_none() {
        return redirect_login();
    }
    Html(DASHBOARD_HTML).into_response()
}

// ── Admin JSON API ──────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct AdminGuildDto {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub is_default: bool,
    pub is_public: bool,
    pub member_count: i64,
}

pub async fn admin_list_guilds(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Response {
    if resolve_session(&state, &headers).await.is_none() {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let rows = sqlx::query(
        "SELECT g.id, g.name, g.description, g.is_default, g.is_public,
                COUNT(m.user_id) as member_count
         FROM guilds g
         LEFT JOIN members m ON m.guild_id = g.id AND m.is_banned = false
         GROUP BY g.id
         ORDER BY g.is_default DESC, g.created_at ASC",
    )
    .fetch_all(&state.db)
    .await;

    match rows {
        Ok(rows) => {
            let guilds: Vec<AdminGuildDto> = rows
                .iter()
                .map(|r| AdminGuildDto {
                    id: r.get("id"),
                    name: r.get("name"),
                    description: r.get("description"),
                    is_default: r.get("is_default"),
                    is_public: r.get("is_public"),
                    member_count: r.get("member_count"),
                })
                .collect();
            Json(guilds).into_response()
        }
        Err(e) => {
            tracing::error!("admin list guilds: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn admin_set_default(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(guild_id): Path<Uuid>,
) -> Response {
    if resolve_session(&state, &headers).await.is_none() {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    match sqlx::query("UPDATE guilds SET is_default = (id = $1)")
        .bind(guild_id)
        .execute(&state.db)
        .await
    {
        Ok(_) => Json(serde_json::json!({ "ok": true })).into_response(),
        Err(e) => {
            tracing::error!("admin set default: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn admin_delete_guild(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(guild_id): Path<Uuid>,
) -> Response {
    if resolve_session(&state, &headers).await.is_none() {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let row = sqlx::query("SELECT is_default FROM guilds WHERE id = $1")
        .bind(guild_id)
        .fetch_optional(&state.db)
        .await;

    let row = match row {
        Ok(Some(r)) => r,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!("{e}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    if row.get::<bool, _>("is_default") {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "cannot delete default guild — set another guild as default first"
            })),
        )
            .into_response();
    }

    match sqlx::query("DELETE FROM guilds WHERE id = $1")
        .bind(guild_id)
        .execute(&state.db)
        .await
    {
        Ok(_) => Json(serde_json::json!({ "ok": true })).into_response(),
        Err(e) => {
            tracing::error!("admin delete guild: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

#[derive(Serialize)]
pub struct AdminMemberDto {
    pub user_id: Uuid,
    pub username: String,
    pub nickname: Option<String>,
    pub joined_at: chrono::DateTime<chrono::Utc>,
    pub is_muted: bool,
    pub is_banned: bool,
    pub roles: Vec<Uuid>,
}

pub async fn admin_list_members(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(guild_id): Path<Uuid>,
) -> Response {
    if resolve_session(&state, &headers).await.is_none() {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let rows = sqlx::query(
        "SELECT m.user_id, m.username, m.nickname, m.joined_at, m.is_muted, m.is_banned,
                COALESCE(ARRAY_AGG(mr.role_id) FILTER (WHERE mr.role_id IS NOT NULL), '{}') as roles
         FROM members m 
         LEFT JOIN member_roles mr ON mr.user_id = m.user_id AND mr.guild_id = m.guild_id
         WHERE m.guild_id = $1 
         GROUP BY m.user_id, m.username, m.nickname, m.joined_at, m.is_muted, m.is_banned
         ORDER BY m.joined_at",
    )
    .bind(guild_id)
    .fetch_all(&state.db)
    .await;

    match rows {
        Ok(rows) => {
            let members: Vec<AdminMemberDto> = rows
                .iter()
                .map(|r| AdminMemberDto {
                    user_id: r.get("user_id"),
                    username: r.get("username"),
                    nickname: r.get("nickname"),
                    joined_at: r.get("joined_at"),
                    is_muted: r.get("is_muted"),
                    is_banned: r.get("is_banned"),
                    roles: r.get("roles"),
                })
                .collect();
            Json(members).into_response()
        }
        Err(e) => {
            tracing::error!("admin list members: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn admin_kick_member(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((guild_id, user_id)): Path<(Uuid, Uuid)>,
) -> Response {
    if resolve_session(&state, &headers).await.is_none() {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    match sqlx::query("DELETE FROM members WHERE user_id = $1 AND guild_id = $2")
        .bind(user_id)
        .bind(guild_id)
        .execute(&state.db)
        .await
    {
        Ok(_) => {
            // Send WS update
            let _ = crate::ws::handler::broadcast_to_guild(&state, guild_id, crate::ws::types::ServerEvent::MemberRemove { guild_id, user_id }).await;
            Json(serde_json::json!({ "ok": true })).into_response()
        }
        Err(e) => {
            tracing::error!("{e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn admin_ban_member(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((guild_id, user_id)): Path<(Uuid, Uuid)>,
) -> Response {
    if resolve_session(&state, &headers).await.is_none() {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    match sqlx::query(
        "UPDATE members SET is_banned = true WHERE user_id = $1 AND guild_id = $2",
    )
    .bind(user_id)
    .bind(guild_id)
    .execute(&state.db)
    .await
    {
        Ok(_) => {
            // Send WS update
            let _ = crate::ws::handler::broadcast_to_guild(&state, guild_id, crate::ws::types::ServerEvent::MemberRemove { guild_id, user_id }).await;
            Json(serde_json::json!({ "ok": true })).into_response()
        }
        Err(e) => {
            tracing::error!("{e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

// ── Вкладка: Глобальные Пользователи ────────────────────────────────────────

#[derive(Serialize)]
pub struct AdminUserListDto {
    pub user_id: Uuid,
    pub username: String,
    pub is_banned: bool,
}

pub async fn admin_list_users(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Response {
    if resolve_session(&state, &headers).await.is_none() {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let rows = sqlx::query(
        "SELECT DISTINCT u.user_id, u.username, 
                EXISTS(SELECT 1 FROM banned_users b WHERE b.user_id = u.user_id) as is_banned 
         FROM (
             SELECT user_id, username FROM members
             UNION
             SELECT user_id, username FROM banned_users
         ) u
         ORDER BY u.username"
    )
    .fetch_all(&state.db)
    .await;

    match rows {
        Ok(rows) => {
            let users: Vec<AdminUserListDto> = rows
                .iter()
                .map(|r| AdminUserListDto {
                    user_id: r.get("user_id"),
                    username: r.get("username"),
                    is_banned: r.get("is_banned"),
                })
                .collect();
            Json(users).into_response()
        }
        Err(e) => {
            tracing::error!("admin list users: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn admin_ban_user_global(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(user_id): Path<Uuid>,
) -> Response {
    if resolve_session(&state, &headers).await.is_none() {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let username: String = sqlx::query_scalar("SELECT username FROM members WHERE user_id = $1 LIMIT 1")
        .bind(user_id)
        .fetch_optional(&state.db)
        .await
        .unwrap_or_default()
        .unwrap_or_else(|| "Unknown".to_string());

    let res = sqlx::query("INSERT INTO banned_users (user_id, username) VALUES ($1, $2) ON CONFLICT (user_id) DO NOTHING")
        .bind(user_id)
        .bind(username)
        .execute(&state.db)
        .await;

    if res.is_ok() {
        // Disconnect immediately if online
        if let Some((_, sender)) = state.connections.remove(&user_id) {
            let _ = sender.send(crate::ws::types::ServerEvent::Error { message: "you have been globally banned".into() });
        }
    }

    match res {
        Ok(_) => Json(serde_json::json!({ "ok": true })).into_response(),
        Err(e) => {
            tracing::error!("ban global: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn admin_unban_user_global(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(user_id): Path<Uuid>,
) -> Response {
    if resolve_session(&state, &headers).await.is_none() {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    match sqlx::query("DELETE FROM banned_users WHERE user_id = $1")
        .bind(user_id)
        .execute(&state.db)
        .await
    {
        Ok(_) => Json(serde_json::json!({ "ok": true })).into_response(),
        Err(e) => {
            tracing::error!("unban global: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

// ── Вкладка: Настройки Сервера ──────────────────────────────────────────────

#[derive(Deserialize)]
pub struct AdminSettingsReq {
    pub server_name: String,
    pub catalog_register: bool,
}

pub async fn admin_get_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Response {
    if resolve_session(&state, &headers).await.is_none() {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let mut name = state.config.server_name.clone();
    let mut register = state.config.catalog_register;

    if let Ok(Some(n)) = sqlx::query_scalar::<_, String>("SELECT value FROM server_settings WHERE key = 'server_name'")
        .fetch_optional(&state.db).await {
        name = n;
    }
    if let Ok(Some(r)) = sqlx::query_scalar::<_, String>("SELECT value FROM server_settings WHERE key = 'catalog_register'")
        .fetch_optional(&state.db).await {
        register = r == "true";
    }

    Json(serde_json::json!({
        "server_name": name,
        "catalog_register": register,
    })).into_response()
}

pub async fn admin_save_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<AdminSettingsReq>,
) -> Response {
    if resolve_session(&state, &headers).await.is_none() {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let _ = sqlx::query("INSERT INTO server_settings (key, value) VALUES ('server_name', $1) ON CONFLICT (key) DO UPDATE SET value = $1")
        .bind(&body.server_name)
        .execute(&state.db)
        .await;

    let _ = sqlx::query("INSERT INTO server_settings (key, value) VALUES ('catalog_register', $1) ON CONFLICT (key) DO UPDATE SET value = $1")
        .bind(if body.catalog_register { "true" } else { "false" })
        .execute(&state.db)
        .await;

    Json(serde_json::json!({ "ok": true })).into_response()
}

// ── Вкладка: Состояние Системы ──────────────────────────────────────────────

pub async fn admin_get_stats(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Response {
    if resolve_session(&state, &headers).await.is_none() {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let active_connections = state.connections.len();

    let total_guilds: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM guilds")
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

    let total_users: i64 = sqlx::query_scalar(
        "SELECT COUNT(DISTINCT user_id) FROM (
            SELECT user_id FROM members
            UNION
            SELECT user_id FROM banned_users
         ) u"
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(0);

    let livekit_status = match state.voice.token(Uuid::nil(), Uuid::nil(), Uuid::nil()).await {
        Ok(_) => "Connected",
        Err(_) => "Disconnected (Check credentials)",
    };

    Json(serde_json::json!({
        "active_connections": active_connections,
        "total_guilds": total_guilds,
        "total_users": total_users,
        "livekit_status": livekit_status,
        "version": env!("CARGO_PKG_VERSION"),
    })).into_response()
}

// ── Полное управление ролями в гильдиях ──────────────────────────────────────

pub async fn admin_list_roles(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(guild_id): Path<Uuid>,
) -> Response {
    if resolve_session(&state, &headers).await.is_none() {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    
    let rows = sqlx::query(
        "SELECT id, guild_id, name, color, permissions, position FROM roles WHERE guild_id = $1 ORDER BY position",
    )
    .bind(guild_id)
    .fetch_all(&state.db)
    .await;

    match rows {
        Ok(rows) => {
            let roles: Vec<crate::api::roles::RoleDto> = rows
                .iter()
                .map(|r| crate::api::roles::RoleDto {
                    id: r.get("id"),
                    guild_id: r.get("guild_id"),
                    name: r.get("name"),
                    color: r.get("color"),
                    permissions: r.get("permissions"),
                    position: r.get("position"),
                })
                .collect();
            Json(roles).into_response()
        }
        Err(e) => {
            tracing::error!("admin list roles: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

#[derive(Deserialize)]
pub struct AdminCreateRoleReq {
    pub name: String,
    pub color: Option<String>,
    pub permissions: i64,
}

pub async fn admin_create_role(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(guild_id): Path<Uuid>,
    Json(body): Json<AdminCreateRoleReq>,
) -> Response {
    if resolve_session(&state, &headers).await.is_none() {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    if body.name.is_empty() {
        return StatusCode::BAD_REQUEST.into_response();
    }

    let position: i32 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(position), 0) + 1 FROM roles WHERE guild_id = $1",
    )
    .bind(guild_id)
    .fetch_one(&state.db)
    .await
    .unwrap_or(0);

    let id = Uuid::new_v4();
    match sqlx::query(
        "INSERT INTO roles (id, guild_id, name, color, permissions, position)
         VALUES ($1, $2, $3, $4, $5, $6)
         RETURNING id, guild_id, name, color, permissions, position"
    )
    .bind(id)
    .bind(guild_id)
    .bind(&body.name)
    .bind(&body.color)
    .bind(body.permissions)
    .bind(position)
    .fetch_one(&state.db)
    .await
    {
        Ok(r) => {
            let role = crate::api::roles::RoleDto {
                id: r.get("id"),
                guild_id: r.get("guild_id"),
                name: r.get("name"),
                color: r.get("color"),
                permissions: r.get("permissions"),
                position: r.get("position"),
            };
            let ws_role = crate::ws::types::WsRole {
                id: role.id,
                guild_id: role.guild_id,
                name: role.name.clone(),
                color: role.color.clone(),
                permissions: role.permissions,
                position: role.position,
            };
            let _ = crate::ws::handler::broadcast_to_guild(&state, guild_id, crate::ws::types::ServerEvent::RoleCreate { guild_id, role: ws_role }).await;
            Json(role).into_response()
        }
        Err(e) => {
            tracing::error!("admin create role: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

#[derive(Deserialize)]
pub struct AdminUpdateRoleReq {
    pub name: Option<String>,
    pub color: Option<String>,
    pub permissions: Option<i64>,
    pub position: Option<i32>,
}

pub async fn admin_update_role(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((guild_id, role_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<AdminUpdateRoleReq>,
) -> Response {
    if resolve_session(&state, &headers).await.is_none() {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let current_name: String = sqlx::query_scalar("SELECT name FROM roles WHERE id = $1")
        .bind(role_id)
        .fetch_one(&state.db)
        .await
        .unwrap_or_default();

    let name = body.name.unwrap_or_else(|| current_name.clone());
    let name = if current_name == "@everyone" {
        "@everyone".to_string()
    } else {
        name
    };

    let color = body.color;
    let permissions = body.permissions.unwrap_or(0);
    let position = body.position.unwrap_or(0);

    match sqlx::query(
        "UPDATE roles SET name = $1, color = $2, permissions = $3, position = $4
         WHERE id = $5 AND guild_id = $6
         RETURNING id, guild_id, name, color, permissions, position"
    )
    .bind(name)
    .bind(color)
    .bind(permissions)
    .bind(position)
    .bind(role_id)
    .bind(guild_id)
    .fetch_one(&state.db)
    .await
    {
        Ok(r) => {
            let role = crate::api::roles::RoleDto {
                id: r.get("id"),
                guild_id: r.get("guild_id"),
                name: r.get("name"),
                color: r.get("color"),
                permissions: r.get("permissions"),
                position: r.get("position"),
            };
            let ws_role = crate::ws::types::WsRole {
                id: role.id,
                guild_id: role.guild_id,
                name: role.name.clone(),
                color: role.color.clone(),
                permissions: role.permissions,
                position: role.position,
            };
            let _ = crate::ws::handler::broadcast_to_guild(&state, guild_id, crate::ws::types::ServerEvent::RoleUpdate { guild_id, role: ws_role }).await;
            Json(role).into_response()
        }
        Err(e) => {
            tracing::error!("admin update role: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn admin_delete_role(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((guild_id, role_id)): Path<(Uuid, Uuid)>,
) -> Response {
    if resolve_session(&state, &headers).await.is_none() {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let name: String = sqlx::query_scalar("SELECT name FROM roles WHERE id = $1")
        .bind(role_id)
        .fetch_one(&state.db)
        .await
        .unwrap_or_default();

    if name == "@everyone" {
        return (StatusCode::BAD_REQUEST, "cannot delete @everyone role").into_response();
    }

    match sqlx::query("DELETE FROM roles WHERE id = $1").bind(role_id).execute(&state.db).await {
        Ok(_) => {
            let _ = crate::ws::handler::broadcast_to_guild(&state, guild_id, crate::ws::types::ServerEvent::RoleDelete { guild_id, role_id }).await;
            Json(serde_json::json!({ "ok": true })).into_response()
        }
        Err(e) => {
            tracing::error!("admin delete role: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn admin_assign_role(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((guild_id, user_id, role_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Response {
    if resolve_session(&state, &headers).await.is_none() {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    match sqlx::query("INSERT INTO member_roles (user_id, guild_id, role_id) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING")
        .bind(user_id)
        .bind(guild_id)
        .bind(role_id)
        .execute(&state.db)
        .await
    {
        Ok(_) => {
            if let Ok(member) = crate::api::members::fetch_ws_member(&state, user_id, guild_id).await {
                let _ = crate::ws::handler::broadcast_to_guild(&state, guild_id, crate::ws::types::ServerEvent::MemberUpdate { guild_id, member }).await;
            }
            Json(serde_json::json!({ "ok": true })).into_response()
        }
        Err(e) => {
            tracing::error!("admin assign role: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn admin_remove_role(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((guild_id, user_id, role_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Response {
    if resolve_session(&state, &headers).await.is_none() {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    match sqlx::query("DELETE FROM member_roles WHERE user_id = $1 AND guild_id = $2 AND role_id = $3")
        .bind(user_id)
        .bind(guild_id)
        .bind(role_id)
        .execute(&state.db)
        .await
    {
        Ok(_) => {
            if let Ok(member) = crate::api::members::fetch_ws_member(&state, user_id, guild_id).await {
                let _ = crate::ws::handler::broadcast_to_guild(&state, guild_id, crate::ws::types::ServerEvent::MemberUpdate { guild_id, member }).await;
            }
            Json(serde_json::json!({ "ok": true })).into_response()
        }
        Err(e) => {
            tracing::error!("admin remove role: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

// ── Discovery (для central pull) ─────────────────────────────────────────────

#[derive(Serialize)]
pub struct DiscoveryResponse {
    pub name: String,
    pub version: &'static str,
    pub online_count: usize,
    pub total_members: i64,
    pub default_guild: Option<DiscoveryGuildSummary>,
}

#[derive(Serialize)]
pub struct DiscoveryGuildSummary {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub member_count: i64,
}

pub async fn discovery(State(state): State<AppState>) -> AppResult<Json<DiscoveryResponse>> {
    let total_members: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(cnt), 0)::bigint FROM (
             SELECT COUNT(m.user_id) as cnt
             FROM guilds g
             LEFT JOIN members m ON m.guild_id = g.id AND m.is_banned = false
             GROUP BY g.id
         ) sub",
    )
    .fetch_one(&state.db)
    .await?;

    let default_row = sqlx::query(
        "SELECT g.id, g.name, g.description,
                COUNT(m.user_id) as member_count
         FROM guilds g
         LEFT JOIN members m ON m.guild_id = g.id AND m.is_banned = false
         WHERE g.is_default = true
         GROUP BY g.id
         LIMIT 1",
    )
    .fetch_optional(&state.db)
    .await?;

    let default_guild = default_row.map(|r| DiscoveryGuildSummary {
        id: r.get("id"),
        name: r.get("name"),
        description: r.get("description"),
        member_count: r.get("member_count"),
    });

    Ok(Json(DiscoveryResponse {
        name: state.config.server_name.clone(),
        version: env!("CARGO_PKG_VERSION"),
        online_count: state.connections.len(),
        total_members,
        default_guild,
    }))
}

// ── HTML ─────────────────────────────────────────────────────────────────────

fn setup_error(msg: &str) -> String {
    SETUP_HTML.replace("<!--ERROR-->", &format!(r#"<p class="error">{msg}</p>"#))
}

fn login_error(msg: &str) -> String {
    LOGIN_HTML.replace("<!--ERROR-->", &format!(r#"<p class="error">{msg}</p>"#))
}

const SETUP_HTML: &str = r##"<!DOCTYPE html>
<html lang="ru">
<head>
    <meta charset="utf-8">
    <title>BeyVox Setup</title>
    <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap" rel="stylesheet">
    <style>
        :root {
            --bg: #090d16;
            --card-bg: #111827;
            --border: #1f2937;
            --text: #f3f4f6;
            --text-muted: #9ca3af;
            --primary: #6366f1;
            --primary-hover: #4f46e5;
            --error: #ef4444;
        }
        * { box-sizing: border-box; margin: 0; padding: 0; }
        body {
            font-family: 'Inter', sans-serif;
            background: var(--bg);
            color: var(--text);
            display: flex;
            align-items: center;
            justify-content: center;
            min-height: 100vh;
        }
        .card {
            background: var(--card-bg);
            border: 1px solid var(--border);
            border-radius: 16px;
            padding: 2.5rem;
            width: 100%;
            max-width: 440px;
            box-shadow: 0 10px 25px -5px rgba(0, 0, 0, 0.3);
        }
        h1 {
            font-size: 1.5rem;
            font-weight: 700;
            margin-bottom: 0.5rem;
            background: linear-gradient(to right, #818cf8, #a78bfa);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
        }
        .sub { color: var(--text-muted); font-size: 0.875rem; margin-bottom: 1.75rem; }
        label { display: block; font-size: 0.775rem; font-weight: 600; color: var(--text-muted); margin-bottom: 0.4rem; margin-top: 1.25rem; text-transform: uppercase; letter-spacing: 0.05em; }
        input {
            width: 100%;
            background: var(--bg);
            border: 1px solid var(--border);
            border-radius: 8px;
            padding: 0.75rem 1rem;
            color: var(--text);
            font-size: 0.95rem;
            outline: none;
            transition: border-color 0.2s;
        }
        input:focus { border-color: var(--primary); }
        button {
            width: 100%;
            margin-top: 2rem;
            padding: 0.85rem;
            background: var(--primary);
            color: #fff;
            border: none;
            border-radius: 8px;
            font-size: 0.95rem;
            font-weight: 600;
            cursor: pointer;
            transition: background 0.2s;
        }
        button:hover { background: var(--primary-hover); }
        .error { color: var(--error); font-size: 0.85rem; margin-top: 1rem; text-align: center; }
    </style>
</head>
<body>
    <div class="card">
        <h1>Настройка BeyVox Server</h1>
        <p class="sub">Первый запуск — создайте аккаунт администратора</p>
        <form method="POST" action="/setup">
            <label>Имя администратора</label>
            <input name="username" required autocomplete="off" placeholder="admin">
            <label>Пароль (минимум 8 символов)</label>
            <input name="password" type="password" required placeholder="••••••••">
            <label>Название сообщества по умолчанию</label>
            <input name="guild_name" placeholder="General" value="General">
            <button type="submit">Создать и продолжить</button>
            <!--ERROR-->
        </form>
    </div>
</body>
</html>"##;

const LOGIN_HTML: &str = r##"<!DOCTYPE html>
<html lang="ru">
<head>
    <meta charset="utf-8">
    <title>Вход в админ-панель BeyVox</title>
    <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap" rel="stylesheet">
    <style>
        :root {
            --bg: #090d16;
            --card-bg: #111827;
            --border: #1f2937;
            --text: #f3f4f6;
            --text-muted: #9ca3af;
            --primary: #6366f1;
            --primary-hover: #4f46e5;
            --error: #ef4444;
        }
        * { box-sizing: border-box; margin: 0; padding: 0; }
        body {
            font-family: 'Inter', sans-serif;
            background: var(--bg);
            color: var(--text);
            display: flex;
            align-items: center;
            justify-content: center;
            min-height: 100vh;
        }
        .card {
            background: var(--card-bg);
            border: 1px solid var(--border);
            border-radius: 16px;
            padding: 2.5rem;
            width: 100%;
            max-width: 400px;
            box-shadow: 0 10px 25px -5px rgba(0, 0, 0, 0.3);
        }
        h1 {
            font-size: 1.5rem;
            font-weight: 700;
            margin-bottom: 1.5rem;
            background: linear-gradient(to right, #818cf8, #a78bfa);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
        }
        label { display: block; font-size: 0.775rem; font-weight: 600; color: var(--text-muted); margin-bottom: 0.4rem; margin-top: 1.25rem; text-transform: uppercase; letter-spacing: 0.05em; }
        input {
            width: 100%;
            background: var(--bg);
            border: 1px solid var(--border);
            border-radius: 8px;
            padding: 0.75rem 1rem;
            color: var(--text);
            font-size: 0.95rem;
            outline: none;
            transition: border-color 0.2s;
        }
        input:focus { border-color: var(--primary); }
        button {
            width: 100%;
            margin-top: 2rem;
            padding: 0.85rem;
            background: var(--primary);
            color: #fff;
            border: none;
            border-radius: 8px;
            font-size: 0.95rem;
            font-weight: 600;
            cursor: pointer;
            transition: background 0.2s;
        }
        button:hover { background: var(--primary-hover); }
        .error { color: var(--error); font-size: 0.85rem; margin-top: 1rem; text-align: center; }
    </style>
</head>
<body>
    <div class="card">
        <h1>Админ-панель BeyVox</h1>
        <form method="POST" action="/admin/login">
            <label>Имя пользователя</label>
            <input name="username" required autocomplete="username" placeholder="admin">
            <label>Пароль</label>
            <input name="password" type="password" required autocomplete="current-password" placeholder="••••••••">
            <button type="submit">Войти в панель</button>
            <!--ERROR-->
        </form>
    </div>
</body>
</html>"##;

const DASHBOARD_HTML: &str = r##"<!DOCTYPE html>
<html lang="ru">
<head>
    <meta charset="utf-8">
    <title>Панель управления BeyVox</title>
    <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap" rel="stylesheet">
    <style>
        :root {
            --bg: #090b11;
            --panel-bg: #111420;
            --border: #1e2335;
            --border-hover: #2e3550;
            --text: #f1f5f9;
            --text-muted: #6b7280;
            --primary: #6366f1;
            --primary-hover: #4f46e5;
            --success: #10b981;
            --warning: #f59e0b;
            --error: #ef4444;
        }
        * { box-sizing: border-box; margin: 0; padding: 0; }
        body {
            font-family: 'Inter', sans-serif;
            background: var(--bg);
            color: var(--text);
            min-height: 100vh;
            display: flex;
            flex-direction: column;
        }
        header {
            background: var(--panel-bg);
            border-bottom: 1px solid var(--border);
            padding: 1.25rem 2rem;
            display: flex;
            align-items: center;
            justify-content: space-between;
        }
        header h1 {
            font-size: 1.25rem;
            font-weight: 700;
            background: linear-gradient(to right, #818cf8, #a78bfa);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
        }
        .header-actions { display: flex; align-items: center; gap: 1.5rem; }
        .header-actions button { background: none; border: none; cursor: pointer; color: var(--text-muted); font-size: 0.875rem; font-weight: 500; transition: color 0.2s; }
        .header-actions button:hover { color: var(--error); }

        .container {
            display: flex;
            flex: 1;
            max-width: 1200px;
            width: 100%;
            margin: 2rem auto;
            gap: 2rem;
            padding: 0 1rem;
        }
        aside {
            width: 240px;
            flex-shrink: 0;
            display: flex;
            flex-direction: column;
            gap: 0.5rem;
        }
        .nav-btn {
            display: flex;
            align-items: center;
            gap: 0.75rem;
            padding: 0.85rem 1rem;
            background: transparent;
            border: 1px solid transparent;
            color: var(--text-muted);
            border-radius: 8px;
            font-size: 0.95rem;
            font-weight: 600;
            text-align: left;
            cursor: pointer;
            transition: all 0.2s;
        }
        .nav-btn:hover { background: var(--panel-bg); color: var(--text); }
        .nav-btn.active { background: var(--panel-bg); border-color: var(--border); color: var(--primary); }

        main { flex: 1; }
        .tab-content { display: none; }
        .tab-content.active { display: block; }

        h2 { font-size: 1.25rem; font-weight: 700; margin-bottom: 1.5rem; }
        .card-list { display: flex; flex-direction: column; gap: 1rem; }
        
        .guild-card, .user-row {
            background: var(--panel-bg);
            border: 1px solid var(--border);
            border-radius: 12px;
            padding: 1.25rem 1.5rem;
            display: flex;
            align-items: center;
            justify-content: space-between;
            gap: 1.5rem;
            transition: border-color 0.2s;
        }
        .guild-card:hover, .user-row:hover { border-color: var(--border-hover); }
        .guild-card.default-guild { border-color: var(--primary); }
        .guild-info, .user-info { flex: 1; }
        .guild-name, .user-name { font-weight: 600; font-size: 1.05rem; display: flex; align-items: center; gap: 0.5rem; }
        .guild-desc { color: var(--text-muted); font-size: 0.875rem; margin-top: 0.35rem; }
        .guild-meta { color: var(--text-muted); font-size: 0.8rem; margin-top: 0.5rem; display: flex; gap: 1rem; }
        
        .badge { font-size: 0.725rem; padding: 0.2rem 0.5rem; border-radius: 4px; font-weight: 600; }
        .badge-primary { background: rgba(99, 102, 241, 0.15); color: #818cf8; }
        .badge-secondary { background: rgba(107, 114, 128, 0.15); color: #9ca3af; }
        .badge-error { background: rgba(239, 68, 68, 0.15); color: #f87171; }

        .btn { padding: 0.5rem 1rem; border: 1px solid transparent; border-radius: 6px; font-size: 0.85rem; font-weight: 600; cursor: pointer; transition: all 0.2s; }
        .btn-primary { background: var(--primary); color: #fff; }
        .btn-primary:hover { background: var(--primary-hover); }
        .btn-outline { background: transparent; border-color: var(--border); color: var(--text); }
        .btn-outline:hover { background: rgba(255,255,255,0.05); }
        .btn-danger { background: transparent; border-color: var(--error); color: var(--error); }
        .btn-danger:hover { background: var(--error); color: #fff; }

        /* Таблица / форма */
        .form-group { display: flex; flex-direction: column; gap: 0.5rem; margin-bottom: 1.5rem; }
        .form-group label { font-size: 0.85rem; color: var(--text-muted); font-weight: 600; }
        .form-group input, .form-group select { background: var(--bg); border: 1px solid var(--border); border-radius: 6px; padding: 0.75rem; color: var(--text); font-size: 0.95rem; outline: none; }
        .form-group input:focus { border-color: var(--primary); }

        .form-row { display: flex; align-items: center; gap: 1.5rem; margin-top: 1rem; }
        .form-checkbox { display: flex; align-items: center; gap: 0.5rem; cursor: pointer; }
        .form-checkbox input { width: 1.2rem; height: 1.2rem; }

        /* Модальные окна */
        .modal-bg { display: none; position: fixed; inset: 0; background: rgba(0, 0, 0, 0.85); z-index: 100; align-items: center; justify-content: center; backdrop-filter: blur(4px); }
        .modal-bg.open { display: flex; }
        .modal { background: var(--panel-bg); border: 1px solid var(--border); border-radius: 16px; padding: 2rem; width: 100%; max-width: 650px; max-height: 90vh; overflow-y: auto; position: relative; }
        .modal-large { max-width: 850px; }
        .modal h3 { font-size: 1.25rem; font-weight: 700; margin-bottom: 1.5rem; }
        .close-btn { position: absolute; top: 1.5rem; right: 1.5rem; background: none; border: none; color: var(--text-muted); font-size: 1.5rem; cursor: pointer; transition: color 0.2s; }
        .close-btn:hover { color: var(--text); }

        /* Настройка прав (Матрица чекбоксов) */
        .perms-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 0.75rem; margin: 1.5rem 0; padding: 1rem; background: var(--bg); border: 1px solid var(--border); border-radius: 8px; }
        .perm-checkbox { display: flex; align-items: center; gap: 0.75rem; cursor: pointer; font-size: 0.875rem; }
        .perm-checkbox input { width: 1.15rem; height: 1.15rem; }

        /* Мониторинг */
        .stats-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 1.5rem; }
        .stat-card { background: var(--panel-bg); border: 1px solid var(--border); border-radius: 12px; padding: 1.5rem; display: flex; flex-direction: column; gap: 0.5rem; }
        .stat-label { font-size: 0.85rem; color: var(--text-muted); font-weight: 600; text-transform: uppercase; }
        .stat-val { font-size: 2rem; font-weight: 700; color: #fff; }

        /* Всплывающие уведомления */
        .toast { position: fixed; bottom: 2rem; right: 2rem; background: var(--success); color: #000; font-weight: 600; padding: 0.85rem 1.5rem; border-radius: 8px; box-shadow: 0 10px 15px -3px rgba(0,0,0,0.5); opacity: 0; pointer-events: none; transition: opacity 0.3s; z-index: 1000; }
        .toast.err { background: var(--error); color: #fff; }
        .toast.show { opacity: 1; }

        /* Стили ролей */
        .roles-list { display: flex; flex-direction: column; gap: 0.75rem; margin-bottom: 1.5rem; }
        .role-row { background: var(--bg); border: 1px solid var(--border); border-radius: 8px; padding: 0.75rem 1rem; display: flex; align-items: center; justify-content: space-between; }
        .role-color-indicator { width: 14px; height: 14px; border-radius: 50%; display: inline-block; }
    </style>
</head>
<body>
    <header>
        <h1>Панель управления BeyVox</h1>
        <div class="header-actions">
            <form method="POST" action="/admin/logout">
                <button type="submit">Выйти</button>
            </form>
        </div>
    </header>

    <div class="container">
        <aside>
            <button class="nav-btn active" onclick="switchTab('guilds')">Сообщества</button>
            <button class="nav-btn" onclick="switchTab('users')">Пользователи</button>
            <button class="nav-btn" onclick="switchTab('settings')">Настройки сервера</button>
            <button class="nav-btn" onclick="switchTab('monitoring')">Мониторинг</button>
        </aside>

        <main>
            <!-- ТАБ: СООБЩЕСТВА -->
            <section id="tab-guilds" class="tab-content active">
                <h2>Управление сообществами</h2>
                <div class="card-list" id="guild-list"><p style="color:var(--text-muted)">Загрузка…</p></div>
            </section>

            <!-- ТАБ: ПОЛЬЗОВАТЕЛИ -->
            <section id="tab-users" class="tab-content">
                <h2>Глобальный список пользователей</h2>
                <div class="card-list" id="user-list"><p style="color:var(--text-muted)">Загрузка…</p></div>
            </section>

            <!-- ТАБ: НАСТРОЙКИ -->
            <section id="tab-settings" class="tab-content">
                <h2>Настройки сервера</h2>
                <div class="card-list" style="background:var(--panel-bg); border:1px solid var(--border); padding:2rem; border-radius:12px;">
                    <div class="form-group">
                        <label>Название инстанса (отображается в клиенте)</label>
                        <input id="setting-name" type="text" placeholder="BeyVox Server">
                    </div>
                    <div class="form-group">
                        <label class="form-checkbox">
                            <input id="setting-register" type="checkbox"> Включить регистрацию в центральном каталоге
                        </label>
                    </div>
                    <button class="btn btn-primary" onclick="saveSettings()" style="width:fit-content; padding:0.75rem 2rem;">Сохранить изменения</button>
                </div>
            </section>

            <!-- ТАБ: МОНИТОРИНГ -->
            <section id="tab-monitoring" class="tab-content">
                <h2>Мониторинг и статус системы</h2>
                <div class="stats-grid">
                    <div class="stat-card">
                        <span class="stat-label">Активные соединения (WS)</span>
                        <div class="stat-val" id="stat-ws">0</div>
                    </div>
                    <div class="stat-card">
                        <span class="stat-label">Всего гильдий</span>
                        <div class="stat-val" id="stat-guilds">0</div>
                    </div>
                    <div class="stat-card">
                        <span class="stat-label">Всего пользователей</span>
                        <div class="stat-val" id="stat-users">0</div>
                    </div>
                    <div class="stat-card">
                        <span class="stat-label">Медиа сервер (LiveKit)</span>
                        <div class="stat-val" id="stat-livekit" style="font-size:1.2rem; color:var(--success); font-weight:600; margin-top:0.5rem;">Unknown</div>
                    </div>
                </div>
            </section>
        </main>
    </div>

    <!-- МОДАЛЬНОЕ ОКНО: УЧАСТНИКИ -->
    <div class="modal-bg" id="members-modal">
        <div class="modal modal-large">
            <button class="close-btn" onclick="closeModal('members-modal')">✕</button>
            <h3 id="members-modal-title">Участники сообщества</h3>
            <div class="card-list" id="member-list"></div>
        </div>
    </div>

    <!-- МОДАЛЬНОЕ ОКНО: УПРАВЛЕНИЕ РОЛЯМИ -->
    <div class="modal-bg" id="roles-modal">
        <div class="modal modal-large">
            <button class="close-btn" onclick="closeModal('roles-modal')">✕</button>
            <h3 id="roles-modal-title">Управление ролями</h3>
            
            <div style="margin-bottom: 1.5rem; display:flex; gap:1rem;">
                <input id="new-role-name" type="text" placeholder="Название роли" style="flex:1;">
                <input id="new-role-color" type="color" value="#ffffff" style="width:45px; height:42px; padding:0; cursor:pointer;">
                <button class="btn btn-primary" onclick="createRole()">Добавить роль</button>
            </div>

            <div class="roles-list" id="roles-list"></div>
        </div>
    </div>

    <!-- МОДАЛЬНОЕ ОКНО: РЕДАКТИРОВАНИЕ ПРАВ РОЛИ -->
    <div class="modal-bg" id="role-edit-modal">
        <div class="modal">
            <button class="close-btn" onclick="closeModal('role-edit-modal')">✕</button>
            <h3 id="role-edit-title">Настройка прав роли</h3>
            
            <div class="form-group">
                <label>Название роли</label>
                <input id="edit-role-name" type="text">
            </div>
            
            <div class="form-group" id="edit-role-color-container">
                <label>Цвет роли</label>
                <input id="edit-role-color" type="color" style="width:100%; height:40px; padding:0; cursor:pointer;">
            </div>

            <div class="form-group">
                <label>Позиция (приоритет в списке)</label>
                <input id="edit-role-position" type="number" min="0">
            </div>

            <h4 style="margin-top:1.5rem; font-size:0.9rem; text-transform:uppercase; color:var(--text-muted); font-weight:600;">Матрица прав доступа:</h4>
            
            <div class="perms-grid">
                <label class="perm-checkbox"><input type="checkbox" id="perm-admin"> ADMINISTRATOR</label>
                <label class="perm-checkbox"><input type="checkbox" id="perm-channels"> MANAGE_CHANNELS</label>
                <label class="perm-checkbox"><input type="checkbox" id="perm-roles"> MANAGE_ROLES</label>
                <label class="perm-checkbox"><input type="checkbox" id="perm-members"> MANAGE_MEMBERS</label>
                <label class="perm-checkbox"><input type="checkbox" id="perm-messages"> SEND_MESSAGES</label>
                <label class="perm-checkbox"><input type="checkbox" id="perm-files"> ATTACH_FILES</label>
                <label class="perm-checkbox"><input type="checkbox" id="perm-voice"> CONNECT_VOICE</label>
                <label class="perm-checkbox"><input type="checkbox" id="perm-stream"> STREAM_SCREEN</label>
                <label class="perm-checkbox"><input type="checkbox" id="perm-mute"> MUTE_MEMBERS</label>
                <label class="perm-checkbox"><input type="checkbox" id="perm-ban"> BAN_MEMBERS</label>
            </div>

            <button class="btn btn-primary" onclick="saveRoleEdit()" style="width:100%;">Сохранить</button>
        </div>
    </div>

    <div class="toast" id="toast"></div>

    <script>
        let currentGuildId = null;
        let currentRoleId = null;
        let currentMembersList = [];
        let currentRolesList = [];

        // Константы битов прав
        const PERMS = {
            ADMINISTRATOR: 1 << 0,
            MANAGE_CHANNELS: 1 << 1,
            MANAGE_ROLES: 1 << 2,
            MANAGE_MEMBERS: 1 << 3,
            SEND_MESSAGES: 1 << 4,
            ATTACH_FILES: 1 << 5,
            CONNECT_VOICE: 1 << 6,
            STREAM_SCREEN: 1 << 7,
            MUTE_MEMBERS: 1 << 8,
            BAN_MEMBERS: 1 << 9
        };

        function switchTab(tab) {
            document.querySelectorAll('.nav-btn').forEach(btn => btn.classList.remove('active'));
            document.querySelectorAll('.tab-content').forEach(c => c.classList.remove('active'));
            
            // Находим кнопку
            const btn = Array.from(document.querySelectorAll('.nav-btn')).find(b => b.onclick.toString().includes(tab));
            if (btn) btn.classList.add('active');
            
            document.getElementById('tab-' + tab).classList.add('active');
            
            if (tab === 'guilds') loadGuilds();
            if (tab === 'users') loadUsers();
            if (tab === 'settings') loadSettings();
            if (tab === 'monitoring') loadStats();
        }

        async function loadGuilds() {
            const res = await fetch('/admin/api/guilds');
            const guilds = await res.json();
            const el = document.getElementById('guild-list');
            if (!guilds.length) { el.innerHTML = '<p style="color:var(--text-muted)">Нет сообществ.</p>'; return; }
            el.innerHTML = guilds.map(g => `
                <div class="guild-card ${g.is_default ? 'default-guild' : ''}" id="guild-${g.id}">
                    <div class="guild-info">
                        <div class="guild-name">
                            ${esc(g.name)}
                            ${g.is_default ? '<span class="badge badge-primary">По умолчанию</span>' : ''}
                            ${!g.is_public ? '<span class="badge badge-secondary">Приватный</span>' : ''}
                        </div>
                        ${g.description ? `<div class="guild-desc">${esc(g.description)}</div>` : ''}
                        <div class="guild-meta">
                            <span>Участников: <strong>${g.member_count}</strong></span>
                        </div>
                    </div>
                    <div style="display:flex; gap:0.5rem;">
                        ${!g.is_default ? `<button class="btn btn-outline" onclick="setDefault('${g.id}')">Дефолтный</button>` : ''}
                        <button class="btn btn-outline" onclick="openRoles('${g.id}', '${esc(g.name)}')">Роли</button>
                        <button class="btn btn-outline" onclick="openMembers('${g.id}', '${esc(g.name)}')">Участники</button>
                        ${!g.is_default ? `<button class="btn btn-danger" onclick="deleteGuild('${g.id}', '${esc(g.name)}')">Удалить</button>` : ''}
                    </div>
                </div>
            `).join('');
        }

        async function setDefault(id) {
            const res = await fetch(`/admin/api/guilds/${id}/set-default`, { method: 'POST' });
            if (res.ok) { showToast('Сообщество по умолчанию успешно обновлено'); loadGuilds(); }
            else showToast('Ошибка при установке гильдии по умолчанию', true);
        }

        async function deleteGuild(id, name) {
            if (!confirm(`Удалить сообщество "${name}"? Это действие невозможно отменить!`)) return;
            const res = await fetch(`/admin/api/guilds/${id}`, { method: 'DELETE' });
            const data = await res.json();
            if (res.ok) { showToast('Сообщество удалено'); loadGuilds(); }
            else showToast(data.error || 'Ошибка при удалении', true);
        }

        // ── УЧАСТНИКИ ──

        async function openMembers(guildId, guildName) {
            currentGuildId = guildId;
            document.getElementById('members-modal-title').textContent = `Участники — ${guildName}`;
            document.getElementById('member-list').innerHTML = '<p style="color:var(--text-muted)">Загрузка…</p>';
            document.getElementById('members-modal').classList.add('open');
            await loadMembers();
        }

        async function loadMembers() {
            // Запрашиваем сначала роли, чтобы показывать чекбоксы
            const rolesRes = await fetch(`/admin/api/guilds/${currentGuildId}/roles`);
            currentRolesList = await rolesRes.json();

            const res = await fetch(`/admin/api/guilds/${currentGuildId}/members`);
            const members = await res.json();
            currentMembersList = members;
            const el = document.getElementById('member-list');
            if (!members.length) { el.innerHTML = '<p style="color:var(--text-muted)">Участников нет.</p>'; return; }
            el.innerHTML = members.map(m => {
                // Строим селектор ролей для каждого участника
                const rolesCheckboxes = currentRolesList.map(r => {
                    const hasRole = m.roles.includes(r.id);
                    return `
                        <label class="form-checkbox" style="font-size:0.75rem; margin-right:1rem; display:inline-flex;">
                            <input type="checkbox" ${hasRole ? 'checked' : ''} onchange="toggleMemberRole('${m.user_id}', '${r.id}', this.checked)">
                            <span style="color:${r.color || '#fff'}">${esc(r.name)}</span>
                        </label>
                    `;
                }).join('');

                return `
                    <div class="user-row" id="member-${m.user_id}">
                        <div class="user-info">
                            <div class="user-name">
                                ${esc(m.nickname || m.username)}
                                ${m.is_muted ? '<span class="badge badge-secondary">Muted</span>' : ''}
                                ${m.is_banned ? '<span class="badge badge-error">Забанен</span>' : ''}
                            </div>
                            <div style="font-size:0.75rem; color:var(--text-muted); margin-top:0.25rem;">
                                @${esc(m.username)} · Зашел: ${new Date(m.joined_at).toLocaleDateString()}
                            </div>
                            <div style="margin-top:0.75rem; border-top:1px solid var(--border); padding-top:0.5rem;">
                                ${rolesCheckboxes}
                            </div>
                        </div>
                        <div style="display:flex; gap:0.5rem;">
                            <button class="btn btn-outline" onclick="kickMember('${m.user_id}')">Кикнуть</button>
                            <button class="btn btn-danger" onclick="banMember('${m.user_id}')">Забанить</button>
                        </div>
                    </div>
                `;
            }).join('');
        }

        async function toggleMemberRole(userId, roleId, checked) {
            const method = checked ? 'PUT' : 'DELETE';
            const res = await fetch(`/admin/api/guilds/${currentGuildId}/members/${userId}/roles/${roleId}`, { method });
            if (res.ok) {
                showToast(checked ? 'Роль добавлена участнику' : 'Роль снята с участника');
            } else {
                showToast('Не удалось обновить роль участника', true);
            }
        }

        async function kickMember(userId) {
            if (!confirm('Выгнать этого участника из гильдии?')) return;
            const res = await fetch(`/admin/api/guilds/${currentGuildId}/members/${userId}/kick`, { method: 'POST' });
            if (res.ok) { showToast('Участник кикнут'); loadMembers(); }
            else showToast('Ошибка при выполнении операции', true);
        }

        async function banMember(userId) {
            if (!confirm('Забанить этого участника в этой гильдии?')) return;
            const res = await fetch(`/admin/api/guilds/${currentGuildId}/members/${userId}/ban`, { method: 'POST' });
            if (res.ok) { showToast('Участник забанен'); loadMembers(); }
            else showToast('Ошибка при выполнении операции', true);
        }

        // ── УПРАВЛЕНИЕ РОЛЯМИ ──

        async function openRoles(guildId, guildName) {
            currentGuildId = guildId;
            document.getElementById('roles-modal-title').textContent = `Роли сообщества — ${guildName}`;
            document.getElementById('roles-list').innerHTML = '<p style="color:var(--text-muted)">Загрузка…</p>';
            document.getElementById('roles-modal').classList.add('open');
            await loadRoles();
        }

        async function loadRoles() {
            const res = await fetch(`/admin/api/guilds/${currentGuildId}/roles`);
            currentRolesList = await res.json();
            const el = document.getElementById('roles-list');
            if (!currentRolesList.length) { el.innerHTML = '<p style="color:var(--text-muted)">Нет ролей.</p>'; return; }
            el.innerHTML = currentRolesList.map(r => `
                <div class="role-row">
                    <div style="display:flex; align-items:center; gap:0.75rem;">
                        <span class="role-color-indicator" style="background:${r.color || '#9ca3af'}"></span>
                        <strong style="color:${r.color || '#fff'}">${esc(r.name)}</strong>
                        <span style="font-size:0.75rem; color:var(--text-muted)">Позиция: ${r.position}</span>
                    </div>
                    <div style="display:flex; gap:0.5rem;">
                        <button class="btn btn-outline" onclick="editRolePerms('${r.id}')">Настроить</button>
                        ${r.name !== '@everyone' ? `<button class="btn btn-danger" onclick="deleteRole('${r.id}')">✕</button>` : ''}
                    </div>
                </div>
            `).join('');
        }

        async function createRole() {
            const name = document.getElementById('new-role-name').value.trim();
            const color = document.getElementById('new-role-color').value;
            if (!name) { showToast('Введите название роли', true); return; }

            const res = await fetch(`/admin/api/guilds/${currentGuildId}/roles`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ name, color, permissions: 16 | 32 | 64 }) // default text + attach + voice
            });

            if (res.ok) {
                showToast('Роль успешно создана');
                document.getElementById('new-role-name').value = '';
                loadRoles();
            } else {
                showToast('Ошибка создания роли', true);
            }
        }

        async function deleteRole(roleId) {
            if (!confirm('Удалить эту роль? Все пользователи её потеряют.')) return;
            const res = await fetch(`/admin/api/guilds/${currentGuildId}/roles/${roleId}`, { method: 'DELETE' });
            if (res.ok) { showToast('Роль удалена'); loadRoles(); }
            else showToast('Ошибка при удалении роли', true);
        }

        function editRolePerms(roleId) {
            const role = currentRolesList.find(r => r.id === roleId);
            if (!role) return;

            currentRoleId = roleId;
            document.getElementById('edit-role-name').value = role.name;
            document.getElementById('edit-role-color').value = role.color || '#ffffff';
            document.getElementById('edit-role-position').value = role.position;

            // Запрещаем менять имя и позицию у @everyone
            if (role.name === '@everyone') {
                document.getElementById('edit-role-name').disabled = true;
                document.getElementById('edit-role-position').disabled = true;
                document.getElementById('edit-role-color-container').style.display = 'none';
            } else {
                document.getElementById('edit-role-name').disabled = false;
                document.getElementById('edit-role-position').disabled = false;
                document.getElementById('edit-role-color-container').style.display = 'block';
            }

            // Заполняем чекбоксы на основе битовой маски
            const mask = role.permissions;
            document.getElementById('perm-admin').checked = (mask & PERMS.ADMINISTRATOR) !== 0;
            document.getElementById('perm-channels').checked = (mask & PERMS.MANAGE_CHANNELS) !== 0;
            document.getElementById('perm-roles').checked = (mask & PERMS.MANAGE_ROLES) !== 0;
            document.getElementById('perm-members').checked = (mask & PERMS.MANAGE_MEMBERS) !== 0;
            document.getElementById('perm-messages').checked = (mask & PERMS.SEND_MESSAGES) !== 0;
            document.getElementById('perm-files').checked = (mask & PERMS.ATTACH_FILES) !== 0;
            document.getElementById('perm-voice').checked = (mask & PERMS.CONNECT_VOICE) !== 0;
            document.getElementById('perm-stream').checked = (mask & PERMS.STREAM_SCREEN) !== 0;
            document.getElementById('perm-mute').checked = (mask & PERMS.MUTE_MEMBERS) !== 0;
            document.getElementById('perm-ban').checked = (mask & PERMS.BAN_MEMBERS) !== 0;

            document.getElementById('role-edit-modal').classList.add('open');
        }

        async function saveRoleEdit() {
            const name = document.getElementById('edit-role-name').value.trim();
            const color = document.getElementById('edit-role-color').value;
            const position = parseInt(document.getElementById('edit-role-position').value) || 0;

            // Собираем битовую маску из чекбоксов
            let permissions = 0;
            if (document.getElementById('perm-admin').checked) permissions |= PERMS.ADMINISTRATOR;
            if (document.getElementById('perm-channels').checked) permissions |= PERMS.MANAGE_CHANNELS;
            if (document.getElementById('perm-roles').checked) permissions |= PERMS.MANAGE_ROLES;
            if (document.getElementById('perm-members').checked) permissions |= PERMS.MANAGE_MEMBERS;
            if (document.getElementById('perm-messages').checked) permissions |= PERMS.SEND_MESSAGES;
            if (document.getElementById('perm-files').checked) permissions |= PERMS.ATTACH_FILES;
            if (document.getElementById('perm-voice').checked) permissions |= PERMS.CONNECT_VOICE;
            if (document.getElementById('perm-stream').checked) permissions |= PERMS.STREAM_SCREEN;
            if (document.getElementById('perm-mute').checked) permissions |= PERMS.MUTE_MEMBERS;
            if (document.getElementById('perm-ban').checked) permissions |= PERMS.BAN_MEMBERS;

            const res = await fetch(`/admin/api/guilds/${currentGuildId}/roles/${currentRoleId}`, {
                method: 'PATCH',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ name, color, permissions, position })
            });

            if (res.ok) {
                showToast('Настройки роли сохранены');
                closeModal('role-edit-modal');
                loadRoles();
            } else {
                showToast('Не удалось обновить роль', true);
            }
        }

        // ── ГЛОБАЛЬНЫЕ ПОЛЬЗОВАТЕЛИ ──

        async function loadUsers() {
            const res = await fetch('/admin/api/users');
            const users = await res.json();
            const el = document.getElementById('user-list');
            if (!users.length) { el.innerHTML = '<p style="color:var(--text-muted)">Нет зарегистрированных пользователей.</p>'; return; }
            el.innerHTML = users.map(u => `
                <div class="user-row" id="user-${u.user_id}">
                    <div class="user-info">
                        <div class="user-name">
                            @${esc(u.username)}
                            ${u.is_banned ? '<span class="badge badge-error">Глобально забанен</span>' : ''}
                        </div>
                        <div style="font-size:0.75rem; color:var(--text-muted)">ID: ${u.user_id}</div>
                    </div>
                    <div>
                        ${u.is_banned 
                            ? `<button class="btn btn-outline" onclick="toggleGlobalBan('${u.user_id}', false)">Разбанить</button>` 
                            : `<button class="btn btn-danger" onclick="toggleGlobalBan('${u.user_id}', true)">Глобальный бан</button>`
                        }
                    </div>
                </div>
            `).join('');
        }

        async function toggleGlobalBan(userId, ban) {
            const act = ban ? 'забанить этого пользователя глобально? Он потеряет доступ ко всем серверам.' : 'разбанить пользователя?';
            if (!confirm(`Вы действительно хотите ${act}`)) return;

            const method = 'POST';
            const url = ban ? `/admin/api/users/${userId}/ban` : `/admin/api/users/${userId}/unban`;
            
            const res = await fetch(url, { method });
            if (res.ok) {
                showToast(ban ? 'Пользователь забанен глобально' : 'Глобальный бан снят');
                loadUsers();
            } else {
                showToast('Ошибка при выполнении операции', true);
            }
        }

        // ── НАСТРОЙКИ СЕРВЕРА ──

        async function loadSettings() {
            const res = await fetch('/admin/api/settings');
            const data = await res.json();
            document.getElementById('setting-name').value = data.server_name;
            document.getElementById('setting-register').checked = data.catalog_register;
        }

        async function saveSettings() {
            const server_name = document.getElementById('setting-name').value.trim();
            const catalog_register = document.getElementById('setting-register').checked;
            if (!server_name) { showToast('Имя сервера не может быть пустым', true); return; }

            const res = await fetch('/admin/api/settings', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ server_name, catalog_register })
            });

            if (res.ok) {
                showToast('Настройки сервера сохранены');
            } else {
                showToast('Не удалось сохранить настройки', true);
            }
        }

        // ── МОНИТОРИНГ ──

        async function loadStats() {
            const res = await fetch('/admin/api/stats');
            const data = await res.json();
            document.getElementById('stat-ws').textContent = data.active_connections;
            document.getElementById('stat-guilds').textContent = data.total_guilds;
            document.getElementById('stat-users').textContent = data.total_users;
            
            const lkEl = document.getElementById('stat-livekit');
            lkEl.textContent = data.livekit_status;
            if (data.livekit_status === 'Connected') {
                lkEl.style.color = 'var(--success)';
            } else {
                lkEl.style.color = 'var(--error)';
            }
        }

        // ── ВСПОМОГАТЕЛЬНЫЕ ──

        function closeModal(id) {
            document.getElementById(id).classList.remove('open');
        }

        function esc(s) {
            return String(s).replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;').replace(/"/g,'&quot;');
        }

        function showToast(msg, isErr) {
            const el = document.getElementById('toast');
            el.textContent = msg;
            el.className = 'toast' + (isErr ? ' err' : '') + ' show';
            setTimeout(() => el.classList.remove('show'), 2500);
        }

        // Инициализация
        loadGuilds();
    </script>
</body>
</html>"##;
