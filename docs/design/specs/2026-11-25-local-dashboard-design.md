# Local dashboard — design

Date: 2026-11-25 · Agent: codex · [Overview](../OVERVIEW.md)

## Problem and goal
The owner requested visual testing in the local dashboard, with evidence that the API is actually being checked. The existing [server](../../../src/server.rs) renders a stored-status table; the [scheduler](../../../src/scheduler.rs) performs checks, and [Executor](../../../src/checker/mod.rs) already produces safe status, timing, assertion errors, and flow-step results.

Extend the local status page into an operational dashboard: live scheduled observations, real recent history, incident state, and on-demand tests of configured targets. Keep the public static Next.js site independent.

## Decisions
| ID | Choice and reason | Status / source |
| --- | --- | --- |
| D1 | Visual local testing and real execution evidence | Decided: owner's current request |
| D2 | Reuse the configured executor; never accept arbitrary request URLs or credentials from the browser | PROPOSED implementation default: existing config boundary |
| D3 | Manual tests follow ordinary CLI checks: no incident/state/outbox writes. Scheduled checks remain the incident writer | PROPOSED implementation default: existing CLI semantics and project transaction rule |
| D4 | Serve embedded HTML/CSS/JS from the Rust daemon; no Node installation or external asset requests to use the dashboard | PROPOSED implementation default: existing Axum page and self-hosted binary |
| D5 | Enable dashboard data/actions only on loopback-bound listeners, enforce local Host and same-origin action headers; retain heartbeat behavior for nonlocal binds | PROPOSED implementation default: no authenticated remote administration in scope |
| D6 | Inherit warm paper, charcoal, signal orange, Lucide icons, responsive density, and reduced-motion behavior | Established design system; owner asked for real icons and no emojis |

Implementation is authorized by the owner's request and repository instructions. These defaults do not change public hosting or alert policy.

## Data and rules
No schema changes. Read targets, target_state, checks, check_steps, incidents, and notification counts in a read transaction. Bound history to 24 observations and incidents to five per target. Never expose target URLs, request/response bodies, headers, credentials, or extracted flow values. Manual results and in-flight activity are process-local and disappear on restart.

Serialize all executions per target with a shared gate; keep the existing global 16-check concurrency bound. A scheduled check persists through Store::record, preserving its one-transaction check/state/incident/outbox write. A manual check returns its real CheckResult without calling record. Notification delivery remains outside these operations. Concurrent manual requests return a categorical busy response instead of building a queue.

## API and UI
- GET /: local dashboard, embedded assets; the existing basic status page remains available on nonloopback listeners.
- GET /api/dashboard: safe configured-target metadata, active execution, latest manual result, persisted history/steps/incidents, pending alert count, server timestamp. No-store.
- POST /api/targets/{name}/check: configured target only, same-origin custom-header request. Returns the actual result or categorical busy/not-configured/unavailable errors. Does not alter incident state.
- GET /dashboard.css and /dashboard.js: bundled assets, no CDN. Restrictive CSP and frame protection.

Operate mode: compact masthead with connection freshness, target list with state/last result, selected target's real latency and observation strip, manual test action/result, flow steps, and recent history/incidents. Label scheduled incident state separately from individual PASS/FAIL observations. Show waiting, running, stale, disconnected, empty, and failed states without claiming a fabricated test ran. Refresh every two seconds; pause hidden-page polling and retain the last snapshot as visibly disconnected on errors. Reconcile DOM without stealing keyboard focus.

## Out of scope
Browser-based configuration editing, arbitrary API playground requests, a hosted multiuser dashboard, remote dashboard authentication, changed notification thresholds, fabricated progress steps, and new check protocols.

## Testing / done when
1. Local mock HTTP requests prove manual PASS/FAIL/body assertions, timeout, target allowlist, concurrent-run rejection, and unchanged check/state/incident/outbox counts.
2. Scheduled executions still persist and advance thresholds; safe snapshots show actual history and flow steps, exclude private target details, and mark stale observations.
3. Host/origin/header checks reject foreign browser actions; nonloopback mode does not expose dashboard data/actions. Existing heartbeat tests still pass.
4. Browser tests drive real mock API checks, verify changed outcomes and history, clipboard-free controls, keyboard access, disconnected/empty states, 375px/desktop overflow, reduced motion, and detectable accessibility issues.
5. Rust and website gates pass. Manual visual verification uses an isolated demo monitor without the owner's Telegram credentials. Record evidence and remaining limitations in verification notes.
