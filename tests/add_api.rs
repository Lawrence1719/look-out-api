use look_out_api::{config::Config, dashboard::Dashboard, registration::NewApi, store::Store};
use std::path::Path;

fn input(name: &str) -> NewApi {
    serde_json::from_value(serde_json::json!({"name":name,"url":"http://127.0.0.1:3001/health","method":"GET","expected_status":200,"expected_body_contains":"ok","interval_secs":30,"timeout_secs":5})).expect("input")
}
async fn setup(path: &Path) -> (Store, Dashboard) {
    let config = Config::load(path).expect("config");
    let store = Store::connect("sqlite::memory:").await.expect("store");
    store.sync(&config).await.expect("sync");
    let dashboard = Dashboard::new(&config, store.clone())
        .expect("dashboard")
        .with_config_file(path.to_path_buf(), &config);
    (store, dashboard)
}
#[tokio::test]
async fn adding_api_preserves_raw_config_and_registers_only_the_new_target() {
    let directory = tempfile::tempdir().expect("dir");
    let path = directory.path().join("monitor.toml");
    let original = "# Preserve this comment\n[defaults]\nfailure_threshold=4\n[[target]]\nname='original'\nurl='http://127.0.0.1:3001/health'\n[target.headers]\nx-local-path='${PATH}'\n# An untouched example: ${PRIVATE_SETTING}\n";
    std::fs::write(&path, original).expect("write");
    let (store, dashboard) = setup(&path).await;
    for _ in 0..4 {
        store
            .record(
                "original",
                &look_out_api::checker::CheckResult::failure("simulated local failure"),
                4,
                2,
            )
            .await
            .expect("seed incident");
    }
    let before_state =
        serde_json::to_value(store.statuses().await.expect("before states")).expect("json");
    let before_outbox = store
        .notifications()
        .await
        .expect("before outbox")
        .into_iter()
        .map(|n| {
            (
                n.id,
                n.target_id,
                n.event_key,
                n.kind,
                n.message,
                n.attempts,
                n.last_error,
                n.permanent_failure,
            )
        })
        .collect::<Vec<_>>();
    let mut schedule = dashboard.subscribe_schedules();
    let result = dashboard.add_api(input("added-api")).await.expect("add");
    assert!(result.monitoring);
    schedule.changed().await.expect("new schedule");
    assert_eq!(schedule.borrow().len(), 2);
    let text = std::fs::read_to_string(&path).expect("read");
    assert!(text.starts_with(original));
    assert!(text.contains("${PRIVATE_SETTING}"));
    assert!(text.contains("${PATH}"));
    let reopened = Config::load(&path).expect("reload");
    assert_eq!(reopened.targets.len(), 2);
    assert_eq!(store.statuses().await.expect("states").len(), 2);
    assert!(store
        .history("added-api", 10)
        .await
        .expect("history")
        .is_empty());
    assert_eq!(
        store
            .notifications()
            .await
            .expect("outbox")
            .into_iter()
            .map(|n| (
                n.id,
                n.target_id,
                n.event_key,
                n.kind,
                n.message,
                n.attempts,
                n.last_error,
                n.permanent_failure
            ))
            .collect::<Vec<_>>(),
        before_outbox
    );
    let after = store.statuses().await.expect("states");
    assert_eq!(
        serde_json::to_value(
            after
                .into_iter()
                .filter(|state| state.name == "original")
                .collect::<Vec<_>>()
        )
        .expect("json"),
        before_state
    );
    assert!(dashboard.add_api(input("added-api")).await.is_err());
    assert_eq!(std::fs::read_to_string(&path).expect("unchanged"), text);
    let snapshot =
        serde_json::to_string(&dashboard.snapshot().await.expect("snapshot")).expect("json");
    assert!(!snapshot.contains("127.0.0.1:3001"));
    assert!(!snapshot.contains("PRIVATE_SETTING"));
}
#[tokio::test]
async fn invalid_additions_and_external_edits_do_not_overwrite_configuration() {
    let directory = tempfile::tempdir().expect("dir");
    let path = directory.path().join("monitor.toml");
    let original = "[[target]]\nname='original'\nurl='http://127.0.0.1:3001/health'\n";
    std::fs::write(&path, original).expect("write");
    let (store, dashboard) = setup(&path).await;
    for bad in [
        serde_json::json!({"url":"file:///private/file"}),
        serde_json::json!({"name":"bad name"}),
        serde_json::json!({"method":"POST"}),
        serde_json::json!({"interval_secs":5,"timeout_secs":5}),
        serde_json::json!({"url":"https://user:password@example.invalid"}),
        serde_json::json!({"expected_body_contains":"${PRIVATE_SETTING}"}),
    ] {
        let mut value = serde_json::to_value(input("invalid-api")).expect("json");
        for (key, item) in bad.as_object().expect("object") {
            value[key] = item.clone();
        }
        assert!(dashboard
            .add_api(serde_json::from_value(value).expect("input"))
            .await
            .is_err());
        assert_eq!(std::fs::read_to_string(&path).expect("read"), original);
    }
    let edited = format!("{original}\n# External edit\n");
    std::fs::write(&path, &edited).expect("edit");
    assert!(dashboard.add_api(input("new-api")).await.is_err());
    assert_eq!(std::fs::read_to_string(&path).expect("read"), edited);
    assert_eq!(store.statuses().await.expect("states").len(), 1);
}
#[tokio::test]
async fn committed_configuration_recovers_after_registration_failure() {
    let directory = tempfile::tempdir().expect("dir");
    let path = directory.path().join("monitor.toml");
    std::fs::write(
        &path,
        "[[target]]\nname='original'\nurl='http://127.0.0.1:3001/health'\n",
    )
    .expect("write");
    let (store, dashboard) = setup(&path).await;
    sqlx::query("CREATE TRIGGER reject_new BEFORE INSERT ON targets WHEN NEW.name='saved-api' BEGIN SELECT RAISE(ABORT,'simulated registration failure'); END").execute(&store.pool).await.expect("failure trigger");
    let result = dashboard.add_api(input("saved-api")).await.expect("saved");
    assert!(!result.monitoring);
    let config = Config::load(&path).expect("reopen");
    assert!(config
        .targets
        .iter()
        .any(|target| target.name == "saved-api"));
    let reopened = Store::connect("sqlite::memory:").await.expect("store");
    reopened.sync(&config).await.expect("startup sync");
    assert_eq!(reopened.statuses().await.expect("states").len(), 2);
}

#[tokio::test]
async fn add_endpoint_requires_local_action_and_rejects_unknown_or_oversized_input() {
    let directory = tempfile::tempdir().expect("dir");
    let path = directory.path().join("monitor.toml");
    std::fs::write(
        &path,
        "[[target]]\nname='original'\nurl='http://127.0.0.1:3001/health'\n",
    )
    .expect("write");
    let (store, dashboard) = setup(&path).await;
    let _schedule = dashboard.subscribe_schedules();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("listen");
    let address = listener.local_addr().expect("address");
    let base = format!("http://{address}");
    let app = look_out_api::server::dashboard_router(store.clone(), None, Some(dashboard));
    let handle = tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve");
    });
    let client = reqwest::Client::new();
    for (origin, action, host) in [
        (
            "https://foreign.example".to_owned(),
            true,
            address.to_string(),
        ),
        (base.clone(), false, address.to_string()),
        (base.clone(), true, "foreign.example".into()),
    ] {
        let mut request = client
            .post(format!("{base}/api/targets"))
            .header("origin", origin)
            .header("host", host)
            .json(&input("blocked"));
        if action {
            request = request.header("x-look-out-action", "add-api");
        }
        assert_eq!(request.send().await.expect("request").status(), 403);
    }
    for (body, status) in [
        ("{\"private-input\":\"secret\"}".into(), 400),
        ("x".repeat(17_000), 413),
    ] {
        let response = client
            .post(format!("{base}/api/targets"))
            .header("origin", &base)
            .header("x-look-out-action", "add-api")
            .header("content-type", "application/json")
            .body(body)
            .send()
            .await
            .expect("request");
        assert_eq!(response.status(), status);
        assert!(!response.text().await.expect("text").contains("secret"));
    }
    let mut new = input("from-api");
    new.expected_status = None;
    new.expected_body_contains = None;
    let response = client
        .post(format!("{base}/api/targets"))
        .header("origin", &base)
        .header("x-look-out-action", "add-api")
        .json(&new)
        .send()
        .await
        .expect("add");
    assert_eq!(response.status(), 201);
    assert!(!response.text().await.expect("text").contains("3001"));
    let config = Config::load(&path).expect("persisted");
    assert_eq!(config.targets.len(), 2);
    assert_eq!(config.targets[1].expected_status, None);
    handle.abort();
}

#[tokio::test]
async fn concurrent_saves_do_not_lose_targets_or_schedule_updates() {
    let directory = tempfile::tempdir().expect("dir");
    let path = directory.path().join("monitor.toml");
    std::fs::write(
        &path,
        "[[target]]\nname='original'\nurl='http://127.0.0.1:3001/health'\n",
    )
    .expect("write");
    let (store, dashboard) = setup(&path).await;
    let schedule = dashboard.subscribe_schedules();
    let (first, second) = tokio::join!(
        dashboard.add_api(input("first")),
        dashboard.add_api(input("second"))
    );
    for (name, result) in [("first", first), ("second", second)] {
        if result.is_err() {
            dashboard.add_api(input(name)).await.expect("retry busy");
        }
    }
    assert_eq!(Config::load(&path).expect("reload").targets.len(), 3);
    assert_eq!(store.statuses().await.expect("states").len(), 3);
    assert_eq!(schedule.borrow().len(), 3);
}
