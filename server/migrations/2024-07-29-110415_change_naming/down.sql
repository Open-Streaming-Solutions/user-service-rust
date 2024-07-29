-- This file should undo anything in `up.sql`
ALTER TABLE users RENAME COLUMN name TO user_name;
ALTER TABLE users RENAME COLUMN email TO user_email;

