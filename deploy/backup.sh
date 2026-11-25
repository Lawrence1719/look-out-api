#!/usr/bin/env bash
set -euo pipefail
if [ "$#" -ne 2 ]; then
  echo 'Usage: deploy/backup.sh /path/to/look-out-api.db /path/to/backup-directory' >&2
  exit 2
fi
python3 - "$1" "$2" <<'PY'
import datetime, os, pathlib, sqlite3, sys
source = pathlib.Path(sys.argv[1]).resolve(strict=True)
directory = pathlib.Path(sys.argv[2]).resolve()
directory.mkdir(parents=True, exist_ok=True, mode=0o700)
stamp = datetime.datetime.now(datetime.timezone.utc).strftime('%Y%m%dT%H%M%S%fZ')
destination = directory / ('look-out-api-' + stamp + '.db')
fd = os.open(destination, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
os.close(fd)
try:
    with sqlite3.connect(source.as_uri() + '?mode=ro', uri=True) as origin:
        with sqlite3.connect(destination) as backup:
            origin.backup(backup)
            check = backup.execute('PRAGMA integrity_check').fetchone()
            if check != ('ok',):
                raise RuntimeError('backup integrity check failed')
except Exception:
    destination.unlink(missing_ok=True)
    raise
for old in sorted(directory.glob('look-out-api-????????T????????????Z.db'), reverse=True)[7:]:
    old.unlink()
print('SQLite backup created and integrity checked; keeping seven copies.')
PY
