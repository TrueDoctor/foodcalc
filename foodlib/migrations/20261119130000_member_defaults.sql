-- Default roles for new group members (role-based defaults):
--   groups.default_roles  NULL = inherit the global default, '[]' = no permissions
--   app_settings          generic key/value store; seeded with the global
--                         member default ("viewer" = least privilege).
-- Personal groups are exempt: their owner always holds `manager`.

ALTER TABLE groups ADD COLUMN IF NOT EXISTS default_roles TEXT[];

CREATE TABLE IF NOT EXISTS app_settings
(
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

INSERT INTO app_settings (key, value)
VALUES ('groups.default_member_roles', '["viewer"]')
ON CONFLICT (key) DO NOTHING;
