-- event_ingredients_before_prep_time_resolve: drop the double unit
-- conversion. resolved_recipes.weight is ALREADY the ingredient amount
-- converted to kg (amount × kg-per-unit via ingredient_weight). The previous
-- definition multiplied (weight column) and divided (price column) by
-- ingredient_weight.weight a second time — invisible for recipe ingredient
-- rows in kg (factor 1), but wrong by the unit factor for everything else
-- (1000 g rows came out as 0.001× the real demand, L as 0.001×, pieces as
-- 1/n×). Price matched the same faulty shape.
-- The ingredient_weight join is gone with it (no remaining reference).
CREATE OR REPLACE VIEW public.event_ingredients_before_prep_time_resolve AS
 SELECT events.event_id,
    events.event_name AS event,
    event_meals.recipe_id,
    recipes.name as recipe,
    event_meals.place_id,
    places.name AS place,
    event_meals.start_time,
    event_meals.end_time,
    resolved_recipes.ingredient_id,
    resolved_recipes.ingredient,
    resolved_recipes.weight * recipe_multipliers.recipe_multiplier AS weight,
    recipe_multipliers.recipe_multiplier * resolved_recipes.weight * ingredients.energy * 1000::numeric AS energy,
    ingredient_sources.price * (recipe_multipliers.recipe_multiplier * resolved_recipes.weight / NULLIF(COALESCE(ingredient_sources.package_size, 1::numeric), 0::numeric)) AS price,
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
     LEFT JOIN ( SELECT event_meals_1.meal_id AS meal_id,
            event_meals_1.recipe_id AS recipe_id,
            event_meals_1.energy_per_serving * event_meals_1.servings::numeric / NULLIF(recipe_stats.energy, 0::numeric) AS recipe_multiplier
           FROM event_meals event_meals_1
             JOIN recipe_stats USING (recipe_id)) recipe_multipliers ON recipe_multipliers.meal_id = event_meals.meal_id;
