-- Start transaction
BEGIN;

-- Disable triggers temporarily
SET session_replication_role = 'replica';

-- Drop dependent views
DROP VIEW IF EXISTS user_groups_view;

-- Alter user_groups table
ALTER TABLE user_groups
    ALTER COLUMN user_id TYPE bigint;

-- Recreate dependent views
CREATE VIEW user_groups_view AS
SELECT 
    ug.user_id,
    u.username AS user_name,
    ug.group_id,
    g.name AS group_name
FROM user_groups ug
JOIN users u ON ug.user_id = u.id
JOIN groups g ON ug.group_id = g.id;

-- Re-enable triggers
SET session_replication_role = 'origin';

COMMIT;
