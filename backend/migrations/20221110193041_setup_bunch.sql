-- Add migration script here
CREATE TABLE bunch (
    id INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    title VARCHAR(35) NOT NULL,
    description VARCHAR(280),
    date DATE NOT NULL,
    expiration DATE,
    clickcounter INTEGER NOT NULL,
    uri CHAR(6) NOT NULL,
    password CHAR(60),
    fetchOpenGraph BOOLEAN NOT NULL,
    incognito BOOLEAN NOT NULL
);