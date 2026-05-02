CREATE TABLE network_identity (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  keypair BLOB NOT NULL,
  created_at TEXT NOT NULL
);

CREATE TABLE network_trusted_peers (
  peer_id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  authorized_at TEXT NOT NULL
);
