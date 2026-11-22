-- Restore the source-unit → kg conversion in the price column of
-- event_ingredients_before_prep_time_resolve.
--
-- 20261102230000 correctly removed the ingredient_weight factor from the
-- WEIGHT column (resolved_recipes.weight is already in kg, so multiplying
-- again was a double conversion). But it removed the same factor from the
-- PRICE column, where `ingredient_weight.weight * package_size` is not a
-- double conversion — it is the package weight in kg:
--     price_per_kg = source.price / (kg-per-unit × package_size)
-- Without it, every source whose package is not sold in kg is mispriced:
-- pieces by 1/n (e.g. a 10-piece egg carton charged as if 10 kg cost 4.99 €),
-- g/L packages by ×1000. kg sources are unaffected (factor 1), which is why
-- the regression slipped through the kg-only fixtures.
--
-- Weight column stays as 20261102230000 left it (kg, no double conversion);
-- the meal-scoped recipe_multiplier shape is kept as well.
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
    ingredient_sources.price * (recipe_multipliers.recipe_multiplier * resolved_recipes.weight
        / NULLIF(ingredient_weight.weight * COALESCE(ingredient_sources.package_size, 1::numeric), 0::numeric)) AS price,
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
     LEFT JOIN ( SELECT event_meals_1.meal_id AS meal_id,
            event_meals_1.recipe_id AS recipe_id,
            event_meals_1.energy_per_serving * event_meals_1.servings::numeric / NULLIF(recipe_stats.energy, 0::numeric) AS recipe_multiplier
           FROM event_meals event_meals_1
             JOIN recipe_stats USING (recipe_id)) recipe_multipliers ON recipe_multipliers.meal_id = event_meals.meal_id;
