CREATE TABLE IF NOT EXISTS events (
    id TEXT PRIMARY KEY,
    pubkey TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    kind INTEGER NOT NULL,
    content TEXT NOT NULL,
    sig TEXT NOT NULL,
    raw_json TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS wot_trust (
    source_pubkey TEXT NOT NULL,
    target_pubkey TEXT NOT NULL,
    trust_score REAL NOT NULL,
    updated_at INTEGER NOT NULL,
    PRIMARY KEY (source_pubkey, target_pubkey)
);
