-- Add migration script here
UPDATE food_properties
SET name = 'null'
WHERE name IS NULL;

ALTER TABLE food_properties
ALTER COLUMN name SET NOT NULL;
