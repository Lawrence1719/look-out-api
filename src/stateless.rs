use crate::{
    alert::{retry_delay, Telegram},
    checker::Executor,
    config::{Config, Kind},
    scheduler,
    state::{advance, State, Status},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashSet},
    fs::OpenOptions,
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pending {
    pub target: String,
    pub event_key: String,
    pub message: String,
    pub attempts: i64,
    pub next_attempt_at: i64,
    pub permanent_failure: bool,
    pub last_error: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileState {
    pub version: u32,
    pub next_event: u64,
    pub targets: BTreeMap<String, State>,
    pub pending: Vec<Pending>,
}
impl Default for FileState {
    fn default() -> Self {
        Self {
            version: 1,
            next_event: 1,
            targets: BTreeMap::new(),
            pending: vec![],
        }
    }
}
pub fn load(path: &Path) -> anyhow::Result<FileState> {
    match std::fs::read(path) {
        Ok(bytes) => match serde_json::from_slice::<FileState>(&bytes) {
            Ok(state) if state.version == 1 && state.next_event > 0 => Ok(state),
            _ => {
                tracing::warn!("state file is corrupt or unsupported; starting with unknown state");
                Ok(FileState::default())
            }
        },
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            tracing::warn!("state file is missing; starting with unknown state");
            Ok(FileState::default())
        }
        Err(_) => anyhow::bail!("cannot read state file"),
    }
}
pub fn save(path: &Path, state: &FileState) -> anyhow::Result<()> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let temp = parent.join(format!(
        ".look-out-state-{}-{}.tmp",
        std::process::id(),
        fastrand::u64(..)
    ));
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let result = (|| -> anyhow::Result<()> {
        let mut file = options.open(&temp)?;
        file.write_all(&serde_json::to_vec_pretty(state)?)?;
        file.sync_all()?;
        drop(file);
        std::fs::rename(&temp, path)?;
        #[cfg(unix)]
        {
            std::fs::File::open(parent)?.sync_all()?;
        }
        Ok(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temp);
    }
    result
}
pub async fn run(
    config: &Config,
    path: &Path,
    sender: Option<&Telegram>,
    quiet: bool,
    retry_alerts: bool,
) -> anyhow::Result<bool> {
    if config
        .targets
        .iter()
        .any(|target| target.kind == Kind::Heartbeat)
    {
        anyhow::bail!("heartbeat targets require daemon mode and SQLite");
    }
    let _lock = scheduler::lock(&PathBuf::from(format!("{}.lock", path.display())))?;
    let mut state = load(path)?;
    if retry_alerts {
        for pending in &mut state.pending {
            pending.permanent_failure = false;
            pending.next_attempt_at = 0;
        }
    }
    for target in &config.targets {
        let executor = Executor::new(target.clone(), config.defaults.clone(), None)?;
        let result = executor.check().await;
        let current = state.targets.entry(target.name.clone()).or_default();
        let previous_status = current.status;
        let previous_changed = current.last_changed_at.clone();
        let transition = advance(
            current,
            result.ok,
            target.failure(&config.defaults),
            target.recovery(&config.defaults),
        );
        current.last_checked_at = Some(result.checked_at.clone());
        if transition.is_some() || previous_status != current.status {
            current.last_changed_at = Some(result.checked_at.clone());
        }
        if let Some(event) = transition {
            if sender.is_some() {
                let event_key = format!("{}:{}", target.name, state.next_event);
                state.next_event = state
                    .next_event
                    .checked_add(1)
                    .ok_or_else(|| anyhow::anyhow!("state event counter exhausted"))?;
                let duration = previous_changed
                    .and_then(|time| chrono::DateTime::parse_from_rfc3339(&time).ok())
                    .filter(|_| event.as_str() == "RECOVERED")
                    .map(|start| {
                        format!(
                            "; incident duration {}s",
                            (Utc::now() - start.with_timezone(&Utc))
                                .num_seconds()
                                .max(0)
                        )
                    })
                    .unwrap_or_default();
                state.pending.push(Pending {
                    target: target.name.clone(),
                    event_key: event_key.clone(),
                    message: format!(
                        "{}: {} · event {}{}",
                        event.as_str(),
                        target.name,
                        event_key,
                        duration
                    ),
                    attempts: 0,
                    next_attempt_at: 0,
                    permanent_failure: false,
                    last_error: None,
                });
            }
        }
        if !quiet {
            println!(
                "{:<24} {:<7} {:>6} ms {}",
                target.name,
                if result.ok { "PASS" } else { "FAIL" },
                result.latency_ms,
                result.error.as_deref().unwrap_or("")
            );
        }
    }
    // Persist the transition and outbox before making an external delivery attempt.
    save(path, &state)?;
    if let Some(sender) = sender {
        let mut blocked = HashSet::new();
        let mut index = 0;
        while index < state.pending.len() {
            let pending = state.pending[index].clone();
            if !blocked.insert(pending.target.clone()) {
                index += 1;
                continue;
            }
            if pending.permanent_failure || pending.next_attempt_at > Utc::now().timestamp() {
                index += 1;
                continue;
            }
            match sender.send(&pending.message).await {
                Ok(()) => {
                    state.pending.remove(index);
                    blocked.remove(&pending.target);
                }
                Err(error) => {
                    let pending = &mut state.pending[index];
                    pending.next_attempt_at = Utc::now().timestamp()
                        + error
                            .retry_after
                            .unwrap_or_else(|| retry_delay(pending.attempts))
                            as i64;
                    pending.attempts = pending.attempts.saturating_add(1);
                    pending.permanent_failure = error.permanent;
                    pending.last_error = Some(error.reason.into());
                    tracing::warn!(
                        reason = error.reason,
                        "stateless notification remains queued"
                    );
                    index += 1;
                }
            }
            save(path, &state)?;
        }
    }
    Ok(config.targets.iter().all(|target| {
        state
            .targets
            .get(&target.name)
            .is_some_and(|state| state.status == Status::Up)
    }))
}
