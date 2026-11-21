-- CREATE TABLE roles
-- (
--     slug      VARCHAR(32) PRIMARY KEY,           -- e.g. 'viewer', 'editor', 'manager', or custom 'kitchen'
--     name      VARCHAR(64) NOT NULL,
--     is_system BOOLEAN     NOT NULL DEFAULT FALSE -- protects built-in presets from deletion
-- );
--
-- CREATE TABLE role_permissions
-- (
--     role_slug  VARCHAR(32) NOT NULL REFERENCES roles (slug) ON DELETE CASCADE,
--     permission VARCHAR(64) NOT NULL, -- matches Permission::as_str(), e.g. 'events:edit'
--     PRIMARY KEY (role_slug, permission)
-- );
--
--
-- -- 2. Seed built-in roles BEFORE backfilling user_group_roles
-- INSERT INTO roles (slug, name, is_system)
-- VALUES ('viewer', 'Viewer', TRUE),
--        ('editor', 'Editor', TRUE),
--        ('manager', 'Manager', TRUE);
--
-- INSERT INTO role_permissions (role_slug, permission)
-- VALUES
--     -- Viewer
--     ('viewer', 'events:view'),
--     ('viewer', 'events:view_calculation'),
--     ('viewer', 'recipes:view'),
--     ('viewer', 'ingredients:view'),
--     ('viewer', 'inventories:view'),
--     -- Editor (includes Viewer)
--     ('editor', 'events:view'),
--     ('editor', 'events:view_calculation'),
--     ('editor', 'recipes:view'),
--     ('editor', 'ingredients:view'),
--     ('editor', 'inventories:view'),
--     ('editor', 'events:create'),
--     ('editor', 'events:edit'),
--     ('editor', 'events:clone'),
--     ('editor', 'events:manage_meals'),
--     ('editor', 'events:manage_shopping'),
--     ('editor', 'events:shopping:add_tour'),
--     ('editor', 'events:shopping:edit_tour'),
--     ('editor', 'status_board:operate'),
--     ('editor', 'recipes:create'),
--     ('editor', 'recipes:edit'),
--     ('editor', 'ingredients:create'),
--     ('editor', 'ingredients:edit'),
--     ('editor', 'inventories:create'),
--     ('editor', 'inventories:edit'),
--     -- Manager (includes Editor + destructive/admin group actions)
--     ('manager', 'events:view'),
--     ('manager', 'events:view_calculation'),
--     ('manager', 'recipes:view'),
--     ('manager', 'ingredients:view'),
--     ('manager', 'inventories:view'),
--     ('manager', 'events:create'),
--     ('manager', 'events:edit'),
--     ('manager', 'events:clone'),
--     ('manager', 'events:manage_meals'),
--     ('manager', 'events:manage_shopping'),
--     ('manager', 'events:shopping:add_tour'),
--     ('manager', 'events:shopping:edit_tour'),
--     ('manager', 'status_board:operate'),
--     ('manager', 'recipes:create'),
--     ('manager', 'recipes:edit'),
--     ('manager', 'ingredients:create'),
--     ('manager', 'ingredients:edit'),
--     ('manager', 'inventories:create'),
--     ('manager', 'inventories:edit'),
--     ('manager', 'events:delete'),
--     ('manager', 'events:transfer_owner'),
--     ('manager', 'recipes:delete'),
--     ('manager', 'recipes:transfer_owner'),
--     ('manager', 'ingredients:delete'),
--     ('manager', 'ingredients:transfer_owner'),
--     ('manager', 'inventories:delete'),
--     ('manager', 'inventories:transfer_owner'),
--     ('manager', 'group:manage_members'),
--     ('manager', 'group:delete');


-- 3. Multi-role group memberships (replaces plain user_groups)
CREATE TABLE IF NOT EXISTS user_group_roles
(
    user_id   bigint      NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    group_id  bigint     NOT NULL REFERENCES groups (id) ON DELETE CASCADE,
    role_slug VARCHAR(32) NOT NULL, --REFERENCES roles (slug) ON DELETE RESTRICT,
    PRIMARY KEY (user_id, group_id, role_slug)
);

-- Backfill: existing user_groups members previously had full access, so grant 'manager'
INSERT INTO user_group_roles (user_id, group_id, role_slug)
SELECT user_id, group_id, 'manager'
FROM user_groups
ON CONFLICT DO NOTHING;

-- Drop dependent views
DROP VIEW IF EXISTS user_groups_view;

DROP TABLE IF EXISTS user_groups;

-- 4. Direct per-event collaborators
CREATE TABLE IF NOT EXISTS event_user_roles
(
    user_id   BIGINT      NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    event_id  INTEGER     NOT NULL REFERENCES events (event_id) ON DELETE CASCADE,
    role_slug VARCHAR(32) NOT NULL, -- REFERENCES roles (slug) ON DELETE RESTRICT,
    PRIMARY KEY (user_id, event_id, role_slug)
);
-- 5. Granular global permissions
CREATE TABLE IF NOT EXISTS user_global_permissions (
                                         user_id    BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                                         permission VARCHAR(64) NOT NULL,
                                         PRIMARY KEY (user_id, permission)
);

INSERT INTO user_global_permissions (user_id, permission)
SELECT id, 'admin:manage_users' FROM users WHERE is_admin = TRUE
UNION ALL
SELECT id, 'admin:manage_master_data' FROM users WHERE is_admin = TRUE
ON CONFLICT DO NOTHING;

-- 6 Shoppingtour completion
ALTER TABLE shopping_tours
    ADD COLUMN IF NOT EXISTS completed BOOLEAN NOT NULL DEFAULT FALSE;