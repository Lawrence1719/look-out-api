use crate::{
    alert::{deliver_pending, Telegram},
    config::{Config, Kind},
    dashboard::Dashboard,
    server,
    store::Store,
};
use std::{
    fs::{File, OpenOptions},
    path::Path,
    time::Duration,
};
use tokio::{sync::watch, task::JoinSet, time::MissedTickBehavior};

pub fn lock(path: &Path) -> anyhow::Result<File> {
    let mut options = OpenOptions::new();
    options.read(true).write(true).create(true).truncate(false);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let file = options.open(path)?;
    fs2::FileExt::try_lock_exclusive(&file)
        .map_err(|_| anyhow::anyhow!("another monitor is already using this state or database"))?;
    Ok(file)
}
pub async fn run(
    config: Config,
    store: Store,
    sender: Option<Telegram>,
    bind: Option<String>,
) -> anyhow::Result<()> {
    let heartbeat = config
        .targets
        .iter()
        .any(|target| target.kind == Kind::Heartbeat);
    let token = std::env::var("LOOK_OUT_API_HEARTBEAT_TOKEN")
        .ok()
        .filter(|value| !value.is_empty());
    if heartbeat && token.is_none() {
        anyhow::bail!("heartbeat targets require LOOK_OUT_API_HEARTBEAT_TOKEN");
    }
    let dashboard = Dashboard::new(&config, store.clone())?;
    let listener = if heartbeat || bind.is_some() {
        let address = bind.unwrap_or_else(|| "127.0.0.1:8080".into());
        let listener = tokio::net::TcpListener::bind(&address)
            .await
            .map_err(|_| anyhow::anyhow!("cannot bind status/heartbeat server"))?;
        tracing::info!(bind=%address,"local status/heartbeat server started");
        Some(listener)
    } else {
        None
    };
    store.sync(&config).await?;
    store.prune(config.defaults.retention_days).await?;
    let (shutdown, receiver) = watch::channel(false);
    let mut tasks: JoinSet<anyhow::Result<()>> = JoinSet::new();
    for target in &config.targets {
        let dashboard = dashboard.clone();
        let name = target.name.clone();
        let period = target.interval(&config.defaults);
        let mut stop = receiver.clone();
        tasks.spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(period));
            interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
            tokio::select! { _ = tokio::time::sleep(Duration::from_millis(fastrand::u64(0..1000))) => (), _ = stop.changed() => return Ok(()) }
            loop {
                tokio::select! {
                    _ = stop.changed() => break,
                    _ = interval.tick() => {
                        tokio::select! {
                            _ = stop.changed() => break,
                            result = dashboard.scheduled(&name) => result?,
                        }
                    },
                }
            }
            Ok(())
        });
    }
    if let Some(sender) = sender {
        let store = store.clone();
        let mut stop = receiver.clone();
        tasks.spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(1));
            interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
            loop {
                tokio::select! {
                    _ = stop.changed() => break,
                    _ = interval.tick() => deliver_pending(&store,&sender).await?,
                }
            }
            Ok(())
        });
    } else {
        tracing::warn!("Telegram is not configured; incident notifications will remain queued");
    }
    {
        let store = store.clone();
        let retention = config.defaults.retention_days;
        let mut stop = receiver.clone();
        tasks.spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(86400));
            interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
            interval.tick().await;
            loop {
                tokio::select! { _ = stop.changed() => break, _ = interval.tick() => { store.prune(retention).await?; } }
            }
            Ok(())
        });
    }
    if let Some(listener) = listener {
        let local = listener.local_addr()?.ip().is_loopback();
        let app = server::dashboard_router(store.clone(), token, local.then_some(dashboard));
        let mut stop = receiver.clone();
        tasks.spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async move {
                    let _ = stop.changed().await;
                })
                .await?;
            Ok(())
        });
    }
    tracing::info!(
        targets = config.targets.len(),
        "monitor started; press Ctrl+C to stop"
    );
    let mut failure = None;
    tokio::select! {
        result = shutdown_signal() => result?,
        result = tasks.join_next() => {
            failure = Some(match result { Some(Ok(Err(error))) => error, Some(Err(_)) => anyhow::anyhow!("monitor task failed"), _ => anyhow::anyhow!("monitor task stopped unexpectedly") });
        }
    }
    let _ = shutdown.send(true);
    while let Some(result) = tasks.join_next().await {
        if !matches!(result, Ok(Ok(()))) && failure.is_none() {
            failure = Some(anyhow::anyhow!("monitor task failed during shutdown"));
        }
    }
    store.pool.close().await;
    tracing::info!("monitor stopped; persistent state and notifications retained");
    if let Some(error) = failure {
        return Err(error);
    }
    Ok(())
}
async fn shutdown_signal() -> anyhow::Result<()> {
    #[cfg(unix)]
    {
        let mut terminate =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
        tokio::select! { result=tokio::signal::ctrl_c() => result?, _=terminate.recv() => () }
    }
    #[cfg(not(unix))]
    {
        tokio::signal::ctrl_c().await?;
    }
    Ok(())
}
