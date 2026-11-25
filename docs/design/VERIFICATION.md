# Look Out API — initial build verification

Checked: 2026-11-25 · Agent: codex · Initial implementation: `feat/initial-build` ([PR #1](https://github.com/Lawrence1719/look-out-api/pull/1), now merged). Documentation follow-up: `docs/cross-platform-setup`.

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

Automated accessibility checks cover detectable issues, not a full human accessibility audit. Automated Telegram tests use local mocks. The later manual live-delivery verification is recorded below.

## Stage status

| Stage | Status |
| --- | --- |
| Groundwork, schema, config, HTTP/TCP/DNS/flow/heartbeat/initial security | Verified locally |
| Incident state, outbox, retries, reports and state-file checks | Verified locally |
| Responsive static information site and docs | Verified locally |
| GitHub workflows, systemd, backup and container assets | Built; remote execution/host setup pending |
| Postgres/Redis protocol, gRPC and browser checks | Not implemented; retained in roadmap |
| Expanded security rules and per-finding escalation alerts | Not implemented; initial availability-based security alerts only |
| Public personal repository and initial PR | Published; PR #1 merged |
| Live Telegram delivery with local demo | Verified manually; see follow-up below |
| Tagged release and production monitor host | Needs setup |
| Public information site | Pages configured; current hosted deployment evidence is separate from local site checks |

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
| Review cross-platform documentation follow-up | Lawrence | Documentation merge |
| Verify site and tagged release jobs | Lawrence + Codex | Verified hosted site and release |
| Configure intended real targets | Lawrence | Monitoring beyond the local demo |
| Start a Docker engine and run container/Compose verification | Lawrence + Codex | Verified container delivery; engine unavailable during this build |
| Provision and validate a host that stays within current free allowances | Lawrence | Always-on monitoring |
| Configure second-watcher secrets and validate cache continuity | Lawrence + Codex | Independent coverage |

No account credentials, hosted identifiers or unverified public URLs were invented. No usage telemetry is enabled.

## Personal GitHub setup

2026-11-25 · Agent: codex. Owner confirmed `Lawrence1719` and public visibility. Scoped authentication verified the GitHub login before repository creation. GitHub created a starter README on `main`; implementation is committed and pushed only on `feat/initial-build`. Global Git and GitHub identities are preserved. Rust and website gates were rerun successfully before publication: 23 Rust tests and 6 browser tests. Hosted CI [run 45637411403](https://github.com/Lawrence1719/look-out-api/actions/runs/45637411403) passed for initial implementation commit `9eff4b3`: Rust fmt/Clippy/tests on Linux, macOS and Windows, plus the complete website gates. Later PR commits receive their own required checks.

The initial public draft uses MIT as the recommended implementation default following the owner's open-source direction; bundled font licenses are retained. Release archives include the source license.

At initial publication, draft PR [#1](https://github.com/Lawrence1719/look-out-api/pull/1) was open and `main` retained only the GitHub-created starter README. The owner has since merged it. Required checks are Rust on Linux/macOS/Windows and Website, with an up-to-date branch required and admin enforcement enabled; force pushes and deletions are disabled. Solo-owner PRs do not require an additional reviewer approval. Pages is configured with `build_type=workflow`; hosted deployment results must be verified separately. Font copyright and license notices are included in the static export.

Hosted release packaging remains unverified until a version tag is intentionally published. The successful PR run does not claim release or deployment validation.

Local preview restoration exposed generated Next.js type references changing between development and builds. Following the installed Next.js guide, `next-env.d.ts` is now ignored and the typecheck command runs `next typegen` before TypeScript, so a fresh checkout does not depend on generated files being committed. Default source links now point to the verified public repository, and the roadmap no longer lists repository publication as pending.

## Manual Telegram setup and delivery

2026-11-25 · Agent: codex. The owner created a dedicated bot, saved its token locally, and initiated its private chat. The destination was retrieved without logging the token or full API response. Telegram confirmed a setup message. The monitor was restarted with delivery enabled; stopping only the local demo produced DOWN after three consecutive failures, and restoring it produced RECOVERED after two successes. SQLite recorded two ordered transition events, both delivered, with zero open incidents or pending notifications afterward. The demo and monitor were left running and the target was UP. Private credentials, chat identifiers, and operational database files remain untracked. No production API was disrupted.

## Cross-platform documentation follow-up

2026-11-25 · Agent: codex · Branch: `docs/cross-platform-setup`, based on merged PR #1.

- Added [Windows](../getting-started/windows.md), [macOS](../getting-started/macos.md), and [Linux](../getting-started/linux.md) source-setup guides with native shell syntax, prerequisites, local configuration, demo/daemon terminals, and stop/restart behavior.
- Added shared [usage/database/backup](../usage.md), [Telegram](../telegram.md), and [troubleshooting](../troubleshooting.md) documentation. The static website exposes the same OS entry points and setup explanations.
- Checked relative Markdown links across the documentation; parsed the documented Python examples; executed the backup example against a temporary database and proved an existing backup is preserved.
- Ran the documented validate/status/check/run/history/report/alerts flow against a temporary database and the local demo, with Telegram credentials explicitly absent. Automatic schema creation, recorded history, UP status, and graceful shutdown passed.
- Rust fmt, Clippy, and all 23 tests passed in an isolated checkout without the owner's local credentials. The website clean install, formatting, type generation/typecheck, static build, and all six browser/accessibility tests passed.
- Desktop/mobile documentation layout, OS anchor navigation, and Windows command rendering were checked against the static export. Native Windows and Linux installation walkthroughs were not manually executed on this Mac. Rust CI covers the three hosted OS runners; native service installers, extra platform/architecture combinations, tagged release packaging, and Docker runtime remain separate verification work.

## Documentation density refinement

2026-11-25 · Agent: codex · Branch: `docs/cross-platform-setup`.

- Removed the narrow documentation width cap and paired instructions with command examples on wide containers. Reduced heading, section, and code-block spacing; retained a readable prose measure and stacked layout on smaller containers.
- Preserved all existing setup examples, OS anchors, configuration explanations, and implemented/planned distinctions. Updated one directional reference after moving the build example.
- Website formatting, type generation/typecheck, and static export passed. All six browser tests passed, including automated accessibility checks. The initial accessibility run identified scrollable examples without keyboard access; focusable code regions fixed the issue before the passing rerun.
- Measured the static export at 320, 375, 768, 1024, 1440, and 1920px with fonts loaded: no horizontal page overflow. Visually reviewed desktop, intermediate, mobile, and OS-anchor views. Verified Windows command copying preserves the complete text and keyboard focus reaches its scrolling code region. The scoped Impeccable layout scan reported no findings.
- This change affects the information site only; local monitoring credentials and runtime data were not changed.

## Local dashboard and visual testing

2026-11-26 · Agent: codex · Branch: `feat/local-dashboard` · [Design](./specs/2026-11-25-local-dashboard-design.md) · [Plan](./plans/2026-11-25-local-dashboard.md) · [Surface record](./surfaces/local-dashboard.md).

| Rule / decision | Status | Evidence |
| --- | --- | --- |
| Manual tests perform real configured checks | Verified locally | `manual_checks_make_real_requests_without_changing_incidents_or_history`; browser request counter, actual HTTP 200 PASS and response-text mismatch FAIL |
| Manual tests do not advance scheduled history/incidents/outbox | Verified locally | Same Rust test compares stored state and check/incident/notification counts; isolated release-daemon smoke preserved history count and created zero notifications |
| Manual and scheduled execution share per-target serialization | Verified locally | `execution_is_serialized_and_scheduled_results_remain_durable_and_safe`; overlapping manual requests rejected, scheduled request waits and persists |
| Real flow-step and timeout results; stale observations remain unknown | Verified locally | `dashboard_exposes_real_flow_steps_timeouts_and_stale_state`; browser flow and timeout checks |
| Dashboard excludes private request details | Verified locally | Snapshot assertions exclude mock URL/query, header, expected response text, and extracted flow values; browser uses text nodes for safe rendering |
| Dashboard actions require local Host, same Origin, and custom header | Verified locally | `dashboard_api_checks_origin_host_and_configured_target_boundary`; unknown configured target rejected |
| Nonloopback daemon listeners disable dashboard data/actions | Built and locally checked | Scheduler enables dashboard runtime only for loopback listener addresses; `dashboard_actions_are_unavailable_without_loopback_runtime` proves disabled routing. Existing heartbeat integration still passes |
| Scheduled transactions, incidents, and outbox behavior remain intact | Verified locally | Existing persistence tests; browser observes actual local mock DOWN after failure threshold and RECOVERED after success threshold |
| Responsive UI, connection failure, empty state, focus and detectable accessibility | Verified locally | Dashboard browser suite, 320–1920px overflow measurements, desktop/mobile captures, reduced-motion mode; automated accessibility is not a full human audit |
| Embedded assets need no Node runtime or CDN | Verified locally | Release-binary smoke serves page and bundled fonts from the daemon; fonts and icon license notices retained |

Final local gates: Rust fmt, Clippy with warnings denied, and **28 tests passed** in an isolated source checkout with the owner's credentials absent. Website clean install, formatting (including embedded dashboard assets), type generation/typecheck, static export, and **6 existing browser/accessibility tests passed**. The separate real-daemon dashboard browser suite has **3 passing tests**. `actionlint` passed after adding dashboard browser coverage to CI. Relative documentation links were checked.

Visual QA used real local mock executions: PASS, HTTP 200 with a failed text assertion, running feedback, and completed flow steps. The Impeccable reviewer initially requested truthful immediate activity text and the established Manrope/DM Sans pairing; its verdict pass scored both fixes resolved, disposition **ship** at that scope. A later check of the owner's 24-observation history exposed the state machine's saturated counters; copy now explicitly calls them capped threshold counters rather than a full success streak. Review screenshots are transient local QA and are not committed.

An isolated release-daemon run against the included local demo verified dashboard routing, a real manual HTTP 200 PASS, unchanged scheduled history, zero notifications created by the manual action, bundled font delivery, and clean SIGTERM shutdown. The owner's local monitor was restarted with the new binary, preserving its existing private config/database and healthy demo. The dashboard was opened at `http://127.0.0.1:8080/`; clicking Run check returned a real PASS. No automated test used the owner's Telegram credentials or private database, and no new Telegram delivery test was run for this feature.

Still needs people: Lawrence reviews/merges the feature PR and its documentation dependency. Tagged release packaging, container execution, an always-on host, remote administration/authentication, native Windows/Linux dashboard walkthroughs, and broader assistive-technology checks remain separate work. The public information site remains independently static; planned protocols and expanded assertions remain on the full roadmap.

Publication follow-up · 2026-11-26 · Agent: codex. The dashboard implementation was committed and pushed to `feat/local-dashboard` on the verified personal repository. GitHub returned GraphQL server errors and HTTP 500 when creating a draft PR, including a standard-main-base attempt. Read-only checks after the uncertain responses confirmed no PR was created. Therefore hosted CI has not run for this feature branch yet. The branch and local dashboard are available; draft PR creation and its hosted checks remain pending GitHub recovery. No main push or merge was performed.


## Add APIs from the local dashboard

2026-11-26 · Agent: codex · Branch: `feat/local-dashboard` · [Design](./specs/2026-11-26-dashboard-add-api-design.md) · [Usage](../dashboard.md#add-an-api-without-restarting).

- Implemented an inline Add API form for GET/HEAD checks: unique name, HTTP/HTTPS URL, optional status/text expectations, interval, and timeout. Saving persists the target and schedules it without restarting; advanced TOML changes still require restart.
- Five new Rust tests verify validation, duplicate rejection, bounded/same-origin action requests, concurrent additions without lost schedule updates, preserved raw comments and real environment placeholders, untouched existing incident/counter/outbox state, and restart recovery after a simulated database registration failure. Errors remain categorical; request configuration stays out of snapshots.
- Writes preserve original file permissions and use a sibling advisory lock plus atomic replacement. Detected external edits are rejected. This is not a compare-and-swap guarantee against an uncoordinated editor racing the final replacement; avoid simultaneous manual edits. Symlinked/read-only configuration disables additions.
- Rust fmt, Clippy with warnings denied, and **33 tests passed** in an isolated checkout without the owner's credentials. Website clean install, formatting, typecheck, static export, and **6 static-site tests passed**. The dashboard browser suite has **4 passing tests**, including actual immediate scheduled HTTP 200/PASS after saving, draft preservation during polling/errors, timeout validation, duplicate focus recovery, mobile layout, and Axe checks.
- Visually inspected real fixture captures at 320, 375, 1024, 1440, and 1920px, plus duplicate-error and saved-result states. No horizontal page overflow. The single Impeccable detector pass returned no findings; a fresh finish reviewer returned **ship**, no material fixes. Existing Manrope/DM Sans, paper palette, and Lucide conventions are retained. Captures remain transient local QA.
- An isolated release-daemon smoke verified real scheduled PASS after adding, unchanged mode 0600, persisted target/history across graceful shutdown/restart, and resumed scheduled checks. It used a temporary config/database and the included local demo with Telegram absent.
- Rebuilt and restarted only the owner's local monitor. Add API is enabled at `http://127.0.0.1:8080/`; the original single-target TOML was byte-for-byte preserved. No test target was added to the owner's configuration and no Telegram test was sent.

Native Windows/Linux interactive walkthroughs, broad assistive-technology review, target editing/deletion, authentication/POST forms, and remote dashboard administration remain separate work. Publication follow-up: GitHub recovered and [draft PR #3](https://github.com/Lawrence1719/look-out-api/pull/3) was created against the documentation branch. [Hosted CI run 45713032969](https://github.com/Lawrence1719/look-out-api/actions/runs/45713032969) was in progress for feature commit `f567722` when checked; local results do not establish its hosted outcome.
