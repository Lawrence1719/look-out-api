use crate::{
    checker::CheckResult,
    config::Config,
    state::{advance, State, Status, Transition},
};
use chrono::Utc;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
    FromRow, Row, SqlitePool,
};
use std::{str::FromStr, time::Duration};

#[derive(Clone)]
pub struct Store {
    pub pool: SqlitePool,
}

#[derive(Debug, FromRow)]
pub struct TargetStatus {
    pub name: String,
    pub r#type: String,
    pub status: String,
    pub interval_secs: i64,
    pub last_checked_at: Option<String>,
    pub consecutive_fail: i64,
    pub consecutive_ok: i64,
}
#[derive(Debug, Clone, FromRow)]
pub struct Notification {
    pub id: i64,
    pub target_id: i64,
    pub event_key: String,
    pub kind: String,
    pub message: String,
    pub attempts: i64,
    pub last_error: Option<String>,
    pub permanent_failure: bool,
}
#[derive(Debug, FromRow)]
pub struct HistoryRow {
    pub checked_at: String,
    pub ok: bool,
    pub status_code: Option<i64>,
    pub latency_ms: i64,
    pub failed_step: Option<String>,
    pub error: Option<String>,
}
impl Store {
    pub async fn connect(database: &str) -> anyhow::Result<Self> {
        let options = SqliteConnectOptions::from_str(database)?
            .create_if_missing(true)
            .foreign_keys(true)
            .journal_mode(SqliteJournalMode::Wal)
            .busy_timeout(Duration::from_secs(10));
        let pool = SqlitePoolOptions::new()
            .max_connections(if database.contains(":memory:") { 1 } else { 5 })
            .connect_with(options)
            .await?;
        sqlx::migrate!().run(&pool).await?;
        Ok(Self { pool })
    }
    pub async fn sync(&self, config: &Config) -> anyhow::Result<()> {
        let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;
        sqlx::query("UPDATE targets SET active = 0")
            .execute(&mut *tx)
            .await?;
        for target in &config.targets {
            let id: i64 = sqlx::query_scalar("INSERT INTO targets(name,type,interval_secs,created_at) VALUES(?,?,?,?) ON CONFLICT(name) DO UPDATE SET active=1,type=excluded.type,interval_secs=excluded.interval_secs RETURNING id")
                .bind(&target.name).bind(target.kind.as_str()).bind(target.interval(&config.defaults) as i64).bind(Utc::now().to_rfc3339()).fetch_one(&mut *tx).await?;
            sqlx::query("INSERT INTO target_state(target_id) VALUES(?) ON CONFLICT DO NOTHING")
                .bind(id)
                .execute(&mut *tx)
                .await?;
        }
        tx.commit().await?;
        Ok(())
    }
    pub async fn record(
        &self,
        name: &str,
        result: &CheckResult,
        failure: u32,
        recovery: u32,
    ) -> anyhow::Result<Option<Transition>> {
        let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;
        let row = sqlx::query("SELECT t.id,s.status,s.consecutive_fail,s.consecutive_ok,s.last_changed_at,s.last_checked_at FROM targets t JOIN target_state s ON s.target_id=t.id WHERE t.name=? AND t.active=1")
            .bind(name).fetch_one(&mut *tx).await?;
        let target_id: i64 = row.get("id");
        let previous: String = row.get("status");
        let mut state = State {
            status: match previous.as_str() {
                "up" => Status::Up,
                "down" => Status::Down,
                _ => Status::Unknown,
            },
            consecutive_fail: row.get::<i64, _>("consecutive_fail") as u32,
            consecutive_ok: row.get::<i64, _>("consecutive_ok") as u32,
            last_changed_at: row.get("last_changed_at"),
            last_checked_at: row.get("last_checked_at"),
        };
        let transition = advance(&mut state, result.ok, failure, recovery);
        if transition.is_some() || (previous == "unknown" && state.status == Status::Up) {
            state.last_changed_at = Some(result.checked_at.clone());
        }
        let check_id: i64 = sqlx::query_scalar("INSERT INTO checks(target_id,checked_at,ok,status_code,latency_ms,failed_step,error) VALUES(?,?,?,?,?,?,?) RETURNING id")
            .bind(target_id).bind(&result.checked_at).bind(result.ok).bind(result.status_code.map(i64::from)).bind(result.latency_ms).bind(&result.failed_step).bind(&result.error).fetch_one(&mut *tx).await?;
        for step in &result.steps {
            sqlx::query("INSERT INTO check_steps(check_id,step_name,ok,status_code,latency_ms,error) VALUES(?,?,?,?,?,?)")
                .bind(check_id).bind(&step.name).bind(step.ok).bind(step.status_code.map(i64::from)).bind(step.latency_ms).bind(&step.error).execute(&mut *tx).await?;
        }
        if result.findings_complete {
            sqlx::query("UPDATE security_findings SET resolved_at=? WHERE target_id=? AND resolved_at IS NULL").bind(&result.checked_at).bind(target_id).execute(&mut *tx).await?;
        }
        for finding in &result.findings {
            sqlx::query("INSERT INTO security_findings(target_id,check_id,rule,severity,detail,first_seen_at,last_seen_at) VALUES(?,?,?,?,?,?,?) ON CONFLICT(target_id,rule,detail) DO UPDATE SET severity=excluded.severity,last_seen_at=excluded.last_seen_at,check_id=excluded.check_id,resolved_at=NULL")
                .bind(target_id).bind(check_id).bind(&finding.rule).bind(&finding.severity).bind(&finding.detail).bind(&result.checked_at).bind(&result.checked_at).execute(&mut *tx).await?;
        }
        sqlx::query("UPDATE target_state SET status=?,consecutive_fail=?,consecutive_ok=?,last_changed_at=?,last_checked_at=? WHERE target_id=?")
            .bind(state.status.as_str()).bind(i64::from(state.consecutive_fail)).bind(i64::from(state.consecutive_ok)).bind(&state.last_changed_at).bind(&result.checked_at).bind(target_id).execute(&mut *tx).await?;
        if let Some(event) = transition {
            let (incident_id, duration) = match event {
                Transition::Down => {
                    let id: i64 = sqlx::query_scalar("INSERT INTO incidents(target_id,started_at,reason) VALUES(?,?,?) RETURNING id")
                        .bind(target_id).bind(&result.checked_at).bind(&result.error).fetch_one(&mut *tx).await?;
                    (id, None)
                }
                Transition::Recovered => {
                    let row = sqlx::query("UPDATE incidents SET resolved_at=? WHERE target_id=? AND resolved_at IS NULL RETURNING id,started_at")
                        .bind(&result.checked_at).bind(target_id).fetch_one(&mut *tx).await?;
                    let start: String = row.get("started_at");
                    let elapsed = chrono::DateTime::parse_from_rfc3339(&start)
                        .ok()
                        .zip(chrono::DateTime::parse_from_rfc3339(&result.checked_at).ok())
                        .map(|(start, end)| (end - start).num_seconds().max(0));
                    (row.get("id"), elapsed)
                }
            };
            let duration = duration
                .map(|seconds| format!("; incident duration {seconds}s"))
                .unwrap_or_default();
            let message = format!(
                "{}: {} · incident {}{}",
                event.as_str(),
                name,
                incident_id,
                duration
            );
            let key = format!("incident:{incident_id}:{}", event.as_str());
            sqlx::query("INSERT INTO notifications(target_id,event_key,kind,message,created_at,next_attempt_at) VALUES(?,?,?,?,?,?) ON CONFLICT(event_key) DO NOTHING")
                .bind(target_id).bind(key).bind(event.as_str()).bind(message).bind(&result.checked_at).bind(&result.checked_at).execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(transition)
    }
    pub async fn statuses(&self) -> anyhow::Result<Vec<TargetStatus>> {
        Ok(sqlx::query_as("SELECT t.name,t.type,t.interval_secs,s.status,s.last_checked_at,s.consecutive_fail,s.consecutive_ok FROM targets t JOIN target_state s ON s.target_id=t.id WHERE t.active=1 ORDER BY t.name").fetch_all(&self.pool).await?)
    }
    pub async fn history(&self, name: &str, limit: u32) -> anyhow::Result<Vec<HistoryRow>> {
        Ok(sqlx::query_as("SELECT c.checked_at,c.ok,c.status_code,c.latency_ms,c.failed_step,c.error FROM checks c JOIN targets t ON c.target_id=t.id WHERE t.name=? ORDER BY c.id DESC LIMIT ?").bind(name).bind(i64::from(limit)).fetch_all(&self.pool).await?)
    }
    pub async fn pending(&self) -> anyhow::Result<Vec<Notification>> {
        Ok(sqlx::query_as("SELECT n.id,n.target_id,n.event_key,n.kind,n.message,n.attempts,n.last_error,n.permanent_failure FROM notifications n WHERE n.delivered_at IS NULL AND n.permanent_failure=0 AND n.next_attempt_at <= ? AND NOT EXISTS(SELECT 1 FROM notifications earlier WHERE earlier.target_id=n.target_id AND earlier.id<n.id AND earlier.delivered_at IS NULL) ORDER BY n.id LIMIT 20")
            .bind(Utc::now().to_rfc3339()).fetch_all(&self.pool).await?)
    }
    pub async fn notifications(&self) -> anyhow::Result<Vec<Notification>> {
        Ok(sqlx::query_as("SELECT id,target_id,event_key,kind,message,attempts,last_error,permanent_failure FROM notifications WHERE delivered_at IS NULL ORDER BY id").fetch_all(&self.pool).await?)
    }
    pub async fn delivered(&self, notification: &Notification) -> anyhow::Result<()> {
        let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?;
        let now = Utc::now().to_rfc3339();
        sqlx::query("UPDATE notifications SET delivered_at=?,last_error=NULL,attempts=attempts+1 WHERE id=? AND delivered_at IS NULL").bind(&now).bind(notification.id).execute(&mut *tx).await?;
        sqlx::query("UPDATE target_state SET last_alerted_at=? WHERE target_id=?")
            .bind(now)
            .bind(notification.target_id)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(())
    }
    pub async fn delivery_failed(
        &self,
        notification: &Notification,
        reason: &str,
        permanent: bool,
        delay_secs: u64,
    ) -> anyhow::Result<()> {
        let next = (Utc::now() + chrono::Duration::seconds(delay_secs as i64)).to_rfc3339();
        sqlx::query("UPDATE notifications SET attempts=attempts+1,last_error=?,permanent_failure=?,next_attempt_at=? WHERE id=? AND delivered_at IS NULL")
            .bind(reason).bind(permanent).bind(next).bind(notification.id).execute(&self.pool).await?;
        Ok(())
    }
    pub async fn retry_notifications(&self) -> anyhow::Result<u64> {
        Ok(sqlx::query("UPDATE notifications SET permanent_failure=0,next_attempt_at=? WHERE delivered_at IS NULL").bind(Utc::now().to_rfc3339()).execute(&self.pool).await?.rows_affected())
    }
    pub async fn ping(&self, name: &str) -> anyhow::Result<bool> {
        let count = sqlx::query("INSERT INTO heartbeats(target_id,last_ping_at) SELECT id,? FROM targets WHERE name=? AND type='heartbeat' AND active=1 ON CONFLICT(target_id) DO UPDATE SET last_ping_at=excluded.last_ping_at")
            .bind(Utc::now().to_rfc3339()).bind(name).execute(&self.pool).await?.rows_affected();
        Ok(count > 0)
    }
    pub async fn heartbeat_age(&self, name: &str) -> anyhow::Result<Option<i64>> {
        let timestamp: Option<String> = sqlx::query_scalar("SELECT COALESCE(h.last_ping_at,t.created_at) FROM targets t LEFT JOIN heartbeats h ON h.target_id=t.id WHERE t.name=? AND t.type='heartbeat' AND t.active=1").bind(name).fetch_optional(&self.pool).await?;
        Ok(timestamp
            .and_then(|time| chrono::DateTime::parse_from_rfc3339(&time).ok())
            .map(|time| (Utc::now() - time.with_timezone(&Utc)).num_seconds().max(0)))
    }
    pub async fn prune(&self, days: u32) -> anyhow::Result<u64> {
        if days == 0 {
            return Ok(0);
        }
        let cutoff = (Utc::now() - chrono::Duration::days(i64::from(days))).to_rfc3339();
        Ok(sqlx::query("DELETE FROM checks WHERE checked_at < ?")
            .bind(cutoff)
            .execute(&self.pool)
            .await?
            .rows_affected())
    }
}
