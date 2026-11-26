use look_out_api::{config::Config, dashboard::Dashboard, server, store::Store};
use serde_json::Value;
use wiremock::{matchers::method, Mock, MockServer, ResponseTemplate};

async fn fixture(delay_ms: u64) -> (MockServer, Store, Dashboard) {
    let mock = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string("private-response-ok")
                .set_delay(std::time::Duration::from_millis(delay_ms)),
        )
        .mount(&mock)
        .await;
    let config = Config::parse(&format!("[defaults]\ninterval_secs=5\ntimeout_secs=2\nfailure_threshold=2\nrecovery_threshold=2\n[[target]]\nname='local-api'\nurl='{}/health?private=secret-query'\nexpected_status=200\nexpected_body_contains='private-response-ok'\n[target.headers]\nauthorization='secret-header'", mock.uri())).expect("config");
    let store = Store::connect("sqlite::memory:").await.expect("store");
    store.sync(&config).await.expect("sync");
    let dashboard = Dashboard::new(&config, store.clone()).expect("dashboard");
    (mock, store, dashboard)
}

#[tokio::test]
async fn manual_checks_make_real_requests_without_changing_incidents_or_history() {
    let (mock, store, dashboard) = fixture(0).await;
    let result = dashboard.manual("local-api").await.expect("run");
    assert!(result.ok);
    assert_eq!(result.status_code, Some(200));
    assert_eq!(mock.received_requests().await.expect("requests").len(), 1);
    assert!(store
        .history("local-api", 10)
        .await
        .expect("history")
        .is_empty());
    assert_eq!(store.statuses().await.expect("state")[0].status, "unknown");
    for table in ["checks", "incidents", "notifications"] {
        let count: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {table}"))
            .fetch_one(&store.pool)
            .await
            .expect("count");
        assert_eq!(count, 0);
    }
    mock.reset().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string("wrong response"))
        .mount(&mock)
        .await;
    let result = dashboard.manual("local-api").await.expect("run mismatch");
    assert!(!result.ok);
    assert_eq!(
        result.error.as_deref(),
        Some("expected response text was not found")
    );
    assert!(dashboard.manual("not-configured").await.is_err());
    assert_eq!(mock.received_requests().await.expect("requests").len(), 1);
}

#[tokio::test]
async fn execution_is_serialized_and_scheduled_results_remain_durable_and_safe() {
    let (mock, store, dashboard) = fixture(150).await;
    let first = dashboard.clone();
    let running = tokio::spawn(async move { first.manual("local-api").await });
    for _ in 0..100 {
        if dashboard.snapshot().await.expect("snapshot").targets[0]
            .activity
            .is_some()
        {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    }
    assert!(dashboard.manual("local-api").await.is_err());
    let scheduled = dashboard.clone();
    let scheduled = tokio::spawn(async move { scheduled.scheduled("local-api").await });
    assert!(running.await.expect("join").expect("first").ok);
    scheduled.await.expect("join").expect("scheduled");
    assert_eq!(mock.received_requests().await.expect("requests").len(), 2);
    assert_eq!(
        store.history("local-api", 10).await.expect("history").len(),
        1
    );
    let snapshot = dashboard.snapshot().await.expect("snapshot");
    assert!(snapshot.targets[0].activity.is_none());
    let text = serde_json::to_string(&snapshot).expect("json");
    for secret in [
        "secret-query",
        "secret-header",
        "private-response-ok",
        &mock.uri(),
    ] {
        assert!(!text.contains(secret));
    }
    assert!(text.contains("status_code"));
}

#[tokio::test]
async fn dashboard_api_checks_origin_host_and_configured_target_boundary() {
    let (_mock, store, dashboard) = fixture(0).await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("listen");
    let address = listener.local_addr().expect("address");
    let base = format!("http://{address}");
    let app = server::dashboard_router(store, None, Some(dashboard));
    let handle = tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve");
    });
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{base}/api/dashboard"))
        .send()
        .await
        .expect("snapshot");
    assert_eq!(response.status(), 200);
    assert_eq!(response.headers()["cache-control"], "no-store");
    let body: Value = response.json().await.expect("json");
    assert_eq!(body["targets"][0]["name"], "local-api");
    let page = client.get(&base).send().await.expect("page");
    assert!(page.headers()["content-security-policy"]
        .to_str()
        .expect("csp")
        .contains("frame-ancestors 'none'"));
    assert!(page.text().await.expect("html").contains("dashboard.js"));
    let font = client
        .get(format!("{base}/fonts/manrope.woff2"))
        .send()
        .await
        .expect("font");
    assert_eq!(font.status(), 200);
    assert_eq!(font.headers()["content-type"], "font/woff2");
    assert!(font.bytes().await.expect("font bytes").len() > 1000);
    for (origin, action, host, expected) in [
        (base.as_str(), true, address.to_string(), 200),
        ("https://foreign.example", true, address.to_string(), 403),
        (base.as_str(), false, address.to_string(), 403),
        (base.as_str(), true, "foreign.example".into(), 403),
    ] {
        let mut request = client
            .post(format!("{base}/api/targets/local-api/check"))
            .header("origin", origin)
            .header("host", host);
        if action {
            request = request.header("x-look-out-action", "run-check");
        }
        assert_eq!(
            request.send().await.expect("request").status().as_u16(),
            expected
        );
    }
    let missing = client
        .post(format!("{base}/api/targets/missing/check"))
        .header("origin", &base)
        .header("x-look-out-action", "run-check")
        .send()
        .await
        .expect("missing");
    assert_eq!(missing.status(), 404);
    let foreign = client
        .get(format!("{base}/api/dashboard"))
        .header("host", "foreign.example")
        .send()
        .await
        .expect("foreign");
    assert_eq!(foreign.status(), 403);
    handle.abort();
}

#[tokio::test]
async fn dashboard_actions_are_unavailable_without_loopback_runtime() {
    let (_mock, store, _) = fixture(0).await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("listen");
    let address = listener.local_addr().expect("address");
    let app = server::dashboard_router(store, None, None);
    let handle = tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve");
    });
    let client = reqwest::Client::new();
    assert_eq!(
        client
            .get(format!("http://{address}/api/dashboard"))
            .send()
            .await
            .expect("snapshot")
            .status(),
        404
    );
    assert_eq!(
        client
            .post(format!("http://{address}/api/targets/local-api/check"))
            .send()
            .await
            .expect("run")
            .status(),
        404
    );
    handle.abort();
}

#[tokio::test]
async fn dashboard_exposes_real_flow_steps_timeouts_and_stale_state() {
    use wiremock::matchers::path;
    let mock = MockServer::start().await;
    Mock::given(path("/session"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"token":"extracted-private-value"})),
        )
        .mount(&mock)
        .await;
    Mock::given(path("/orders"))
        .respond_with(ResponseTemplate::new(503))
        .mount(&mock)
        .await;
    Mock::given(path("/slow"))
        .respond_with(ResponseTemplate::new(200).set_delay(std::time::Duration::from_secs(2)))
        .mount(&mock)
        .await;
    let config = Config::parse(&format!("[defaults]\ninterval_secs=5\ntimeout_secs=1\n[[target]]\nname='checkout'\ntype='flow'\n[[target.step]]\nname='session'\nurl='{0}/session'\nextract={{token='/token'}}\n[[target.step]]\nname='orders'\nurl='{0}/orders'\nheaders={{authorization='Bearer {{{{token}}}}'}}\n[[target]]\nname='slow'\nurl='{0}/slow'", mock.uri())).expect("config");
    let store = Store::connect("sqlite::memory:").await.expect("store");
    store.sync(&config).await.expect("sync");
    let dashboard = Dashboard::new(&config, store.clone()).expect("dashboard");
    let result = dashboard.manual("checkout").await.expect("flow");
    assert!(!result.ok);
    assert_eq!(result.steps.len(), 2);
    assert!(result.steps[0].ok);
    assert_eq!(result.steps[1].status_code, Some(503));
    dashboard.scheduled("checkout").await.expect("scheduled");
    let slow = dashboard.manual("slow").await.expect("timeout");
    assert!(!slow.ok);
    assert!(slow.error.as_deref().expect("reason").contains("timed out"));
    sqlx::query("UPDATE target_state SET last_checked_at='2000-01-01T00:00:00Z'")
        .execute(&store.pool)
        .await
        .expect("stale");
    let snapshot =
        serde_json::to_value(dashboard.snapshot().await.expect("snapshot")).expect("json");
    assert_eq!(snapshot["targets"][0]["stale"], true);
    assert_eq!(snapshot["targets"][0]["steps"][1]["status_code"], 503);
    assert!(!snapshot.to_string().contains("extracted-private-value"));
    assert_eq!(
        snapshot["targets"][0]["history"]
            .as_array()
            .expect("history")
            .len(),
        1
    );
}
