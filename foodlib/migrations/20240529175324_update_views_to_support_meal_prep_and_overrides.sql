CREATE OR REPLACE VIEW public.event_ingredient_sources AS 
    SELECT * FROM (
        SELECT overrides.event_id AS event_id,
            overrides.ingredient_id AS ingredient_id,
            COALESCE(overrides.overriding_source, ingredient_sources.ingredient_source_id) AS ingredient_source_id
        FROM (
            SELECT e_i_cross.event_id,
                e_i_cross.ingredient_id,
                subquery.ingredient_source_id AS overriding_source
            FROM (
                SELECT events.event_id, ingredients.ingredient_id 
                FROM events
                CROSS JOIN ingredients) e_i_cross
                LEFT JOIN (
                    SELECT event_source_overrides.event_id,
                        ingredients.ingredient_id,
                        event_source_overrides.ingredient_source_id
                    FROM ingredients
                    LEFT JOIN ingredient_sources USING (ingredient_id)
                    LEFT JOIN event_source_overrides USING (ingredient_source_id)
                    LEFT JOIN ingredient_sources ingredient_sources2 
                        ON ingredient_sources2.ingredient_source_id = event_source_overrides.ingredient_source_id) subquery
                    ON subquery.ingredient_id = e_i_cross.ingredient_id AND subquery.event_id = e_i_cross.event_id) overrides 
        LEFT JOIN ingredient_sources USING (ingredient_id)
    )
    GROUP BY event_id, ingredient_id, ingredient_source_id;

CREATE OR REPLACE VIEW public.resolved_meta AS
 WITH RECURSIVE meta AS (
         SELECT meta_recipes.parent_id AS recipe_id,
            meta_recipes.child_id,
            meta_recipes.weight,
            (meta_recipes.parent_id::text || '.'::text) || meta_recipes.child_id::text  AS acc,
            meta_recipes.parent_id
           FROM meta_recipes
        UNION
         SELECT meta_1.recipe_id,
            meta_recipes.child_id,
            meta_recipes.weight * (meta_1.weight / recipe_weight.weight) AS weight,
            (meta_1.acc || '.'::text) || meta_recipes.child_id AS acc,
            meta_1.child_id AS parent_id
           FROM meta meta_1
             JOIN meta_recipes ON meta_1.child_id = meta_recipes.parent_id
             JOIN recipe_weight ON recipe_weight.recipe_id = meta_1.child_id
        )
 SELECT DISTINCT meta.recipe_id,
    meta.child_id AS subrecipe_id,
    meta.weight,
    r.name AS recipe,
    mr.name AS subrecipe,
    meta.acc,
    meta.parent_id,
    r2.name AS parent
   FROM meta
     JOIN recipe_ingredients ON meta.child_id = recipe_ingredients.recipe_id
     JOIN recipes r ON r.recipe_id = meta.recipe_id
     JOIN recipes r2 ON r2.recipe_id = meta.parent_id
     JOIN recipes mr ON mr.recipe_id = meta.child_id;

CREATE OR REPLACE VIEW public.source_price_per_kilo AS
    SELECT ingredient_sources.ingredient_source_id,
        ingredient_sources.price / (ingredient_sources.package_size * ingredient_weight.weight)::double precision AS price_per_kilo
    FROM ingredient_sources
    LEFT JOIN ingredient_weight USING (ingredient_id, unit_id);

CREATE OR REPLACE VIEW public.event_ingredients_with_source_options AS
    SELECT events.event_id,
        resolved_recipes.ingredient_id,
        source_price_per_kilo.price_per_kilo AS price,
        source_price_per_kilo.ingredient_source_id
    FROM events
    LEFT JOIN event_meals USING (event_id)
    LEFT JOIN resolved_recipes USING (recipe_id)
    LEFT JOIN ingredient_sources USING (ingredient_id)
    LEFT JOIN source_price_per_kilo USING (ingredient_source_id);

CREATE OR REPLACE VIEW public.best_event_ingredient_sources AS
    SELECT * FROM event_ingredients_with_source_options
    WHERE price = ( 
        SELECT MIN(price) FROM event_ingredients_with_source_options o2
        WHERE o2.event_id = event_ingredients_with_source_options.event_id
            AND o2.ingredient_id = event_ingredients_with_source_options.ingredient_id)
    GROUP BY event_id, ingredient_id, price, ingredient_source_id;
        

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
        resolved_recipes.weight * recipe_multipliers.recipe_multiplier AS weight,
        recipe_multipliers.recipe_multiplier * resolved_recipes.weight * ingredients.energy * 1000::numeric AS energy,
        ingredient_sources.price * (recipe_multipliers.recipe_multiplier * resolved_recipes.weight / (ingredient_weight.weight * COALESCE(ingredient_sources.package_size, 1::numeric)))::double precision AS price,
        event_meals.servings,
        event_meals.meal_id,
        resolved_recipes.acc AS subrecipe_hierarchy,
        event_meals.start_time AS buy_by,
        ingredient_sources.store_id AS store_id
    FROM events
        LEFT JOIN event_meals USING (event_id)
        LEFT JOIN places USING (place_id)
        LEFT JOIN resolved_recipes USING (recipe_id)
        LEFT JOIN ingredients USING (ingredient_id)
        LEFT JOIN best_event_ingredient_sources USING (event_id, ingredient_id)
        LEFT JOIN ingredient_sources USING (ingredient_source_id, ingredient_id)
        LEFT JOIN ingredient_weight USING (ingredient_id, unit_id)
        LEFT JOIN ( 
            SELECT event_meals_1.recipe_id,
                event_meals_1.event_id,
                event_meals_1.place_id,
                event_meals_1.start_time,
                event_meals_1.energy_per_serving * event_meals_1.servings::numeric / recipe_stats.energy AS recipe_multiplier
                FROM event_meals event_meals_1
                    JOIN recipe_stats USING (recipe_id)) recipe_multipliers 
            USING (event_id, recipe_id, place_id, start_time);

CREATE OR REPLACE VIEW public.resolved_prep_starts AS
    SELECT food_prep.prep_id,
        food_prep.event_id,
        food_prep.recipe_id,
        food_prep.prep_date,
        COALESCE(food_prep.use_from, food_prep.prep_date) AS use_from,
        food_prep.use_until
    FROM food_prep;

CREATE OR REPLACE VIEW public.prep_ingredients AS
    SELECT resolved_prep_starts.event_id,
        resolved_prep_starts.prep_id,
        resolved_prep_starts.recipe_id,
        resolved_prep_starts.prep_date,
        event_ingredients_before_prep_time_resolve.ingredient_id,
        event_ingredients_before_prep_time_resolve.ingredient,
        SUM(event_ingredients_before_prep_time_resolve.weight) AS total_weight, 
        SUM(event_ingredients_before_prep_time_resolve.price) AS total_price
    FROM resolved_prep_starts
    LEFT JOIN event_ingredients_before_prep_time_resolve 
        ON event_ingredients_before_prep_time_resolve.event_id = resolved_prep_starts.event_id 
            AND event_ingredients_before_prep_time_resolve.subrecipe_hierarchy LIKE CONCAT('%', resolved_prep_starts.recipe_id, '%')
    WHERE event_ingredients_before_prep_time_resolve.start_time >= resolved_prep_starts.use_from
        AND event_ingredients_before_prep_time_resolve.start_time <= resolved_prep_starts.use_until
    GROUP BY 
        resolved_prep_starts.event_id,
        resolved_prep_starts.prep_id,
        resolved_prep_starts.recipe_id,
        resolved_prep_starts.prep_date,
        event_ingredients_before_prep_time_resolve.ingredient_id,
        event_ingredients_before_prep_time_resolve.ingredient;

CREATE OR REPLACE VIEW public.event_ingredients AS
    SELECT 
        event_ingredients_before_prep_time_resolve.event_id,
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
    LEFT JOIN prep_ingredients USING (event_id, recipe_id, ingredient_id);

CREATE OR REPLACE VIEW public.all_later_tours AS
    SELECT shopping_tours.event_id,
        shopping_tours.tour_id,
        shopping_tours.tour_date,
        shopping_tours.store_id,
        shopping_tours_1.tour_date AS next_tour_date
    FROM shopping_tours
    LEFT JOIN shopping_tours shopping_tours_1 
        ON shopping_tours_1.tour_date > shopping_tours.tour_date 
            AND shopping_tours_1.event_id = shopping_tours.event_id 
            AND shopping_tours_1.store_id = shopping_tours.store_id;

CREATE OR REPLACE VIEW public.shopping_tour_ranges AS
    SELECT event_id, tour_id, tour_date, store_id, next_tour_date FROM all_later_tours
    WHERE all_later_tours.next_tour_date = (
        SELECT MIN(next_tour_date) FROM all_later_tours all_later_tours_1
        WHERE all_later_tours.tour_id = all_later_tours_1.tour_id) 
        OR all_later_tours.next_tour_date IS NULL;
    
CREATE OR REPLACE VIEW public.shopping_tour_ingredients AS
    SELECT 
        event_ingredients.event_id,
        event_ingredients.event,
        event_ingredients.recipe_id,
        event_ingredients.recipe,
        event_ingredients.place_id,
        event_ingredients.place,
        event_ingredients.start_time,
        event_ingredients.end_time,
        event_ingredients.ingredient_id,
        event_ingredients.ingredient,
        event_ingredients.weight,
        event_ingredients.energy,
        event_ingredients.price,
        event_ingredients.servings,
        event_ingredients.meal_id,
        event_ingredients.subrecipe_hierarchy,
        event_ingredients.buy_by,
        event_ingredients.store_id,
        shopping_tour_ranges.tour_id
    FROM event_ingredients
    LEFT JOIN shopping_tour_ranges ON 
        event_ingredients.event_id = shopping_tour_ranges.event_id
        AND event_ingredients.store_id = shopping_tour_ranges.store_id
        AND shopping_tour_ranges.tour_date <= event_ingredients.buy_by
        AND (shopping_tour_ranges.next_tour_date IS NULL 
            OR event_ingredients.buy_by <= shopping_tour_ranges.next_tour_date);

CREATE OR REPLACE VIEW public.shopping_list AS
    SELECT full_weight.event_id,
        full_weight.event_name,
        full_weight.ingredient_id,
        full_weight.ingredient,
        SUM(full_weight.weight) AS weight,
        ROUND(SUM(full_weight.weight) * ingredients.energy * 1000::numeric, 2) AS energy,
        (SUM(full_weight.weight) / price_per_ingredient_weight.weight)::double precision * COALESCE(price_per_ingredient_weight.price, '-1,00 €'::money) AS price,
        full_weight.tour_id
    FROM ( 
        SELECT prefetch.event_id,
            prefetch.tour_id,
            prefetch.event_name,
            prefetch.ingredient_id,
            prefetch.ingredient,
            prefetch.store_id,
            prefetch.buy_by,
            ROUND(GREATEST(prefetch.weight::double precision - COALESCE(event_storage.amount::double precision, 0::double precision), 0::double precision)::numeric, 2) AS weight
        FROM ( 
            SELECT shopping_tour_ingredients.event_id,
                shopping_tour_ingredients.tour_id,
                shopping_tour_ingredients.event AS event_name,
                shopping_tour_ingredients.ingredient_id,
                shopping_tour_ingredients.ingredient,
                shopping_tour_ingredients.store_id,
                shopping_tour_ingredients.buy_by,
                SUM(shopping_tour_ingredients.weight) AS weight
            FROM shopping_tour_ingredients
            WHERE shopping_tour_ingredients.tour_id IS NOT NULL
            GROUP BY 
                shopping_tour_ingredients.event_id, 
                shopping_tour_ingredients.tour_id,
                shopping_tour_ingredients.event, 
                shopping_tour_ingredients.ingredient_id, 
                shopping_tour_ingredients.ingredient,
                shopping_tour_ingredients.buy_by,
                shopping_tour_ingredients.store_id) prefetch 
        LEFT JOIN event_storage USING (event_id, ingredient_id)) full_weight
    LEFT JOIN LATERAL ( 
        SELECT ingredient_sources.ingredient_id,
            ingredient_sources.package_size * ingredient_weight.weight AS weight,
            ingredient_sources.price
        FROM ingredient_sources
        LEFT JOIN ingredient_weight USING (unit_id, ingredient_id)) price_per_ingredient_weight USING (ingredient_id)
    LEFT JOIN ingredients USING (ingredient_id)
    GROUP BY full_weight.event_id,
        full_weight.tour_id,
        full_weight.event_name,
        full_weight.ingredient_id,
        full_weight.ingredient,
        ingredients.energy,
        price_per_ingredient_weight.weight,
        price_per_ingredient_weight.price;

CREATE OR REPLACE VIEW public.shopping_tour_ingredients_without_tour AS
    SELECT * FROM shopping_tour_ingredients WHERE tour_id IS NULL;
