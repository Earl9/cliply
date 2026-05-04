CREATE TABLE IF NOT EXISTS clipboard_items (
  id TEXT PRIMARY KEY,
  type TEXT NOT NULL,
  title TEXT,
  preview_text TEXT,
  normalized_text TEXT,
  source_app TEXT,
  source_window TEXT,
  hash TEXT NOT NULL,
  size_bytes INTEGER DEFAULT 0,
  is_pinned INTEGER DEFAULT 0,
  is_favorite INTEGER DEFAULT 0,
  is_deleted INTEGER DEFAULT 0,
  sensitive_score INTEGER DEFAULT 0,
  copied_at TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  used_count INTEGER DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_clipboard_items_hash ON clipboard_items(hash);
CREATE INDEX IF NOT EXISTS idx_clipboard_items_type ON clipboard_items(type);
CREATE INDEX IF NOT EXISTS idx_clipboard_items_copied_at ON clipboard_items(copied_at);
CREATE INDEX IF NOT EXISTS idx_clipboard_items_pinned ON clipboard_items(is_pinned);

CREATE TABLE IF NOT EXISTS clipboard_formats (
  id TEXT PRIMARY KEY,
  item_id TEXT NOT NULL,
  format_name TEXT NOT NULL,
  mime_type TEXT,
  data_kind TEXT NOT NULL,
  data_text TEXT,
  data_path TEXT,
  size_bytes INTEGER DEFAULT 0,
  priority INTEGER DEFAULT 0,
  created_at TEXT NOT NULL,
  FOREIGN KEY (item_id) REFERENCES clipboard_items(id)
);

CREATE TABLE IF NOT EXISTS clipboard_tags (
  item_id TEXT NOT NULL,
  tag TEXT NOT NULL,
  created_at TEXT NOT NULL,
  PRIMARY KEY (item_id, tag),
  FOREIGN KEY (item_id) REFERENCES clipboard_items(id)
);

CREATE TABLE IF NOT EXISTS settings (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
