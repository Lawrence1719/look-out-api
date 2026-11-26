use crate::error::ConfigError;
use serde::Deserialize;
use std::{
    collections::{BTreeMap, HashSet},
    path::Path,
};

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Defaults {
    pub interval_secs: u64,
    pub timeout_secs: u64,
    pub failure_threshold: u32,
    pub recovery_threshold: u32,
    pub retention_days: u32,
}
impl Default for Defaults {
    fn default() -> Self {
        Self {
            interval_secs: 60,
            timeout_secs: 10,
            failure_threshold: 3,
            recovery_threshold: 2,
            retention_days: 30,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    #[default]
    Http,
    Tcp,
    Dns,
    Flow,
    Security,
    Heartbeat,
}
impl Kind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Http => "http",
            Self::Tcp => "tcp",
            Self::Dns => "dns",
            Self::Flow => "flow",
            Self::Security => "security",
            Self::Heartbeat => "heartbeat",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Step {
    pub name: String,
    pub url: String,
    #[serde(default = "get")]
    pub method: String,
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
    pub json: Option<serde_json::Value>,
    pub expected_status: Option<u16>,
    pub expected_body_contains: Option<String>,
    #[serde(default)]
    pub extract: BTreeMap<String, String>,
}
fn get() -> String {
    "GET".into()
}
fn cert_warn() -> i64 {
    21
}
fn cert_critical() -> i64 {
    7
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Target {
    pub name: String,
    #[serde(default, rename = "type")]
    pub kind: Kind,
    pub url: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
    #[serde(default = "get")]
    pub method: String,
    pub expected_status: Option<u16>,
    pub expected_body_contains: Option<String>,
    pub json: Option<serde_json::Value>,
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
    pub interval_secs: Option<u64>,
    pub timeout_secs: Option<u64>,
    pub failure_threshold: Option<u32>,
    pub recovery_threshold: Option<u32>,
    pub proxy_url: Option<String>,
    pub ca_cert_path: Option<String>,
    #[serde(default)]
    pub insecure_skip_verify: bool,
    #[serde(default)]
    pub expect_ips: Vec<std::net::IpAddr>,
    #[serde(default, rename = "step")]
    pub steps: Vec<Step>,
    pub grace_secs: Option<u64>,
    #[serde(default = "cert_warn")]
    pub cert_warn_days: i64,
    #[serde(default = "cert_critical")]
    pub cert_critical_days: i64,
    #[serde(default)]
    pub required_headers: Vec<String>,
    #[serde(default)]
    pub forbidden_headers: Vec<String>,
    #[serde(default)]
    pub must_be_blocked: Vec<String>,
    #[serde(default)]
    pub protected_paths: Vec<String>,
    #[serde(default)]
    pub cors_must_not_be_wildcard: bool,
}
impl Target {
    pub fn interval(&self, d: &Defaults) -> u64 {
        self.interval_secs.unwrap_or(d.interval_secs)
    }
    pub fn timeout(&self, d: &Defaults) -> u64 {
        self.timeout_secs.unwrap_or(d.timeout_secs)
    }
    pub fn failure(&self, d: &Defaults) -> u32 {
        self.failure_threshold.unwrap_or(d.failure_threshold)
    }
    pub fn recovery(&self, d: &Defaults) -> u32 {
        self.recovery_threshold.unwrap_or(d.recovery_threshold)
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default)]
    pub defaults: Defaults,
    #[serde(rename = "target")]
    pub targets: Vec<Target>,
}
impl Config {
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        let text = std::fs::read_to_string(path).map_err(|_| ConfigError::Read)?;
        Self::parse(&text)
    }
    pub fn parse(text: &str) -> Result<Self, ConfigError> {
        Self::parse_with(text, |key| std::env::var(key).ok())
    }
    pub fn parse_with(
        text: &str,
        env: impl Fn(&str) -> Option<String>,
    ) -> Result<Self, ConfigError> {
        let mut value: toml::Value = toml::from_str(text).map_err(|_| ConfigError::Syntax)?;
        interpolate_value(&mut value, &env)?;
        let config: Self = value.try_into().map_err(|_| ConfigError::Fields)?;
        config.validate()?;
        Ok(config)
    }
    pub fn validate(&self) -> Result<(), ConfigError> {
        let err = |msg: String| ConfigError::Validation(msg);
        if self.targets.is_empty() {
            return Err(err("at least one target is required".into()));
        }
        if self.defaults.interval_secs > 31_536_000
            || self.defaults.interval_secs < 5
            || self.defaults.timeout_secs == 0
            || self.defaults.timeout_secs >= self.defaults.interval_secs
        {
            return Err(err(
                "defaults require interval_secs >= 5 and 0 < timeout_secs < interval_secs".into(),
            ));
        }
        if !(1..=1000).contains(&self.defaults.failure_threshold)
            || !(1..=1000).contains(&self.defaults.recovery_threshold)
        {
            return Err(err("default thresholds must be between 1 and 1000".into()));
        }
        let mut names = HashSet::new();
        for target in &self.targets {
            if !valid_name(&target.name) {
                return Err(err("target names must contain 1–80 ASCII letters, digits, dots, underscores or hyphens".into()));
            }
            let bad = |msg: &str| err(format!("target {}: {msg}", target.name));
            if !names.insert(&target.name) {
                return Err(bad("name must be unique"));
            }
            if target.interval(&self.defaults) > 31_536_000
                || target.interval(&self.defaults) < 5
                || target.timeout(&self.defaults) == 0
                || target.timeout(&self.defaults) >= target.interval(&self.defaults)
            {
                return Err(bad(
                    "require interval_secs >= 5 and 0 < timeout_secs < interval_secs",
                ));
            }
            if !(1..=1000).contains(&target.failure(&self.defaults))
                || !(1..=1000).contains(&target.recovery(&self.defaults))
            {
                return Err(bad("thresholds must be between 1 and 1000"));
            }
            match target.kind {
                Kind::Http | Kind::Security => {
                    validate_url(
                        target
                            .url
                            .as_deref()
                            .ok_or_else(|| bad("url is required"))?,
                    )
                    .map_err(|_| bad("url must use http or https without embedded credentials"))?;
                }
                Kind::Tcp | Kind::Dns => {
                    if target
                        .host
                        .as_ref()
                        .is_none_or(|host| host.is_empty() || host.chars().any(char::is_whitespace))
                    {
                        return Err(bad("host is required and cannot contain whitespace"));
                    }
                    if target.kind == Kind::Tcp && target.port.is_none_or(|p| p == 0) {
                        return Err(bad("a nonzero port is required"));
                    }
                }
                Kind::Flow => {
                    if target.steps.is_empty() {
                        return Err(bad("flow requires at least one step"));
                    }
                    let mut step_names = HashSet::new();
                    for step in &target.steps {
                        if !valid_name(&step.name) || !step_names.insert(&step.name) {
                            return Err(bad("step names must be valid and unique"));
                        }
                        validate_url(&step.url).map_err(|_| {
                            bad("step url must use http or https without embedded credentials")
                        })?;
                        validate_request(&step.method, step.expected_status).map_err(|_| {
                            bad("step requires GET, HEAD or POST and a valid expected_status")
                        })?;
                        for (key, pointer) in &step.extract {
                            if !valid_name(key) || !(pointer.is_empty() || pointer.starts_with('/'))
                            {
                                return Err(bad(
                                    "extract requires valid variable names and JSON pointers",
                                ));
                            }
                        }
                    }
                }
                Kind::Heartbeat => {
                    if target
                        .grace_secs
                        .is_none_or(|g| !(1..=31_536_000).contains(&g))
                    {
                        return Err(bad("heartbeat requires grace_secs >= 1"));
                    }
                }
            }
            validate_request(&target.method, target.expected_status)
                .map_err(|_| bad("requires GET, HEAD or POST and a valid expected_status"))?;
            if target.kind == Kind::Security {
                if target.insecure_skip_verify {
                    return Err(bad("security checks require TLS verification"));
                }
                if target.cert_critical_days < 0
                    || target.cert_warn_days <= target.cert_critical_days
                {
                    return Err(bad("certificate thresholds require 0 <= critical < warn"));
                }
                for path in target.must_be_blocked.iter().chain(&target.protected_paths) {
                    if !path.starts_with('/')
                        || path.starts_with("//")
                        || path.contains('\\')
                        || path.contains('?')
                        || path.contains('#')
                    {
                        return Err(bad("security paths must be absolute paths on the target origin without query or fragment"));
                    }
                }
            }
            if let Some(proxy) = &target.proxy_url {
                let parsed = url::Url::parse(proxy).map_err(|_| bad("proxy URL is invalid"))?;
                if !["http", "https", "socks5", "socks5h"].contains(&parsed.scheme())
                    || parsed.host_str().is_none()
                {
                    return Err(bad("proxy URL must use http, https, socks5 or socks5h"));
                }
            }
        }
        Ok(())
    }
}
fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 80
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
}
fn validate_url(text: &str) -> Result<(), ()> {
    let url = url::Url::parse(text).map_err(|_| ())?;
    if !["http", "https"].contains(&url.scheme())
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err(());
    }
    Ok(())
}
fn validate_request(method: &str, status: Option<u16>) -> Result<(), ()> {
    if !["GET", "HEAD", "POST"].contains(&method)
        || status.is_some_and(|s| !(100..=599).contains(&s))
    {
        return Err(());
    }
    Ok(())
}
fn interpolate_value(
    value: &mut toml::Value,
    env: &impl Fn(&str) -> Option<String>,
) -> Result<(), ConfigError> {
    match value {
        toml::Value::String(text) => *text = interpolate(text, env)?,
        toml::Value::Array(values) => {
            for value in values {
                interpolate_value(value, env)?;
            }
        }
        toml::Value::Table(table) => {
            for (_, value) in table.iter_mut() {
                interpolate_value(value, env)?;
            }
        }
        _ => (),
    }
    Ok(())
}
fn interpolate(text: &str, env: &impl Fn(&str) -> Option<String>) -> Result<String, ConfigError> {
    let mut rest = text;
    let mut out = String::new();
    while let Some(start) = rest.find("${") {
        out.push_str(&rest[..start]);
        let variable = &rest[start + 2..];
        let end = variable
            .find('}')
            .ok_or_else(|| ConfigError::Validation("unterminated environment variable".into()))?;
        let name = &variable[..end];
        if name.is_empty() || !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_') {
            return Err(ConfigError::Validation(
                "invalid environment variable name".into(),
            ));
        }
        out.push_str(&env(name).ok_or_else(|| ConfigError::MissingEnv(name.into()))?);
        rest = &variable[end + 1..];
    }
    out.push_str(rest);
    Ok(out)
}
