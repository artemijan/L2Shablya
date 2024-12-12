-- Add up migration script here
CREATE TABLE character
(
    id      INTEGER PRIMARY KEY AUTOINCREMENT,
    name    TEXT NOT NULL,
    level   INT  NOT NULL DEFAULT 1,
    user_id INT  NOT NULL REFERENCES user (id) ON DELETE CASCADE
);