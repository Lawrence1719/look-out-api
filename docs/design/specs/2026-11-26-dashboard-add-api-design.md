# Add API from the local dashboard

2026-11-26 · Agent: codex · Extension of [local dashboard](./2026-11-25-local-dashboard-design.md).

## Goal and sources
The owner requested adding APIs directly in the dashboard after the current guide required editing TOML and restarting. Reuse [Config validation](../../../src/config.rs), [dashboard execution gates](../../../src/dashboard.rs), the [scheduler](../../../src/scheduler.rs), and the established [Operate surface](../surfaces/local-dashboard.md).

## Implementation decisions
- D1 (owner request): an Add API form on the local dashboard saves a target.
- D2 (implementation default): first version supports HTTP GET/HEAD, nonsecret unique name, URL, optional expected status/text, interval, timeout. Existing advanced config remains available through TOML. No credentials/header/JSON-body form or URL-based auto-discovery.
- D3 (implementation default): append a safely serialized target block to the original raw TOML, preserving comments and environment placeholders. Use a private sibling write lock, temporary file, fsync, atomic replacement, and original permissions. Reject changed/symlinked/unwritable configuration; never rewrite interpolated secrets back to disk.
- D4 (implementation default): save first, then register the target/state in a database transaction, install its shared runner, and notify the scheduler. Existing runners and state stay intact. A crash after file save is recovered by normal startup sync. If activation fails after save, return an explicit saved-but-restart-needed result; never claim monitoring started.
- D5 (existing boundary): loopback only, local Host, exact Origin, custom action header, bounded JSON payload, categorical errors, no request configuration in snapshots/logs. Saving authorizes scheduled requests and normal incident alerts for the added target.

## Data and workflow
No schema migration. New target and initial unknown state register together; checks/incident/outbox continue through Store::record. Dynamic runner registration is serialized, shares existing target gates/global capacity, and uses a watch channel so scheduler registration cannot lose additions. Snapshot includes whether adding is available, but never the config path or raw private configuration.

Add API opens an inline form in the main area: name/URL lead, HTTP expectations and timing follow. Labels, validation, save/cancel, busy/error states, focus return, and a success status are explicit. Preserve draft values during polling/errors. Show the new target immediately, with waiting/running/actual first result; never fabricate success. On cancellation no request is sent. No automatic network validation before Save; a legitimate unavailable API can still be monitored.

## Verification plan
1. Test-first: invalid/duplicate input, no file writes on validation failure, raw comments/placeholders preserved, external file changes rejected, concurrent additions serialized, config survives reopen/restart, no writes to existing counters/outbox.
2. Register only after file commit; prove new scheduled execution reaches a local mock and persists real history, old target state remains unchanged, snapshot omits request details.
3. Prove Host/Origin/action/body-limit checks; cancellation/response loss must not cause duplicate configuration or misleading success.
4. Browser: add local API, observe target and actual scheduled PASS, run manual test, reject duplicates/invalid timeout, preserve form draft through polling, cancel, keyboard/focus, mobile and Axe. Use temporary configuration/SQLite and local mocks, no owner's credentials.
5. Rust fmt/Clippy/tests, website gates and dashboard browser suite. Review rendered desktop/mobile form and saved state; record evidence and native platform limits.

Done when the owner can add a configured API in the local browser, see actual monitoring start without restart, and retain it after a daemon restart. Editing/deleting targets, POST/body/auth forms, remote administration, and general config hot reload remain outside this change.
