CREATE OR REPLACE VIEW public.event_ingredients_before_prep_time_resolve AS
 SELECT events.event_id,
    events.event_name AS event,
    event_meals.recipe_id,
    resolved_recipes.recipe,
    event_meals.place_id,
    places.name AS place,
    event_meals.start_time,
    event_meals.end_time,
    resolved_recipes.ingredient_id,
    resolved_recipes.ingredient,
    resolved_recipes.weight * recipe_multipliers.recipe_multiplier * ingredient_weight.weight AS weight,
    recipe_multipliers.recipe_multiplier * resolved_recipes.weight * ingredients.energy * 1000::numeric AS energy,
    ingredient_sources.price * (recipe_multipliers.recipe_multiplier * resolved_recipes.weight / (ingredient_weight.weight * COALESCE(ingredient_sources.package_size, 1::numeric)))::double precision AS price,
    event_meals.servings,
    event_meals.meal_id,
    resolved_recipes.acc AS subrecipe_hierarchy,
    event_meals.start_time AS buy_by,
    ingredient_sources.store_id
   FROM events
     LEFT JOIN event_meals USING (event_id)
     LEFT JOIN places USING (place_id)
     LEFT JOIN resolved_recipes USING (recipe_id)
     LEFT JOIN ingredients USING (ingredient_id)
     LEFT JOIN best_event_ingredient_sources USING (event_id, ingredient_id)
     LEFT JOIN ingredient_sources USING (ingredient_source_id, ingredient_id)
     LEFT JOIN ingredient_weight USING (ingredient_id, unit_id)
     LEFT JOIN ( SELECT event_meals_1.recipe_id,
            event_meals_1.event_id,
            event_meals_1.place_id,
            event_meals_1.start_time,
            event_meals_1.energy_per_serving * event_meals_1.servings::numeric / recipe_stats.energy AS recipe_multiplier
           FROM event_meals event_meals_1
             JOIN recipe_stats USING (recipe_id)) recipe_multipliers USING (event_id, recipe_id, place_id, start_time);

CREATE OR REPLACE VIEW public.shopping_list AS
 SELECT full_weight.event_id,
    full_weight.event_name,
    full_weight.ingredient_id,
    full_weight.ingredient,
    sum(full_weight.weight) AS weight,
    round(sum(full_weight.weight) * ingredients.energy * 1000::numeric, 2) AS energy,
    sum(full_weight.weight)::double precision * COALESCE(price_per_ingredient_weight.price, '-1.00'::float8::numeric::money) AS price,
    full_weight.tour_id
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
                  WHERE shopping_tour_ingredients.tour_id IS NOT NULL
                  GROUP BY shopping_tour_ingredients.event_id, shopping_tour_ingredients.tour_id, shopping_tour_ingredients.event, shopping_tour_ingredients.ingredient_id, shopping_tour_ingredients.ingredient, shopping_tour_ingredients.buy_by, shopping_tour_ingredients.store_id) prefetch
             LEFT JOIN event_storage USING (event_id, ingredient_id)) full_weight
     LEFT JOIN best_event_ingredient_sources price_per_ingredient_weight USING (ingredient_id, event_id)
     LEFT JOIN ingredients USING (ingredient_id)
  GROUP BY full_weight.event_id, full_weight.tour_id, full_weight.event_name, full_weight.ingredient_id, full_weight.ingredient, ingredients.energy, price_per_ingredient_weight.price;
