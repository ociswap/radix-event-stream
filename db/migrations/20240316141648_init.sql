-- migrate:up
CREATE TABLE IF NOT EXISTS items (
    id INTEGER PRIMARY KEY, value BLOB NOT NULL
)
-- migrate:down