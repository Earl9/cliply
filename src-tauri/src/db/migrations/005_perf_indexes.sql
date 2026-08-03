-- Partial index backing the hot list query:
--   WHERE is_deleted = 0 AND deleted_at IS NULL
--   ORDER BY is_pinned DESC, copied_at DESC
CREATE INDEX IF NOT EXISTS idx_clipboard_items_active_order
  ON clipboard_items(is_pinned DESC, copied_at DESC)
  WHERE is_deleted = 0 AND deleted_at IS NULL;

-- Duplicate-hash lookup only ever targets live rows.
CREATE INDEX IF NOT EXISTS idx_clipboard_items_active_hash
  ON clipboard_items(hash)
  WHERE is_deleted = 0 AND deleted_at IS NULL;

-- Sync export reads events in creation order; pending scan filters on synced_at.
CREATE INDEX IF NOT EXISTS idx_sync_events_created_at ON sync_events(created_at);

-- Orphan-blob cleanup and per-item blob lookups filter on (item_id, deleted_at).
CREATE INDEX IF NOT EXISTS idx_sync_blobs_item_deleted ON sync_blobs(item_id, deleted_at);
