-- Add migration script here
CREATE TABLE entry (
    id INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    url TEXT NOT NULL,
    clickcounter INTEGER,
    description VARCHAR(280),
    title VARCHAR(35),
    bunch_id INTEGER REFERENCES bunch(id)
);