# Pause and resume individual targets

2026-11-26 · Agent: codex. Owner asked to stop targets added to the monitor. Extend the incumbent local dashboard; preserve its form, shared dropdown, and information-site access.

## Decisions

- D1: Pause/Resume is reversible and per target. Keep configuration, history, incident state, threshold counters, and outbox. No target deletion in this change.
- D2: Store a constrained `paused` boolean on SQLite targets with a new migration. Configuration sync preserves it. New targets default to running; retaining the same database retains pauses across restart. TOML is untouched.
- D3: Use the shared target execution gate for pause/resume. An active execution finishes before pause is acknowledged. Scheduled execution checks the durable pause flag under the same gate immediately before making requests. No database transaction spans network work. Once pause succeeds, subsequent scheduled ticks skip execution and persistence.
- D4: Manual checks remain explicit actions and do not change scheduled state. Resume continues on the saved schedule. Existing incidents/counters remain frozen during pause; resuming advances them from actual observations. Already queued notifications continue in order; pausing does not delete or mute them.
- D5: Idempotent POST pause/resume endpoints require existing loopback, Host, Origin, and action-header boundaries. Finish admitted changes if the browser disconnects; categorical errors and uncertain-response recovery. Missing targets return 404; failed persistence never claims success.

## Interface

A secondary Pause monitoring / Resume monitoring button beside the selected target header uses Lucide icons. The target list and activity show PAUSED independently from incident evidence. Paused copy explains manual checks and queued alerts. Pending actions disable duplicate control/manual actions and show progress; errors retain the known state and suggest refreshing if uncertain. Keep controls mounted through polling, keyboard focus visible, and mobile wrapping contained. CLI/basic status also identify paused targets.

## Verification

Test before implementation: durable pause with sync/reopen, zero scheduled requests or new checks/incidents/outbox while paused, unchanged other targets, intentional manual tests, in-flight draining, idempotent controls, invalid database values, endpoint guards, failed writes and disconnected response. Browser proves history stops over more than an interval, survives reload, then resumes with real observations; include failure, focus, mobile and Axe. Release smoke upgrades a database made by the previous binary, pauses/restarts/resumes against the local demo. Run repository Rust/website gates in isolated/no-credential fixtures, inspect running/paused/pending/error desktop/mobile captures, then finish review/documentation.
