use look_out_api::{config::Config, server, store::Store};

#[tokio::test]
async fn heartbeat_endpoint_requires_bearer_token_and_known_target() {
    let config = Config::parse("[[target]]\nname='backup'\ntype='heartbeat'\ngrace_secs=60")
        .expect("config");
    let store = Store::connect("sqlite::memory:").await.expect("store");
    store.sync(&config).await.expect("sync");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("listen");
    let address = listener.local_addr().expect("address");
    let app = server::router(store.clone(), Some("heartbeat-test-token".into()));
    let handle = tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve");
    });
    let client = reqwest::Client::new();
    for (name, token, expected) in [
        ("backup", None, 401),
        ("backup", Some("wrong"), 401),
        ("missing", Some("heartbeat-test-token"), 404),
        ("backup", Some("heartbeat-test-token"), 202),
    ] {
        let mut request = client.post(format!("http://{address}/heartbeat/{name}"));
        if let Some(token) = token {
            request = request.bearer_auth(token);
        }
        assert_eq!(
            request.send().await.expect("request").status().as_u16(),
            expected
        );
    }
    assert!(store
        .heartbeat_age("backup")
        .await
        .expect("age")
        .is_some_and(|age| age < 5));
    handle.abort();
}
