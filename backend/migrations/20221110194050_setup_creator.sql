-- Add migration script here
CREATE TABLE creator (
    id SERIAL PRIMARY KEY,
    username VARCHAR(15) NOT NULL UNIQUE,
    password CHAR(60) NOT NULL,
    admin BOOLEAN NOT NULL
);

ALTER TABLE bunch ADD creator_id INTEGER REFERENCES bunch(id);