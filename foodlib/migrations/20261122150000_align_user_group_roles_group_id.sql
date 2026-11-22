-- Align `user_group_roles.group_id` with the typed id and the rest of the
-- schema: 20261118190000 created it as `bigint`, but `GroupId` is `i32` and
-- every other group reference (`events`, `recipes`, `ingredients`,
-- `inventories`, `audit_log`) is `integer`. On fresh databases (sqlx-test,
-- CI) the sqlx compile-time type check therefore rejected every `GroupId`
-- parameter against this table; the dev database had been manually altered
-- to `integer`, hiding the drift.
ALTER TABLE user_group_roles
    ALTER COLUMN group_id TYPE integer;
