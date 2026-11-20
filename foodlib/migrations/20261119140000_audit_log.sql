-- Audit log for security-relevant events (ISO 27001:2022 A.8.15 Logging,
-- A.5.15-A.5.18 access control, A.5.28 evidence collection).
--
-- Append-only by design: the API exposes no UPDATE/DELETE; retention purges
-- are explicit admin actions guarded by `admin:manage_settings`.
-- `actor_username` is a write-time snapshot so entries stay readable after
-- the referenced user is deleted (`actor_user_id` then becomes NULL).

CREATE TABLE IF NOT EXISTS audit_log
(
    id              BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    actor_user_id   BIGINT      NULL REFERENCES users (id) ON DELETE SET NULL,
    actor_username  TEXT        NOT NULL,
    action          TEXT        NOT NULL,
    target_user_id  BIGINT      NULL REFERENCES users (id) ON DELETE SET NULL,
    entity_type     TEXT        NULL,
    entity_id       INTEGER     NULL,
    group_id        INTEGER     NULL,
    event_id        INTEGER     NULL,
    details         TEXT        NOT NULL DEFAULT '{}'
);

CREATE INDEX IF NOT EXISTS audit_log_created_at_idx ON audit_log (created_at DESC);
CREATE INDEX IF NOT EXISTS audit_log_action_idx ON audit_log (action);
CREATE INDEX IF NOT EXISTS audit_log_actor_idx ON audit_log (actor_user_id);
CREATE INDEX IF NOT EXISTS audit_log_group_idx ON audit_log (group_id);
CREATE INDEX IF NOT EXISTS audit_log_event_idx ON audit_log (event_id);
