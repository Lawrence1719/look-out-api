use look_out_api::{
    report::{percentile95, since_duration},
    scheduler,
    stateless::{load, save, FileState},
};
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn independent_processes_alert_once_and_keep_credentials_out_of_output() {
    let api = MockServer::start().await;
    let telegram = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"ok":true})))
        .mount(&telegram)
        .await;
    let dir = tempfile::tempdir().expect("directory");
    let config_path = dir.path().join("monitor.toml");
    let state_path = dir.path().join("state.json");
    std::fs::write(&config_path,format!("[defaults]\nfailure_threshold=1\nrecovery_threshold=2\n[[target]]\nname='api'\nurl='{}/health?token=private-url-secret'\nheaders={{Authorization='Bearer private-header-secret'}}",api.uri())).expect("config");
    let run = || {
        let mut command =
            tokio::process::Command::new(assert_cmd::cargo::cargo_bin!("look-out-api"));
        command
            .current_dir(dir.path())
            .arg("--config")
            .arg(&config_path)
            .arg("--quiet")
            .args(["check", "--state-file"])
            .arg(&state_path)
            .arg("--alert")
            .env("RUST_LOG", "trace")
            .env("TELEGRAM_BOT_TOKEN", "private-bot-secret")
            .env("TELEGRAM_CHAT_ID", "chat")
            .env("TELEGRAM_API_BASE_URL", telegram.uri());
        command
    };
    for expected_code in [1, 1] {
        api.reset().await;
        Mock::given(path("/health"))
            .respond_with(ResponseTemplate::new(500).set_body_string("private-body-secret"))
            .mount(&api)
            .await;
        let output = run().output().await.expect("process");
        assert_eq!(output.status.code(), Some(expected_code));
        let combined = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(!combined.contains("private-"), "output leaked a secret");
    }
    assert_eq!(
        telegram.received_requests().await.expect("requests").len(),
        1
    );
    api.reset().await;
    Mock::given(path("/health"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&api)
        .await;
    assert_eq!(
        run().output().await.expect("recovery 1").status.code(),
        Some(1)
    );
    assert_eq!(
        run().output().await.expect("recovery 2").status.code(),
        Some(0)
    );
    assert_eq!(
        run().output().await.expect("still up").status.code(),
        Some(0)
    );
    assert_eq!(
        telegram.received_requests().await.expect("requests").len(),
        2
    );
    let saved = std::fs::read_to_string(&state_path).expect("saved");
    assert!(!saved.contains("private-"));
    assert!(
        !dir.path().join("look-out-api.db").exists(),
        "stateless mode must not create SQLite"
    );
}

#[test]
fn state_round_trip_corruption_and_locking() {
    let dir = tempfile::tempdir().expect("directory");
    let path = dir.path().join("state.json");
    save(&path, &FileState::default()).expect("save");
    assert_eq!(load(&path).expect("load").version, 1);
    save(&path, &FileState::default()).expect("replace atomically");
    std::fs::write(&path, "{broken").expect("corrupt");
    assert!(load(&path).expect("fresh").targets.is_empty());
    let lock_path = dir.path().join("lock");
    let first = scheduler::lock(&lock_path).expect("lock");
    assert!(scheduler::lock(&lock_path).is_err());
    drop(first);
    assert!(scheduler::lock(&lock_path).is_ok());
}

#[test]
fn percentile_and_duration_handle_empty_and_unusual_input() {
    assert_eq!(percentile95(&[]), None);
    assert_eq!(percentile95(&[42]), Some(42));
    assert_eq!(percentile95(&(1..=100).collect::<Vec<_>>()), Some(95));
    assert_eq!(since_duration("24h").expect("duration").num_hours(), 24);
    for value in ["", "0h", "-1d", "20λ", "99999999999999999999d"] {
        assert!(since_duration(value).is_err());
    }
}

#[tokio::test]
async fn cli_reports_configuration_and_unknown_target_errors() {
    let dir = tempfile::tempdir().expect("directory");
    let path = dir.path().join("valid.toml");
    std::fs::write(&path, "[[target]]\nname='api'\nurl='http://127.0.0.1:1'").expect("config");
    let output = tokio::process::Command::new(assert_cmd::cargo::cargo_bin!("look-out-api"))
        .arg("--config")
        .arg(&path)
        .arg("validate")
        .output()
        .await
        .expect("validate");
    assert!(output.status.success());
    let output = tokio::process::Command::new(assert_cmd::cargo::cargo_bin!("look-out-api"))
        .arg("--config")
        .arg(&path)
        .args(["check", "--name", "missing"])
        .output()
        .await
        .expect("check");
    assert_eq!(output.status.code(), Some(2));
}
