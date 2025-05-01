-- Add migration script here
CREATE OR REPLACE VIEW public.resolved_meta AS
 WITH RECURSIVE meta AS (
         SELECT meta_recipes.parent_id AS recipe_id,
            meta_recipes.child_id,
            meta_recipes.weight,
            (meta_recipes.parent_id::text || '.'::text) || meta_recipes.child_id::text AS acc,
            meta_recipes.parent_id
           FROM meta_recipes
        UNION
         SELECT meta_1.recipe_id,
            meta_recipes.child_id,
            meta_recipes.weight * (meta_1.weight / recipe_weight.weight) AS weight,
            (meta_1.acc || '.'::text) || meta_recipes.child_id AS acc,
            meta_1.child_id AS parent_id
           FROM meta meta_1
             JOIN meta_recipes ON meta_1.child_id = meta_recipes.parent_id
             JOIN recipe_weight ON recipe_weight.recipe_id = meta_1.child_id
        )
 SELECT DISTINCT meta.recipe_id,
    meta.child_id AS subrecipe_id,
    meta.weight,
    r.name AS recipe,
    mr.name AS subrecipe,
    meta.acc,
    meta.parent_id,
    r2.name AS parent
   FROM meta
     JOIN recipes r ON r.recipe_id = meta.recipe_id
     JOIN recipes r2 ON r2.recipe_id = meta.parent_id
     JOIN recipes mr ON mr.recipe_id = meta.child_id

