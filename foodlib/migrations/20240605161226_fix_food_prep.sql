-- Add migration script here
CREATE OR REPLACE VIEW public.prep_ingredients AS
 SELECT event_id,
    prep_id,
    recipe_id,
    prep_date,
    ingredient_id,
    ingredient,
    sum(weight) AS total_weight,
    sum(price) AS total_price
   FROM prep_ingredients_with_duplicates
   JOIN ( SELECT event_id, ingredient_id, subrecipe_hierarchy,meal_id, min(prep_date) as min_date
        FROM prep_ingredients_with_duplicates GROUP BY event_id, ingredient_id, subrecipe_hierarchy, meal_id) USING (event_id, ingredient_id, subrecipe_hierarchy, meal_id)
  WHERE prep_date = min_date
  GROUP BY event_id, prep_id, recipe_id, prep_date, ingredient_id, ingredient


CREATE OR REPLACE VIEW public.event_ingredients AS
 SELECT event_ingredients_before_prep_time_resolve.event_id,
    event_ingredients_before_prep_time_resolve.event,
    event_ingredients_before_prep_time_resolve.recipe_id,
    event_ingredients_before_prep_time_resolve.recipe,
    event_ingredients_before_prep_time_resolve.place_id,
    event_ingredients_before_prep_time_resolve.place,
    event_ingredients_before_prep_time_resolve.start_time,
    event_ingredients_before_prep_time_resolve.end_time,
    event_ingredients_before_prep_time_resolve.ingredient_id,
    event_ingredients_before_prep_time_resolve.ingredient,
    event_ingredients_before_prep_time_resolve.weight,
    event_ingredients_before_prep_time_resolve.energy,
    event_ingredients_before_prep_time_resolve.price,
    event_ingredients_before_prep_time_resolve.servings,
    event_ingredients_before_prep_time_resolve.meal_id,
    event_ingredients_before_prep_time_resolve.subrecipe_hierarchy,
    COALESCE(prep_ingredients.prep_date, event_ingredients_before_prep_time_resolve.buy_by) AS buy_by,
    event_ingredients_before_prep_time_resolve.store_id
   FROM event_ingredients_before_prep_time_resolve
     LEFT JOIN prep_ingredients USING (event_id, ingredient_id)

CREATE OR REPLACE VIEW public.prep_ingredients_with_duplicates AS
 SELECT resolved_prep_starts.event_id,
    resolved_prep_starts.prep_id,
    resolved_prep_starts.recipe_id,
    resolved_prep_starts.prep_date,
    event_ingredients_before_prep_time_resolve.ingredient_id,
    event_ingredients_before_prep_time_resolve.ingredient,
    event_ingredients_before_prep_time_resolve.weight,
    event_ingredients_before_prep_time_resolve.price,
    event_ingredients_before_prep_time_resolve.meal_id,
    event_ingredients_before_prep_time_resolve.subrecipe_hierarchy
   FROM resolved_prep_starts
     LEFT JOIN event_ingredients_before_prep_time_resolve USING (event_id)
  WHERE event_ingredients_before_prep_time_resolve.subrecipe_hierarchy ~ concat('^(.*\.)?', resolved_prep_starts.recipe_id, '(\..*)?$') AND event_ingredients_before_prep_time_resolve.start_time >= resolved_prep_starts.use_from AND event_ingredients_before_prep_time_resolve.start_time <= resolved_prep_starts.use_until
