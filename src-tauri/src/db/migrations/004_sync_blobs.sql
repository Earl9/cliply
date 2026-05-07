CREATE TABLE IF NOT EXISTS sync_blobs (
  id TEXT PRIMARY KEY,
  item_id TEXT NOT NULL,
  blob_type TEXT NOT NULL,
  local_path TEXT,
  remote_path TEXT,
  size_bytes INTEGER DEFAULT 0,
  hash TEXT NOT NULL,
  encrypted INTEGER DEFAULT 0,
  sync_status TEXT DEFAULT 'pending',
  created_at TEXT NOT NULL,
  uploaded_at TEXT NULL,
  deleted_at TEXT NULL,
  FOREIGN KEY (item_id) REFERENCES clipboard_items(id)
);

CREATE INDEX IF NOT EXISTS idx_sync_blobs_item_id ON sync_blobs(item_id);
CREATE INDEX IF NOT EXISTS idx_sync_blobs_blob_type ON sync_blobs(blob_type);
CREATE INDEX IF NOT EXISTS idx_sync_blobs_hash ON sync_blobs(hash);
CREATE INDEX IF NOT EXISTS idx_sync_blobs_sync_status ON sync_blobs(sync_status);
CREATE UNIQUE INDEX IF NOT EXISTS idx_sync_blobs_item_type_hash
  ON sync_blobs(item_id, blob_type, hash);
