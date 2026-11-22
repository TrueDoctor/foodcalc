-- RBAC has no implicit superuser bypass (`AuthCtx::can_global` consults only
-- the explicit grants), so every admin account must carry the full admin:*
-- set. The original RBAC migration only seeded manage_users/master_data.
INSERT INTO user_global_permissions (user_id, permission)
SELECT id, perm FROM users, (VALUES
    ('admin:manage_users'),
    ('admin:manage_master_data'),
    ('admin:manage_settings'),
    ('admin:view_audit_log')
) AS admin_perms(perm)
WHERE is_admin = TRUE
ON CONFLICT DO NOTHING;
