-- Fix pricing for negative ingredient quantities (e.g. removing an ingredient
-- from an event via a negative amount). The old formula multiplied the source
-- price by the (signed) ratio, so MIN() over sources picked the most expensive
-- source for negative quantities, and the magnitude grew with the price.
-- New semantics: pick the cheapest unit price first, then scale linearly.
-- For positive quantities the value is algebraically identical to before.
CREATE OR REPLACE VIEW event_ingredients_before_prep_time_resolve AS
SELECT events.event_id,
       events.event_name AS event,
       event_meals.recipe_id,
       recipes.name AS recipe,
       event_meals.place_id,
       places.name AS place,
       event_meals.start_time,
       event_meals.end_time,
       resolved_recipes.ingredient_id,
       resolved_recipes.ingredient,
       resolved_recipes.weight * recipe_multipliers.recipe_multiplier * ingredient_weight.weight AS weight,
       recipe_multipliers.recipe_multiplier * resolved_recipes.weight * ingredients.energy * 1000::numeric AS energy,
       (ingredient_sources.price /
         NULLIF(ingredient_weight.weight * COALESCE(ingredient_sources.package_size, 1::numeric), 0::numeric))
         * (recipe_multipliers.recipe_multiplier * resolved_recipes.weight) AS price,
       event_meals.servings,
       event_meals.meal_id,
       resolved_recipes.acc AS subrecipe_hierarchy,
       event_meals.start_time AS buy_by,
       ingredient_sources.store_id
FROM events
  LEFT JOIN event_meals USING (event_id)
  LEFT JOIN places USING (place_id)
  LEFT JOIN resolved_recipes USING (recipe_id)
  LEFT JOIN recipes USING (recipe_id)
  LEFT JOIN ingredients USING (ingredient_id)
  LEFT JOIN best_event_ingredient_sources USING (event_id, ingredient_id)
  LEFT JOIN ingredient_sources USING (ingredient_source_id, ingredient_id)
  LEFT JOIN ingredient_weight USING (ingredient_id, unit_id)
  LEFT JOIN (SELECT event_meals_1.recipe_id,
                    event_meals_1.event_id,
                    event_meals_1.place_id,
                    event_meals_1.start_time,
                    event_meals_1.energy_per_serving * event_meals_1.servings::numeric / NULLIF(recipe_stats.energy, 0::numeric) AS recipe_multiplier
             FROM event_meals event_meals_1
               JOIN recipe_stats USING (recipe_id)) recipe_multipliers
    USING (event_id, recipe_id, place_id, start_time);
