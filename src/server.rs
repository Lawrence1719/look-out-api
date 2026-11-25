use crate::store::Store;
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::Html,
    routing::{get, post},
    Router,
};

#[derive(Clone)]
struct ServerState {
    store: Store,
    token: Option<String>,
}

pub fn router(store: Store, token: Option<String>) -> Router {
    Router::new()
        .route("/", get(status))
        .route("/healthz", get(|| async { "ok" }))
        .route("/heartbeat/{name}", post(heartbeat))
        .with_state(ServerState { store, token })
}
async fn heartbeat(
    State(state): State<ServerState>,
    Path(name): Path<String>,
    headers: HeaderMap,
) -> StatusCode {
    let Some(token) = state.token else {
        return StatusCode::NOT_FOUND;
    };
    let authorized = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .is_some_and(|provided| provided.as_bytes() == token.as_bytes());
    if !authorized {
        return StatusCode::UNAUTHORIZED;
    }
    match state.store.ping(&name).await {
        Ok(true) => StatusCode::ACCEPTED,
        Ok(false) => StatusCode::NOT_FOUND,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
async fn status(State(state): State<ServerState>) -> Result<Html<String>, StatusCode> {
    let statuses = state
        .store
        .statuses()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut rows = String::new();
    for target in statuses {
        let stale = target
            .last_checked_at
            .as_deref()
            .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
            .is_none_or(|time| {
                (chrono::Utc::now() - time.with_timezone(&chrono::Utc)).num_seconds()
                    > target.interval_secs * 2
            });
        let label = if stale {
            "unknown / stale"
        } else {
            &target.status
        };
        rows.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td></tr>",
            escape(&target.name),
            escape(&target.r#type),
            escape(label)
        ));
    }
    Ok(Html(format!("<!doctype html><html lang=\"en\"><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>Look Out API status</title><style>body{{font:16px system-ui;background:#f7f7f2;color:#232721;margin:3rem auto;padding:0 1rem;max-width:60rem}}table{{width:100%;border-collapse:collapse}}td,th{{text-align:left;padding:1rem;border-bottom:1px solid #ccd0c5}}a{{color:#b33f1e}}</style><body><h1>Look Out API</h1><p>Local monitoring status. Stale observations are shown as unknown.</p><table><thead><tr><th>Target</th><th>Check</th><th>Observed status</th></tr></thead><tbody>{rows}</tbody></table></body></html>")))
}
fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
