use crate::{
    config::{Defaults, Kind, Target},
    store::Store,
};
use async_trait::async_trait;
use chrono::Utc;
use futures_util::StreamExt;
use reqwest::{Client, Method};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    time::{Duration, Instant},
};

const MAX_BODY: usize = 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub name: String,
    pub ok: bool,
    pub status_code: Option<u16>,
    pub latency_ms: i64,
    pub error: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub rule: String,
    pub severity: String,
    pub detail: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub ok: bool,
    pub status_code: Option<u16>,
    pub latency_ms: i64,
    pub error: Option<String>,
    pub checked_at: String,
    pub failed_step: Option<String>,
    pub steps: Vec<StepResult>,
    pub findings: Vec<Finding>,
    pub findings_complete: bool,
}
impl CheckResult {
    pub fn success() -> Self {
        Self {
            ok: true,
            status_code: None,
            latency_ms: 0,
            error: None,
            checked_at: Utc::now().to_rfc3339(),
            failed_step: None,
            steps: vec![],
            findings: vec![],
            findings_complete: false,
        }
    }
    pub fn failure(reason: impl Into<String>) -> Self {
        Self {
            ok: false,
            error: Some(reason.into()),
            ..Self::success()
        }
    }
}
#[async_trait]
pub trait Check: Send + Sync {
    async fn run(&self) -> CheckResult;
}

#[derive(Clone)]
pub struct Executor {
    pub target: Target,
    defaults: Defaults,
    client: Client,
    store: Option<Store>,
}
impl Executor {
    pub fn new(target: Target, defaults: Defaults, store: Option<Store>) -> anyhow::Result<Self> {
        let mut builder = Client::builder()
            .timeout(Duration::from_secs(target.timeout(&defaults)))
            .redirect(reqwest::redirect::Policy::limited(5))
            .tls_info(true)
            .user_agent(concat!("look-out-api/", env!("CARGO_PKG_VERSION")));
        if let Some(proxy) = target
            .proxy_url
            .clone()
            .or_else(|| std::env::var("LOOK_OUT_API_HTTP_PROXY").ok())
            .filter(|p| !p.is_empty())
        {
            builder = builder.proxy(
                reqwest::Proxy::all(proxy)
                    .map_err(|_| anyhow::anyhow!("invalid HTTP proxy configuration"))?,
            );
        }
        if let Some(path) = &target.ca_cert_path {
            let bytes = std::fs::read(path)
                .map_err(|_| anyhow::anyhow!("cannot read target CA certificate"))?;
            let cert = reqwest::Certificate::from_pem(&bytes)
                .map_err(|_| anyhow::anyhow!("target CA certificate is invalid"))?;
            builder = builder.add_root_certificate(cert);
        }
        builder = builder.danger_accept_invalid_certs(target.insecure_skip_verify);
        let client = builder
            .build()
            .map_err(|_| anyhow::anyhow!("cannot initialize target HTTP client"))?;
        Ok(Self {
            target,
            defaults,
            client,
            store,
        })
    }
    pub async fn check(&self) -> CheckResult {
        let started = Instant::now();
        if self.target.insecure_skip_verify {
            tracing::warn!(target_name = %self.target.name, "TLS verification disabled for this check");
        }
        let mut result = tokio::time::timeout(
            Duration::from_secs(self.target.timeout(&self.defaults)),
            self.run(),
        )
        .await
        .unwrap_or_else(|_| CheckResult::failure("check timed out"));
        result.latency_ms = started.elapsed().as_millis().min(i64::MAX as u128) as i64;
        result.checked_at = Utc::now().to_rfc3339();
        result
    }
    async fn http(&self) -> CheckResult {
        let Some(url) = self.target.url.as_deref() else {
            return CheckResult::failure("HTTP URL is missing");
        };
        match request(
            &self.client,
            &self.target.method,
            url,
            &self.target.headers,
            self.target.json.as_ref(),
            self.target.expected_status,
            self.target.expected_body_contains.as_deref(),
        )
        .await
        {
            Ok((status, _)) => CheckResult {
                status_code: Some(status),
                ..CheckResult::success()
            },
            Err((status, message)) => CheckResult {
                status_code: status,
                ..CheckResult::failure(message)
            },
        }
    }
    async fn tcp(&self) -> CheckResult {
        let Some(host) = &self.target.host else {
            return CheckResult::failure("TCP host is missing");
        };
        let Some(port) = self.target.port else {
            return CheckResult::failure("TCP port is missing");
        };
        match tokio::net::TcpStream::connect((host.as_str(), port)).await {
            Ok(_) => CheckResult::success(),
            Err(_) => CheckResult::failure("TCP connection failed"),
        }
    }
    async fn dns(&self) -> CheckResult {
        let Some(host) = &self.target.host else {
            return CheckResult::failure("DNS host is missing");
        };
        match tokio::net::lookup_host((host.as_str(), 0)).await {
            Ok(addresses) => {
                let ips: Vec<_> = addresses.map(|a| a.ip()).collect();
                if ips.is_empty() {
                    CheckResult::failure("DNS returned no addresses")
                } else if self.target.expect_ips.iter().any(|ip| !ips.contains(ip)) {
                    CheckResult::failure("DNS expected address was not returned")
                } else {
                    CheckResult::success()
                }
            }
            Err(_) => CheckResult::failure("DNS resolution failed"),
        }
    }
    async fn heartbeat(&self) -> CheckResult {
        let Some(store) = &self.store else {
            return CheckResult::failure("heartbeat checks require daemon persistence");
        };
        let age = store.heartbeat_age(&self.target.name).await;
        match age {
            Ok(Some(age)) if age <= self.target.grace_secs.unwrap_or(0) as i64 => {
                CheckResult::success()
            }
            Ok(_) => CheckResult::failure("heartbeat grace period exceeded"),
            Err(_) => CheckResult::failure("cannot read heartbeat state"),
        }
    }
    async fn flow(&self) -> CheckResult {
        let mut variables = BTreeMap::new();
        let mut result = CheckResult::success();
        for step in &self.target.steps {
            let started = Instant::now();
            let expanded = (|| -> Result<_, String> {
                let url = template(&step.url, &variables)?;
                let headers = step
                    .headers
                    .iter()
                    .map(|(key, value)| Ok((key.clone(), template(value, &variables)?)))
                    .collect::<Result<BTreeMap<_, _>, String>>()?;
                let body = step
                    .json
                    .as_ref()
                    .map(|json| template_json(json, &variables))
                    .transpose()?;
                Ok((url, headers, body))
            })();
            let outcome = match expanded {
                Ok((url, headers, json)) => {
                    request(
                        &self.client,
                        &step.method,
                        &url,
                        &headers,
                        json.as_ref(),
                        step.expected_status,
                        step.expected_body_contains.as_deref(),
                    )
                    .await
                }
                Err(reason) => Err((None, reason)),
            };
            let (status, failure) = match outcome {
                Ok((status, bytes)) => {
                    if step.extract.is_empty() {
                        (Some(status), None)
                    } else {
                        match serde_json::from_slice::<serde_json::Value>(&bytes) {
                            Ok(body) => {
                                let mut missing = false;
                                for (key, pointer) in &step.extract {
                                    match body.pointer(pointer) {
                                        Some(value) if !value.is_null() => {
                                            variables.insert(
                                                key.clone(),
                                                value
                                                    .as_str()
                                                    .map(String::from)
                                                    .unwrap_or_else(|| value.to_string()),
                                            );
                                        }
                                        _ => missing = true,
                                    }
                                }
                                (
                                    Some(status),
                                    missing.then(|| "flow extraction value is missing".to_string()),
                                )
                            }
                            Err(_) => (
                                Some(status),
                                Some("flow extraction requires a JSON response".into()),
                            ),
                        }
                    }
                }
                Err((status, reason)) => (status, Some(reason)),
            };
            result.steps.push(StepResult {
                name: step.name.clone(),
                ok: failure.is_none(),
                status_code: status,
                latency_ms: started.elapsed().as_millis().min(i64::MAX as u128) as i64,
                error: failure.clone(),
            });
            result.status_code = status;
            if let Some(reason) = failure {
                result.ok = false;
                result.error = Some(reason);
                result.failed_step = Some(step.name.clone());
                break;
            }
        }
        result
    }
    async fn security(&self) -> CheckResult {
        let Some(url_text) = &self.target.url else {
            return CheckResult::failure("security URL is missing");
        };
        let Ok(base) = url::Url::parse(url_text) else {
            return CheckResult::failure("security URL is invalid");
        };
        let response = match self.client.get(base.clone()).send().await {
            Ok(response) => response,
            Err(error) => return CheckResult::failure(http_error(&error)),
        };
        let mut result = CheckResult {
            status_code: Some(response.status().as_u16()),
            ..CheckResult::success()
        };
        for header in &self.target.required_headers {
            if !response.headers().contains_key(header) {
                result.findings.push(finding(
                    "missing_header",
                    "medium",
                    &format!("required header {header} is absent"),
                ));
            }
        }
        for header in &self.target.forbidden_headers {
            if response.headers().contains_key(header) {
                result.findings.push(finding(
                    "leaky_header",
                    "low",
                    &format!("forbidden header {header} is present"),
                ));
            }
        }
        if self.target.cors_must_not_be_wildcard
            && response
                .headers()
                .get("access-control-allow-origin")
                .is_some_and(|v| v == "*")
        {
            result.findings.push(finding(
                "cors_wildcard",
                "medium",
                "wildcard origin is allowed",
            ));
        }
        if base.scheme() == "https" {
            if let Some(der) = response
                .extensions()
                .get::<reqwest::tls::TlsInfo>()
                .and_then(|info| info.peer_certificate())
            {
                if let Ok((_, cert)) = x509_parser::parse_x509_certificate(der) {
                    let days =
                        (cert.validity().not_after.timestamp() - Utc::now().timestamp()) / 86400;
                    if days <= self.target.cert_warn_days {
                        let severity = if days <= self.target.cert_critical_days {
                            "high"
                        } else {
                            "medium"
                        };
                        result.findings.push(finding(
                            "tls_cert_expiry",
                            severity,
                            "certificate is inside the configured expiry threshold",
                        ));
                    }
                }
            }
        }
        let mut http_url = base.clone();
        if base.scheme() == "https" {
            let _ = http_url.set_scheme("http");
        }
        tokio::time::sleep(Duration::from_millis(150)).await;
        match self.client.get(http_url).send().await {
            Ok(redirect) if redirect.url().scheme() == "https" => (),
            _ => result.findings.push(finding(
                "https_redirect",
                "medium",
                "HTTP request did not reach HTTPS",
            )),
        }
        for (protected, paths) in [
            (false, &self.target.must_be_blocked),
            (true, &self.target.protected_paths),
        ] {
            for (index, path) in paths.iter().enumerate() {
                tokio::time::sleep(Duration::from_millis(150)).await;
                let mut probe = base.clone();
                probe.set_path(path);
                probe.set_query(None);
                probe.set_fragment(None);
                match self.client.get(probe).send().await {
                    Ok(probe) => {
                        let status = probe.status().as_u16();
                        let allowed = if protected {
                            [401, 403].contains(&status)
                        } else {
                            [401, 403, 404].contains(&status)
                        };
                        if !allowed {
                            result.findings.push(finding(
                                if protected {
                                    "protected_path"
                                } else {
                                    "exposed_path"
                                },
                                "high",
                                &format!(
                                    "configured path probe {} returned an unexpected status",
                                    index + 1
                                ),
                            ));
                        }
                    }
                    Err(_) => {
                        result.ok = false;
                        result.error = Some("security path probe failed".into());
                    }
                }
            }
        }
        result.findings_complete = result.error.is_none();
        if !result.findings.is_empty() {
            result.ok = false;
            result.error = Some(format!(
                "{} passive security finding(s)",
                result.findings.len()
            ));
        }
        result
    }
}
#[async_trait]
impl Check for Executor {
    async fn run(&self) -> CheckResult {
        match self.target.kind {
            Kind::Http => self.http().await,
            Kind::Tcp => self.tcp().await,
            Kind::Dns => self.dns().await,
            Kind::Flow => self.flow().await,
            Kind::Security => self.security().await,
            Kind::Heartbeat => self.heartbeat().await,
        }
    }
}
fn finding(rule: &str, severity: &str, detail: &str) -> Finding {
    Finding {
        rule: rule.into(),
        severity: severity.into(),
        detail: detail.into(),
    }
}
fn http_error(error: &reqwest::Error) -> String {
    if error.is_timeout() {
        "HTTP request timed out"
    } else if error.is_connect() {
        "HTTP connection or TLS validation failed"
    } else if error.is_redirect() {
        "HTTP redirect limit exceeded"
    } else {
        "HTTP request failed"
    }
    .into()
}
async fn request(
    client: &Client,
    method: &str,
    url: &str,
    headers: &BTreeMap<String, String>,
    json: Option<&serde_json::Value>,
    expected: Option<u16>,
    contains: Option<&str>,
) -> Result<(u16, Vec<u8>), (Option<u16>, String)> {
    let method =
        Method::from_bytes(method.as_bytes()).map_err(|_| (None, "invalid HTTP method".into()))?;
    let mut request = client.request(method, url);
    for (key, value) in headers {
        request = request.header(key, value);
    }
    if let Some(json) = json {
        request = request.json(json);
    }
    let response = request.send().await.map_err(|e| (None, http_error(&e)))?;
    let status = response.status().as_u16();
    let status_ok = expected.map_or((200..300).contains(&status), |expected| status == expected);
    if !status_ok {
        return Err((Some(status), "unexpected HTTP status".into()));
    }
    let mut bytes = Vec::new();
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|_| (Some(status), "cannot read HTTP response".into()))?;
        if bytes.len().saturating_add(chunk.len()) > MAX_BODY {
            return Err((Some(status), "HTTP response exceeds 1 MiB limit".into()));
        }
        bytes.extend_from_slice(&chunk);
    }
    if contains.is_some_and(|needle| !String::from_utf8_lossy(&bytes).contains(needle)) {
        return Err((Some(status), "expected response text was not found".into()));
    }
    Ok((status, bytes))
}
fn template(text: &str, values: &BTreeMap<String, String>) -> Result<String, String> {
    let mut rest = text;
    let mut output = String::new();
    while let Some(start) = rest.find("{{") {
        output.push_str(&rest[..start]);
        let variable = &rest[start + 2..];
        let end = variable
            .find("}}")
            .ok_or("flow variable template is incomplete")?;
        output.push_str(
            values
                .get(&variable[..end])
                .ok_or("flow variable is missing")?,
        );
        rest = &variable[end + 2..];
    }
    output.push_str(rest);
    Ok(output)
}
fn template_json(
    value: &serde_json::Value,
    variables: &BTreeMap<String, String>,
) -> Result<serde_json::Value, String> {
    Ok(match value {
        serde_json::Value::String(text) => serde_json::Value::String(template(text, variables)?),
        serde_json::Value::Array(array) => serde_json::Value::Array(
            array
                .iter()
                .map(|v| template_json(v, variables))
                .collect::<Result<_, _>>()?,
        ),
        serde_json::Value::Object(object) => serde_json::Value::Object(
            object
                .iter()
                .map(|(k, v)| Ok((k.clone(), template_json(v, variables)?)))
                .collect::<Result<_, String>>()?,
        ),
        value => value.clone(),
    })
}
