use look_out_api::{config::Config, dashboard::Dashboard, store::Store};
use wiremock::{matchers::method, Mock, MockServer, ResponseTemplate};

async fn fixture(delay: u64) -> (MockServer, Config, Store, Dashboard) {
    let mock = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_delay(std::time::Duration::from_millis(delay)))
        .mount(&mock)
        .await;
    let config = Config::parse(&format!("[defaults]\ninterval_secs=5\ntimeout_secs=2\nfailure_threshold=2\nrecovery_threshold=2\n[[target]]\nname='first'\nurl='{0}/health'\n[[target]]\nname='second'\nurl='{0}/health'",mock.uri())).expect("config");
    let store = Store::connect("sqlite::memory:").await.expect("store");
    store.sync(&config).await.expect("sync");
    let dashboard = Dashboard::new(&config, store.clone()).expect("dashboard");
    (mock, config, store, dashboard)
}
#[tokio::test]
async fn pause_freezes_scheduled_evidence_and_survives_sync_and_reopen() {
    let (mock, config, store, dashboard) = fixture(0).await;
    for _ in 0..2 {
        store
            .record(
                "first",
                &look_out_api::checker::CheckResult::failure("simulated local failure"),
                2,
                2,
            )
            .await
            .expect("seed");
    }
    let before = serde_json::to_value(store.history("first", 20).await.expect("before history"))
        .expect("json");
    let outbox = store.notifications().await.expect("outbox").len();
    dashboard.set_paused("first", true).await.expect("pause");
    dashboard
        .set_paused("first", true)
        .await
        .expect("idempotent pause");
    dashboard.scheduled("first").await.expect("skip paused");
    assert!(mock.received_requests().await.expect("requests").is_empty());
    assert_eq!(
        serde_json::to_value(store.history("first", 20).await.expect("history")).expect("json"),
        before
    );
    assert_eq!(store.notifications().await.expect("outbox").len(), outbox);
    let snapshot =
        serde_json::to_value(dashboard.snapshot().await.expect("snapshot")).expect("json");
    assert_eq!(snapshot["targets"][0]["paused"], true);
    assert_eq!(snapshot["targets"][0]["status"], "down");
    assert_eq!(snapshot["targets"][0]["consecutive_fail"], 2);
    assert!(
        dashboard
            .manual("first")
            .await
            .expect("intentional manual")
            .ok
    );
    dashboard.scheduled("second").await.expect("other target");
    assert_eq!(mock.received_requests().await.expect("requests").len(), 2);
    store.sync(&config).await.expect("restart sync");
    let restarted = Dashboard::new(&config, store.clone()).expect("restart runtime");
    restarted.scheduled("first").await.expect("still paused");
    assert_eq!(mock.received_requests().await.expect("requests").len(), 2);
    restarted.set_paused("first", false).await.expect("resume");
    restarted
        .scheduled("first")
        .await
        .expect("resumed real request");
    assert_eq!(mock.received_requests().await.expect("requests").len(), 3);
    assert_eq!(store.history("first", 20).await.expect("history").len(), 3);
    assert!(
        sqlx::query("UPDATE targets SET paused=2 WHERE name='first'")
            .execute(&store.pool)
            .await
            .is_err()
    );
}
#[tokio::test]
async fn pause_waits_for_an_active_execution_before_acknowledging() {
    let (mock, _, store, dashboard) = fixture(200).await;
    let scheduled = dashboard.clone();
    let running = tokio::spawn(async move { scheduled.scheduled("first").await });
    for _ in 0..100 {
        if !mock.received_requests().await.expect("requests").is_empty() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    }
    let control = dashboard.clone();
    let mut paused = tokio::spawn(async move { control.set_paused("first", true).await });
    assert!(
        tokio::time::timeout(std::time::Duration::from_millis(20), &mut paused)
            .await
            .is_err()
    );
    running.await.expect("check join").expect("check");
    paused.await.expect("pause join").expect("pause");
    dashboard.scheduled("first").await.expect("skip");
    assert_eq!(mock.received_requests().await.expect("requests").len(), 1);
    assert_eq!(store.history("first", 10).await.expect("history").len(), 1);
}
#[tokio::test]
async fn pause_endpoints_enforce_action_boundaries_and_report_failed_writes() {
    let (_, _, store, dashboard) = fixture(0).await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("listen");
    let address = listener.local_addr().expect("address");
    let base = format!("http://{address}");
    let app = look_out_api::server::dashboard_router(store.clone(), None, Some(dashboard));
    let server = tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve");
    });
    let client = reqwest::Client::new();
    for action in ["pause", "resume"] {
        for (origin, header, host) in [
            (
                "https://foreign.example".to_owned(),
                true,
                address.to_string(),
            ),
            (base.clone(), false, address.to_string()),
            (base.clone(), true, "foreign.example".into()),
        ] {
            let mut request = client
                .post(format!("{base}/api/targets/first/{action}"))
                .header("origin", origin)
                .header("host", host);
            if header {
                request = request.header("x-look-out-action", format!("{action}-target"));
            }
            assert_eq!(request.send().await.expect("guard").status(), 403);
        }
        for name in ["first", "missing"] {
            let response = client
                .post(format!("{base}/api/targets/{name}/{action}"))
                .header("origin", &base)
                .header("x-look-out-action", format!("{action}-target"))
                .send()
                .await
                .expect("control");
            assert_eq!(response.status(), if name == "missing" { 404 } else { 200 });
            assert!(!response.text().await.expect("body").contains("/health"));
        }
    }
    sqlx::query("CREATE TRIGGER reject_pause BEFORE UPDATE OF paused ON targets BEGIN SELECT RAISE(ABORT,'simulated local write failure'); END").execute(&store.pool).await.expect("trigger");
    let response = client
        .post(format!("{base}/api/targets/first/pause"))
        .header("origin", &base)
        .header("x-look-out-action", "pause-target")
        .send()
        .await
        .expect("failed change");
    assert_eq!(response.status(), 500);
    assert!(!store.statuses().await.expect("states")[0].paused);
    server.abort();
}

#[tokio::test]
async fn pause_finishes_when_the_http_client_disconnects() {
    use tokio::io::AsyncWriteExt;
    let (mock, _, store, dashboard) = fixture(200).await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("listen");
    let address = listener.local_addr().expect("address");
    let app = look_out_api::server::dashboard_router(store.clone(), None, Some(dashboard.clone()));
    let server = tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve");
    });
    let running = tokio::spawn(async move { dashboard.scheduled("first").await });
    for _ in 0..100 {
        if !mock.received_requests().await.expect("requests").is_empty() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    }
    let mut client = tokio::net::TcpStream::connect(address)
        .await
        .expect("connect");
    client.write_all(format!("POST /api/targets/first/pause HTTP/1.1\r\nHost: {address}\r\nOrigin: http://{address}\r\nX-Look-Out-Action: pause-target\r\nContent-Length: 0\r\n\r\n").as_bytes()).await.expect("send");
    tokio::time::sleep(std::time::Duration::from_millis(25)).await;
    drop(client);
    running.await.expect("join").expect("check");
    for _ in 0..100 {
        if store.paused("first").await.expect("state") {
            server.abort();
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    }
    server.abort();
    panic!("admitted pause did not finish after disconnect");
}
