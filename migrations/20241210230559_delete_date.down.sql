-- Add down migration script here
ALTER TABLE character
    DROP COLUMN delete_at;