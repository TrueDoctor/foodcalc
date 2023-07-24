-- Add migration script here
CREATE TABLE IF NOT EXISTS users (
  id SERIAL PRIMARY KEY,
  username VARCHAR(255) NOT NULL,
  email VARCHAR(255) NOT NULL,
  password_hash VARCHAR(255) NOT NULL,
  is_admin BOOLEAN DEFAULT FALSE,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Add unique constraint to username and email
ALTER TABLE users ADD CONSTRAINT unique_username UNIQUE (username);
ALTER TABLE users ADD CONSTRAINT unique_email UNIQUE (email);

CREATE TABLE IF NOT EXISTS groups (
  id SERIAL PRIMARY KEY,
  name VARCHAR(255) NOT NULL
);

-- Add unique constraint to group name
ALTER TABLE groups ADD CONSTRAINT unique_group_name UNIQUE (name)

CREATE TABLE IF NOT EXISTS user_groups (
  user_id INTEGER REFERENCES users(id),
  group_id INTEGER REFERENCES groups(id),
  PRIMARY KEY (user_id, group_id)
);

