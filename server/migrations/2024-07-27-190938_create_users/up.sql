-- Your SQL goes here
CREATE TABLE users (
                       id UUID PRIMARY KEY,
                       username VARCHAR NOT NULL,
                       email VARCHAR NOT NULL
);