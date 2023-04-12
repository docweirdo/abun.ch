-- Add migration script here
CREATE TABLE token (
    id CHAR(16) PRIMARY KEY,
    valid BOOLEAN NOT NULL,
    admin BOOLEAN NOT NULL,
    creator_id INTEGER NOT NULL REFERENCES creator(id)
);