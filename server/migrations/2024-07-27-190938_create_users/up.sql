-- Your SQL goes here
CREATE TABLE users (
                       id UUID PRIMARY KEY,
                       username VARCHAR NOT NULL UNIQUE,
                       email VARCHAR NOT NULL UNIQUE
);