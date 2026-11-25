//! Safe local dashboard snapshots and the execution gate shared with the scheduler.
use crate::{
    checker::{CheckResult, Executor, StepResult},
    config::Config,
    store::{HistoryRow, Store, TargetStatus},
};
use chrono::Utc;
use serde::Serialize;
use sqlx::FromRow;
use std::{collections::BTreeMap, sync::Arc};
use tokio::sync::{watch, Mutex, Semaphore};

#[derive(Clone)]
pub struct Dashboard {
    targets: Arc<BTreeMap<String, Runner>>,
    store: Store,
    capacity: Arc<Semaphore>,
}
struct Runner {
    executor: Executor,
    gate: Mutex<()>,
    activity: watch::Sender<Option<Activity>>,
    manual: watch::Sender<Option<CheckResult>>,
    metadata: Metadata,
}
#[derive(Clone, Serialize)]
pub struct Activity {
    source: &'static str,
    started_at: String,
}
#[derive(Serialize)]
pub struct Snapshot {
    server_time: String,
    pub targets: Vec<TargetView>,
}
#[derive(Clone, Serialize)]
struct Metadata {
    timeout_secs: u64,
    failure_threshold: u32,
    recovery_threshold: u32,
    expected_status: Option<u16>,
    body_assertion: bool,
    flow_steps: usize,
}
#[derive(Serialize)]
pub struct TargetView {
    #[serde(flatten)]
    state: TargetStatus,
    #[serde(flatten)]
    metadata: Metadata,
    stale: bool,
    pub activity: Option<Activity>,
    manual_result: Option<CheckResult>,
    history: Vec<HistoryRow>,
    steps: Vec<StepResult>,
    incidents: Vec<Incident>,
    pending_alerts: i64,
}
#[derive(Serialize, FromRow)]
struct Incident {
    id: i64,
    started_at: String,
    resolved_at: Option<String>,
}
#[derive(FromRow)]
struct StoredStep {
    step_name: String,
    ok: bool,
    status_code: Option<i64>,
    latency_ms: i64,
    error: Option<String>,
}
#[derive(Debug)]
pub enum RunError {
    Missing,
    Busy,
}
struct ActivityGuard<'a>(&'a watch::Sender<Option<Activity>>);
impl Drop for ActivityGuard<'_> {
    fn drop(&mut self) {
        self.0.send_replace(None);
    }
}
impl Dashboard {
    pub fn new(config: &Config, store: Store) -> anyhow::Result<Self> {
        let mut targets = BTreeMap::new();
        for target in &config.targets {
            let executor =
                Executor::new(target.clone(), config.defaults.clone(), Some(store.clone()))?;
            let (activity, _) = watch::channel(None);
            let (manual, _) = watch::channel(None);
            targets.insert(
                target.name.clone(),
                Runner {
                    executor,
                    gate: Mutex::new(()),
                    activity,
                    manual,
                    metadata: Metadata {
                        timeout_secs: target.timeout(&config.defaults),
                        failure_threshold: target.failure(&config.defaults),
                        recovery_threshold: target.recovery(&config.defaults),
                        expected_status: target.expected_status,
                        body_assertion: target.expected_body_contains.is_some(),
                        flow_steps: target.steps.len(),
                    },
                },
            );
        }
        Ok(Self {
            targets: Arc::new(targets),
            store,
            capacity: Arc::new(Semaphore::new(16)),
        })
    }
    pub async fn manual(&self, name: &str) -> Result<CheckResult, RunError> {
        let runner = self.targets.get(name).ok_or(RunError::Missing)?;
        let _gate = runner.gate.try_lock().map_err(|_| RunError::Busy)?;
        let _permit = self.capacity.try_acquire().map_err(|_| RunError::Busy)?;
        let _activity = runner.start("manual");
        let result = runner.executor.check().await;
        runner.manual.send_replace(Some(result.clone()));
        Ok(result)
    }
    pub async fn scheduled(&self, name: &str) -> anyhow::Result<()> {
        let runner = self
            .targets
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("target is not configured"))?;
        let _gate = runner.gate.lock().await;
        let _permit = self.capacity.acquire().await?;
        let _activity = runner.start("scheduled");
        let result = runner.executor.check().await;
        let transition = self
            .store
            .record(
                name,
                &result,
                runner.metadata.failure_threshold,
                runner.metadata.recovery_threshold,
            )
            .await?;
        tracing::info!(target_name=%name,ok=result.ok,latency_ms=result.latency_ms,"check completed");
        if let Some(transition) = transition {
            tracing::info!(target_name=%name,transition=transition.as_str(),"incident transition queued");
        }
        Ok(())
    }
    pub async fn snapshot(&self) -> anyhow::Result<Snapshot> {
        let mut tx = self.store.pool.begin().await?;
        let states: Vec<TargetStatus> = sqlx::query_as("SELECT t.name,t.type,t.interval_secs,s.status,s.last_checked_at,s.consecutive_fail,s.consecutive_ok FROM targets t JOIN target_state s ON s.target_id=t.id WHERE t.active=1 ORDER BY t.name").fetch_all(&mut *tx).await?;
        let now = Utc::now();
        let mut targets = Vec::new();
        for state in states {
            let Some(runner) = self.targets.get(&state.name) else {
                continue;
            };
            let history = sqlx::query_as("SELECT c.checked_at,c.ok,c.status_code,c.latency_ms,c.failed_step,c.error FROM checks c JOIN targets t ON c.target_id=t.id WHERE t.name=? ORDER BY c.id DESC LIMIT 24").bind(&state.name).fetch_all(&mut *tx).await?;
            let step_rows: Vec<StoredStep> = sqlx::query_as("SELECT step_name,ok,status_code,latency_ms,error FROM check_steps WHERE check_id=(SELECT c.id FROM checks c JOIN targets t ON c.target_id=t.id WHERE t.name=? ORDER BY c.id DESC LIMIT 1) ORDER BY id").bind(&state.name).fetch_all(&mut *tx).await?;
            let steps = step_rows
                .into_iter()
                .map(|step| StepResult {
                    name: step.step_name,
                    ok: step.ok,
                    status_code: step.status_code.map(|status| status as u16),
                    latency_ms: step.latency_ms,
                    error: step.error,
                })
                .collect();
            let incidents = sqlx::query_as("SELECT i.id,i.started_at,i.resolved_at FROM incidents i JOIN targets t ON i.target_id=t.id WHERE t.name=? ORDER BY i.id DESC LIMIT 5").bind(&state.name).fetch_all(&mut *tx).await?;
            let pending_alerts = sqlx::query_scalar("SELECT COUNT(*) FROM notifications n JOIN targets t ON n.target_id=t.id WHERE t.name=? AND n.delivered_at IS NULL").bind(&state.name).fetch_one(&mut *tx).await?;
            let stale = state
                .last_checked_at
                .as_deref()
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .is_none_or(|time| {
                    (now - time.with_timezone(&Utc)).num_seconds() > state.interval_secs * 2
                });
            targets.push(TargetView {
                metadata: runner.metadata.clone(),
                activity: runner.activity.borrow().clone(),
                manual_result: runner.manual.borrow().clone(),
                state,
                stale,
                history,
                steps,
                incidents,
                pending_alerts,
            });
        }
        tx.commit().await?;
        Ok(Snapshot {
            server_time: now.to_rfc3339(),
            targets,
        })
    }
}
impl Runner {
    fn start(&self, source: &'static str) -> ActivityGuard<'_> {
        self.activity.send_replace(Some(Activity {
            source,
            started_at: Utc::now().to_rfc3339(),
        }));
        ActivityGuard(&self.activity)
    }
}
