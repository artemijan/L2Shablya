-- Add down migration script here
ALTER TABLE user
    DROP COLUMN ban_ip;
ALTER TABLE user
    DROP COLUMN ban_duration;