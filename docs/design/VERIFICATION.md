# Look Out API — initial build verification

Checked: 2026-11-25 · Agent: codex · Local branch: `feat/initial-build` (draft PR [#1](https://github.com/Lawrence1719/look-out-api/pull/1))

Sources: [overview](./OVERVIEW.md), [project spec](../../spec.md), and the actual commands/tests below. This records an initial working increment, not completion of the full roadmap or a live deployment.

## Decisions

| ID | Decision | Status | Evidence |
| --- | --- | --- | --- |
| D1 | Rust CLI/daemon and local SQLite | Verified for initial six types | [checks](../../tests/checks.rs), [persistence](../../tests/persistence.rs), local release build |
| D2 | Transactional incidents and persistent notification queue | Verified locally | Restart, rollback, local event uniqueness, rejected credentials, temporary delivery errors and ordering tests |
| D3 | Next.js static information site, Motion, Lucide, no emojis | Verified | Static export; six Playwright tests against exported files; desktop/mobile screenshots inspected |
| D4 | Personal local commit identity | Verified locally and through GitHub | `git config --local user.email` is `lawrence.dizon@proton.me`; GitHub API authenticated as `Lawrence1719`; public remote `Lawrence1719/look-out-api` created; active global GitHub account unchanged |
| D5 | GitHub CI/CD | Initial PR CI passed; Pages configured | [CI](../../.github/workflows/ci.yml), [site](../../.github/workflows/deploy-main.yml), [release](../../.github/workflows/release.yml), [watcher](../../.github/workflows/watch.yml); actionlint passed |
| D6 | Safe local build | Verified | Feature branch, mock services, temporary databases; no live targets, production credentials or cloud hosts; verified personal GitHub repository setup authorized by owner |
| D7 | Observation-log visual direction | Built, needs owner feedback | [visual system](../../DESIGN.md); rendered desktop 1440px and mobile 375px layouts |

## Rules and evidence

| Rule | Enforced by | Evidence |
| --- | --- | --- |
| One open incident and unique local alert event | SQLite partial unique index and unique event key | `database_constraints_and_failed_transaction_protect_incidents` |
| Check/state/incident/outbox commit together | Immediate SQLite transaction | Rollback test leaves unknown state and no queued event |
| Startup/failed/recovered thresholds | Pure state machine | Flapping, initial failure and restart tests |
| Pending alerts survive failure and keep order | Persistent outbox and worker | Temporary failure and rejected credential tests |
| Independent process runs do not repeat confirmed events | Locked/atomic state file with pending queue | `independent_processes_alert_once_and_keep_credentials_out_of_output` |
| Secrets are not copied into check results or dependency traces | Categorical errors, safe metadata, tracing target filter | URL/header/body/bot/extracted-token assertions with `RUST_LOG=trace` |
| Heartbeat requires token and registered target | HTTP handler plus target lookup | Authenticated endpoint and persisted grace/ping tests |
| Passive probes are read-only | Security executor uses GET only | Local mock request assertions |
| Certificate warnings and chain validation work | reqwest TLS and X.509 parsing | Locally signed near-expiry certificate flagged; untrusted certificate rejected |
| Reports do not claim missing coverage was healthy | Sample-based calculation and stale labels | Percentile/duration tests; CLI report smoke |
| Single daemon and graceful restart | File lock, signals, drained tasks | Manual local smoke: duplicate daemon rejected, SIGTERM exit 0, stored DOWN then RECOVERED events retained |

## Commands actually run

- `cargo test --locked`: **23 tests passed** (plus empty unit/doc targets).
- `cargo fmt --check`: passed.
- `cargo clippy --locked --all-targets -- -D warnings`: passed.
- `cargo build --release --locked`: passed on local macOS ARM64; executable produced.
- Website `npm run format:check`, `npm run typecheck`, `npm run build`: passed.
- Website `npm test`: **6 tests passed against the static export**. Includes reduced-motion/manual sequence behavior, keyboard tab selection, clipboard, mobile navigation/overflow, script errors and automated WCAG A/AA checks on both pages.
- GitHub Pages base-path build: `/look-out-api` assets, documentation navigation, and fully qualified social image URL passed a local static-host smoke. The normal root export was rebuilt afterward.
- `actionlint`: passed on all four workflows. `shellcheck deploy/backup.sh`: passed.
- npm dependency audit at installation: zero reported vulnerabilities.
- Local daemon smoke: initial UP, DOWN, stop/restart while DOWN, confirmed recovery, two persisted notification events, successful status/history/report/alerts queries, duplicate daemon lock, and clean SIGTERM shutdown.
- Backup smoke: nine successful online backups, seven retained copies, restored-copy SQLite integrity check passed.

Automated accessibility checks cover detectable issues, not a full human accessibility audit. Telegram tests use local mocks; no real message was sent.

## Stage status

| Stage | Status |
| --- | --- |
| Groundwork, schema, config, HTTP/TCP/DNS/flow/heartbeat/initial security | Verified locally |
| Incident state, outbox, retries, reports and state-file checks | Verified locally |
| Responsive static information site and docs | Verified locally |
| GitHub workflows, systemd, backup and container assets | Built; remote execution/host setup pending |
| Postgres/Redis protocol, gRPC and browser checks | Not implemented; retained in roadmap |
| Expanded security rules and per-finding escalation alerts | Not implemented; initial availability-based security alerts only |
| Public personal repository and draft PR | Published; review pending |
| Tagged release, real Telegram test and public site | Needs setup |

## Repository standards adapted to this personal project

| Practice | Status | Evidence / remaining work |
| --- | --- | --- |
| Enforced rules | Verified locally | SQLite constraints and transactional writer |
| Ops memory | Not connected | Local spec/design/verification are used; no work-organization services introduced |
| Named sign-offs | Partial | Implementation/local QA: Codex. Review, merge and release: Lawrence |
| SQL rule tests and CI | Built | SQL constraints exercised in Rust integration tests; CI runs them on three operating systems |
| Facts sheet | Built | [CLAUDE.md](../../CLAUDE.md) |
| Branch/PR habits | Partial | Feature branch and personal local email; verified public personal remote and draft PR #1 |
| Agent safety | Built | [AGENTS.md](../../AGENTS.md) |
| Governed MCP | Not applicable | No MCP server |
| GitHub DevOps | Partially verified | CI/site/release/watcher workflows; main protected with four required checks; Pages workflow publishing enabled; initial hosted CI passed; PR preview hosting not configured |

## Still needs people

| Item | Owner | Blocks |
| --- | --- | --- |
| Review initial implementation and visual direction | Lawrence | Initial merge |
| Review and merge PR #1; verify site and tagged release jobs | Lawrence + Codex | Published site and release |
| Provide real target config and Telegram credentials locally | Lawrence | Live DOWN/RECOVERED delivery test |
| Start a Docker engine and run container/Compose verification | Lawrence + Codex | Verified container delivery; engine unavailable during this build |
| Provision and validate a host that stays within current free allowances | Lawrence | Always-on monitoring |
| Configure second-watcher secrets and validate cache continuity | Lawrence + Codex | Independent coverage |

No account credentials, hosted identifiers or unverified public URLs were invented. No usage telemetry is enabled.

## Personal GitHub setup

2026-11-25 · Agent: codex. Owner confirmed `Lawrence1719` and public visibility. Scoped authentication verified the GitHub login before repository creation. GitHub created a starter README on `main`; implementation is committed and pushed only on `feat/initial-build`. Global Git and GitHub identities are preserved. Rust and website gates were rerun successfully before publication: 23 Rust tests and 6 browser tests. Hosted CI [run 45637411403](https://github.com/Lawrence1719/look-out-api/actions/runs/45637411403) passed for initial implementation commit `9eff4b3`: Rust fmt/Clippy/tests on Linux, macOS and Windows, plus the complete website gates. Later PR commits receive their own required checks.

The initial public draft uses MIT as the recommended implementation default following the owner's open-source direction; bundled font licenses are retained. Release archives include the source license.

Draft PR [#1](https://github.com/Lawrence1719/look-out-api/pull/1) is open. `main` retains only the GitHub-created starter README. Required checks are Rust on Linux/macOS/Windows and Website, with an up-to-date branch required and admin enforcement enabled; force pushes and deletions are disabled. Solo-owner PRs do not require an additional reviewer approval. Pages is configured with `build_type=workflow` and awaits a reviewed merge followed by successful main CI. No site deployment or release was started. Font copyright and license notices are included in the static export.

Hosted release packaging remains unverified until a version tag is intentionally published. The successful PR run does not claim release or deployment validation.
