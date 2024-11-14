-- Add down migration script here
ALTER TABLE user
DROP COLUMN access_level;