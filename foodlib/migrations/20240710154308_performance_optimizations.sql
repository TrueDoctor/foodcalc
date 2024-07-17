CREATE OR REPLACE VIEW public.best_event_ingredient_sources AS
WITH ranked_sources AS (
  SELECT 
    event_id,
    ingredient_id,
    price,
    ingredient_source_id,
    ROW_NUMBER() OVER (PARTITION BY event_id, ingredient_id ORDER BY price) as rn
  FROM event_ingredients_with_source_options
)
SELECT 
  event_id,
  ingredient_id,
  price,
  ingredient_source_id
FROM ranked_sources
WHERE rn = 1;

CREATE OR REPLACE VIEW public.prep_ingredients AS
SELECT event_id,
    prep_id,
    recipe_id,
    prep_date,
    ingredient_id,
    ingredient,
    subrecipe_hierarchy,
    meal_id
   FROM ( SELECT rps.event_id,
            rps.prep_id,
            rps.recipe_id,
            rps.prep_date,
            eibptr.ingredient_id,
            eibptr.ingredient,
            eibptr.subrecipe_hierarchy,
            eibptr.meal_id,
            row_number() OVER (PARTITION BY rps.event_id, eibptr.ingredient_id, eibptr.subrecipe_hierarchy, eibptr.meal_id ORDER BY rps.prep_date) AS rn
           FROM resolved_prep_starts rps
             LEFT JOIN event_ingredients_before_prep_time_resolve eibptr ON rps.event_id = eibptr.event_id
          WHERE eibptr.subrecipe_hierarchy ~ concat('^(.*\.)?', rps.recipe_id, '(\..*)?$') AND eibptr.start_time >= rps.use_from AND eibptr.start_time <= rps.use_until) subquery
  WHERE rn = 1
  ORDER BY event_id, prep_date, ingredient_id;
