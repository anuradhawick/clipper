CREATE TABLE IF NOT EXISTS tag_items (
    id TEXT PRIMARY KEY,
    tag_id TEXT NOT NULL,
    item_kind TEXT NOT NULL CHECK (item_kind IN ('clipboard', 'bookmark', 'note')),
    item_id TEXT NOT NULL,
    timestamp TEXT,
    UNIQUE(tag_id, item_kind, item_id),
    FOREIGN KEY(tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_tag_items_item
ON tag_items (item_kind, item_id);

CREATE INDEX IF NOT EXISTS idx_tag_items_tag
ON tag_items (tag_id);
