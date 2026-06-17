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
        "SELECT user_id, username, nickname, joined_at, is_muted, is_banned
         FROM members WHERE guild_id = $1 ORDER BY joined_at",
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
        Ok(_) => Json(serde_json::json!({ "ok": true })).into_response(),
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
        Ok(_) => Json(serde_json::json!({ "ok": true })).into_response(),
        Err(e) => {
            tracing::error!("{e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

// ── Discovery (публичный endpoint для central pull) ─────────────────────────

#[derive(Serialize)]
pub struct DiscoveryResponse {
    pub name: String,
    pub version: &'static str,
    pub online_count: usize,
    pub total_members: i64,
    pub default_guild: Option<GuildSummary>,
}

#[derive(Serialize)]
pub struct GuildSummary {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub member_count: i64,
}

pub async fn discovery(State(state): State<AppState>) -> AppResult<Json<DiscoveryResponse>> {
    // Суммарное число участников по всем гильдиям
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

    // Только дефолтная гильдия — точка входа для новых участников
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

    let default_guild = default_row.map(|r| GuildSummary {
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

const SETUP_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head><meta charset="utf-8"><title>BeyVox Setup</title>
<style>
*{box-sizing:border-box;margin:0;padding:0}
body{font-family:system-ui,sans-serif;background:#0f0f17;color:#e0e0e0;display:flex;align-items:center;justify-content:center;min-height:100vh}
.card{background:#1a1a2e;border:1px solid #2a2a45;border-radius:12px;padding:2rem;width:100%;max-width:420px}
h1{font-size:1.4rem;margin-bottom:.25rem}
.sub{color:#888;font-size:.85rem;margin-bottom:1.5rem}
label{display:block;font-size:.85rem;color:#aaa;margin-bottom:.3rem;margin-top:1rem}
input{width:100%;background:#0f0f17;border:1px solid #2a2a45;border-radius:8px;padding:.65rem .9rem;color:#e0e0e0;font-size:.95rem;outline:none}
input:focus{border-color:#5865f2}
button{width:100%;margin-top:1.5rem;padding:.75rem;background:#5865f2;color:#fff;border:none;border-radius:8px;font-size:.95rem;cursor:pointer}
button:hover{background:#4752c4}
.error{color:#f04747;font-size:.85rem;margin-top:.75rem}
</style></head>
<body><div class="card">
<h1>BeyVox Server Setup</h1>
<p class="sub">First launch — create admin account</p>
<form method="POST" action="/setup">
  <label>Admin username</label>
  <input name="username" required autocomplete="off">
  <label>Password (min 8 chars)</label>
  <input name="password" type="password" required>
  <label>Default guild name</label>
  <input name="guild_name" placeholder="General" value="General">
  <button type="submit">Create &amp; Continue</button>
  <!--ERROR-->
</form>
</div></body></html>"#;

const LOGIN_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head><meta charset="utf-8"><title>BeyVox Admin</title>
<style>
*{box-sizing:border-box;margin:0;padding:0}
body{font-family:system-ui,sans-serif;background:#0f0f17;color:#e0e0e0;display:flex;align-items:center;justify-content:center;min-height:100vh}
.card{background:#1a1a2e;border:1px solid #2a2a45;border-radius:12px;padding:2rem;width:100%;max-width:380px}
h1{font-size:1.4rem;margin-bottom:1.5rem}
label{display:block;font-size:.85rem;color:#aaa;margin-bottom:.3rem;margin-top:1rem}
input{width:100%;background:#0f0f17;border:1px solid #2a2a45;border-radius:8px;padding:.65rem .9rem;color:#e0e0e0;font-size:.95rem;outline:none}
input:focus{border-color:#5865f2}
button{width:100%;margin-top:1.5rem;padding:.75rem;background:#5865f2;color:#fff;border:none;border-radius:8px;font-size:.95rem;cursor:pointer}
button:hover{background:#4752c4}
.error{color:#f04747;font-size:.85rem;margin-top:.75rem}
</style></head>
<body><div class="card">
<h1>Admin Panel</h1>
<form method="POST" action="/admin/login">
  <label>Username</label>
  <input name="username" required autocomplete="username">
  <label>Password</label>
  <input name="password" type="password" required autocomplete="current-password">
  <button type="submit">Sign in</button>
  <!--ERROR-->
</form>
</div></body></html>"#;

const DASHBOARD_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head><meta charset="utf-8"><title>BeyVox Admin</title>
<style>
*{box-sizing:border-box;margin:0;padding:0}
body{font-family:system-ui,sans-serif;background:#0f0f17;color:#e0e0e0;min-height:100vh}
header{background:#1a1a2e;border-bottom:1px solid #2a2a45;padding:1rem 2rem;display:flex;align-items:center;justify-content:space-between}
header h1{font-size:1.1rem}
header a{color:#888;font-size:.85rem;text-decoration:none}
header a:hover{color:#e0e0e0}
main{padding:2rem;max-width:1000px;margin:0 auto}
h2{font-size:1rem;color:#aaa;margin-bottom:1rem;text-transform:uppercase;letter-spacing:.05em}
.guild-list{display:flex;flex-direction:column;gap:.5rem}
.guild{background:#1a1a2e;border:1px solid #2a2a45;border-radius:10px;padding:1rem 1.25rem;display:flex;align-items:center;gap:1rem}
.guild.default-guild{border-color:#5865f2}
.guild-info{flex:1}
.guild-name{font-weight:600;display:flex;align-items:center;gap:.5rem}
.badge{font-size:.7rem;background:#5865f2;color:#fff;padding:.15rem .45rem;border-radius:4px}
.guild-desc{color:#888;font-size:.82rem;margin-top:.2rem}
.guild-meta{font-size:.8rem;color:#666;margin-top:.3rem}
.actions{display:flex;gap:.5rem;flex-shrink:0}
button{padding:.4rem .9rem;border:none;border-radius:6px;font-size:.82rem;cursor:pointer}
.btn-default{background:#5865f2;color:#fff}
.btn-default:hover{background:#4752c4}
.btn-members{background:#2a2a45;color:#e0e0e0}
.btn-members:hover{background:#3a3a5a}
.btn-delete{background:transparent;border:1px solid #f04747;color:#f04747}
.btn-delete:hover{background:#f04747;color:#fff}
.modal-bg{display:none;position:fixed;inset:0;background:rgba(0,0,0,.7);z-index:10;align-items:center;justify-content:center}
.modal-bg.open{display:flex}
.modal{background:#1a1a2e;border:1px solid #2a2a45;border-radius:12px;padding:1.5rem;width:100%;max-width:600px;max-height:80vh;overflow-y:auto}
.modal h3{margin-bottom:1rem}
.member-list{display:flex;flex-direction:column;gap:.4rem}
.member{background:#0f0f17;border-radius:6px;padding:.6rem .9rem;display:flex;align-items:center;gap:.75rem}
.member-name{flex:1;font-size:.9rem}
.member-meta{font-size:.75rem;color:#666}
.btn-sm{padding:.3rem .6rem;font-size:.75rem}
.btn-kick{background:transparent;border:1px solid #faa61a;color:#faa61a}
.btn-kick:hover{background:#faa61a;color:#000}
.btn-ban{background:transparent;border:1px solid #f04747;color:#f04747}
.btn-ban:hover{background:#f04747;color:#fff}
.close-btn{float:right;background:none;border:none;color:#888;font-size:1.2rem;cursor:pointer;padding:0}
.toast{position:fixed;bottom:1.5rem;right:1.5rem;background:#23d160;color:#000;padding:.6rem 1.2rem;border-radius:8px;font-size:.85rem;opacity:0;transition:opacity .3s;pointer-events:none}
.toast.err{background:#f04747;color:#fff}
.toast.show{opacity:1}
</style></head>
<body>
<header>
  <h1>BeyVox Admin Panel</h1>
  <form method="POST" action="/admin/logout" style="display:inline">
    <button type="submit" style="background:none;border:none;cursor:pointer;color:#888;font-size:.85rem">Sign out</button>
  </form>
</header>
<main>
  <h2>Guilds</h2>
  <div class="guild-list" id="guild-list"><p style="color:#666">Loading…</p></div>
</main>

<div class="modal-bg" id="members-modal">
  <div class="modal">
    <button class="close-btn" onclick="closeModal()">✕</button>
    <h3 id="modal-title">Members</h3>
    <div class="member-list" id="member-list"></div>
  </div>
</div>

<div class="toast" id="toast"></div>

<script>
let currentGuildId = null;

async function loadGuilds() {
  const res = await fetch('/admin/api/guilds');
  const guilds = await res.json();
  const el = document.getElementById('guild-list');
  if (!guilds.length) { el.innerHTML = '<p style="color:#666">No guilds yet.</p>'; return; }
  el.innerHTML = guilds.map(g => `
    <div class="guild ${g.is_default ? 'default-guild' : ''}" id="guild-${g.id}">
      <div class="guild-info">
        <div class="guild-name">
          ${esc(g.name)}
          ${g.is_default ? '<span class="badge">Default</span>' : ''}
          ${!g.is_public ? '<span class="badge" style="background:#888">Private</span>' : ''}
        </div>
        ${g.description ? `<div class="guild-desc">${esc(g.description)}</div>` : ''}
        <div class="guild-meta">${g.member_count} member${g.member_count !== 1 ? 's' : ''}</div>
      </div>
      <div class="actions">
        ${!g.is_default ? `<button class="btn-default" onclick="setDefault('${g.id}')">Set default</button>` : ''}
        <button class="btn-members" onclick="openMembers('${g.id}', '${esc(g.name)}')">Members</button>
        ${!g.is_default ? `<button class="btn-delete" onclick="deleteGuild('${g.id}', '${esc(g.name)}')">Delete</button>` : ''}
      </div>
    </div>
  `).join('');
}

async function setDefault(id) {
  const res = await fetch(`/admin/api/guilds/${id}/set-default`, { method: 'POST' });
  if (res.ok) { toast('Default guild updated'); loadGuilds(); }
  else toast('Error', true);
}

async function deleteGuild(id, name) {
  if (!confirm(`Delete guild "${name}"? This cannot be undone.`)) return;
  const res = await fetch(`/admin/api/guilds/${id}`, { method: 'DELETE' });
  const data = await res.json();
  if (res.ok) { toast('Guild deleted'); loadGuilds(); }
  else toast(data.error || 'Error', true);
}

async function openMembers(guildId, guildName) {
  currentGuildId = guildId;
  document.getElementById('modal-title').textContent = `Members — ${guildName}`;
  document.getElementById('member-list').innerHTML = '<p style="color:#666">Loading…</p>';
  document.getElementById('members-modal').classList.add('open');
  await loadMembers(guildId);
}

async function loadMembers(guildId) {
  const res = await fetch(`/admin/api/guilds/${guildId}/members`);
  const members = await res.json();
  const el = document.getElementById('member-list');
  if (!members.length) { el.innerHTML = '<p style="color:#666">No members.</p>'; return; }
  el.innerHTML = members.map(m => `
    <div class="member" id="member-${m.user_id}">
      <div>
        <div class="member-name">${esc(m.nickname || m.username)}</div>
        <div class="member-meta">@${esc(m.username)} · joined ${new Date(m.joined_at).toLocaleDateString()}
          ${m.is_muted ? ' · muted' : ''}${m.is_banned ? ' · <span style="color:#f04747">banned</span>' : ''}</div>
      </div>
      <div class="actions">
        <button class="btn-sm btn-kick" onclick="kickMember('${currentGuildId}','${m.user_id}')">Kick</button>
        <button class="btn-sm btn-ban" onclick="banMember('${currentGuildId}','${m.user_id}')">Ban</button>
      </div>
    </div>
  `).join('');
}

async function kickMember(guildId, userId) {
  if (!confirm('Kick this member?')) return;
  const res = await fetch(`/admin/api/guilds/${guildId}/members/${userId}/kick`, { method: 'POST' });
  if (res.ok) { toast('Member kicked'); loadMembers(guildId); }
  else toast('Error', true);
}

async function banMember(guildId, userId) {
  if (!confirm('Ban this member?')) return;
  const res = await fetch(`/admin/api/guilds/${guildId}/members/${userId}/ban`, { method: 'POST' });
  if (res.ok) { toast('Member banned'); loadMembers(guildId); }
  else toast('Error', true);
}

function closeModal() {
  document.getElementById('members-modal').classList.remove('open');
  currentGuildId = null;
}

function esc(s) {
  return String(s).replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;').replace(/"/g,'&quot;');
}

function toast(msg, err) {
  const el = document.getElementById('toast');
  el.textContent = msg;
  el.className = 'toast' + (err ? ' err' : '') + ' show';
  setTimeout(() => el.classList.remove('show'), 2500);
}

document.getElementById('members-modal').addEventListener('click', e => {
  if (e.target === e.currentTarget) closeModal();
});

loadGuilds();
</script>
</body></html>"#;
