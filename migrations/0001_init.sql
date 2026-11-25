CREATE TABLE targets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    type TEXT NOT NULL CHECK(type IN ('http','tcp','dns','flow','security','heartbeat')),
    active INTEGER NOT NULL DEFAULT 1 CHECK(active IN (0,1)),
    interval_secs INTEGER NOT NULL CHECK(interval_secs >= 5),
    created_at TEXT NOT NULL
);
CREATE TABLE checks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    target_id INTEGER NOT NULL REFERENCES targets(id) ON DELETE CASCADE,
    checked_at TEXT NOT NULL,
    ok INTEGER NOT NULL CHECK(ok IN (0,1)),
    status_code INTEGER,
    latency_ms INTEGER NOT NULL CHECK(latency_ms >= 0),
    failed_step TEXT,
    error TEXT
);
CREATE INDEX idx_checks_target_time ON checks(target_id, checked_at DESC);
CREATE TABLE check_steps (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    check_id INTEGER NOT NULL REFERENCES checks(id) ON DELETE CASCADE,
    step_name TEXT NOT NULL,
    ok INTEGER NOT NULL CHECK(ok IN (0,1)),
    status_code INTEGER,
    latency_ms INTEGER NOT NULL CHECK(latency_ms >= 0),
    error TEXT
);
CREATE TABLE target_state (
    target_id INTEGER PRIMARY KEY REFERENCES targets(id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'unknown' CHECK(status IN ('up','down','unknown')),
    consecutive_fail INTEGER NOT NULL DEFAULT 0 CHECK(consecutive_fail >= 0),
    consecutive_ok INTEGER NOT NULL DEFAULT 0 CHECK(consecutive_ok >= 0),
    last_changed_at TEXT,
    last_checked_at TEXT,
    last_alerted_at TEXT
);
CREATE TABLE incidents (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    target_id INTEGER NOT NULL REFERENCES targets(id) ON DELETE CASCADE,
    started_at TEXT NOT NULL,
    resolved_at TEXT,
    reason TEXT
);
CREATE UNIQUE INDEX idx_one_open_incident ON incidents(target_id) WHERE resolved_at IS NULL;
CREATE TABLE heartbeats (
    target_id INTEGER PRIMARY KEY REFERENCES targets(id) ON DELETE CASCADE,
    last_ping_at TEXT NOT NULL
);
CREATE TABLE security_findings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    target_id INTEGER NOT NULL REFERENCES targets(id) ON DELETE CASCADE,
    check_id INTEGER REFERENCES checks(id) ON DELETE SET NULL,
    rule TEXT NOT NULL,
    severity TEXT NOT NULL CHECK(severity IN ('info','low','medium','high')),
    detail TEXT NOT NULL,
    first_seen_at TEXT NOT NULL,
    last_seen_at TEXT NOT NULL,
    resolved_at TEXT,
    UNIQUE(target_id, rule, detail)
);
CREATE TABLE notifications (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    target_id INTEGER NOT NULL REFERENCES targets(id) ON DELETE CASCADE,
    event_key TEXT NOT NULL UNIQUE,
    kind TEXT NOT NULL CHECK(kind IN ('DOWN','RECOVERED')),
    message TEXT NOT NULL,
    created_at TEXT NOT NULL,
    delivered_at TEXT,
    attempts INTEGER NOT NULL DEFAULT 0 CHECK(attempts >= 0),
    next_attempt_at TEXT NOT NULL,
    permanent_failure INTEGER NOT NULL DEFAULT 0 CHECK(permanent_failure IN (0,1)),
    last_error TEXT
);
CREATE INDEX idx_notifications_pending ON notifications(delivered_at, next_attempt_at);
