CREATE VIRTUAL TABLE IF NOT EXISTS clipboard_items_fts USING fts5(
  item_id UNINDEXED,
  title,
  preview_text,
  normalized_text,
  source_app
);
