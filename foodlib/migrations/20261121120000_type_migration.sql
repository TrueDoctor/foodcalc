ALTER TABLE audit_log
    ALTER COLUMN entity_id TYPE TEXT USING entity_id::TEXT;