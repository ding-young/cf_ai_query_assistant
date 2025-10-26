-- migration 0001: Create the initial history table
CREATE TABLE history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    natural TEXT NOT NULL,
    sql TEXT NOT NULL,
    executed INTEGER DEFAULT 0 NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL
);