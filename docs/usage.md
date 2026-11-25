# Configuration and everyday use

[All guides](./README.md) · [Telegram](./telegram.md) · [Troubleshooting](./troubleshooting.md)

## Choose the executable for your shell

The command examples below use `look-out-api` as shorthand. From a source checkout, substitute the appropriate executable prefix:

| Shell | Executable prefix |
| --- | --- |
| Windows PowerShell | `.\target\release\look-out-api.exe` |
| macOS / Linux | `./target/release/look-out-api` |

For example, `look-out-api status` becomes `.\target\release\look-out-api.exe status` in PowerShell. Optionally run `cargo install --path . --locked` to install the command into Cargo's binary directory, if that directory is on your PATH.

Run commands from the same working directory. By default, `.env`, `look-out-api.toml`, and the SQLite path are resolved there. Opening a different directory can select a different database or fail to find the config.

## Configure targets

Edit the private `look-out-api.toml` copied from [the example](../look-out-api.example.toml). Start with its local demo, then replace it with targets you control:

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

Target names are nonsecret labels with ASCII letters, digits, dots, hyphens, or underscores. They must be unique. Each interval is at least five seconds and its timeout is shorter than the interval.

Strings may reference `${VARIABLE_NAME}` from your environment or `.env`. Put secrets there instead of in shared configuration. A variable already set in the process environment takes precedence over the `.env` value. Both Telegram fields must be set together or both left empty.

Validate and then restart the daemon after editing target configuration or `.env`:

```sh
look-out-api validate
look-out-api run --bind 127.0.0.1:8080
```

Validation does not contact targets. An ordinary `check` makes one observation per target but does **not** advance daemon incident counters or send transition alerts. Use `run` for persistent monitoring.

Implemented checks are HTTP, TCP, DNS, multi-step flows, heartbeat, and initial passive security. Response expectations define what success means; a 200 response alone does not prove business correctness. See [coverage and limitations](../README.md#check-coverage) before relying on a planned feature.

## Database setup and storage

The default setting in [.env.example](../.env.example) is:

```dotenv
DATABASE_URL=sqlite://look-out-api.db
```

`status` or `run` creates the database file and applies the bundled schema migrations. `status` alone does not register targets or run checks; start the daemon to record observations. There is no database password, separate server, or manual SQL setup.

The database contains observation history, target state, incidents, findings, and pending notifications. Its WAL/SHM sidecar files are part of SQLite's operation. Keep the database on persistent local storage, give the monitor's OS user access to its parent directory, and keep only one daemon on that database. `run` rejects an in-memory database.

For a custom location, create its parent directory first. Use forward slashes in the SQLite URL, including on Windows. Examples below are illustrative paths to replace with your own:

| Platform | Example `DATABASE_URL` |
| --- | --- |
| Windows | `sqlite://C:/Users/your-user/look-out-data/look-out-api.db` |
| macOS | `sqlite:///Users/your-user/look-out-data/look-out-api.db` |
| Linux | `sqlite:///var/lib/look-out-api/look-out-api.db` |

You can override locations with global `--config PATH` and `--database sqlite://PATH` options. Use the same locations for `run`, `status`, `history`, `report`, and `alerts`.

## Inspect results

```sh
look-out-api status
look-out-api history --name local-api --limit 20
look-out-api report --since 24h
look-out-api report --security
look-out-api alerts
```

`status` marks observations older than two target intervals as unknown/stale. Reports calculate sample-based availability and successful-sample latency; gaps are not healthy time. Check commands return 0 on success, 1 on failed checks, and 2 for configuration/execution errors.

Stop a foreground daemon with Ctrl+C. Restarting with the same database preserves its state and pending alerts. Keep the host awake and reachable. The information site cannot run the monitor for you, and an unavailable monitor cannot report its own outage through that same process.

## Backups and restore

On macOS/Linux, the supplied [backup script](../deploy/backup.sh) uses SQLite's online backup API, checks integrity, and retains seven copies:

```sh
./deploy/backup.sh ./look-out-api.db /path/to/private-backups
```

Replace the backup directory with a location you control. The script requires bash and Python 3 with `sqlite3`.

On Windows, save the following as `backup_local.py` outside the repository. It uses Python's online backup API, refuses to overwrite an existing destination, and checks the result:

```python
import os
from pathlib import Path
import sqlite3
import sys

if len(sys.argv) != 3:
    raise SystemExit("Usage: backup_local.py SOURCE_DATABASE NEW_BACKUP_FILE")
try:
    source = Path(sys.argv[1]).resolve(strict=True)
    destination = Path(sys.argv[2]).resolve()
    destination.parent.mkdir(parents=True, exist_ok=True)
    fd = os.open(destination, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    os.close(fd)
except OSError:
    raise SystemExit("Cannot open source or create a new backup destination")
try:
    with sqlite3.connect(source.as_uri() + "?mode=ro", uri=True) as origin:
        with sqlite3.connect(destination) as backup:
            origin.backup(backup)
            if backup.execute("PRAGMA integrity_check").fetchone() != ("ok",):
                raise RuntimeError("Backup integrity check failed")
except Exception:
    destination.unlink(missing_ok=True)
    raise SystemExit("Backup failed; incomplete destination removed")
print("Backup created; integrity ok")
```

From PowerShell, replace the helper path below with its actual location and choose a **new** filename for every backup:

```powershell
py -3 "C:\path\to\backup_local.py" .\look-out-api.db "$env:LOCALAPPDATA\LookOutAPI\backups\look-out-api-backup-01.db"
```

Expected result: `Backup created; integrity ok`. The Windows example does not rotate backups automatically. Restrict the directory to your OS account and copy backups to a separate storage location. Do not copy just an actively written `.db` file while ignoring its WAL file. See [Python's backup API](https://docs.python.org/3/library/sqlite3.html#sqlite3.Connection.backup).

For restore, stop the daemon. Move the current database and its `-wal`/`-shm` companions aside, put a verified backup at the configured database path, restore its file access permissions, and restart. Preserve the old files until the restored monitor works. Test a restore before depending on a backup.

## Update a source installation

Back up the database and preserve `.env` and the private target config. Stop the daemon before replacing its executable:

```sh
git pull --ff-only
cargo build --release --locked
```

Run validation, then restart using the same config and database locations. Review the change notes before an update; migrations run when the new binary opens the database. If you installed the optional Cargo command, repeat `cargo install --path . --locked` too. Never replace a failed build with an assumed new executable.

For Linux service management and host-specific configuration, see [deployment](../deploy/README.md). Native Windows Service and macOS launchd templates are not supplied yet.
