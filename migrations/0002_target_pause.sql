-- Durable per-target scheduling control; configuration membership remains separate.
-- Pause/resume never rewrites incident evidence or deletes queued notifications.
ALTER TABLE targets ADD COLUMN paused INTEGER NOT NULL DEFAULT 0 CHECK(paused IN (0,1));
