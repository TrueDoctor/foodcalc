-- Add migration script here
CREATE OR REPLACE VIEW public.shopping_list AS
 SELECT full_weight.event_id,
    full_weight.event_name,
    full_weight.ingredient_id,
    full_weight.ingredient,
    sum(full_weight.weight) AS weight,
    round(sum(full_weight.weight) * ingredients.energy * 1000::numeric, 2) AS energy,
    sum(full_weight.weight)::double precision * COALESCE(price_per_ingredient_weight.price, '-1'::money) AS price,
    full_weight.tour_id,
    category
   FROM ( SELECT prefetch.event_id,
            prefetch.tour_id,
            prefetch.event_name,
            prefetch.ingredient_id,
            prefetch.ingredient,
            prefetch.store_id,
            prefetch.buy_by,
            round(GREATEST(prefetch.weight::double precision - COALESCE(event_storage.amount::double precision, 0::double precision), 0::double precision)::numeric, 2) AS weight
           FROM ( SELECT shopping_tour_ingredients.event_id,
                    shopping_tour_ingredients.tour_id,
                    shopping_tour_ingredients.event AS event_name,
                    shopping_tour_ingredients.ingredient_id,
                    shopping_tour_ingredients.ingredient,
                    shopping_tour_ingredients.store_id,
                    shopping_tour_ingredients.buy_by,
                    sum(shopping_tour_ingredients.weight) AS weight
                   FROM shopping_tour_ingredients
                  GROUP BY shopping_tour_ingredients.event_id, shopping_tour_ingredients.tour_id, shopping_tour_ingredients.event, shopping_tour_ingredients.ingredient_id, shopping_tour_ingredients.ingredient, shopping_tour_ingredients.buy_by, shopping_tour_ingredients.store_id) prefetch
             LEFT JOIN event_storage USING (event_id, ingredient_id)) full_weight
     LEFT JOIN best_event_ingredient_sources price_per_ingredient_weight USING (ingredient_id, event_id)
     LEFT JOIN metro_categories USING (ingredient_source_id)
     LEFT JOIN ingredients USING (ingredient_id)
  GROUP BY full_weight.event_id, full_weight.tour_id, full_weight.event_name, full_weight.ingredient_id, full_weight.ingredient, ingredients.energy, price_per_ingredient_weight.price, category;
