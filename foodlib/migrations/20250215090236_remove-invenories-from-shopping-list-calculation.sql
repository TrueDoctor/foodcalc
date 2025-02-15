CREATE OR REPLACE VIEW public.shopping_list AS
 SELECT full_weight.event_id,
     full_weight.event as event_name,
     full_weight.ingredient_id,
     full_weight.ingredient,
     sum(full_weight.weight) AS weight,
     round(sum(full_weight.weight) * ingredients.energy * 1000::numeric, 2) AS energy,
     sum(full_weight.weight) * COALESCE(price_per_ingredient_weight.price, '-1'::integer::numeric) AS price,
     full_weight.tour_id,
     metro_categories.category
     FROM   shopping_tour_ingredients as full_weight
     LEFT JOIN best_event_ingredient_sources price_per_ingredient_weight USING (ingredient_id, event_id)
     LEFT JOIN metro_categories USING (ingredient_source_id)
     LEFT JOIN ingredients USING (ingredient_id)
     GROUP BY full_weight.event_id, full_weight.tour_id, full_weight.event, full_weight.ingredient_id, full_weight.ingredient, ingredients.energy, price_per_ingredient_weight.price, metro_categories.category
