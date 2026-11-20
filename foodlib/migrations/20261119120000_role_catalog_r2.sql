-- R2 role catalog: replaces the viewer/editor/manager trio with
-- viewer, cook, shopper, stocker, serving, team, manager (defined in
-- foodlib_new/src/policy.rs and served via GET /api/roles).
--
-- `viewer` and `manager` slugs are unchanged, so existing rows stay valid.
-- `editor` mapped onto its successor `team` (the Rust catalog also keeps an
-- alias, but stored rows should be migrated cleanly).

UPDATE user_group_roles SET role_slug = 'team' WHERE role_slug = 'editor';

UPDATE event_user_roles SET role_slug = 'team' WHERE role_slug = 'editor';
