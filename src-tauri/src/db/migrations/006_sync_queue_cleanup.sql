-- Runtime migration adds sync_events.abandoned_at only when missing because
-- SQLite does not support ALTER TABLE ADD COLUMN IF NOT EXISTS.
--
-- ALTER TABLE sync_events ADD COLUMN abandoned_at TEXT NULL;

CREATE INDEX IF NOT EXISTS idx_sync_events_abandoned_at
  ON sync_events(abandoned_at);

CREATE INDEX IF NOT EXISTS idx_sync_events_active_pending
  ON sync_events(synced_at, abandoned_at, created_at);

CREATE INDEX IF NOT EXISTS idx_sync_events_cleanup
  ON sync_events(item_id, synced_at, created_at);

CREATE INDEX IF NOT EXISTS idx_clipboard_items_sync_cleanup
  ON clipboard_items(sync_status, deleted_at)
  WHERE is_deleted = 1;

CREATE INDEX IF NOT EXISTS idx_sync_blobs_cleanup
  ON sync_blobs(item_id, sync_status, deleted_at);
