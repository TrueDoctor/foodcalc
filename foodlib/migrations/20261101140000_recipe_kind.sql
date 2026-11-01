-- Categorize recipes as 'meal' (Gericht) or 'component' (Komponente) so
-- both can be filtered in the UI and freely combined (a component as a
-- side for a buffet, a meal as a side of another meal, ...).
-- Existing recipes are meals, which matches the previous implicit behavior.
DO $$
BEGIN
    CREATE TYPE recipe_kind AS ENUM ('meal', 'component');
EXCEPTION
    WHEN duplicate_object THEN NULL; -- already created
END $$;

-- Transitional: nullable TEXT column, upgraded to the enum below.
ALTER TABLE recipes ADD COLUMN IF NOT EXISTS kind TEXT;

-- Migrate any legacy values, then convert the column type.
UPDATE recipes SET kind = 'meal' WHERE kind IS NULL OR kind = '';
ALTER TABLE recipes ALTER COLUMN kind DROP DEFAULT;
ALTER TABLE recipes
    ALTER COLUMN kind TYPE recipe_kind
    USING COALESCE(NULLIF(kind, ''), 'meal')::recipe_kind;
ALTER TABLE recipes ALTER COLUMN kind SET DEFAULT 'meal';
ALTER TABLE recipes ALTER COLUMN kind SET NOT NULL;
