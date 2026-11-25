use look_out_api::{
    alert::{deliver_pending, Telegram},
    checker::{CheckResult, Executor},
    config::Config,
    state::Transition,
    store::Store,
};
use sqlx::Row;
use wiremock::{matchers::method, Mock, MockServer, ResponseTemplate};

fn config() -> Config {
    Config::parse("[defaults]\nfailure_threshold=2\nrecovery_threshold=2\n[[target]]\nname='api'\nurl='http://127.0.0.1:1'").expect("config")
}
#[tokio::test]
async fn restart_preserves_counters_and_deduplicates_incident_events() {
    let dir = tempfile::tempdir().expect("directory");
    let url = format!("sqlite://{}", dir.path().join("test.db").display());
    let store = Store::connect(&url).await.expect("store");
    store.sync(&config()).await.expect("sync");
    let failure = CheckResult::failure("HTTP connection failed");
    assert_eq!(
        store.record("api", &failure, 2, 2).await.expect("record"),
        None
    );
    store.pool.close().await;
    let store = Store::connect(&url).await.expect("reopen");
    store.sync(&config()).await.expect("sync");
    assert_eq!(
        store.record("api", &failure, 2, 2).await.expect("record"),
        Some(Transition::Down)
    );
    store.record("api", &failure, 2, 2).await.expect("record");
    assert_eq!(store.notifications().await.expect("queue").len(), 1);
    store
        .record("api", &CheckResult::success(), 2, 2)
        .await
        .expect("record");
    assert_eq!(
        store
            .record("api", &CheckResult::success(), 2, 2)
            .await
            .expect("record"),
        Some(Transition::Recovered)
    );
    assert_eq!(store.notifications().await.expect("queue").len(), 2);
    let incidents = sqlx::query("SELECT started_at,resolved_at FROM incidents")
        .fetch_all(&store.pool)
        .await
        .expect("incidents");
    assert_eq!(incidents.len(), 1);
    assert!(incidents[0]
        .get::<Option<String>, _>("resolved_at")
        .is_some());
    assert_eq!(
        store.pending().await.expect("pending").len(),
        1,
        "recovery waits for down delivery"
    );
}
#[tokio::test]
async fn database_constraints_and_failed_transaction_protect_incidents() {
    let store = Store::connect("sqlite::memory:").await.expect("store");
    store.sync(&config()).await.expect("sync");
    for sql in [
        "UPDATE target_state SET status='bad'",
        "UPDATE target_state SET consecutive_fail=-1",
        "INSERT INTO checks(target_id,checked_at,ok,latency_ms) VALUES(999,'now',1,1)",
    ] {
        assert!(sqlx::query(sql).execute(&store.pool).await.is_err());
    }
    let mut invalid = CheckResult::failure("failed");
    invalid.latency_ms = -1;
    assert!(store.record("api", &invalid, 1, 1).await.is_err());
    assert_eq!(store.statuses().await.expect("states")[0].status, "unknown");
    assert_eq!(store.notifications().await.expect("queue").len(), 0);
    store
        .record("api", &CheckResult::failure("failed"), 1, 1)
        .await
        .expect("down");
    assert!(
        sqlx::query("INSERT INTO incidents(target_id,started_at) VALUES(1,'now')")
            .execute(&store.pool)
            .await
            .is_err()
    );
    assert!(sqlx::query("INSERT INTO notifications(target_id,event_key,kind,message,created_at,next_attempt_at) SELECT target_id,event_key,kind,message,created_at,next_attempt_at FROM notifications").execute(&store.pool).await.is_err());
}
#[tokio::test]
async fn unavailable_telegram_retains_queue_then_delivers_in_order() {
    let server = MockServer::start().await;
    let store = Store::connect("sqlite::memory:").await.expect("store");
    store.sync(&config()).await.expect("sync");
    store
        .record("api", &CheckResult::failure("failed"), 1, 1)
        .await
        .expect("down");
    store
        .record("api", &CheckResult::success(), 1, 1)
        .await
        .expect("up");
    let telegram =
        Telegram::new("private-bot-token".into(), "chat".into(), server.uri()).expect("telegram");
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(503).set_body_string("private-service-error"))
        .mount(&server)
        .await;
    deliver_pending(&store, &telegram).await.expect("attempt");
    let notifications = store.notifications().await.expect("queue");
    assert_eq!(notifications.len(), 2);
    assert_eq!(notifications[0].attempts, 1);
    assert_eq!(notifications[1].attempts, 0);
    assert!(!notifications[0]
        .last_error
        .as_deref()
        .expect("reason")
        .contains("private"));
    server.reset().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"ok":true})))
        .mount(&server)
        .await;
    store.retry_notifications().await.expect("retry");
    deliver_pending(&store, &telegram)
        .await
        .expect("down delivery");
    deliver_pending(&store, &telegram)
        .await
        .expect("recovery delivery");
    assert!(store.notifications().await.expect("queue").is_empty());
    let requests = server.received_requests().await.expect("requests");
    assert_eq!(requests.len(), 2);
    let body1: serde_json::Value = serde_json::from_slice(&requests[0].body).expect("json");
    let body2: serde_json::Value = serde_json::from_slice(&requests[1].body).expect("json");
    assert!(body1["text"].as_str().expect("text").starts_with("DOWN"));
    assert!(body2["text"]
        .as_str()
        .expect("text")
        .starts_with("RECOVERED"));
}
#[tokio::test]
async fn heartbeat_grace_and_ping_are_persistent() {
    let config = Config::parse("[[target]]\nname='backup'\ntype='heartbeat'\ngrace_secs=60")
        .expect("config");
    let store = Store::connect("sqlite::memory:").await.expect("store");
    store.sync(&config).await.expect("sync");
    let executor = Executor::new(
        config.targets[0].clone(),
        config.defaults,
        Some(store.clone()),
    )
    .expect("executor");
    assert!(executor.check().await.ok, "initial startup grace period");
    sqlx::query("UPDATE targets SET created_at='2000-01-01T00:00:00+00:00'")
        .execute(&store.pool)
        .await
        .expect("age");
    assert!(!executor.check().await.ok);
    assert!(!store.ping("not-a-target").await.expect("unknown"));
    assert!(store.ping("backup").await.expect("ping"));
    assert!(executor.check().await.ok);
}

#[tokio::test]
async fn rejected_credentials_block_later_notifications_until_explicit_retry() {
    let server = MockServer::start().await;
    let store = Store::connect("sqlite::memory:").await.expect("store");
    store.sync(&config()).await.expect("sync");
    store
        .record("api", &CheckResult::failure("failed"), 1, 1)
        .await
        .expect("down");
    store
        .record("api", &CheckResult::success(), 1, 1)
        .await
        .expect("recovery");
    let telegram =
        Telegram::new("test-token".into(), "chat".into(), server.uri()).expect("telegram");
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(403).set_body_json(
            serde_json::json!({"ok":false,"error_code":403,"description":"private-error-content"}),
        ))
        .mount(&server)
        .await;
    deliver_pending(&store, &telegram).await.expect("attempt");
    deliver_pending(&store, &telegram).await.expect("blocked");
    assert_eq!(server.received_requests().await.expect("requests").len(), 1);
    assert!(store.notifications().await.expect("queue")[0].permanent_failure);
    server.reset().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"ok":true})))
        .mount(&server)
        .await;
    store.retry_notifications().await.expect("retry");
    deliver_pending(&store, &telegram).await.expect("down");
    deliver_pending(&store, &telegram).await.expect("up");
    assert!(store.notifications().await.expect("empty").is_empty());
}

#[tokio::test]
async fn telegram_rate_limits_and_unconfirmed_responses_remain_failures() {
    let server = MockServer::start().await;
    let telegram =
        Telegram::new("test-token".into(), "chat".into(), server.uri()).expect("telegram");
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(429).set_body_json(
            serde_json::json!({"ok":false,"error_code":429,"parameters":{"retry_after":60}}),
        ))
        .mount(&server)
        .await;
    let error = telegram.send("message").await.expect_err("limited");
    assert!(!error.permanent);
    assert_eq!(error.retry_after, Some(60));
    server.reset().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_string("private-malformed-response"))
        .mount(&server)
        .await;
    let error = telegram.send("message").await.expect_err("unconfirmed");
    assert!(!error.permanent);
    assert!(!error.reason.contains("private"));
}
