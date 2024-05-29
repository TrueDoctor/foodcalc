CREATE OR REPLACE VIEW public.event_ingredients_with_source_options AS
 SELECT events.event_id,
    resolved_recipes.ingredient_id,
    source_price_per_kilo.price_per_kilo AS price,
    source_price_per_kilo.ingredient_source_id
   FROM events
     LEFT JOIN event_meals USING (event_id)
     LEFT JOIN resolved_recipes USING (recipe_id)
     LEFT JOIN event_ingredient_sources USING (event_id, ingredient_id)
     LEFT JOIN source_price_per_kilo USING (ingredient_source_id)
