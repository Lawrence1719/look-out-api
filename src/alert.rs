use crate::store::Store;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct DeliveryError {
    pub reason: &'static str,
    pub permanent: bool,
    pub retry_after: Option<u64>,
}
#[derive(Clone)]
pub struct Telegram {
    client: reqwest::Client,
    token: String,
    chat_id: String,
    base_url: String,
}
impl Telegram {
    pub fn from_env() -> anyhow::Result<Option<Self>> {
        let token = std::env::var("TELEGRAM_BOT_TOKEN").unwrap_or_default();
        let chat = std::env::var("TELEGRAM_CHAT_ID").unwrap_or_default();
        if token.is_empty() && chat.is_empty() {
            return Ok(None);
        }
        if token.is_empty() || chat.is_empty() {
            anyhow::bail!("TELEGRAM_BOT_TOKEN and TELEGRAM_CHAT_ID must both be configured");
        }
        Ok(Some(Self::new(
            token,
            chat,
            std::env::var("TELEGRAM_API_BASE_URL")
                .unwrap_or_else(|_| "https://api.telegram.org".into()),
        )?))
    }
    pub fn new(token: String, chat_id: String, base_url: String) -> anyhow::Result<Self> {
        let parsed = url::Url::parse(&base_url)
            .map_err(|_| anyhow::anyhow!("invalid Telegram API base URL"))?;
        let loopback = parsed.host_str().is_some_and(|host| {
            host == "localhost"
                || host
                    .parse::<std::net::IpAddr>()
                    .is_ok_and(|ip| ip.is_loopback())
        });
        if parsed.scheme() != "https" && !(parsed.scheme() == "http" && loopback) {
            anyhow::bail!("Telegram API must use HTTPS, except on loopback");
        }
        if !parsed.username().is_empty()
            || parsed.password().is_some()
            || parsed.query().is_some()
            || parsed.fragment().is_some()
        {
            anyhow::bail!("Telegram API URL must not contain credentials, query or fragment");
        }
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|_| anyhow::anyhow!("cannot initialize notification client"))?;
        Ok(Self {
            client,
            token,
            chat_id,
            base_url,
        })
    }
    pub async fn send(&self, message: &str) -> Result<(), DeliveryError> {
        let url = format!(
            "{}/bot{}/sendMessage",
            self.base_url.trim_end_matches('/'),
            self.token
        );
        let response = self
            .client
            .post(url)
            .json(&serde_json::json!({"chat_id": self.chat_id, "text": message}))
            .send()
            .await
            .map_err(|_| DeliveryError {
                reason: "notification transport failed; delivery may be uncertain",
                permanent: false,
                retry_after: None,
            })?;
        let status = response.status();
        let bytes = response.bytes().await.map_err(|_| DeliveryError {
            reason: "notification response unreadable; delivery may be uncertain",
            permanent: false,
            retry_after: None,
        })?;
        let body = serde_json::from_slice::<serde_json::Value>(&bytes).unwrap_or_default();
        if status.is_success() && body.get("ok").and_then(|value| value.as_bool()) == Some(true) {
            return Ok(());
        }
        let retry_after = body
            .pointer("/parameters/retry_after")
            .and_then(|value| value.as_u64())
            .map(|delay| delay.min(86400));
        let code = body
            .get("error_code")
            .and_then(|value| value.as_u64())
            .unwrap_or(u64::from(status.as_u16()));
        Err(DeliveryError {
            reason: if code == 429 {
                "notification rate limited"
            } else if [400, 401, 403, 404].contains(&code) {
                "notification configuration rejected; fix credentials or chat and run alerts --retry"
            } else {
                "notification service failed"
            },
            permanent: [400, 401, 403, 404].contains(&code),
            retry_after,
        })
    }
}
pub fn retry_delay(attempts: i64) -> u64 {
    5_u64
        .saturating_mul(2_u64.saturating_pow(attempts.clamp(0, 10) as u32))
        .min(3600)
}
pub async fn deliver_pending(store: &Store, sender: &Telegram) -> anyhow::Result<()> {
    for notification in store.pending().await? {
        match sender.send(&notification.message).await {
            Ok(()) => store.delivered(&notification).await?,
            Err(error) => {
                let delay = error
                    .retry_after
                    .unwrap_or_else(|| retry_delay(notification.attempts) + fastrand::u64(0..5));
                store
                    .delivery_failed(&notification, error.reason, error.permanent, delay)
                    .await?;
                tracing::warn!(
                    notification_id = notification.id,
                    permanent = error.permanent,
                    reason = error.reason,
                    "notification remains queued"
                );
            }
        }
    }
    Ok(())
}
