use serde::Deserialize;
use sqlx::Row;
use uuid::Uuid;

use crate::AppState;

#[derive(Deserialize)]
struct DiscoveryResponse {
    online_count: Option<i32>,
    total_members: Option<i64>,
}

pub fn spawn(state: AppState) {
    tokio::spawn(async move {
        // Первый pull через 30 секунд после старта (серверы могут ещё не подняться)
        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        loop {
            pull_all(&state).await;
            tokio::time::sleep(tokio::time::Duration::from_secs(5 * 60)).await;
        }
    });
}

async fn pull_all(state: &AppState) {
    let servers = sqlx::query("SELECT id, address FROM servers WHERE is_public = true")
        .fetch_all(&state.db)
        .await;

    let servers = match servers {
        Ok(rows) => rows,
        Err(e) => {
            tracing::error!("puller: fetch servers: {e}");
            return;
        }
    };

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap_or_default();

    for row in &servers {
        let server_id: Uuid = row.get("id");
        let address: String = row.get("address");

        let url = normalize_address(&address);
        pull_server(state, &client, server_id, &url).await;
    }
}

async fn pull_server(
    state: &AppState,
    client: &reqwest::Client,
    server_id: Uuid,
    base_url: &str,
) {
    let url = format!("{base_url}/api/discovery");
    let resp = match client.get(&url).send().await {
        Ok(r) => r,
        Err(e) => {
            tracing::debug!("puller: {url}: {e}");
            return;
        }
    };

    let discovery: DiscoveryResponse = match resp.json().await {
        Ok(d) => d,
        Err(e) => {
            tracing::debug!("puller: parse {url}: {e}");
            return;
        }
    };

    let online = discovery.online_count.unwrap_or(0);
    let total_members = discovery.total_members.unwrap_or(0) as i32;

    let _ = sqlx::query(
        "UPDATE servers SET last_ping = NOW(), online_count = $1, total_members = $2 WHERE id = $3",
    )
    .bind(online)
    .bind(total_members)
    .bind(server_id)
    .execute(&state.db)
    .await;

    tracing::debug!("puller: {} → {} members, {} online", base_url, total_members, online);
}

fn normalize_address(address: &str) -> String {
    if address.starts_with("http://") || address.starts_with("https://") {
        address.trim_end_matches('/').to_owned()
    } else {
        format!("https://{}", address.trim_end_matches('/'))
    }
}
