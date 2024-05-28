-- Update event_meals Table
alter table public.event_meals
    add meal_id int generated always as IDENTITY;

alter table public.event_meals
    drop constraint event_meals_pk;

alter table public.event_meals
    add constraint event_meals_pk
        primary key (meal_id);

-- Update Depending Views
CREATE OR REPLACE VIEW event_ingredients AS
SELECT events.event_id,
       events.event_name                                                                                           AS event,
       event_meals.recipe_id,
       resolved_recipes.recipe,
       event_meals.place_id,
       places.name                                                                                                 AS place,
       event_meals.start_time,
       event_meals.end_time,
       resolved_recipes.ingredient_id,
       resolved_recipes.ingredient,
       resolved_recipes.weight * recipe_multipliers.recipe_multiplier                                              AS weight,
       recipe_multipliers.recipe_multiplier * resolved_recipes.weight * ingredients.energy *
       1000::numeric                                                                                               AS energy,
       min(ingredient_sources.price * (recipe_multipliers.recipe_multiplier * resolved_recipes.weight /
                                       (ingredient_weight.weight *
                                        COALESCE(ingredient_sources.package_size, 1::numeric)))::double precision) AS price,
       event_meals.servings,
       event_meals.meal_id
FROM events
         LEFT JOIN event_meals USING (event_id)
         LEFT JOIN places USING (place_id)
         LEFT JOIN resolved_recipe_ingredients resolved_recipes USING (recipe_id)
         LEFT JOIN ingredients USING (ingredient_id)
         LEFT JOIN ingredient_sources USING (ingredient_id)
         LEFT JOIN ingredient_weight USING (ingredient_id, unit_id)
         LEFT JOIN (SELECT event_meals_1.recipe_id,
                           event_meals_1.event_id,
                           event_meals_1.place_id,
                           event_meals_1.start_time,
                           event_meals_1.energy_per_serving * event_meals_1.servings::numeric /
                           recipe_stats.energy AS recipe_multiplier
                    FROM event_meals event_meals_1
                             JOIN recipe_stats USING (recipe_id)) recipe_multipliers
                   USING (event_id, recipe_id, place_id, start_time)
GROUP BY events.event_id, events.event_name, event_meals.recipe_id, resolved_recipes.recipe, event_meals.place_id,
         places.name, event_meals.start_time, event_meals.end_time, resolved_recipes.ingredient_id,
         resolved_recipes.ingredient, resolved_recipes.weight, ingredients.energy, recipe_multipliers.recipe_multiplier,
         event_meals.servings, event_meals.meal_id;

CREATE OR REPLACE VIEW event_recipes AS
SELECT event_id,
       event,
       recipe_id,
       recipe,
       round(sum(weight), 2) AS weights,
       round(sum(energy), 2) AS energy,
       sum(price)            AS price,
       meal_id
FROM event_ingredients
GROUP BY event_id, event, recipe_id, recipe, meal_id
