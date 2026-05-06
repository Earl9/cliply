-- Runtime migration applies these columns only when missing because SQLite
-- does not support ALTER TABLE ADD COLUMN IF NOT EXISTS.
--
-- ALTER TABLE clipboard_items ADD COLUMN sync_id TEXT;
-- ALTER TABLE clipboard_items ADD COLUMN device_id TEXT;
-- ALTER TABLE clipboard_items ADD COLUMN revision INTEGER DEFAULT 1;
-- ALTER TABLE clipboard_items ADD COLUMN deleted_at TEXT NULL;
-- ALTER TABLE clipboard_items ADD COLUMN sync_status TEXT DEFAULT 'pending';
-- ALTER TABLE clipboard_items ADD COLUMN last_synced_at TEXT NULL;

CREATE TABLE IF NOT EXISTS devices (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  platform TEXT NOT NULL,
  created_at TEXT NOT NULL,
  last_seen_at TEXT NULL
);

CREATE TABLE IF NOT EXISTS sync_state (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS sync_events (
  id TEXT PRIMARY KEY,
  item_id TEXT,
  event_type TEXT NOT NULL,
  payload_json TEXT,
  created_at TEXT NOT NULL,
  synced_at TEXT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_clipboard_items_sync_id ON clipboard_items(sync_id);
CREATE INDEX IF NOT EXISTS idx_clipboard_items_device_id ON clipboard_items(device_id);
CREATE INDEX IF NOT EXISTS idx_clipboard_items_deleted_at ON clipboard_items(deleted_at);
CREATE INDEX IF NOT EXISTS idx_clipboard_items_sync_status ON clipboard_items(sync_status);
CREATE INDEX IF NOT EXISTS idx_sync_events_item_id ON sync_events(item_id);
CREATE INDEX IF NOT EXISTS idx_sync_events_synced_at ON sync_events(synced_at);
