-- Add up migration script here

ALTER TABLE user
    ADD COLUMN ban_duration INT NULL;
ALTER TABLE user
    ADD COLUMN ban_ip TEXT NULL;
