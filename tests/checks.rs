use look_out_api::{checker::Executor, config::Config};
use wiremock::{
    matchers::{header, method, path},
    Mock, MockServer, ResponseTemplate,
};

fn config(url: &str, extra: &str) -> Config {
    Config::parse(&format!("[[target]]\nname='api'\nurl='{url}'\n{extra}")).expect("config")
}
#[tokio::test]
async fn http_assertions_and_errors_are_safe() {
    let server = MockServer::start().await;
    Mock::given(path("/ok"))
        .respond_with(ResponseTemplate::new(200).set_body_string("all ok"))
        .mount(&server)
        .await;
    Mock::given(path("/bad"))
        .respond_with(ResponseTemplate::new(500).set_body_string("private-response-secret"))
        .mount(&server)
        .await;
    for (endpoint, extra, expected) in [
        ("/ok", "expected_body_contains='ok'", true),
        ("/ok", "expected_body_contains='missing'", false),
        ("/bad?token=private-url-secret", "", false),
    ] {
        let config = config(&format!("{}{endpoint}", server.uri()), extra);
        let result = Executor::new(config.targets[0].clone(), config.defaults, None)
            .expect("executor")
            .check()
            .await;
        assert_eq!(result.ok, expected);
        let rendered = serde_json::to_string(&result).expect("render");
        assert!(!rendered.contains("private-"));
    }
}
#[tokio::test]
async fn timeout_is_a_failed_observation() {
    let server = MockServer::start().await;
    Mock::given(path("/slow"))
        .respond_with(ResponseTemplate::new(200).set_delay(std::time::Duration::from_secs(2)))
        .mount(&server)
        .await;
    let config = config(&format!("{}/slow", server.uri()), "timeout_secs=1");
    let result = Executor::new(config.targets[0].clone(), config.defaults, None)
        .expect("executor")
        .check()
        .await;
    assert!(!result.ok);
    assert!(result
        .error
        .as_deref()
        .expect("reason")
        .contains("timed out"));
}
#[tokio::test]
async fn flow_reuses_token_and_names_failed_step_without_storing_secrets() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/login"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"data":{"token":"private-extracted-secret"}})),
        )
        .mount(&server)
        .await;
    Mock::given(path("/orders"))
        .and(header("authorization", "Bearer private-extracted-secret"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"orders":[]})))
        .mount(&server)
        .await;
    let text = format!(
        r#"[[target]]
name='flow'
type='flow'
[[target.step]]
name='login'
method='POST'
url='{}/login'
extract={{token='/data/token'}}
[[target.step]]
name='orders'
url='{}/orders'
headers={{Authorization='Bearer {{{{token}}}}'}}
expected_body_contains='orders'
"#,
        server.uri(),
        server.uri()
    );
    let config = Config::parse(&text).expect("config");
    let result = Executor::new(config.targets[0].clone(), config.defaults.clone(), None)
        .expect("executor")
        .check()
        .await;
    assert!(result.ok, "{result:?}");
    assert_eq!(result.steps.len(), 2);
    assert!(!serde_json::to_string(&result)
        .expect("render")
        .contains("private-extracted-secret"));
    let broken = text.replace("/data/token", "/data/missing");
    let config = Config::parse(&broken).expect("config");
    let result = Executor::new(config.targets[0].clone(), config.defaults, None)
        .expect("executor")
        .check()
        .await;
    assert!(!result.ok);
    assert_eq!(result.failed_step.as_deref(), Some("login"));
}
#[tokio::test]
async fn tcp_and_dns_use_local_services() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("listen");
    let port = listener.local_addr().expect("address").port();
    let config = Config::parse(&format!("[[target]]\nname='tcp'\ntype='tcp'\nhost='127.0.0.1'\nport={port}\n[[target]]\nname='dns'\ntype='dns'\nhost='localhost'\nexpect_ips=['127.0.0.1']")).expect("config");
    for target in &config.targets {
        assert!(
            Executor::new(target.clone(), config.defaults.clone(), None)
                .expect("executor")
                .check()
                .await
                .ok
        );
    }
    drop(listener);
    assert!(
        !Executor::new(config.targets[0].clone(), config.defaults, None)
            .expect("executor")
            .check()
            .await
            .ok
    );
}
#[tokio::test]
async fn security_probes_are_read_only_and_find_exposure() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).insert_header("access-control-allow-origin", "*"))
        .mount(&server)
        .await;
    let config = config(&server.uri(),"type='security'\nrequired_headers=['strict-transport-security']\nmust_be_blocked=['/.env']\nprotected_paths=['/orders']\ncors_must_not_be_wildcard=true");
    let result = Executor::new(config.targets[0].clone(), config.defaults, None)
        .expect("executor")
        .check()
        .await;
    assert!(!result.ok);
    assert!(result.findings_complete);
    for rule in [
        "missing_header",
        "exposed_path",
        "protected_path",
        "cors_wildcard",
        "https_redirect",
    ] {
        assert!(result.findings.iter().any(|f| f.rule == rule), "{rule}");
    }
    for request in server.received_requests().await.expect("requests") {
        assert_eq!(request.method.as_str(), "GET");
        assert!(request.body.is_empty());
    }
}

#[tokio::test]
async fn failed_login_stops_before_the_next_flow_step() {
    let server = MockServer::start().await;
    Mock::given(path("/login"))
        .respond_with(ResponseTemplate::new(401).set_body_string("private-login-error"))
        .mount(&server)
        .await;
    let text = format!("[[target]]\nname='flow'\ntype='flow'\n[[target.step]]\nname='login'\nmethod='POST'\nurl='{}/login'\n[[target.step]]\nname='orders'\nurl='{}/orders'",server.uri(),server.uri());
    let config = Config::parse(&text).expect("config");
    let result = Executor::new(config.targets[0].clone(), config.defaults, None)
        .expect("executor")
        .check()
        .await;
    assert!(!result.ok);
    assert_eq!(result.failed_step.as_deref(), Some("login"));
    assert_eq!(result.steps.len(), 1);
    assert_eq!(server.received_requests().await.expect("requests").len(), 1);
    assert!(!serde_json::to_string(&result)
        .expect("render")
        .contains("private-login-error"));
}
