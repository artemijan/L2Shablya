-- Add up migration script here
ALTER TABLE user
    ADD COLUMN access_level INT NOT NULL DEFAULT 0;