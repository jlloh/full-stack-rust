-- Your SQL goes here
CREATE TABLE queue (
    id INTEGER NOT NULL PRIMARY KEY,
    user TEXT NOT NULL, 
    is_selected BOOLEAN NOT NULL,
    is_processed BOOLEAN NOT NULL,
    is_abandoned BOOLEAN NOT NULL,
    updated_at TIMESTAMP NOT NULL
);


CREATE INDEX idx_is_selected ON queue(is_selected);

CREATE INDEX idx_is_processed on queue(is_processed);

CREATE INDEX idx_assigned_number ON queue(user, is_processed, is_abandoned);