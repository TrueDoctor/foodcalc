-- DB-backed role → permission assignments (RBAC R3).
--
-- Until now the role→permission mapping was hardcoded in
-- foodlib_new/src/policy/roles.rs (BuiltInRole::permissions). This migration
-- creates the previously-only-commented tables and seeds them from that
-- catalog, making the DATABASE the source of truth. The permission-management
-- UI (legacy /admin/permissions matrix) edits these rows directly;
-- BuiltInRole::permissions remains only as a fallback for legacy slugs
-- missing from the table (e.g. the old 'editor' alias).
--
-- NOTE: keep the seed in sync with BuiltInRole::permissions() — it is a
-- one-time snapshot of the catalog at migration time.

CREATE TABLE IF NOT EXISTS roles
(
    slug      VARCHAR(32) PRIMARY KEY,           -- e.g. 'viewer', 'cook', or custom 'kitchen'
    name      VARCHAR(64) NOT NULL,
    is_system BOOLEAN     NOT NULL DEFAULT FALSE -- protects built-in presets from deletion
);

CREATE TABLE IF NOT EXISTS role_permissions
(
    role_slug  VARCHAR(32) NOT NULL REFERENCES roles (slug) ON DELETE CASCADE,
    permission VARCHAR(64) NOT NULL, -- matches Permission::as_str(), e.g. 'events:edit'
    PRIMARY KEY (role_slug, permission)
);

INSERT INTO roles (slug, name, is_system) VALUES
    ('viewer',  'Leser',       TRUE),
    ('finance', 'Finanzen',    TRUE),
    ('cook',    'Koch',        TRUE),
    ('shopper', 'Einkauf',     TRUE),
    ('stocker', 'Lager',       TRUE),
    ('serving', 'Ausgabe',     TRUE),
    ('team',    'Teammitglied',TRUE),
    ('manager', 'Leitung',     TRUE)
ON CONFLICT (slug) DO NOTHING;

INSERT INTO role_permissions (role_slug, permission) VALUES
    -- viewer (Leser)
    ('viewer', 'events:view'),
    ('viewer', 'events:export'),
    ('viewer', 'recipes:view'),
    ('viewer', 'recipes:export'),
    ('viewer', 'ingredients:view'),
    ('viewer', 'inventories:view'),
    -- finance (Finanzen)
    ('finance', 'events:view'),
    ('finance', 'events:view_calculation'),
    ('finance', 'events:export'),
    ('finance', 'recipes:view'),
    ('finance', 'recipes:export'),
    ('finance', 'ingredients:view'),
    ('finance', 'inventories:view'),
    -- cook (Koch)
    ('cook', 'events:view'),
    ('cook', 'events:view_calculation'),
    ('cook', 'events:export'),
    ('cook', 'events:manage_meals'),
    ('cook', 'status_board:operate'),
    ('cook', 'recipes:view'),
    ('cook', 'recipes:export'),
    ('cook', 'recipes:create'),
    ('cook', 'recipes:edit'),
    ('cook', 'ingredients:view'),
    ('cook', 'ingredients:create'),
    ('cook', 'ingredients:edit'),
    ('cook', 'inventories:view'),
    ('cook', 'inventories:edit_stock'),
    -- shopper (Einkauf)
    ('shopper', 'events:view'),
    ('shopper', 'events:view_calculation'),
    ('shopper', 'events:export'),
    ('shopper', 'events:shopping:add_tour'),
    ('shopper', 'events:shopping:edit_tour'),
    ('shopper', 'events:shopping:complete_tour'),
    ('shopper', 'recipes:view'),
    ('shopper', 'ingredients:view'),
    ('shopper', 'inventories:view'),
    ('shopper', 'inventories:edit_stock'),
    -- stocker (Lager)
    ('stocker', 'events:view'),
    ('stocker', 'events:view_calculation'),
    ('stocker', 'events:shopping:complete_tour'),
    ('stocker', 'recipes:view'),
    ('stocker', 'ingredients:view'),
    ('stocker', 'ingredients:edit'),
    ('stocker', 'inventories:view'),
    ('stocker', 'inventories:create'),
    ('stocker', 'inventories:edit'),
    ('stocker', 'inventories:edit_stock'),
    -- serving (Ausgabe)
    ('serving', 'events:view'),
    ('serving', 'events:export'),
    ('serving', 'status_board:operate'),
    ('serving', 'recipes:view'),
    ('serving', 'recipes:export'),
    ('serving', 'ingredients:view'),
    ('serving', 'inventories:view'),
    -- team (Teammitglied)
    ('team', 'events:view'),
    ('team', 'events:view_calculation'),
    ('team', 'events:export'),
    ('team', 'events:create'),
    ('team', 'events:edit'),
    ('team', 'events:clone'),
    ('team', 'events:manage_meals'),
    ('team', 'events:shopping:add_tour'),
    ('team', 'events:shopping:edit_tour'),
    ('team', 'events:shopping:complete_tour'),
    ('team', 'status_board:operate'),
    ('team', 'recipes:view'),
    ('team', 'recipes:export'),
    ('team', 'recipes:create'),
    ('team', 'recipes:edit'),
    ('team', 'ingredients:view'),
    ('team', 'ingredients:create'),
    ('team', 'ingredients:edit'),
    ('team', 'inventories:view'),
    ('team', 'inventories:create'),
    ('team', 'inventories:edit'),
    ('team', 'inventories:edit_stock'),
    -- manager (Leitung)
    ('manager', 'events:view'),
    ('manager', 'events:view_calculation'),
    ('manager', 'events:export'),
    ('manager', 'events:create'),
    ('manager', 'events:edit'),
    ('manager', 'events:clone'),
    ('manager', 'events:manage_meals'),
    ('manager', 'events:shopping:add_tour'),
    ('manager', 'events:shopping:edit_tour'),
    ('manager', 'events:shopping:complete_tour'),
    ('manager', 'status_board:operate'),
    ('manager', 'events:delete'),
    ('manager', 'events:transfer_owner'),
    ('manager', 'recipes:view'),
    ('manager', 'recipes:export'),
    ('manager', 'recipes:create'),
    ('manager', 'recipes:edit'),
    ('manager', 'recipes:delete'),
    ('manager', 'recipes:transfer_owner'),
    ('manager', 'ingredients:view'),
    ('manager', 'ingredients:create'),
    ('manager', 'ingredients:edit'),
    ('manager', 'ingredients:delete'),
    ('manager', 'ingredients:transfer_owner'),
    ('manager', 'inventories:view'),
    ('manager', 'inventories:create'),
    ('manager', 'inventories:edit'),
    ('manager', 'inventories:edit_stock'),
    ('manager', 'inventories:delete'),
    ('manager', 'inventories:transfer_owner'),
    ('manager', 'group:manage_members'),
    ('manager', 'group:manage_settings'),
    ('manager', 'group:delete')
ON CONFLICT DO NOTHING;
