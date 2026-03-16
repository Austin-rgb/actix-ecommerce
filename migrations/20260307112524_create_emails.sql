-- Add migration script here
CREATE TABLE IF NOT EXISTS emails (
            user TEXT NOT NULL,
            email TEXT NOT NULL UNIQUE,
            PRIMARY KEY (user, email)
)