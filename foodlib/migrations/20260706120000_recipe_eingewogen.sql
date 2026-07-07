-- Marks a recipe whose ingredient amounts have all been weighed and verified
-- accurate to the gram ("eingewogen"). Defaults to false; recipes must be
-- explicitly confirmed. Surfaced as a warning in the recipe edit tab while
-- false and as a badge on the printed recipe sheet once true.
ALTER TABLE recipes ADD COLUMN weighed_in BOOLEAN NOT NULL DEFAULT false;
