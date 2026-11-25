use anyhow::Context;
use clap::Parser;
use look_out_api::{
    alert::Telegram,
    checker::Executor,
    cli::{Cli, Command},
    config::Config,
    report, scheduler, stateless,
    store::Store,
};
use std::{path::PathBuf, str::FromStr};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

#[tokio::main]
async fn main() -> std::process::ExitCode {
    if let Err(error) = dotenvy::dotenv() {
        if !error.not_found() {
            eprintln!("error: .env file could not be loaded");
            return std::process::ExitCode::from(2);
        }
    }
    let cli = Cli::parse();
    let filter = if cli.quiet {
        EnvFilter::new("warn")
    } else {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("look_out_api=info"))
    };
    // Dependency HTTP traces can contain request URLs. Only this crate's safe events are emitted.
    tracing_subscriber::registry()
        .with(filter)
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .with_writer(std::io::stderr)
                .with_filter(tracing_subscriber::filter::filter_fn(|metadata| {
                    metadata.target().starts_with("look_out_api")
                })),
        )
        .init();
    match execute(cli).await {
        Ok(true) => std::process::ExitCode::SUCCESS,
        Ok(false) => std::process::ExitCode::from(1),
        Err(error) => {
            eprintln!("error: {error}");
            std::process::ExitCode::from(2)
        }
    }
}
async fn execute(cli: Cli) -> anyhow::Result<bool> {
    match cli.command {
        Command::Validate => {
            let config = Config::load(&cli.config)?;
            if !cli.quiet {
                println!("Configuration valid: {} target(s).", config.targets.len());
            }
        }
        Command::Check {
            name,
            state_file,
            alert,
            retry_alerts,
        } => {
            let config = Config::load(&cli.config)?;
            if let Some(path) = state_file {
                if name.is_some() {
                    anyhow::bail!("state-file mode checks all configured targets; omit --name");
                }
                let sender = if alert {
                    Some(
                        Telegram::from_env()?
                            .context("--alert requires Telegram bot token and chat ID")?,
                    )
                } else {
                    None
                };
                return stateless::run(&config, &path, sender.as_ref(), cli.quiet, retry_alerts)
                    .await;
            }
            if name
                .as_ref()
                .is_some_and(|name| !config.targets.iter().any(|target| &target.name == name))
            {
                anyhow::bail!("requested target is not configured");
            }
            let heartbeat_store = if config.targets.iter().any(|target| {
                target.kind == look_out_api::config::Kind::Heartbeat
                    && name.as_ref().is_none_or(|name| name == &target.name)
            }) {
                Some(
                    Store::connect(&cli.database)
                        .await
                        .context("cannot read heartbeat database")?,
                )
            } else {
                None
            };
            let mut ok = true;
            if !cli.quiet {
                println!("TARGET                   RESULT  LATENCY  DETAIL");
            }
            for target in &config.targets {
                if name.as_ref().is_some_and(|name| name != &target.name) {
                    continue;
                }
                let result = Executor::new(
                    target.clone(),
                    config.defaults.clone(),
                    heartbeat_store.clone(),
                )?
                .check()
                .await;
                ok &= result.ok;
                if !cli.quiet {
                    println!(
                        "{:<24} {:<7} {:>6} ms {}",
                        target.name,
                        if result.ok { "PASS" } else { "FAIL" },
                        result.latency_ms,
                        result.error.as_deref().unwrap_or("")
                    );
                }
            }
            return Ok(ok);
        }
        Command::Run { bind } => {
            let config = Config::load(&cli.config)?;
            let sender = Telegram::from_env()?;
            let options = sqlx::sqlite::SqliteConnectOptions::from_str(&cli.database)?;
            if options.get_filename().to_string_lossy() == ":memory:" {
                anyhow::bail!("daemon mode requires a persistent SQLite file");
            }
            let lock_path = PathBuf::from(format!("{}.lock", options.get_filename().display()));
            let _lock = scheduler::lock(&lock_path)?;
            let store = Store::connect(&cli.database)
                .await
                .context("cannot initialize monitor database")?;
            scheduler::run(config, store, sender, bind).await?;
        }
        Command::Status => {
            let store = Store::connect(&cli.database)
                .await
                .context("cannot open monitor database")?;
            println!("TARGET                   TYPE       STATUS          LAST OBSERVATION");
            let statuses = store.statuses().await?;
            if statuses.is_empty() {
                println!("No stored targets. Start look-out-api run first.");
            }
            for target in statuses {
                let stale = target
                    .last_checked_at
                    .as_deref()
                    .and_then(|time| chrono::DateTime::parse_from_rfc3339(time).ok())
                    .is_none_or(|time| {
                        (chrono::Utc::now() - time.with_timezone(&chrono::Utc)).num_seconds()
                            > target.interval_secs * 2
                    });
                let label = if stale {
                    "unknown/stale"
                } else {
                    &target.status
                };
                println!(
                    "{:<24} {:<10} {:<15} {}",
                    target.name,
                    target.r#type,
                    label,
                    target.last_checked_at.as_deref().unwrap_or("never")
                );
            }
        }
        Command::History { name, limit } => {
            let store = Store::connect(&cli.database)
                .await
                .context("cannot open monitor database")?;
            println!("CHECKED AT                         OK    STATUS  LATENCY  STEP / DETAIL");
            for row in store.history(&name, limit).await? {
                println!(
                    "{:<34} {:<5} {:<7} {:>6} ms {} {}",
                    row.checked_at,
                    row.ok,
                    row.status_code
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| "-".into()),
                    row.latency_ms,
                    row.failed_step.as_deref().unwrap_or(""),
                    row.error.as_deref().unwrap_or("")
                );
            }
        }
        Command::Report {
            since,
            name,
            security,
        } => {
            let store = Store::connect(&cli.database)
                .await
                .context("cannot open monitor database")?;
            report::print(&store, &since, name.as_deref(), security).await?;
        }
        Command::Alerts { retry } => {
            let store = Store::connect(&cli.database)
                .await
                .context("cannot open monitor database")?;
            if retry {
                println!(
                    "{} notification(s) made eligible for retry by the daemon.",
                    store.retry_notifications().await?
                );
            }
            for notification in store.notifications().await? {
                println!(
                    "{} {} attempts={} blocked={} {}",
                    notification.id,
                    notification.event_key,
                    notification.attempts,
                    notification.permanent_failure,
                    notification.last_error.as_deref().unwrap_or("pending")
                );
            }
        }
    }
    Ok(true)
}
