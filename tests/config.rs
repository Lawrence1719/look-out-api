use look_out_api::config::Config;

#[test]
fn validates_defaults_interpolation_and_target_specific_fields() {
    let config = Config::parse_with("[[target]]\nname='api'\nurl='https://example.com/health'\nheaders={Authorization='Bearer ${TOKEN}'}",|key| (key=="TOKEN").then(|| "sensitive'\"value".into())).expect("config");
    assert_eq!(
        config.targets[0].headers["Authorization"],
        "Bearer sensitive'\"value"
    );
    assert_eq!(config.targets[0].interval(&config.defaults), 60);
    assert!(
        Config::parse("[[target]]\nname='db'\ntype='tcp'\nhost='127.0.0.1'\nport=5432").is_ok()
    );
    assert!(Config::parse("[[target]]\nname='backup'\ntype='heartbeat'\ngrace_secs=90000").is_ok());
}
#[test]
fn rejects_invalid_targets_without_echoing_secrets() {
    let invalid = [
        "[[target]]\nname='api'\nurl='ftp://example.com'",
        "[[target]]\nname='api'\nurl='https://user:secret@example.com'",
        "[[target]]\nname='api'\nurl='https://example.com'\ninterval_secs=4",
        "[[target]]\nname='api'\nurl='https://example.com'\ntimeout_secs=60",
        "[[target]]\nname='api'\nurl='https://example.com'\nfailure_threshold=0",
        "[[target]]\nname='api'\ntype='tcp'\nhost='localhost'",
        "[[target]]\nname='api'\ntype='flow'",
        "[[target]]\nname='api'\ntype='security'\nurl='https://example.com'\nprotected_paths=['//evil.example.com']",
        "[[target]]\nname='api'\nurl='https://example.com'\n[[target]]\nname='api'\nurl='https://example.com'",
        "[[target]]\nname='api'\nurl='https://example.com'\nexpectd_status=200",
    ];
    for text in invalid {
        let error = Config::parse(text).expect_err("reject").to_string();
        assert!(!error.contains("user:secret"));
    }
}
#[test]
fn missing_environment_is_clear_and_syntax_does_not_echo_content() {
    let error = Config::parse_with("[[target]]\nname='api'\nurl='${ABSENT}'", |_| None)
        .expect_err("missing");
    assert!(error.to_string().contains("ABSENT"));
    let error = Config::parse("secret-token = [broken").expect_err("syntax");
    assert!(!error.to_string().contains("secret-token"));
}
