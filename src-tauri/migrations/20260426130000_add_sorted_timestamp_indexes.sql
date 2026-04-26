CREATE INDEX IF NOT EXISTS idx_clipboard_timestamp_desc
ON clipboard (timestamp DESC);

CREATE INDEX IF NOT EXISTS idx_bookmarks_timestamp_desc
ON bookmarks (timestamp DESC);

CREATE INDEX IF NOT EXISTS idx_notes_created_time_desc
ON notes (created_time DESC);

CREATE INDEX IF NOT EXISTS idx_filters_created_date_desc
ON filters (created_date DESC);

CREATE INDEX IF NOT EXISTS idx_tags_timestamp_desc
ON tags (timestamp DESC);
