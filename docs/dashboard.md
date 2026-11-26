# Local dashboard and visual API tests

The daemon includes a local dashboard. Its data comes from actual check executions and SQLite history. The public information website is separate and continues to use illustrative examples.

## Open it

Start the [local demo and monitor](./README.md#start-on-your-operating-system) using your OS guide. Enable the dashboard when starting the daemon:

| Shell | Command from the source checkout |
| --- | --- |
| Windows PowerShell | `.\target\release\look-out-api.exe run --bind 127.0.0.1:8080` |
| macOS / Linux | `./target/release/look-out-api run --bind 127.0.0.1:8080` |

Open **http://127.0.0.1:8080/** on the same machine. The `LOOK_OUT_API_BIND` environment setting can enable the same listener. Stop an existing daemon before restarting it; two daemons cannot share one database. No Node.js server or cloud account is needed to use the dashboard.

## Website and documentation

The dashboard header includes **Website** and **Documentation** links. They open in a new tab, keeping any unfinished API form available. By default they point to the public information website. To use a local preview instead, set `LOOK_OUT_API_SITE_URL=http://127.0.0.1:3000/` in your local environment and restart the daemon. This is optional; the dashboard itself does not need Node.js or the information-site server. The configured URL must be HTTP/HTTPS with no credentials, query, or fragment; invalid values fall back to the public site.

## Run a visual test

1. Select a configured target.
2. Click **Run check**. This runs the same real check engine as the CLI, including its timeout and configured assertions.
3. Read PASS/FAIL, completion time, latency, HTTP status when applicable, and the categorical failure reason. Flow targets show executed steps and stop at the first failed step. Passive security tests show returned findings. Heartbeat tests read the last ping; they do not send a synthetic ping.

An HTTP 200 can still fail when the expected response text is missing. A PASS proves only the assertions you configured; it does not prove every business function works. Configure a multi-step flow to test an actual sequence such as session creation followed by an authenticated request.

Manual tests do **not** write scheduled history, change incident counters, or send alerts. Their latest result is kept in the running daemon's memory and disappears on restart. Tests use the saved configuration. The Add API form saves basic HTTP checks; headers, request bodies, tokens, and advanced targets are configured in TOML. Be aware that a configured POST or multi-step flow executes its configured API operations when you run it.

## Add an API without restarting

1. Click **Add API** in the target list.
2. Enter a unique nonsecret name and the full HTTP/HTTPS URL. Choose GET or HEAD.
3. Set an expected status (blank accepts any 2xx), optional case-sensitive response text for GET, interval (at least five seconds), and a timeout shorter than that interval.
4. Click **Save & start monitoring**. The target is saved to the same TOML file used to start the daemon and scheduled immediately. Select it to see the first real observation or use **Run check** for a manual test.

Saving starts scheduled requests and uses your existing incident thresholds and alert settings. A failing endpoint can still be saved and monitored. Nothing is requested just by filling out the form. Keep URLs and expected text private; they are stored locally in your configuration, but excluded from dashboard snapshots and logs. Environment placeholders and embedded URL credentials are not accepted in this form.

Dropdowns support arrow keys, Home/End, and typing an option name. Enter or Space confirms, Tab confirms and moves on, and Escape closes without changing the selection.

Existing comments, secret placeholders, file permissions, and target state are preserved. Saves use an atomic file replacement and a sibling write lock. Avoid editing the configuration in another editor while adding APIs: if a changed file is detected, the save is rejected and you must restart the monitor to load that configuration. Symlinked, read-only, missing, or oversized configuration disables additions. Duplicate names, including names retained in local history, are rejected.

If a file saves but database registration fails, the dashboard explicitly says that a restart is needed. Check database access and restart; startup loads the saved API. If the connection drops during a save, refresh and check the target list before retrying. The form keeps your draft when a save fails.

Authentication headers, POST bodies, multi-step flows, and other check types remain available through TOML; restart after those edits. Editing or deleting targets through the dashboard is not yet implemented.

## Watch ongoing monitoring

The dashboard refreshes every two seconds while its tab is visible. It displays the latest 24 scheduled observations, the latest scheduled flow steps, up to five incidents, and the count of pending notifications. Each colored observation mark is one actual completed check, with a timestamp and result; the strip is not continuous uptime.

A single failed observation can appear while incident state is still UP: an incident opens only after the configured failure threshold. Recovery similarly waits for its success threshold. Unobserved or old state is labeled unknown/stale after two configured intervals. The connection banner explicitly identifies disconnected snapshots; old data is retained for inspection.

To verify a DOWN/RECOVERED transition safely, use only the included local demo. Leave the monitor running, stop the demo, wait for the failure threshold, restart the demo, and wait for recovery. Scheduled transitions can send Telegram alerts when delivery is configured. The manual **Run check** action does not replace this incident test.

## Pause or resume a target

Select a target and click **Pause monitoring**. An active check finishes before pause is acknowledged; the button shows progress while waiting. Once the target is marked **PAUSED**, its scheduled checks stop. Other targets keep running.

The target stays configured. History, incidents, threshold counters, and pending notifications are preserved. Manual dashboard tests and explicit one-off CLI checks remain available. Already queued alerts may still be delivered; pause does not clear or mute the notification queue. Open incidents remain as recorded until actual resumed observations reach the recovery threshold.

Click **Resume monitoring** to continue on the saved schedule. Use **Run check** if you want an immediate manual result. Pause state is saved in SQLite and survives restart when using the same database; it is not written into TOML. The CLI `status` command also labels paused targets. If a connection drops while changing monitoring, refresh and inspect the state before trying again. Pausing is separate from deleting a target; dashboard deletion is not yet available.

## Access boundary and troubleshooting

The dashboard and its action/data endpoints are enabled only when the daemon binds to a loopback IP. Use `127.0.0.1:8080` (or `[::1]:8080`) and visit that address directly. Host checks and same-origin action headers reject foreign browser requests; the dashboard does not enable cross-origin access. A nonloopback bind keeps the basic read-only status page and heartbeat endpoint but disables the dashboard APIs. Remote dashboard administration and authentication are not implemented.

If you run the daemon on another machine, use an SSH tunnel to its loopback port and open the forwarded local address. Keep the dashboard bound to loopback.

- **Checking…**: a manual or scheduled check is already in progress for that target. Wait for it to finish; target executions are serialized.
- **Disconnected**: confirm the daemon is running, then click Refresh. The information site at port 3000 is a different service.
- **No targets**: use Add API for an HTTP check, or configure advanced targets in TOML and restart.
- **Old configuration**: dashboard additions activate immediately; manual TOML edits require a daemon restart.
- **Uncertain manual result**: reconnect and inspect the latest manual result before retrying an API operation.

Target URLs, headers, bodies, tokens, and extracted flow values are excluded from dashboard responses. Keep target and flow-step names nonsecret. See [troubleshooting](./troubleshooting.md) for database, build-tool, and Telegram issues.

## Contributor verification

The dashboard assets are embedded in the Rust binary. Tests use the local-only [browser fixture](../examples/dashboard_fixture.rs), temporary SQLite, and mock API ports 4181/4182, with no Telegram delivery. From the source checkout run `cargo test --locked`; from the website directory, after `npm ci` and `npx playwright install chromium`, run `npm run test:dashboard`. Rust must be available on PATH for that browser suite. The normal `npm test` suite continues to test only the static information site.
