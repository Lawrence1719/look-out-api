//! Safe local dashboard snapshots and the execution gate shared with the scheduler.
use crate::{
    checker::{CheckResult, Executor, StepResult},
    config::{Config, Defaults, Target},
    registration::{AddError, ConfigEditor, NewApi},
    store::{HistoryRow, Store, TargetStatus},
};
use chrono::Utc;
use serde::Serialize;
use sqlx::FromRow;
use std::{collections::BTreeMap, path::PathBuf, sync::Arc};
use tokio::sync::{watch, Mutex, RwLock, Semaphore};

#[derive(Clone)]
pub struct Dashboard {
    targets: Arc<RwLock<BTreeMap<String, Arc<Runner>>>>,
    defaults: Defaults,
    editor: Option<Arc<ConfigEditor>>,
    add_unavailable: Option<&'static str>,
    registrations: Arc<Mutex<()>>,
    schedules: watch::Sender<Vec<Schedule>>,
    store: Store,
    capacity: Arc<Semaphore>,
}
#[derive(Clone)]
pub struct Schedule {
    pub name: String,
    pub interval_secs: u64,
}
#[derive(Serialize)]
pub struct AddedApi {
    pub name: String,
    pub monitoring: bool,
    pub message: &'static str,
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
    add_api_enabled: bool,
    add_api_unavailable: Option<&'static str>,
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
        let mut schedules = Vec::new();
        for target in &config.targets {
            targets.insert(
                target.name.clone(),
                Arc::new(Runner::new(target, &config.defaults, store.clone())?),
            );
            schedules.push(Schedule {
                name: target.name.clone(),
                interval_secs: target.interval(&config.defaults),
            });
        }
        let (schedules, _) = watch::channel(schedules);
        Ok(Self {
            targets: Arc::new(RwLock::new(targets)),
            defaults: config.defaults.clone(),
            editor: None,
            add_unavailable: Some("This monitor does not have a writable configuration file."),
            registrations: Arc::new(Mutex::new(())),
            schedules,
            store,
            capacity: Arc::new(Semaphore::new(16)),
        })
    }
    pub fn with_config_file(mut self, path: PathBuf, running: &Config) -> Self {
        match ConfigEditor::open(path, running) {
            Ok(editor) => {
                self.editor = Some(Arc::new(editor));
                self.add_unavailable = None;
            }
            Err(error) => self.add_unavailable = Some(error.message()),
        }
        self
    }
    pub fn subscribe_schedules(&self) -> watch::Receiver<Vec<Schedule>> {
        self.schedules.subscribe()
    }
    pub async fn add_api(&self, input: NewApi) -> Result<AddedApi, AddError> {
        input.validate()?;
        let editor = self.editor.clone().ok_or(AddError::Disabled)?;
        let _registration = self.registrations.try_lock().map_err(|_| AddError::Busy)?;
        if self.targets.read().await.contains_key(&input.name) {
            return Err(AddError::Duplicate);
        }
        let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM targets WHERE name=?)")
            .bind(&input.name)
            .fetch_one(&self.store.pool)
            .await
            .map_err(|_| AddError::Storage)?;
        if exists {
            return Err(AddError::Duplicate);
        }
        let block = input.block()?;
        let mut standalone = Config::parse(&block)
            .map_err(|_| AddError::Invalid("url", "The API configuration is invalid."))?;
        let target = standalone.targets.remove(0);
        let runner = Runner::new(&target, &self.defaults, self.store.clone())
            .map_err(|_| AddError::Storage)?;
        tokio::task::spawn_blocking(move || editor.append(&block))
            .await
            .map_err(|_| AddError::Storage)??;
        if self
            .store
            .register_target(&target, &self.defaults)
            .await
            .is_err()
        {
            return Ok(AddedApi { name: target.name, monitoring: false, message: "API saved, but monitoring could not start. Restart the daemon after checking database access." });
        }
        let schedule = Schedule {
            name: target.name.clone(),
            interval_secs: target.interval(&self.defaults),
        };
        self.targets
            .write()
            .await
            .insert(target.name.clone(), Arc::new(runner));
        let monitoring = self.schedules.receiver_count() > 0;
        self.schedules
            .send_modify(|schedules| schedules.push(schedule));
        Ok(AddedApi {
            name: target.name,
            monitoring,
            message: if monitoring {
                "API saved. Monitoring starts now."
            } else {
                "API saved. Restart the daemon to start scheduled monitoring."
            },
        })
    }
    pub async fn manual(&self, name: &str) -> Result<CheckResult, RunError> {
        let runner = self
            .targets
            .read()
            .await
            .get(name)
            .cloned()
            .ok_or(RunError::Missing)?;
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
            .read()
            .await
            .get(name)
            .cloned()
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
        let runners = self.targets.read().await.clone();
        let mut tx = self.store.pool.begin().await?;
        let states: Vec<TargetStatus> = sqlx::query_as("SELECT t.name,t.type,t.interval_secs,s.status,s.last_checked_at,s.consecutive_fail,s.consecutive_ok FROM targets t JOIN target_state s ON s.target_id=t.id WHERE t.active=1 ORDER BY t.name").fetch_all(&mut *tx).await?;
        let now = Utc::now();
        let mut targets = Vec::new();
        for state in states {
            let Some(runner) = runners.get(&state.name) else {
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
            add_api_enabled: self.editor.is_some(),
            add_api_unavailable: self.add_unavailable,
            targets,
        })
    }
}
impl Runner {
    fn new(target: &Target, defaults: &Defaults, store: Store) -> anyhow::Result<Self> {
        let (activity, _) = watch::channel(None);
        let (manual, _) = watch::channel(None);
        Ok(Self {
            executor: Executor::new(target.clone(), defaults.clone(), Some(store))?,
            gate: Mutex::new(()),
            activity,
            manual,
            metadata: Metadata {
                timeout_secs: target.timeout(defaults),
                failure_threshold: target.failure(defaults),
                recovery_threshold: target.recovery(defaults),
                expected_status: target.expected_status,
                body_assertion: target.expected_body_contains.is_some(),
                flow_steps: target.steps.len(),
            },
        })
    }
    fn start(&self, source: &'static str) -> ActivityGuard<'_> {
        self.activity.send_replace(Some(Activity {
            source,
            started_at: Utc::now().to_rfc3339(),
        }));
        ActivityGuard(&self.activity)
    }
}
