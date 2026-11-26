use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "look-out-api",
    version,
    about = "API checks, durable incidents, and Telegram alerts"
)]
pub struct Cli {
    #[arg(
        long,
        global = true,
        env = "LOOK_OUT_API_CONFIG",
        default_value = "look-out-api.toml"
    )]
    pub config: PathBuf,
    #[arg(
        long,
        global = true,
        env = "DATABASE_URL",
        default_value = "sqlite://look-out-api.db"
    )]
    pub database: String,
    #[arg(
        long,
        global = true,
        help = "Suppress check tables and informational logs"
    )]
    pub quiet: bool,
    #[command(subcommand)]
    pub command: Command,
}
#[derive(Subcommand)]
pub enum Command {
    /// Start scheduled checks and persistent notification delivery.
    Run {
        #[arg(
            long,
            env = "LOOK_OUT_API_BIND",
            help = "Enable local dashboard/status/heartbeat server, e.g. 127.0.0.1:8080"
        )]
        bind: Option<String>,
    },
    /// Run checks once. Normal mode does not change daemon state.
    Check {
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        state_file: Option<PathBuf>,
        #[arg(long, requires = "state_file")]
        alert: bool,
        #[arg(
            long,
            requires = "alert",
            help = "Retry notifications rejected due to configuration errors"
        )]
        retry_alerts: bool,
    },
    /// Validate configuration without making network requests.
    Validate,
    /// Show stored status; old observations are marked stale.
    Status,
    /// Show sample-based availability, successful latency, and incidents.
    Report {
        #[arg(long, default_value = "24h")]
        since: String,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        security: bool,
    },
    /// Show recent stored observations.
    History {
        #[arg(long)]
        name: String,
        #[arg(long,default_value_t=50,value_parser=clap::value_parser!(u32).range(1..=10000))]
        limit: u32,
    },
    /// Inspect pending notifications or make them eligible for retry.
    Alerts {
        #[arg(long)]
        retry: bool,
    },
}
