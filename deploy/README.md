# Deployment

Agent: codex. Deployment templates require live verification on the chosen host. No cloud resources have been created.

For a first local run, start with the [Windows, macOS, or Linux setup guides](../docs/README.md). Shared instructions cover [database storage and backups](../docs/usage.md) and [Telegram setup and delivery verification](../docs/telegram.md). These deployment templates are a separate step from running the demo in a terminal.

## Persistent host

Run the daemon on a host that can reach the targets, separate from the services it monitors. Before choosing a free host, verify its current free-resource limits, account requirements, and idle-reclamation policy. This repository does not provision Oracle resources or guarantee that any provider remains free.

On a Linux host, build or copy the appropriate binary. Create a system account named `look-out-api`, put the binary at `/opt/look-out-api/look-out-api`, and create `/var/lib/look-out-api` owned by that user. Put the target config at `/opt/look-out-api/look-out-api.toml` and secrets at `/opt/look-out-api/.env`. Make both files readable only by the service user (mode 600). Set the database path to `/var/lib/look-out-api/look-out-api.db`.

Copy [look-out-api.service](./look-out-api.service) to the host's systemd unit directory, then enable and start that unit. Validate the config as the service user before starting. The service handles SIGTERM and Ctrl+C, finishes in-flight work, and retains its outbox for restart.

No inbound port is required for polling checks. An optional dashboard/status/heartbeat endpoint is enabled with `LOOK_OUT_API_BIND` or `--bind`; heartbeat targets enable it automatically at `127.0.0.1:8080`. Use an SSH tunnel to reach a loopback-only endpoint. The [live dashboard and manual test actions](../docs/dashboard.md) are available only on loopback binds; nonloopback binds retain the basic status page and heartbeat endpoint. Heartbeats need a configured `LOOK_OUT_API_HEARTBEAT_TOKEN` and a network path to the daemon. Recovery follows the same configured consecutive-success threshold as other targets.

## Container demonstration

The [Dockerfile](../Dockerfile) builds the monitor and runs it as UID/GID 10001. The Compose example places a local demo API on an internal network and attaches the monitor to that network plus its normal outbound network for Telegram. It publishes no ports.

From the repository root:

```sh
docker compose -f deploy/compose.yml up --build -d
docker compose -f deploy/compose.yml logs monitor
docker compose -f deploy/compose.yml down
```

Set Telegram environment variables before running Compose to enable notification delivery. The named volume retains history across restarts. A bind-mounted data directory instead must be writable by UID 10001. Customize the example config for your own private services.

The local Docker engine was unavailable during the initial build, so container build/run verification remains pending.

## Backups and restore

Schedule the following nightly on the host using its scheduler:

```sh
./deploy/backup.sh /var/lib/look-out-api/look-out-api.db /a/second/backup/location
```

The script uses SQLite's online backup API, checks integrity, creates mode-600 files, and keeps seven copies. Copy these to a separate machine or storage location you control. For restore, stop the daemon, move the current database and its WAL/SHM companions aside, copy a known-good backup into place, restore service-user ownership/permissions, and restart. Test a restore before relying on it.

## GitHub Actions

[CI](../.github/workflows/ci.yml) validates Rust on Linux, macOS and Windows, and builds/tests the static website. Set required branch checks after the first workflow run; keep changes on feature branches and merge through review.

[Site publishing](../.github/workflows/deploy-main.yml) waits for successful CI on a push to `main`, then exports and deploys to GitHub Pages. Enable Pages with GitHub Actions as the build source in repository settings. Build-time public variables derive from the actual repository and Pages configuration, including the repository base path. The information site requires no daemon or database.

[Releases](../.github/workflows/release.yml) validate a version tag, build Linux x86-64 musl, macOS Apple Silicon, and Windows x86-64 binaries, and attach archives and SHA-256 checksums to a GitHub Release. Additional architectures are roadmap work. The source uses the MIT license. Complete the live verification checklist before publishing a release.

[Second watcher](../.github/workflows/watch.yml) remains inactive until `LOOK_OUT_API_CONFIG_B64` is configured as a repository secret. Add `TELEGRAM_BOT_TOKEN`, `TELEGRAM_CHAT_ID`, and any environment secrets referenced by the config. The template passes `API_TOKEN`; explicitly add other referenced variables to that step's environment. Use public targets and a TCP target for the primary monitor's host, with generic nonsecret target names. Serialize runs and save a fresh cache entry even when checks fail; cache keys are immutable, so restoring and saving a single fixed key would lose state updates.

Scheduled Actions are best-effort and caches can be evicted. Missing or corrupt state restarts from unknown and can repeat an incident notification. Verify current Actions availability and billing rules for the selected repository visibility before enabling scheduled work; the project requires a setup that stays within free allowances. Watcher configuration and state are sensitive operational files and must not be committed.

## Live verification

- Configure real targets and Telegram credentials.
- Prove DOWN and RECOVERED arrive in the correct order.
- Stop and restart the daemon while a notification is pending.
- Reboot the host and confirm automatic restart and retained history.
- Test backup restore.
- Enable GitHub Pages and verify the public site and social sharing metadata.
- Verify separate watcher state across two scheduled runs, then stop the primary host and observe its alert.
