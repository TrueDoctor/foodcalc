-- Add is_personal flag to groups
ALTER TABLE groups ADD COLUMN is_personal BOOLEAN NOT NULL DEFAULT FALSE;

-- Fix user_groups to use BIGINT for user_id (matches users.id type)
-- (already done in fix_user_group_key_type migration, but groups.id is INT so group_id stays INT)

-- Create a personal group for each existing user and add them to it
INSERT INTO groups (name, is_personal)
SELECT username, TRUE FROM users
ON CONFLICT (name) DO NOTHING;

INSERT INTO user_groups (user_id, group_id)
SELECT u.id, g.id
FROM users u
JOIN groups g ON g.name = u.username AND g.is_personal = TRUE
ON CONFLICT DO NOTHING;

-- Add group_id to recipes, defaulting each record to the personal group of its owner
ALTER TABLE recipes ADD COLUMN group_id INT REFERENCES groups(id);

UPDATE recipes r
SET group_id = g.id
FROM users u
JOIN groups g ON g.name = u.username AND g.is_personal = TRUE
WHERE u.id = r.owner_id;

ALTER TABLE recipes ALTER COLUMN group_id SET NOT NULL;
ALTER TABLE recipes DROP CONSTRAINT recipe_owner_fk;
ALTER TABLE recipes DROP COLUMN owner_id;

-- Add group_id to events
ALTER TABLE events ADD COLUMN group_id INT REFERENCES groups(id);

UPDATE events e
SET group_id = g.id
FROM users u
JOIN groups g ON g.name = u.username AND g.is_personal = TRUE
WHERE u.id = e.owner_id;

ALTER TABLE events ALTER COLUMN group_id SET NOT NULL;
ALTER TABLE events DROP CONSTRAINT event_owner_fk;
ALTER TABLE events DROP COLUMN owner_id;

-- Add group_id to ingredients
ALTER TABLE ingredients ADD COLUMN group_id INT REFERENCES groups(id);

UPDATE ingredients i
SET group_id = g.id
FROM users u
JOIN groups g ON g.name = u.username AND g.is_personal = TRUE
WHERE u.id = i.owner_id;

ALTER TABLE ingredients ALTER COLUMN group_id SET NOT NULL;
-- ingredients had a constraint named event_owner_fk (typo in previous migration)
ALTER TABLE ingredients DROP CONSTRAINT event_owner_fk;
ALTER TABLE ingredients DROP COLUMN owner_id;

-- Add group_id to inventories
ALTER TABLE inventories ADD COLUMN group_id INT REFERENCES groups(id);

UPDATE inventories inv
SET group_id = g.id
FROM users u
JOIN groups g ON g.name = u.username AND g.is_personal = TRUE
WHERE u.id = inv.owner_id;

ALTER TABLE inventories ALTER COLUMN group_id SET NOT NULL;
ALTER TABLE inventories DROP CONSTRAINT event_owner_fk;
ALTER TABLE inventories DROP COLUMN owner_id;
