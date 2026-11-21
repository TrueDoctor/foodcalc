-- Group-member roles (Level-ABAC): viewer (nur sehen), editor (bearbeiten),
-- manager (verwaltet Mitglieder & Rollen). Bestandsmitglieder werden zu
-- Managern migriert, um das bisherige Verhalten (jedes Mitglied darf alles)
-- verhältniserhaltend abzubilden; neue Einladungen laufen auf 'viewer'.
ALTER TABLE user_groups ADD COLUMN role TEXT NOT NULL DEFAULT 'viewer';

UPDATE user_groups SET role = 'manager';

ALTER TABLE user_groups
    ADD CONSTRAINT user_groups_role_check CHECK (role IN ('viewer', 'editor', 'manager'));
