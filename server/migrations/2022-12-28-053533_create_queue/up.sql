-- Your SQL goes here
CREATE TABLE queue (
    user TEXT NOT NULL PRIMARY KEY, 
    is_selected BOOLEAN NOT NULL,
    is_processed BOOLEAN NOT NULL,
    updated_at TIMESTAMP NOT NULL
);

CREATE INDEX idx_is_selected ON queue(is_selected);

CREATE INDEX idx_is_processed on queue(is_processed);