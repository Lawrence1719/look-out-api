# Look Out API

A self-hosted Rust CLI and daemon for checking API health and configured response behavior. Keep observations in SQLite and send Telegram notifications when a target goes down or recovers. A separate static Next.js website introduces the project and documents setup.

**Status:** initial development build. Six check types and the monitoring core are implemented. CI checks Rust on Windows, macOS, and Linux, and the website on Linux. Live Telegram DOWN/RECOVERED delivery was verified with the local demo. There is no verified tagged binary release or production monitor deployment yet. The complete roadmap remains in [spec.md](./spec.md).

## Start on your operating system

| Operating system | Setup guide |
| --- | --- |
| Windows (PowerShell) | [Install, configure, and run](./docs/getting-started/windows.md) |
| macOS (Terminal) | [Install, configure, and run](./docs/getting-started/macos.md) |
| Linux (Debian/Ubuntu examples) | [Install, configure, and run](./docs/getting-started/linux.md) |

The [documentation index](./docs/README.md) also covers [Telegram setup](./docs/telegram.md), [SQLite, commands, backups, and updates](./docs/usage.md), and [troubleshooting](./docs/troubleshooting.md). The monitor is self-hosted; each user supplies their own targets and optional Telegram bot. Database creation is automatic. Python is used for the optional demo; Node.js is needed only for the information website.

## Quick start on macOS / Linux

Install stable Rust and your OS build tools using the guides above. If Cargo is not available after installing with rustup, reopen the terminal or load `. "$HOME/.cargo/env"` in zsh/bash. From the source checkout:

```sh
cargo build --release --locked
test -f look-out-api.toml || cp look-out-api.example.toml look-out-api.toml
test -f .env || cp .env.example .env
chmod 600 .env look-out-api.toml
./target/release/look-out-api validate
```

The example checks a local demo API. In a separate terminal:

```sh
python3 examples/demo_api.py
```

Then:

```sh
./target/release/look-out-api check
./target/release/look-out-api run
```

Stop with Ctrl+C. The daemon also handles SIGTERM. State and pending notifications survive restart. Normal one-off checks do not change the daemon's incident state.

## Configure targets

```toml
[defaults]
interval_secs = 60
timeout_secs = 10
failure_threshold = 3
recovery_threshold = 2
retention_days = 30

[[target]]
name = "local-api"
url = "http://127.0.0.1:3001/health"
expected_status = 200
expected_body_contains = "ok"
interval_secs = 30
```

Names must be unique, nonsecret labels containing ASCII letters, digits, dots, hyphens, or underscores. Intervals are between 5 seconds and one year; timeouts must be positive and shorter than the interval. Failure and recovery thresholds are between 1 and 1000. Defaults merge with each target's overrides. HTTP status defaults to any 2xx.

Config strings support `${ENVIRONMENT_VARIABLE}` interpolation, including headers and JSON values. Interpolation happens after TOML parsing so quotation marks in a secret do not break the config. Missing variables fail validation. Unknown fields are rejected.

### Check coverage

| Type | Initial implementation |
| --- | --- |
| `http` | GET/HEAD/POST, status and expected text, timeout, JSON request body, latency |
| `tcp` | Connection acceptance with timeout |
| `dns` | System resolver and optional expected IPv4/IPv6 addresses |
| `flow` | Ordered HTTP steps, JSON pointer extraction, variable reuse, per-step latency and failure labels |
| `security` | TLS validation/expiry, required/forbidden headers, HTTPS redirect, configured sensitive/protected paths, wildcard CORS |
| `heartbeat` | Authenticated push endpoint, grace period, persisted last ping |

Postgres/Redis protocol checks, gRPC, browser journeys, CNAME assertions, TLS-version enumeration, verbose-error/RDAP checks, and finding-specific new/escalation notifications are **planned**. The initial security target follows ordinary availability transitions; findings are stored and can be reported separately. Security checks do not establish that an application is secure. Functional coverage is limited to the assertions and workflows you configure; structured JSON/business-rule assertions are future work.

### Multi-step example

This runs against the included local demo; use a dedicated low-privilege account for a real service.

```toml
[[target]]
name = "login-then-orders"
type = "flow"

[[target.step]]
name = "login"
method = "POST"
url = "http://127.0.0.1:3001/login"
extract = { token = "/access_token" }

[[target.step]]
name = "orders"
url = "http://127.0.0.1:3001/orders"
headers = { Authorization = "Bearer {{token}}" }
expected_status = 200
expected_body_contains = "orders"
```

Response bodies are limited to 1 MiB for HTTP/flow checks. Errors retain categorical reasons, step labels, status codes and timings, not request URLs, response bodies or extracted values.

## Telegram and durable incidents

Set both `TELEGRAM_BOT_TOKEN` and `TELEGRAM_CHAT_ID` in your environment or an untracked `.env`. [.env.example](./.env.example) lists supported settings. Follow the [Telegram guide](./docs/telegram.md) to create a bot, choose a private chat or group, retrieve its chat ID, and verify alerts. The integration sends outbound alerts; it does not reply to `/start` or other commands. Restart the daemon after changing settings. The optional `TELEGRAM_API_BASE_URL` supports a custom Bot API origin; remote origins require HTTPS, and loopback HTTP is permitted for tests.

Initial success establishes UP silently. Three consecutive failures open an incident by default; two consecutive successes close it. Continued failures do not enqueue repeated DOWN events.

Each observation, state update, incident transition, and pending notification is written in one transaction. A unique event key prevents duplicate local enqueueing. A separate worker delivers outside the transaction, retries temporary errors with bounded exponential backoff and jitter, and preserves order per target. Missing Telegram credentials leave notifications queued. Rejected credentials or chat configuration remain visible until corrected:

```sh
look-out-api alerts
look-out-api alerts --retry
```

A lost delivery response can cause a duplicate on retry; exactly-once external delivery is not promised. Only one daemon can hold the lock for a database file. Each target's checks run sequentially with skipped missed intervals; up to 16 target checks execute concurrently.

## Commands

```sh
look-out-api validate
look-out-api check --name local-api
look-out-api run
look-out-api status
look-out-api history --name local-api --limit 50
look-out-api report --since 24h
look-out-api report --security
look-out-api alerts
```

Use `./target/release/look-out-api` unless the binary is installed on your PATH. Global options: `--config PATH`, `--database sqlite://PATH`, `--quiet`. Equivalent configuration environment variables are in the example environment file.

Checks return 0 when successful and 1 on failure; configuration/execution errors return 2. Reports label availability as **sample-based**, exclude missing observations from claims about healthy time, and calculate average/p95 latency from successful checks. Stale observations are displayed as unknown after two configured intervals.

### State-file / CI mode

```sh
look-out-api check --state-file state.json --alert --quiet
```

This uses no SQLite. Preserve the state file between process runs. It stores counters, status, timestamps, and pending notifications. A lock prevents concurrent writers; writes use a synced temporary file and atomic rename. State and outbox are saved before delivery attempts and after each delivery result. Missing/corrupt state resets to unknown with a warning. All configured targets are checked; `--name` is not allowed. Exit 0 requires every configured target to be UP; DOWN and not-yet-established UNKNOWN return 1.

Use `--retry-alerts` together with `--alert` to retry notifications previously rejected for configuration errors. Heartbeats require persistent daemon mode. Retaining state is essential: missing state can repeat an already reported incident. A state file is not a substitute for durable storage when strict continuity is required.

## Private services and heartbeats

Run the daemon inside the network or supply a per-target `proxy_url`. A global `LOOK_OUT_API_HTTP_PROXY` is also supported for HTTP-backed checks. Add `ca_cert_path` for an internal CA. `insecure_skip_verify` is per target, defaults off, warns on every check, and is rejected for security targets.

```toml
[[target]]
name = "nightly-backup"
type = "heartbeat"
grace_secs = 90000
```

Set `LOOK_OUT_API_HEARTBEAT_TOKEN`. The daemon automatically enables its endpoint at `127.0.0.1:8080` for heartbeat targets; override using `--bind` or `LOOK_OUT_API_BIND` when needed. Send `POST /heartbeat/nightly-backup` with `Authorization: Bearer <token>`. A new target receives an initial grace period starting at creation. Subsequent scheduled checks compare the latest ping to `grace_secs`, using the same failure/recovery thresholds. Existing heartbeat targets can be inspected by normal one-off checks using the same database.

The optional HTTP server includes a [local dashboard](./docs/dashboard.md) and `/healthz`. Start with `run --bind 127.0.0.1:8080` and open http://127.0.0.1:8080 to watch real scheduled observations, inspect incidents, run manual tests of configured targets, and add HTTP APIs without restarting. Manual tests do not change incident state or send alerts. See the [deployment guide](./deploy/README.md) for host, container, network, backup, and GitHub setup.

## Website

Requires Node.js 24 and npm:

```sh
npm --prefix website ci
npm --prefix website run dev
```

Open `http://127.0.0.1:3000`. The site uses Next.js App Router, TypeScript, Motion for React, Lucide icons, and locally hosted Manrope, DM Sans and IBM Plex Mono fonts. The interactive sequence is illustrative and never calls a live monitor. Animations honor reduced-motion preferences and have playback controls.

```sh
npm --prefix website run typecheck
npm --prefix website run build
npm --prefix website test
```

Static files are exported to [website/out](./website/out). Production builds accept `NEXT_PUBLIC_BASE_PATH`, `NEXT_PUBLIC_SITE_URL` (origin only), and `NEXT_PUBLIC_REPOSITORY_URL`. GitHub Pages sets those through the publishing workflow. The sharing image is generated locally from the bundled Manrope font and SVG; its font license is included alongside the generator.

## Checks and delivery

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
npm --prefix website run format:check
npm --prefix website run typecheck
npm --prefix website run build
cd website
npx playwright install chromium
npm test
```

Rust tests use local mock servers and listeners, including a locally signed TLS certificate. Browser tests cover interactions, keyboard navigation, clipboard behavior, mobile overflow, reduced motion, and automated accessibility checks.

GitHub Actions runs Rust checks across Linux/macOS/Windows and the website checks on pull requests and `main`. After main CI succeeds, a separate workflow publishes the information site to Pages. Version tags build release archives. The optional scheduled second watcher preserves state between runs using unique cache keys. See [deployment setup](./deploy/README.md) and [verification](./docs/design/VERIFICATION.md) for pending live setup.

## Ownership and contribution

Personal project. The repository-local commit email is `lawrence.dizon@proton.me`; global Git identity is unchanged. The public repository is [Lawrence1719/look-out-api](https://github.com/Lawrence1719/look-out-api). Work on feature branches and use pull requests. Released under the [MIT License](./LICENSE); bundled fonts retain their [included licenses](./website/public/licenses/), which are exported with the site. Crate publishing remains disabled until release preparation.

No usage telemetry is enabled. After publication, repository activity and release-download counts can provide approximate adoption signals without collecting monitored target data.
