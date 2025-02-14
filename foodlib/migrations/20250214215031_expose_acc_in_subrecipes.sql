CREATE OR REPLACE VIEW public.subrecipes AS
 SELECT bar.recipe,
    bar.recipe_id,
    bar.ingredient,
    bar.weight,
    bar.subrecipe,
    bar.subrecipe_id,
    bar.is_subrecipe,
    bar.acc as subrecipe_hierarchy
   FROM ( SELECT rr.recipe_id,
            rr.recipe,
            rr.ingredient,
            sum(rr.weight / recipe_weight.weight) AS weight,
            rr.subrecipe_id,
            recipes_1.name AS subrecipe,
            false AS is_subrecipe,
            acc
           FROM resolved_recipes rr
             JOIN recipe_weight USING (recipe_id)
             JOIN recipes recipes_1 ON rr.subrecipe_id = recipes_1.recipe_id
          GROUP BY rr.recipe_id, rr.subrecipe_id, rr.recipe, rr.ingredient_id, rr.ingredient, recipes_1.name, rr.acc
        UNION ALL
         SELECT resolved_meta.recipe_id,
            resolved_meta.recipe,
            resolved_meta.subrecipe AS ingredient,
            sum(resolved_meta.weight / recipe_weight.weight) AS weight,
            resolved_meta.parent_id,
            resolved_meta.parent,
            true AS is_subrecipe,
            acc
           FROM resolved_meta
             JOIN recipe_weight ON recipe_weight.recipe_id = resolved_meta.recipe_id
          GROUP BY resolved_meta.recipe_id, resolved_meta.recipe, resolved_meta.subrecipe, resolved_meta.subrecipe_id, resolved_meta.parent_id, resolved_meta.parent, acc) bar
     JOIN recipes USING (recipe_id)
  ORDER BY bar.recipe, bar.subrecipe_id, bar.is_subrecipe DESC
