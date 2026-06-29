-- Cache Metro's customer-facing article number ("customerDisplayId") per
-- ingredient source, populated during metro sync. Lets the Metro CSV export
-- read article numbers from the DB instead of calling the Metro API live.
ALTER TABLE metro_categories
    ADD COLUMN article_number VARCHAR;
