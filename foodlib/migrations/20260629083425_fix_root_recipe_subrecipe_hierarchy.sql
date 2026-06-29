-- Seed subrecipe_hierarchy with the root recipe id for a recipe's own
-- direct ingredients.
--
-- resolved_recipes builds `acc` (exposed downstream as subrecipe_hierarchy)
-- from a UNION: subrecipe rows carry an accumulator that begins with the
-- root recipe id (e.g. '402.4'), but the base case for a recipe's *own*
-- direct ingredients set `acc = NULL`. That left those ingredients with an
-- empty hierarchy.
--
-- Food-prep matching keys off this value with the regex
--   subrecipe_hierarchy ~ '^(.*\.)?<recipe_id>(\..*)?$'
-- and a later LEFT JOIN ... USING (subrecipe_hierarchy). A NULL/empty
-- hierarchy matches neither, so prep_date never propagated to the prepped
-- recipe's own direct ingredients: they were scheduled by meal serving time
-- and could land on a later shopping tour than the prep date warrants.
--
-- Seeding the base case with the root recipe id makes those ingredients
-- behave like single-element subrecipe chains, so the existing regex and the
-- USING join both match.
CREATE OR REPLACE VIEW resolved_recipes AS
 SELECT recipes.recipe_id,
    recipes.name AS recipe,
    recipe_ingredients.ingredient_id,
    ingredients.name AS ingredient,
    ((resolved_meta.weight / recipe_weight.weight) * (ingredient_weight.weight * recipe_ingredients.amount)) AS weight,
    resolved_meta.subrecipe AS subrecipe_id,
    subrecipes.name AS subrecipe,
    resolved_meta.acc
   FROM ((((((recipes
     LEFT JOIN ( SELECT resolved_meta_1.recipe_id,
            resolved_meta_1.subrecipe_id AS subrecipe,
            resolved_meta_1.weight,
            resolved_meta_1.acc
           FROM resolved_meta resolved_meta_1
        UNION
         SELECT recipes_1.recipe_id,
            recipes_1.recipe_id,
            recipe_weight_1.weight,
            recipes_1.recipe_id::text AS acc
           FROM (recipes recipes_1
             JOIN recipe_weight recipe_weight_1 USING (recipe_id))) resolved_meta USING (recipe_id))
     JOIN recipe_ingredients ON ((recipe_ingredients.recipe_id = resolved_meta.subrecipe)))
     JOIN recipe_weight ON ((resolved_meta.subrecipe = recipe_weight.recipe_id)))
     LEFT JOIN ingredients USING (ingredient_id))
     LEFT JOIN ingredient_weight USING (ingredient_id, unit_id))
     LEFT JOIN recipes subrecipes ON ((resolved_meta.subrecipe = subrecipes.recipe_id)))
  ORDER BY recipes.recipe_id;
