use crate::{
    dashboard::{Dashboard, RunError},
    store::Store,
};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    middleware,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};

#[derive(Clone)]
struct ServerState {
    store: Store,
    token: Option<String>,
    dashboard: Option<Dashboard>,
}

pub fn router(store: Store, token: Option<String>) -> Router {
    dashboard_router(store, token, None)
}
pub fn dashboard_router(
    store: Store,
    token: Option<String>,
    dashboard: Option<Dashboard>,
) -> Router {
    Router::new()
        .route("/", get(home))
        .route("/dashboard.css", get(styles))
        .route("/status.css", get(|| async { ([("content-type", "text/css")], "body{font:16px system-ui;background:#f7f7f2;color:#232721;margin:3rem auto;padding:0 1rem;max-width:60rem}table{width:100%;border-collapse:collapse}td,th{text-align:left;padding:1rem;border-bottom:1px solid #ccd0c5}") }))
        .route("/dashboard.js", get(script))
        .route("/fonts/{name}", get(font_asset))
        .route("/api/dashboard", get(snapshot))
        .route("/api/targets/{name}/check", post(run_check))
        .route("/healthz", get(|| async { "ok" }))
        .route("/heartbeat/{name}", post(heartbeat))
        .layer(middleware::map_response(security_headers))
        .with_state(ServerState {
            store,
            token,
            dashboard,
        })
}
async fn security_headers(mut response: Response) -> Response {
    let headers = response.headers_mut();
    headers.insert("cache-control", "no-store".parse().expect("static header"));
    headers.insert(
        "x-content-type-options",
        "nosniff".parse().expect("static header"),
    );
    headers.insert(
        "referrer-policy",
        "no-referrer".parse().expect("static header"),
    );
    headers.insert("content-security-policy", "default-src 'self'; script-src 'self'; style-src 'self'; connect-src 'self'; img-src 'self'; object-src 'none'; base-uri 'none'; frame-ancestors 'none'; form-action 'none'".parse().expect("static header"));
    response
}
fn local_host(headers: &HeaderMap) -> Result<String, StatusCode> {
    let authority = headers
        .get("host")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<axum::http::uri::Authority>().ok())
        .ok_or(StatusCode::FORBIDDEN)?;
    if !matches!(
        authority.host(),
        "localhost" | "127.0.0.1" | "[::1]" | "::1"
    ) {
        return Err(StatusCode::FORBIDDEN);
    }
    Ok(authority.to_string())
}
fn local_dashboard<'a>(
    state: &'a ServerState,
    headers: &HeaderMap,
) -> Result<&'a Dashboard, StatusCode> {
    let dashboard = state.dashboard.as_ref().ok_or(StatusCode::NOT_FOUND)?;
    local_host(headers)?;
    Ok(dashboard)
}
async fn home(
    State(state): State<ServerState>,
    headers: HeaderMap,
) -> Result<Html<String>, StatusCode> {
    if state.dashboard.is_none() {
        return status(State(state)).await;
    }
    local_dashboard(&state, &headers)?;
    Ok(Html(include_str!("dashboard/index.html").to_string()))
}
async fn styles(
    State(state): State<ServerState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, StatusCode> {
    local_dashboard(&state, &headers)?;
    Ok((
        [("content-type", "text/css; charset=utf-8")],
        include_str!("dashboard/dashboard.css"),
    ))
}
async fn script(
    State(state): State<ServerState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, StatusCode> {
    local_dashboard(&state, &headers)?;
    Ok((
        [("content-type", "text/javascript; charset=utf-8")],
        include_str!("dashboard/dashboard.js"),
    ))
}
async fn font_asset(
    State(state): State<ServerState>,
    Path(name): Path<String>,
    headers: HeaderMap,
) -> Response {
    if let Err(code) = local_dashboard(&state, &headers) {
        return code.into_response();
    }
    let asset: (&str, &[u8]) = match name.as_str() {
        "manrope.woff2" => (
            "font/woff2",
            include_bytes!("dashboard/fonts/manrope.woff2"),
        ),
        "dm-sans.woff2" => (
            "font/woff2",
            include_bytes!("dashboard/fonts/dm-sans.woff2"),
        ),
        "LICENSE.txt" => (
            "text/plain; charset=utf-8",
            include_bytes!("dashboard/fonts/LICENSE.txt"),
        ),
        _ => return StatusCode::NOT_FOUND.into_response(),
    };
    ([("content-type", asset.0)], asset.1).into_response()
}
async fn snapshot(
    State(state): State<ServerState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, StatusCode> {
    let dashboard = local_dashboard(&state, &headers)?;
    dashboard
        .snapshot()
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
async fn run_check(
    State(state): State<ServerState>,
    Path(name): Path<String>,
    headers: HeaderMap,
) -> Response {
    let dashboard = match local_dashboard(&state, &headers) {
        Ok(dashboard) => dashboard,
        Err(code) => return code.into_response(),
    };
    let host = match local_host(&headers) {
        Ok(host) => host,
        Err(code) => return code.into_response(),
    };
    let origin = format!("http://{host}");
    if headers.get("origin").and_then(|v| v.to_str().ok()) != Some(origin.as_str())
        || headers
            .get("x-look-out-action")
            .and_then(|v| v.to_str().ok())
            != Some("run-check")
    {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error":"Use the local dashboard to run checks."})),
        )
            .into_response();
    }
    match dashboard.manual(&name).await {
        Ok(result) => Json(result).into_response(),
        Err(RunError::Missing) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error":"Target is not configured."})),
        )
            .into_response(),
        Err(RunError::Busy) => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({"error":"A check is already running. Try again shortly."})),
        )
            .into_response(),
    }
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
    Ok(Html(format!("<!doctype html><html lang=\"en\"><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>Look Out API status</title><link rel=\"stylesheet\" href=\"/status.css\"><body><h1>Look Out API</h1><p>Local monitoring status. Stale observations are shown as unknown.</p><table><thead><tr><th>Target</th><th>Check</th><th>Observed status</th></tr></thead><tbody>{rows}</tbody></table></body></html>")))
}
fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
