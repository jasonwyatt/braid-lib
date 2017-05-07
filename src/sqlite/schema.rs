pub const SCHEMA: &'static str = "
BEGIN;

/* Accounts */
CREATE TABLE IF NOT EXISTS accounts (
    id TEXT NOT NULL PRIMARY KEY,
    salt TEXT NOT NULL,
    api_secret_hash TEXT NOT NULL
);

/* Vertices */
CREATE TABLE IF NOT EXISTS vertices (
    id TEXT NOT NULL PRIMARY KEY,
    owner_id TEXT NOT NULL,
    type TEXT NOT NULL
);

/* Edges */
CREATE TABLE IF NOT EXISTS edges (
    id TEXT NOT NULL PRIMARY KEY,
    outbound_id TEXT NOT NULL,
    type TEXT NOT NULL,
    inbound_id TEXT NOT NULL,
    update_timestamp INTEGER NOT NULL,
    weight REAL NOT NULL
);

CREATE INDEX IF NOT EXISTS ix_edges_update_timestamp ON edges (update_timestamp);
CREATE INDEX IF NOT EXISTS ix_edges_inbound_id ON edges (inbound_id);

/* Global metadata */
CREATE TABLE IF NOT EXISTS global_metadata (
    name TEXT NOT NULL PRIMARY KEY,
    value TEXT NOT NULL
);

/* Account metadata */
CREATE TABLE IF NOT EXISTS account_metadata (
    owner_id TEXT NOT NULL,
    name TEXT NOT NULL,
    value TEXT NOT NULL,
    PRIMARY KEY (owner_id, name)
);

/* Vertex metadata */
CREATE TABLE IF NOT EXISTS vertex_metadata (
    owner_id TEXT NOT NULL,
    name TEXT NOT NULL,
    value TEXT NOT NULL,
    PRIMARY KEY (owner_id, name)
);

/* Metadata */
CREATE TABLE IF NOT EXISTS edge_metadata (
    owner_id TEXT NOT NULL,
    name TEXT NOT NULL,
    value TEXT NOT NULL,
    PRIMARY KEY (owner_id, name)
);
END;
";
