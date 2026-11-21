-- Personal API tokens (`Authorization: Bearer fc_...`) for CLI/desktop MCP
-- clients. The raw token is shown once at creation; only the SHA-256 hash is
-- stored. `allowed_risk_classes` optionally scopes what an agent acting with
-- this token may do (see `app::risk`); NULL = all classes the user's own
-- permissions would allow (unrestricted).
--
-- Risk classes are additive capabilities validated against the catalog in
-- the application layer; the DB stores them opaque.

CREATE TABLE IF NOT EXISTS user_api_tokens
(
    id                   BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    user_id              BIGINT      NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    name                 TEXT        NOT NULL,
    token_prefix         TEXT        NOT NULL,
    token_hash           TEXT        NOT NULL UNIQUE,
    allowed_risk_classes TEXT[]      NULL,
    created_at           TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_used_at         TIMESTAMPTZ NULL,
    expires_at           TIMESTAMPTZ NULL
);

-- An earlier draft migration (20261120190000 "user api token") already
-- created the table on some environments; bring those up to the current
-- schema idempotently.
ALTER TABLE user_api_tokens ADD COLUMN IF NOT EXISTS allowed_risk_classes TEXT[];

CREATE INDEX IF NOT EXISTS user_api_tokens_user_idx ON user_api_tokens (user_id);
