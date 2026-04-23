CREATE TABLE IF NOT EXISTS clipboard (
    id TEXT PRIMARY KEY,
    entry BLOB NOT NULL,
    kind TEXT NOT NULL,
    timestamp TEXT
);

CREATE TABLE IF NOT EXISTS bookmarks (
    id TEXT PRIMARY KEY,
    url TEXT NOT NULL,
    text TEXT,
    image BLOB,
    timestamp TEXT
);

CREATE TABLE IF NOT EXISTS notes (
    id TEXT PRIMARY KEY,
    entry TEXT NOT NULL,
    created_time TEXT,
    updated_time TEXT
);

CREATE TABLE IF NOT EXISTS filters (
    id TEXT PRIMARY KEY,
    filter_regex TEXT NOT NULL,
    created_date TEXT
);

CREATE TABLE IF NOT EXISTS tags (
    id TEXT PRIMARY KEY,
    tag TEXT NOT NULL,
    kind TEXT NOT NULL,
    timestamp TEXT
);

CREATE TABLE IF NOT EXISTS settings (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    color TEXT NOT NULL,
    lighting TEXT NOT NULL,
    clipboardHistorySize INTEGER NOT NULL,
    bookmarkHistorySize INTEGER NOT NULL,
    globalShortcut TEXT
);
