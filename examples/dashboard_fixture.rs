//! Local-only browser-test fixture. Explicit demo targets, temporary database, no env credentials.
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
    Json, Router,
};
use look_out_api::{config::Config, dashboard::Dashboard, server, store::Store};
use std::sync::{
    atomic::{AtomicU8, AtomicUsize, Ordering},
    Arc,
};
#[derive(Clone, Default)]
struct Demo {
    mode: Arc<AtomicU8>,
    requests: Arc<AtomicUsize>,
}
async fn health(State(demo): State<Demo>) -> (StatusCode, &'static str) {
    demo.requests.fetch_add(1, Ordering::Relaxed);
    tokio::time::sleep(std::time::Duration::from_millis(250)).await;
    match demo.mode.load(Ordering::Relaxed) {
        1 => (StatusCode::SERVICE_UNAVAILABLE, "unavailable"),
        2 => (StatusCode::OK, "unexpected response"),
        _ => (StatusCode::OK, "demo-ok"),
    }
}
async fn control(State(demo): State<Demo>, Path(mode): Path<String>) -> StatusCode {
    let value = match mode.as_str() {
        "up" => 0,
        "down" => 1,
        "mismatch" => 2,
        _ => return StatusCode::BAD_REQUEST,
    };
    demo.mode.store(value, Ordering::Relaxed);
    StatusCode::NO_CONTENT
}
async fn orders(headers: HeaderMap) -> (StatusCode, Json<serde_json::Value>) {
    if headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        == Some("Bearer demo-flow-value")
    {
        (StatusCode::OK, Json(serde_json::json!({"orders":[]})))
    } else {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error":"authentication required"})),
        )
    }
}
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let demo = Demo::default();
    let app =
        Router::new()
            .route("/health", get(health))
            .route(
                "/session",
                get(|| async { Json(serde_json::json!({"token":"demo-flow-value"})) }),
            )
            .route("/orders", get(orders))
            .route(
                "/slow",
                get(|| async {
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    "demo-ok"
                }),
            )
            .route("/control/{mode}", post(control))
            .route(
                "/requests",
                get(|State(demo): State<Demo>| async move {
                    Json(demo.requests.load(Ordering::Relaxed))
                }),
            )
            .with_state(demo);
    let mock = tokio::net::TcpListener::bind("127.0.0.1:4182").await?;
    tokio::spawn(async move {
        axum::serve(mock, app).await.expect("demo server");
    });
    let config = Config::parse("[defaults]\ninterval_secs=5\ntimeout_secs=1\nfailure_threshold=2\nrecovery_threshold=2\n[[target]]\nname='demo-api'\nurl='http://127.0.0.1:4182/health'\nexpected_status=200\nexpected_body_contains='demo-ok'\n[[target]]\nname='checkout-flow'\ntype='flow'\n[[target.step]]\nname='create-session'\nurl='http://127.0.0.1:4182/session'\nextract={token='/token'}\n[[target.step]]\nname='read-orders'\nurl='http://127.0.0.1:4182/orders'\nheaders={authorization='Bearer {{token}}'}\n[[target]]\nname='slow-api'\nurl='http://127.0.0.1:4182/slow'")?;
    let store = Store::connect("sqlite::memory:").await?;
    store.sync(&config).await?;
    let dashboard = Dashboard::new(&config, store.clone())?;
    for target in config.targets {
        let runner = dashboard.clone();
        tokio::spawn(async move {
            let mut ticks = tokio::time::interval(std::time::Duration::from_secs(5));
            loop {
                ticks.tick().await;
                runner
                    .scheduled(&target.name)
                    .await
                    .expect("scheduled demo");
            }
        });
    }
    let listener = tokio::net::TcpListener::bind("127.0.0.1:4181").await?;
    axum::serve(
        listener,
        server::dashboard_router(store, None, Some(dashboard)),
    )
    .await?;
    Ok(())
}
