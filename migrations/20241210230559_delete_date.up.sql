-- Add up migration script here
ALTER TABLE character
    ADD COLUMN delete_at DATETIME NULL;