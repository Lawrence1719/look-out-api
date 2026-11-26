# spec.md — `look-out-api`: API Health & Security Checker (Rust)

**Name:** Look Out API. Repository, crate, and binary name: `look-out-api`; Rust crate identifier: `look_out_api`. Before publishing to crates.io or GitHub, run `cargo search look-out-api` and check the GitHub org/repo name; choose an alternative if the intended name is unavailable.
**Constraint:** free tiers only (see section 11). No paid services.
**Builder:** this spec is handed to Codex/Claude Code for implementation.
**Ownership:** personal project, owned by the builder's personal GitHub account. Use `lawrence.dizon@proton.me` as the repository-local Git commit email when Git is initialized; preserve global Git identity settings. Verify the personal GitHub account before creating or pushing the remote repository.

A single-binary CLI + daemon that checks configured targets on a schedule, stores results in SQLite, and alerts via Telegram when a service goes down or recovers. A separate Next.js static information site explains and documents the tool.

**Implementation status (2026-11-25):** the initial build implements HTTP, TCP, DNS, flows, heartbeats, initial passive security checks, persistence, durable notification delivery, reports, state-file runs, and the static website. Postgres/Redis protocol checks, gRPC, browser checks, and expanded/finding-specific security alerting remain roadmap work. The checklist below describes the full project; the initial build is not a completed release. See [README](./README.md) and [verification](./docs/design/VERIFICATION.md) for current behavior and evidence.

## 1. Goals / Non-goals

**Goals**
- Learn Rust foundations (ownership, errors, traits), async (`tokio`), HTTP (`reqwest`), persistence (`sqlx` + SQLite), CLI (`clap`).
- Portfolio-quality tool: tested, documented, CI-ready, releasable as a single static binary.
- A public static information website that introduces the project and helps people install and use it through a shareable link.

**Non-goals (v1)**
- No multi-tenant, no auth, no cloud DB.
- No distributed checking from multiple regions.
- Not a vulnerability scanner: no exploit payloads, fuzzing, or brute-forcing.

## 2. Tech Stack

| Concern | Crate |
|---|---|
| Async runtime | `tokio` (features: full) |
| HTTP client | `reqwest` (rustls-tls, no default features) |
| CLI | `clap` (derive) |
| Config | `serde`, `toml` |
| DB | `sqlx` (sqlite, runtime-tokio, migrate) |
| Errors | `thiserror` (lib), `anyhow` (bin) |
| Logging | `tracing`, `tracing-subscriber` |
| Time | `chrono` |
| Status server (phase 5) | `axum` |
| Tests | built-in, `wiremock`, `assert_cmd`, `tempfile` |

## 3. Project Structure

```
look-out-api/
  Cargo.toml
  look-out-api.example.toml
  .env.example
  migrations/
    0001_init.sql
  src/
    main.rs          # entry, wires CLI -> commands
    cli.rs           # clap definitions
    config.rs        # load/validate TOML
    checker/
      mod.rs         # Check trait + CheckResult + factory by type
      http.rs
      tcp.rs
      dns.rs
      db.rs          # postgres, redis
      grpc.rs
      flow.rs        # multi-step, variable extraction
      browser.rs     # cfg(feature = "browser")
      security.rs    # passive security rules
      heartbeat.rs
    scheduler.rs     # interval loop, concurrency
    store.rs         # sqlx queries
    alert.rs         # persistent notification delivery + Telegram
    state.rs         # pure threshold state machine
    stateless.rs     # atomic state-file runner
    report.rs        # uptime / latency stats
    server.rs        # axum status page (phase 5)
    error.rs
  tests/
    integration_check.rs
    cli.rs
  website/          # public static information site, deployed independently
```

## 4. Config File (`look-out-api.toml`)

```toml
[defaults]
interval_secs = 60
timeout_secs = 10
failure_threshold = 3      # consecutive failures before DOWN alert
recovery_threshold = 2     # consecutive successes before RECOVERED alert

[[target]]
name = "my-api"
url = "https://api.example.com/health"
method = "GET"             # GET | HEAD | POST
expected_status = 200      # optional, default: any 2xx
expected_body_contains = "ok"   # optional
interval_secs = 30         # optional override
headers = { Authorization = "Bearer ${API_TOKEN}" }  # env interpolation
```

Validation rules: unique `name`, valid URL (http/https), `interval_secs >= 5`, `timeout_secs < interval_secs`.

### 4.1 Check Types

Each target has a `type` (default `http`). All types produce the same `CheckResult`, so scheduling, storage, and alerting are shared. Implement via a `Check` trait: `async fn run(&self) -> CheckResult`.

| type | What it checks | Crate |
|---|---|---|
| `http` | status, body, latency (default) | `reqwest` |
| `tcp` | port accepts a connection (covers databases, SMTP, Redis, etc.) | `tokio::net` |
| `dns` | name resolves, optionally to an expected IP/CNAME | `hickory-resolver` |
| `postgres` / `redis` | real protocol handshake plus a trivial query (`SELECT 1`, `PING`) | `sqlx`, `redis` |
| `grpc` | standard `grpc.health.v1.Health/Check` | `tonic`, `tonic-health` |
| `flow` | multi-step HTTP sequence (login, extract token, call API) | `reqwest`, `serde_json` |
| `browser` | loads a JS-rendered page in headless Chrome and asserts on the DOM | `chromiumoxide` (cargo feature `browser`) |
| `security` | passive security checks (see 4.3) | `reqwest`, `rustls`, `x509-parser` |

```toml
[[target]]
name = "db-primary"
type = "tcp"
host = "10.0.0.5"
port = 5432

[[target]]
name = "app-dns"
type = "dns"
host = "app.example.com"
expect_ips = ["203.0.113.10"]

[[target]]
name = "orders-db"
type = "postgres"
url = "${ORDERS_DB_URL}"       # read-only user, runs SELECT 1

[[target]]
name = "payments-grpc"
type = "grpc"
url = "http://payments.internal:50051"
service = ""                   # empty = overall server health
```

### 4.2 Multi-step Flows (`type = "flow"`)

Steps run in order and share a variable map. A step can extract values from the JSON response (JSON pointer) and later steps reference them as `{{var}}`. The flow fails on the first failed step, and the result records which step failed and its latency. Total latency and per-step latency are both stored.

```toml
[[target]]
name = "login-then-orders"
type = "flow"
interval_secs = 120

  [[target.step]]
  name = "login"
  method = "POST"
  url = "https://api.example.com/auth/login"
  json = { email = "${LOOK_OUT_API_USER}", password = "${LOOK_OUT_API_PASS}" }
  expected_status = 200
  extract = { token = "/data/access_token" }

  [[target.step]]
  name = "list-orders"
  method = "GET"
  url = "https://api.example.com/orders"
  headers = { Authorization = "Bearer {{token}}" }
  expected_status = 200
  expected_body_contains = "orders"
```

Rules: use a dedicated low-privilege test account; never log extracted tokens or secrets; mask `{{var}}` values in stored errors.

### 4.3 Security Checks (`type = "security"`)

Passive, read-only checks. Only run against systems you own or have written permission to test. Each check yields a finding with severity (`info`, `low`, `medium`, `high`) and alerts follow the same transition and threshold logic as availability.

```toml
[[target]]
name = "my-api-security"
type = "security"
url = "https://api.example.com"
interval_secs = 21600          # every 6 hours is plenty
cert_warn_days = 21
cert_critical_days = 7
required_headers = ["strict-transport-security", "x-content-type-options", "content-security-policy"]
forbidden_headers = ["x-powered-by", "server"]   # flag if they leak versions
must_be_blocked = ["/.env", "/.git/config", "/admin", "/swagger", "/debug"]  # expect 401/403/404
protected_paths = ["/orders"]  # must return 401/403 without credentials
cors_must_not_be_wildcard = true
```

Checks implemented:
- TLS certificate expiry (days remaining), chain validity, hostname match
- Minimum TLS version (flag TLS 1.0/1.1)
- HTTP to HTTPS redirect
- Required security headers present; leaky headers absent
- Sensitive paths not exposed (status 200 is a finding)
- Protected paths enforce auth without a token
- CORS wildcard on non-public endpoints
- Verbose error or stack trace in a deliberately bad request response
- Domain expiry (optional, via RDAP)

Out of scope (use ZAP, Nuclei, `cargo audit` instead): SQL injection, XSS, access-control logic flaws, dependency CVEs. The tool must never send exploit payloads, brute-force, or fuzz.

### 4.4 Private / Internal Targets

`look-out-api` can only check what it can reach, so support these deployment patterns:
- **Run inside the network:** run the daemon on a host or container in the same network or VPC as the targets (documented in README with Docker Compose example).
- **VPN / proxy:** per-target `proxy_url` option (HTTP/SOCKS5) and `LOOK_OUT_API_HTTP_PROXY` env var.
- **Self-signed / internal CA:** `ca_cert_path` option; `insecure_skip_verify` allowed only per target, off by default, and logs a warning on every run.
- **Heartbeat (push) mode:** for services that can't be reached from outside, `look-out-api run` exposes `POST /heartbeat/{name}` (requires `LOOK_OUT_API_HEARTBEAT_TOKEN`). The target is DOWN if no heartbeat arrives within `grace_secs`. Cron jobs and internal services call this URL.

```toml
[[target]]
name = "nightly-backup"
type = "heartbeat"
grace_secs = 90000     # alert if no ping in ~25h
```

### 4.5 JavaScript-rendered Pages (`type = "browser"`)

Behind cargo feature `browser` (off by default so the base binary stays small). Requires Chrome/Chromium on the host (the Docker image variant bundles it).

```toml
[[target]]
name = "storefront"
type = "browser"
url = "https://shop.example.com"
wait_for_selector = "#product-grid"
expect_text = "Add to cart"
timeout_secs = 30
fail_on_console_error = true
```

Records page load time, selector found or not, and console errors. Optional: save a screenshot on failure to `./artifacts/`.

## 5. Database Schema (SQLite)

The schema below is the full-roadmap model. The implemented schema is in [migrations/0001_init.sql](./migrations/0001_init.sql); it currently constrains target types to the six implemented types, stores safe target labels rather than raw endpoints, and includes the durable notification outbox described in section 7.3. Apply additive migrations as roadmap types are introduced.

`migrations/0001_init.sql`

```sql
CREATE TABLE targets (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  name          TEXT NOT NULL UNIQUE,
  type          TEXT NOT NULL DEFAULT 'http'
                CHECK (type IN ('http','tcp','dns','postgres','redis','grpc','flow','browser','security','heartbeat')),
  endpoint      TEXT NOT NULL,          -- url or host:port, for display
  created_at    TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE checks (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  target_id     INTEGER NOT NULL REFERENCES targets(id) ON DELETE CASCADE,
  checked_at    TEXT NOT NULL DEFAULT (datetime('now')),
  ok            INTEGER NOT NULL CHECK (ok IN (0,1)),
  status_code   INTEGER,
  latency_ms    INTEGER,
  failed_step   TEXT,                   -- flow checks: which step failed
  error         TEXT
);
CREATE INDEX idx_checks_target_time ON checks(target_id, checked_at DESC);

CREATE TABLE check_steps (              -- flow checks: per-step detail
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  check_id      INTEGER NOT NULL REFERENCES checks(id) ON DELETE CASCADE,
  step_name     TEXT NOT NULL,
  ok            INTEGER NOT NULL CHECK (ok IN (0,1)),
  status_code   INTEGER,
  latency_ms    INTEGER,
  error         TEXT
);

CREATE TABLE security_findings (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  target_id     INTEGER NOT NULL REFERENCES targets(id) ON DELETE CASCADE,
  check_id      INTEGER REFERENCES checks(id) ON DELETE SET NULL,
  rule          TEXT NOT NULL,          -- e.g. 'tls_cert_expiry', 'missing_hsts', 'exposed_path'
  severity      TEXT NOT NULL CHECK (severity IN ('info','low','medium','high')),
  detail        TEXT,
  first_seen_at TEXT NOT NULL DEFAULT (datetime('now')),
  last_seen_at  TEXT NOT NULL DEFAULT (datetime('now')),
  resolved_at   TEXT,
  UNIQUE (target_id, rule, detail)
);

CREATE TABLE heartbeats (
  target_id     INTEGER PRIMARY KEY REFERENCES targets(id) ON DELETE CASCADE,
  last_ping_at  TEXT NOT NULL
);

CREATE TABLE target_state (
  target_id          INTEGER PRIMARY KEY REFERENCES targets(id) ON DELETE CASCADE,
  status             TEXT NOT NULL CHECK (status IN ('up','down','unknown')) DEFAULT 'unknown',
  consecutive_fail   INTEGER NOT NULL DEFAULT 0,
  consecutive_ok     INTEGER NOT NULL DEFAULT 0,
  last_changed_at    TEXT,
  last_checked_at    TEXT,
  last_alerted_at    TEXT
);

CREATE TABLE incidents (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  target_id     INTEGER NOT NULL REFERENCES targets(id) ON DELETE CASCADE,
  started_at    TEXT NOT NULL,
  resolved_at   TEXT,
  reason        TEXT
);
```

## 6. Environment Variables (`.env.example`)

```
DATABASE_URL=sqlite://look-out-api.db
LOOK_OUT_API_CONFIG=./look-out-api.toml
TELEGRAM_BOT_TOKEN=
TELEGRAM_CHAT_ID=
RUST_LOG=look_out_api=info
LOOK_OUT_API_BIND=127.0.0.1:8080     # status page + heartbeat endpoint
LOOK_OUT_API_HEARTBEAT_TOKEN=        # required for POST /heartbeat/{name}
LOOK_OUT_API_HTTP_PROXY=             # optional global proxy for private networks
LOOK_OUT_API_USER=                   # test account for flow checks
LOOK_OUT_API_PASS=
ORDERS_DB_URL=                # read-only DB user for postgres checks
API_TOKEN=                    # example, referenced by config headers
```

## 7. CLI Commands

```
look-out-api run                 # start daemon (scheduler + alerts)
look-out-api check [--name X]    # one-off check of all/one target, print table, exit code 1 if any fail
look-out-api check --state-file F --alert   # stateless/CI mode: load state from F, run checks, apply thresholds,
                                      # send Telegram alerts on transitions only, write state back to F
look-out-api status              # current state of every target
look-out-api report [--since 24h] [--name X]   # uptime %, avg/p95 latency, incident count
look-out-api validate            # validate config and exit
look-out-api history --name X --limit 50
```

### 7.1 Public Information Website

Build the public information website with Next.js App Router, TypeScript, and `output: "export"`. Use Motion for React for purposeful animation, Lucide React for icons, and locally hosted fonts. No emojis. Honor reduced motion and provide animation playback controls. The site introduces Look Out API to potential users. Content includes:
- What the tool does, who it is for, and how checks and Telegram alerts work.
- Features, with implemented capabilities distinguished from planned work.
- Installation and quickstart instructions, a sample TOML config, and sample CLI output.
- Links to the source repository, documentation, releases, and issue reporting when available.

Use responsive, accessible static pages with a descriptive page title, description, and social sharing metadata. Examples use fictional targets and sample data. The site is independently deployable on free static hosting and requires no running monitoring daemon, database, or user account. Live target status and incident data belong to the separate optional status page in phase 5.

### 7.2 GitHub CI/CD

- Use GitHub Actions in the personal GitHub repository for continuous integration and delivery.
- CI runs on pull requests and pushes to `main`: `cargo fmt --check`, `cargo clippy -- -D warnings`, and `cargo test`. Include browser-feature checks when that feature is implemented.
- Publish the static information website from `main` only after its validation and required CI checks pass.
- Version tags such as `v0.1.0` trigger release validation and builds for Linux, macOS, and Windows, with successful binaries attached to a GitHub Release.
- Pull-request workflows validate changes without deployment credentials. Grant deployment and release jobs only the permissions they require, and keep secrets in GitHub Actions secrets.
- CI/CD handles testing, website deployment, and releases; the monitoring daemon runs on its own host. The scheduled second-watcher workflow in section 11.4 is separate from CI/CD.

### 7.3 Reliable Incidents and Notification Delivery

- Persist target status, counters, last observation, and transition timestamps across restarts. Initial success establishes UP silently; initial failures must reach the failure threshold. Only recovery from DOWN emits RECOVERED.
- In one SQLite write transaction, save the observation and step/finding details, advance state, open or close an incident, and enqueue the transition notification. An initial healthy state is not a recovery.
- Enforce one open incident per target and unique notification event keys in SQLite. Never call Telegram inside a database transaction.
- The outbox stores event identity, kind, safe message, creation/delivery timestamps, attempts, next retry time, permanent-failure flag, and a categorical last error. A single delivery worker retries temporary failures with capped exponential backoff and jitter. Respect Telegram retry delays.
- Preserve notification order per target. Rejected configuration leaves the pending notification visible; after correction, `look-out-api alerts --retry` makes it eligible again. Missing credentials retain the queue.
- Guarantee one local notification event per transition. External delivery can duplicate if Telegram accepts a message but its response is lost; document the ambiguity instead of claiming exactly-once delivery.
- A database lock permits one daemon per SQLite file. Each target runs sequential checks, skips missed schedule ticks, and shares a bounded concurrency limit. Handle Ctrl+C and SIGTERM gracefully.
- Retain the last known incident state internally across observation gaps; display stale observations as unknown. Reports explicitly show sample-based availability and successful-sample latency; do not assume unobserved time was healthy.
- Ordinary one-off checks do not mutate daemon incident state. State-file runs use their own locked, atomically replaced state and pending queue, saved before external sends and after delivery outcomes. All configured targets must be UP for exit 0; unknown/down return 1, execution/configuration errors return 2.
- Tests cover flapping, initial failures, confirmed recovery, restart continuity, transaction rollback, duplicate local events, delivery failures/order, and separate process runs. Test logs and persisted records must not expose request secrets.

## 8. Phased Checklist

### Phase 0 — Setup
- [x] Initialize Cargo project, dependencies, and edition 2021+
- [x] `rustfmt.toml`, `.gitignore` (target, .env, *.db), and local formatting/lint gates
- [x] Initialize Git with repository-local commit email `lawrence.dizon@proton.me`
- [ ] Verify personal GitHub ownership before creating or pushing the remote
- [x] GitHub Actions CI workflow as defined in section 7.2
- [ ] Create personal remote and verify hosted CI
- [ ] Reminder: if you later deploy a status page or add Supabase/Vercel, set up token-based CLI login per machine and per repo

### Phase 1 — Single check (foundations)
- [x] `CheckResult { ok, status_code, latency_ms, error }`
- [x] `Executor::check() -> CheckResult` with timeout, status + body assertions
- [x] Custom error enum with `thiserror`
- [x] Unit tests with `wiremock` (200, 500, timeout, body mismatch)
- [x] `look-out-api check` prints a table and sets exit code

### Phase 2 — Config + CLI
- [x] TOML parsing, defaults merge, env var interpolation
- [x] Validation with clear error messages
- [x] `look-out-api validate`, `look-out-api check --name`
- [x] Tests for bad configs

### Phase 3 — Scheduler + persistence
- [x] sqlx pool, run migrations on startup
- [x] Sync targets from config into DB
- [x] One tokio task per target with `tokio::time::interval`, jitter on start
- [x] Store every result in `checks`
- [x] Graceful shutdown on Ctrl+C (`tokio::signal`)
- [x] `look-out-api status`, `look-out-api history`

### Phase 4 — Alerting
- [x] State machine: thresholds -> `up`/`down` transitions
- [x] Transactional incident/outbox writes, persisted retries, ordering, and `alerts` inspection/retry commands
- [x] Open/close rows in `incidents`
- [x] Telegram sender (`sendMessage` via reqwest) for DOWN and RECOVERED, with duration
- [x] Alert only on transitions, never every failed check
- [x] State-machine tests for flapping, startup failures, recovery, and restart

### Phase 5 — Reports + status page (optional)
- [x] `look-out-api report`: sample-based availability, successful avg/p95 latency, incident count
- [x] `axum` status page at `LOOK_OUT_API_BIND`
- [x] Loopback-only live dashboard with real scheduled history, incident visibility, and manual configured-target tests; manual tests do not advance incident state. See [dashboard guide](./docs/dashboard.md).
- [x] Add GET/HEAD HTTP APIs in the local dashboard, persist configuration, and start scheduled monitoring without a daemon restart. Advanced TOML edits still require restart.
- [x] Pause/resume individual targets from the local dashboard; retain configuration/history and durable pause state across daemon restart with the same database.
- [x] Static information UI: no emojis, sidebar navigation, responsive layouts, custom scrollbars, Motion animation with reduced-motion support, Lucide icons, and the impeccable design workflow

### Phase 5b — Security checks
- [ ] `Check` trait refactor so every type shares scheduler, storage, alerting
- [ ] TLS expiry + chain + hostname check (`rustls`, `x509-parser`), warn and critical thresholds
- [ ] Security header presence and leaky-header checks
- [ ] HTTP to HTTPS redirect check
- [ ] `must_be_blocked` path probe and `protected_paths` auth enforcement
- [ ] CORS wildcard check
- [ ] Store results in `security_findings`; alert on new or escalated findings only
- [ ] `look-out-api report --security` groups findings by severity
- [ ] Safety guardrails: read-only requests only, rate-limited, never sends exploit payloads
- [ ] Tests with `wiremock` simulating missing headers, exposed `/.env`, expiring cert (generate test cert with `rcgen`)

### Phase 5c — Non-HTTP check types
- [x] `tcp` check with connect timeout
- [x] `dns` check with optional expected IPs
- [ ] `postgres` and `redis` protocol checks (read-only, trivial query)
- [ ] `grpc` health check via `tonic-health`
- [ ] Tests using local listeners and a test gRPC server

### Phase 5d — Multi-step flows
- [x] Config parsing for `[[target.step]]`
- [x] Variable map, `{{var}}` templating, JSON pointer extraction
- [x] Per-step latency and failed-step recording in `check_steps`
- [x] Secret masking in logs and stored errors
- [x] Tests: successful token reuse and missing extraction path
- [x] Explicit failed-login flow regression test

### Phase 5e — Private networks and heartbeats
- [x] Per-target `proxy_url`, `ca_cert_path`, guarded `insecure_skip_verify`
- [x] `POST /heartbeat/{name}` with bearer token auth
- [x] Missed-heartbeat detection in scheduler
- [x] Docker Compose example for `look-out-api` inside a private network (execution pending: local engine unavailable)
- [x] README section: deployment patterns for internal targets

### Phase 5f — Browser checks (feature flag)
- [ ] Cargo feature `browser` using `chromiumoxide`
- [ ] Wait for selector, assert text, capture console errors
- [ ] Screenshot on failure
- [ ] Docker image variant that bundles Chromium
- [ ] Integration test against a local page that renders via JavaScript

### Phase 6 — Polish and release
- [x] README with install and config examples
- [ ] Add CLI screenshots to release documentation
- [ ] Public static information website covering section 7.1, with clear feature availability and working documentation/release links
- [x] Data retention: prune `checks` older than N days
- [ ] Dockerfile (multi-stage, distroless or scratch)
- [ ] GitHub release builds for Linux, macOS, Windows
- [ ] GitHub Actions CD workflows for the static website and tagged releases as defined in section 7.2
- [ ] Optional: cross-compile to run on a small VPS or Raspberry Pi

### Phase 7 — Free-tier deployment
- [ ] `Dockerfile` (multi-stage, non-root user, final image `debian:bookworm-slim` or `gcr.io/distroless/cc`), builds for linux/amd64 and linux/arm64
- [ ] `deploy/look-out-api.service` (systemd unit) and `deploy/README.md` with step-by-step Oracle VM setup
- [ ] `deploy/backup.sh`: nightly `sqlite3 look-out-api.db ".backup"` to a second location, keep 7 copies
- [x] Stateless mode: `look-out-api check --state-file state.json --alert` (see 11.3)
- [ ] `.github/workflows/watch.yml` second-watcher workflow (see 11.4)
- [ ] Tests: state-file round trip, alert fires once across two separate process runs, corrupted state file is handled (start fresh and log a warning)
- [ ] Go-live checklist in section 11.5 completed

## 9. Testing Strategy

- **Unit**: state machine, config validation, latency percentile math.
- **Integration**: `wiremock` servers simulating up, down, slow, flapping.
- **CLI**: `assert_cmd` for exit codes and output.
- **Manual**: point at a real URL, kill the network, confirm DOWN then RECOVERED alert arrives once each.

## 10. Definition of Done

- [ ] `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test` all pass in CI
- [ ] All CLI commands in section 7 work against a sample config
- [ ] Flapping target produces no alert spam (verified by test)
- [ ] DOWN and RECOVERED Telegram messages received in a live test
- [ ] Every check type in section 4.1 has at least one passing integration test
- [ ] A flow check against a mock API logs in, reuses the token, and reports the failing step by name
- [ ] Security check flags a mock server missing HSTS, exposing `/.env`, or serving a cert expiring in under 7 days
- [ ] Security checks send only read-only requests; no exploit payloads or fuzzing anywhere in the code
- [ ] Secrets and tokens never appear in logs, the DB, or alerts
- [ ] A heartbeat target goes DOWN after `grace_secs` with no ping and recovers after pings restore successful scheduled checks to the configured recovery threshold
- [ ] Base binary builds without the `browser` feature; browser feature builds and passes its test
- [ ] Graceful shutdown leaves the DB consistent
- [ ] No `unwrap()` in non-test code paths
- [ ] README lets a stranger install and run it in under 5 minutes
- [ ] Static information website meets section 7.1, works on mobile and desktop, and is published at a shareable URL using free hosting
- [ ] Tagged `v0.1.0` release with binaries attached
- [ ] Docker image builds and runs as non-root; container restarts cleanly with the DB volume intact
- [ ] `look-out-api check --state-file ... --alert` creates one DOWN and one RECOVERED event across separate runs with confirmed delivery; uncertain-delivery duplication is documented
- [ ] Every deployment step in section 11 works at $0 cost (no paid service required)

## 11. Free-Tier Deployment (no paid services)

### 11.1 Why not the usual free hosts
- Fly.io has no free tier for new accounts.
- Render and Koyeb free services sleep when idle; a sleeping monitor cannot monitor.
- Vercel and similar serverless platforms cannot run a long-lived daemon with a local SQLite file.
- Therefore the primary host is a free always-on VM, with a free CI scheduler as a second watcher.

### 11.2 Primary: Oracle Cloud Always Free VM
- Use the small x86 Always Free VM (`VM.Standard.E2.1.Micro`, 1 GB RAM). `look-out-api` needs tens of MB, so the Arm A1 allowance is not needed. Oracle has changed the Arm A1 limits before, so do not depend on it.
- Requires an Oracle account with a payment method for verification. Stay within Always Free resources only.
- Read Oracle's current Always Free page and its idle-instance reclamation rules before relying on the VM, and re-check periodically. If the VM is ever reclaimed, the GitHub Actions watcher (11.4) keeps coverage until it is recreated.
- Install: Ubuntu LTS, Docker (or run the static binary under systemd), create a non-root `look-out-api` user.
- Layout on the VM:
  ```
  /opt/look-out-api/look-out-api            # binary (or docker image)
  /opt/look-out-api/look-out-api.toml       # config, chmod 600
  /opt/look-out-api/.env              # secrets, chmod 600, never committed
  /var/lib/look-out-api/look-out-api.db     # SQLite data
  ```
- systemd unit essentials: `User=look-out-api`, `EnvironmentFile=/opt/look-out-api/.env`, `Restart=always`, `RestartSec=5`, `NoNewPrivileges=true`, `ProtectSystem=strict`, `ReadWritePaths=/var/lib/look-out-api`.
- Network: allow inbound SSH only (restrict to your IP if possible). `look-out-api run` needs no inbound ports unless the status page is enabled; bind it to `127.0.0.1` and reach it through an SSH tunnel.
- Backups: nightly `deploy/backup.sh` copies the SQLite file to a second free location (your own machine via `scp`, or a private repo/bucket you control). Test a restore once.
- Do not host `look-out-api` on the same server as the app it monitors.

### 11.3 Stateless / CI mode
For environments with no persistent disk (GitHub Actions):
- `look-out-api check --state-file state.json --alert` runs all targets once, advances the same failure and recovery thresholds as the daemon using the saved state, sends Telegram alerts on transitions only, then writes the state back.
- `state.json` holds a version, event counter, per-target `status`, `consecutive_fail`, `consecutive_ok`, `last_changed_at`, and `last_checked_at`, plus pending notifications and retry metadata. Write atomically (write temp file, then rename). If the file is missing or corrupt, start from `unknown` and log a warning.
- Exit code: 0 if every configured target is up after the run, 1 if any is down or still unknown, 2 for configuration/execution errors.
- No SQLite and no reports in this mode.

### 11.4 Second watcher: GitHub Actions (free)
Purpose: watch the public targets and the VM itself, so you still get alerted if the VM dies.
- Workflow `.github/workflows/watch.yml`: `schedule: cron: "*/7 * * * *"` (minimum allowed is 5 minutes; avoid exact :00 marks, runs can be delayed), plus `workflow_dispatch` for manual runs.
- Steps: checkout, download the released `look-out-api` binary (or `cargo install` from cache), restore `state.json` with `actions/cache`, write the config from a repository secret `LOOK_OUT_API_CONFIG_B64` (base64 TOML) so target URLs are not committed, run `look-out-api check --state-file state.json --alert`, save the cache.
- Include a `tcp` target for the VM's SSH port so the workflow alerts if the VM is unreachable.
- Secrets in repo secrets only: `TELEGRAM_BOT_TOKEN`, `TELEGRAM_CHAT_ID`, `LOOK_OUT_API_CONFIG_B64`, plus any `${VAR}` used in the config.
- Limits to document in the README: runs can be delayed under load; scheduled workflows are disabled after 60 days without repository activity (add a monthly commit or re-enable manually); the runner is wiped every run, so state lives only in the cache; public repositories have public logs, so the tool must never print URLs with tokens, headers, or findings details that reveal client systems. Prefer `--quiet` output in this mode.
- Check GitHub's current rules for scheduled workflows on private repositories before choosing a private repo.

### 11.5 Go-live checklist
- [ ] Telegram bot created, token and chat id in `.env` (never in git)
- [ ] `look-out-api validate` passes on the VM
- [ ] One deliberately broken target proves DOWN and RECOVERED alerts arrive once each
- [ ] Reboot the VM and confirm the service restarts and keeps its history
- [ ] Backup ran once and a restore was tested
- [ ] Second watcher workflow ran green and its state cache persists between runs
- [ ] Stopping the VM triggers an alert from the GitHub Actions watcher
- [ ] `.env`, `look-out-api.toml`, and any state files are in `.gitignore`
- [ ] Setup note for new projects: if a Vercel or Supabase status page or dashboard is added later, set up token-based CLI login per machine and per repo first

## 12. Stretch Ideas

- Slack/Discord/email alert backends behind an `Alerter` trait
- Prometheus `/metrics` endpoint
- ICMP ping, SMTP and IMAP checks, WebSocket checks
- Security finding export to JSON or SARIF for CI
- Hot-reload config on SIGHUP
