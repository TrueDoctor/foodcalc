-- Add owner_id to recipes table
ALTER TABLE recipes ADD COLUMN owner_id BIGINT;

-- Default all existing recipes to the first admin user (update this ID if needed)
UPDATE recipes SET owner_id = (SELECT id FROM users WHERE is_admin = true LIMIT 1);

-- After all data is migrated, make the column NOT NULL
ALTER TABLE recipes ALTER COLUMN owner_id SET NOT NULL;

-- Add foreign key constraint
ALTER TABLE recipes ADD CONSTRAINT recipe_owner_fk FOREIGN KEY (owner_id) REFERENCES users(id);

-- Add owner_id to events table
ALTER TABLE events ADD COLUMN owner_id BIGINT;

-- Default all existing events to the first admin user (update this ID if needed)
UPDATE events SET owner_id = (SELECT id FROM users WHERE is_admin = true LIMIT 1);

-- After all data is migrated, make the column NOT NULL
ALTER TABLE events ALTER COLUMN owner_id SET NOT NULL;

-- Add foreign key constraint
ALTER TABLE events ADD CONSTRAINT event_owner_fk FOREIGN KEY (owner_id) REFERENCES users(id);

-- Add owner_id to ingredients table
ALTER TABLE ingredients ADD COLUMN owner_id BIGINT;

-- Default all existing ingredients to the first admin user (update this ID if needed)
UPDATE ingredients SET owner_id = (SELECT id FROM users WHERE is_admin = true LIMIT 1);

-- After all data is migrated, make the column NOT NULL
ALTER TABLE ingredients ALTER COLUMN owner_id SET NOT NULL;

-- Add foreign key constraint
ALTER TABLE ingredients ADD CONSTRAINT event_owner_fk FOREIGN KEY (owner_id) REFERENCES users(id);


-- Add owner_id to inventories table
ALTER TABLE inventories ADD COLUMN owner_id BIGINT;

-- Default all existing inventories to the first admin user (update this ID if needed)
UPDATE inventories SET owner_id = (SELECT id FROM users WHERE is_admin = true LIMIT 1);

-- After all data is migrated, make the column NOT NULL
ALTER TABLE inventories ALTER COLUMN owner_id SET NOT NULL;

-- Add foreign key constraint
ALTER TABLE inventories ADD CONSTRAINT event_owner_fk FOREIGN KEY (owner_id) REFERENCES users(id);

