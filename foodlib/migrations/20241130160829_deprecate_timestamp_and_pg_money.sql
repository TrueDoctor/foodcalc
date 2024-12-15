-- Start transaction
BEGIN;

-- Disable triggers temporarily
SET session_replication_role = 'replica';

-- Drop views in dependency order
DROP VIEW IF EXISTS 
    shopping_list_per_day,
    shopping_list,
    shopping_list_assumptions,
    shopping_tour_ingredients_without_tour,
    shopping_tour_ingredients,
    event_ingredients,
    prep_ingredients,
    prep_ingredients_with_duplicates,
    event_ingredients_before_prep_time_resolve,
    event_recipes,
    resolved_prep_starts,
    shopping_tour_ranges,
    all_later_tours,
    best_event_ingredient_sources,
    event_ingredients_with_source_options,
    event_ingredient_sources,
    source_price_per_kilo,
    event_storage,
    resolved_recipe_ingredients,
    resolved_recipes,
    resolved_meta,
    recipe_stats,
    recipe_weight,
    subrecipes,
    recipe_ingredients_view,
    ingredient_properties_view,
    ingredients_without_weight,
    ingredients_without_sources,
    ingredient_weight,
    meta_with_names CASCADE;

DROP MATERIALIZED VIEW IF EXISTS conversions;

-- Add temporary columns without NOT NULL initially
ALTER TABLE event_meals 
    ADD COLUMN temp_start timestamptz,
    ADD COLUMN temp_end timestamptz;

-- Update data with proper timezone conversion, handling nulls
UPDATE event_meals SET
    temp_start = COALESCE(start_time AT TIME ZONE COALESCE(current_setting('timezone', true), 'UTC'), '1970-01-01'::timestamptz),
    temp_end = COALESCE(end_time AT TIME ZONE COALESCE(current_setting('timezone', true), 'UTC'), '1970-01-01'::timestamptz);

-- Now add NOT NULL constraints after data is populated
ALTER TABLE event_meals 
    ALTER COLUMN temp_start SET NOT NULL,
    ALTER COLUMN temp_end SET NOT NULL;

ALTER TABLE shopping_tours
    ADD COLUMN temp_tour timestamptz;
  
UPDATE shopping_tours SET
    temp_tour = COALESCE(tour_date AT TIME ZONE COALESCE(current_setting('timezone', true), 'UTC'), '1970-01-01'::timestamptz);

ALTER TABLE shopping_tours 
    ALTER COLUMN temp_tour SET NOT NULL;

ALTER TABLE food_prep
    ADD COLUMN temp_prep timestamptz,
    ADD COLUMN temp_use_from timestamptz,
    ADD COLUMN temp_use_until timestamptz;
    
UPDATE food_prep SET
    temp_prep = COALESCE(prep_date AT TIME ZONE COALESCE(current_setting('timezone', true), 'UTC'), '1970-01-01'::timestamptz),
    temp_use_from = use_from AT TIME ZONE COALESCE(current_setting('timezone', true), 'UTC'),
    temp_use_until = COALESCE(use_until AT TIME ZONE COALESCE(current_setting('timezone', true), 'UTC'), '1970-01-01'::timestamptz);

ALTER TABLE food_prep
    ALTER COLUMN temp_prep SET NOT NULL,
    ALTER COLUMN temp_use_until SET NOT NULL;

-- Convert money columns to numeric(10,2)
ALTER TABLE events ADD COLUMN temp_budget numeric(10,2);
UPDATE events SET temp_budget = budget::numeric;

ALTER TABLE ingredient_sources ADD COLUMN temp_price numeric(10,2);
UPDATE ingredient_sources SET
    temp_price = COALESCE(price::numeric, 0);
ALTER TABLE ingredient_sources 
    ALTER COLUMN temp_price SET NOT NULL;

ALTER TABLE users
    ADD COLUMN temp_created_at timestamptz;
UPDATE users SET
    temp_created_at = COALESCE(created_at AT TIME ZONE COALESCE(current_setting('timezone', true), 'UTC'), '1970-01-01'::timestamptz);
ALTER TABLE users
    ALTER COLUMN temp_created_at SET NOT NULL;

-- Drop original columns
ALTER TABLE event_meals 
    DROP COLUMN start_time,
    DROP COLUMN end_time;

ALTER TABLE shopping_tours 
    DROP COLUMN tour_date;

ALTER TABLE food_prep 
    DROP COLUMN prep_date,
    DROP COLUMN use_from,
    DROP COLUMN use_until;

ALTER TABLE events 
    DROP COLUMN budget;

ALTER TABLE ingredient_sources 
    DROP COLUMN price;

ALTER TABLE users
    DROP COLUMN created_at;

-- Rename temp columns
ALTER TABLE event_meals
    RENAME COLUMN temp_start TO start_time;
ALTER TABLE event_meals  
    RENAME COLUMN temp_end TO end_time;

ALTER TABLE shopping_tours
    RENAME COLUMN temp_tour TO tour_date;

ALTER TABLE food_prep
    RENAME COLUMN temp_prep TO prep_date;
ALTER TABLE food_prep  
    RENAME COLUMN temp_use_from TO use_from;
ALTER TABLE food_prep
    RENAME COLUMN temp_use_until TO use_until;

ALTER TABLE events
    RENAME COLUMN temp_budget TO budget;

ALTER TABLE ingredient_sources
    RENAME COLUMN temp_price TO price;

ALTER TABLE users
    RENAME COLUMN temp_created_at TO created_at;

-- Recreate views in reverse dependency order
-- First recreate the base views that others depend on
CREATE MATERIALIZED VIEW conversions AS
 WITH RECURSIVE conversion_table AS (
         SELECT base_conversions.from_unit,
            base_conversions.to_unit,
            base_conversions.from_amount,
            base_conversions.to_amount
           FROM base_conversions
        UNION
         SELECT base_conversions.to_unit,
            base_conversions.from_unit,
            base_conversions.to_amount,
            base_conversions.from_amount
           FROM base_conversions
        UNION
         SELECT conversion_table_1.from_unit,
            base_conversions.to_unit,
            conversion_table_1.from_amount,
            round((conversion_table_1.to_amount * (base_conversions.to_amount / base_conversions.from_amount)), 6) AS round
           FROM (conversion_table conversion_table_1
             JOIN base_conversions ON ((conversion_table_1.to_unit = base_conversions.from_unit)))
        )
 SELECT conversion_table.from_unit,
    conversion_table.to_unit,
    conversion_table.from_amount,
    conversion_table.to_amount
   FROM conversion_table
  WITH NO DATA;

REFRESH MATERIALIZED VIEW conversions;

-- Now would you like me to continue with recreating the remaining views in proper order?
-- I can provide the SQL for recreating each view, maintaining the proper dependencies.


CREATE VIEW ingredient_weight AS
 SELECT weights.ingredient_id,
    weights.unit_id,
    weights.weight
   FROM weights
UNION
 SELECT ingredients.ingredient_id,
    conversions.from_unit AS unit_id,
    (conversions.to_amount / conversions.from_amount) AS weight
   FROM ingredients,
    conversions
  WHERE (conversions.to_unit = 0);

CREATE VIEW meta_with_names AS
 SELECT meta_recipes.parent_id,
    r1.name AS parent,
    meta_recipes.child_id,
    r2.name AS child,
    meta_recipes.weight
   FROM ((meta_recipes
     JOIN recipes r1 ON ((r1.recipe_id = meta_recipes.parent_id)))
     JOIN recipes r2 ON ((r2.recipe_id = meta_recipes.child_id)));

CREATE VIEW recipe_weight AS
 SELECT recipes.recipe_id,
    recipes.name,
    recipes.comment,
    sum(weights.weight) AS weight
   FROM (recipes
     JOIN ( SELECT meta_recipes.parent_id AS recipe_id,
            meta_recipes.weight
           FROM meta_recipes
        UNION ALL
         SELECT recipe_ingredients.recipe_id,
            (weights_1.weight * recipe_ingredients.amount)
           FROM (recipe_ingredients
             JOIN ingredient_weight weights_1 USING (unit_id, ingredient_id))) weights USING (recipe_id))
  GROUP BY recipes.recipe_id, recipes.name, recipes.comment;

CREATE VIEW resolved_meta AS
 WITH RECURSIVE meta AS (
         SELECT meta_recipes.parent_id AS recipe_id,
            meta_recipes.child_id,
            meta_recipes.weight,
            (meta_recipes.parent_id::text || '.'::text) || meta_recipes.child_id::text AS acc,
            meta_recipes.parent_id
           FROM meta_recipes
        UNION
         SELECT meta_1.recipe_id,
            meta_recipes.child_id,
            meta_recipes.weight * (meta_1.weight / recipe_weight.weight) AS weight,
            (meta_1.acc || '.'::text) || meta_recipes.child_id AS acc,
            meta_1.child_id AS parent_id
           FROM ((meta meta_1
             JOIN meta_recipes ON ((meta_1.child_id = meta_recipes.parent_id)))
             JOIN recipe_weight ON ((recipe_weight.recipe_id = meta_1.child_id)))
        )
 SELECT DISTINCT meta.recipe_id,
    meta.child_id AS subrecipe_id,
    meta.weight,
    r.name AS recipe,
    mr.name AS subrecipe,
    meta.acc,
    meta.parent_id,
    r2.name AS parent
   FROM ((((meta
     JOIN recipe_ingredients ON ((meta.child_id = recipe_ingredients.recipe_id)))
     JOIN recipes r ON ((r.recipe_id = meta.recipe_id)))
     JOIN recipes r2 ON ((r2.recipe_id = meta.parent_id)))
     JOIN recipes mr ON ((mr.recipe_id = meta.child_id)));

CREATE VIEW resolved_recipes AS
 SELECT recipes.recipe_id,
    recipes.name AS recipe,
    recipe_ingredients.ingredient_id,
    ingredients.name AS ingredient,
    ((resolved_meta.weight / recipe_weight.weight) * (ingredient_weight.weight * recipe_ingredients.amount)) AS weight,
    resolved_meta.subrecipe AS subrecipe_id,
    subrecipes.name AS subrecipe,
    resolved_meta.acc
   FROM ((((((recipes
     LEFT JOIN ( SELECT resolved_meta_1.recipe_id,
            resolved_meta_1.subrecipe_id AS subrecipe,
            resolved_meta_1.weight,
            resolved_meta_1.acc
           FROM resolved_meta resolved_meta_1
        UNION
         SELECT recipes_1.recipe_id,
            recipes_1.recipe_id,
            recipe_weight_1.weight,
            NULL::text AS acc
           FROM (recipes recipes_1
             JOIN recipe_weight recipe_weight_1 USING (recipe_id))) resolved_meta USING (recipe_id))
     JOIN recipe_ingredients ON ((recipe_ingredients.recipe_id = resolved_meta.subrecipe)))
     JOIN recipe_weight ON ((resolved_meta.subrecipe = recipe_weight.recipe_id)))
     LEFT JOIN ingredients USING (ingredient_id))
     LEFT JOIN ingredient_weight USING (ingredient_id, unit_id))
     LEFT JOIN recipes subrecipes ON ((resolved_meta.subrecipe = subrecipes.recipe_id)))
  ORDER BY recipes.recipe_id;

CREATE VIEW resolved_recipe_ingredients AS
 SELECT resolved_recipes.recipe_id,
    resolved_recipes.recipe,
    resolved_recipes.ingredient_id,
    resolved_recipes.ingredient,
    sum(resolved_recipes.weight) AS weight
   FROM resolved_recipes
  GROUP BY resolved_recipes.recipe_id, resolved_recipes.recipe, resolved_recipes.ingredient_id, resolved_recipes.ingredient;

CREATE VIEW recipe_stats AS
 SELECT rr.recipe_id,
    rr.recipe,
    sum(rr.weight) AS weight,
    sum(((ingredients.energy * rr.weight) * 1000.0)) AS energy
   FROM (resolved_recipes rr
     JOIN ingredients USING (ingredient_id))
  GROUP BY rr.recipe_id, rr.recipe
  ORDER BY rr.recipe_id;

CREATE VIEW event_storage AS
 SELECT event_inventories.event_id,
    inventory_ingredients.ingredient_id,
    sum(inventory_ingredients.amount) AS amount
   FROM (inventory_ingredients
     JOIN event_inventories USING (inventory_id))
  GROUP BY event_inventories.event_id, inventory_ingredients.ingredient_id;

CREATE VIEW source_price_per_kilo AS
    SELECT ingredient_sources.ingredient_source_id,
        ingredient_sources.price / (ingredient_sources.package_size * ingredient_weight.weight)::numeric AS price_per_kilo
    FROM ingredient_sources
    LEFT JOIN ingredient_weight USING (ingredient_id, unit_id);

CREATE VIEW event_ingredient_sources AS 
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

CREATE VIEW event_ingredients_with_source_options AS
    SELECT events.event_id,
        resolved_recipes.ingredient_id,
        source_price_per_kilo.price_per_kilo AS price,
        source_price_per_kilo.ingredient_source_id
    FROM events
    LEFT JOIN event_meals USING (event_id)
    LEFT JOIN resolved_recipes USING (recipe_id)
    LEFT JOIN event_ingredient_sources USING (event_id, ingredient_id)
    LEFT JOIN source_price_per_kilo USING (ingredient_source_id);

CREATE VIEW best_event_ingredient_sources AS
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

CREATE VIEW all_later_tours AS
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

CREATE VIEW shopping_tour_ranges AS
    SELECT event_id, tour_id, tour_date, store_id, next_tour_date 
    FROM all_later_tours
    WHERE all_later_tours.next_tour_date = (
        SELECT MIN(next_tour_date) FROM all_later_tours all_later_tours_1
        WHERE all_later_tours.tour_id = all_later_tours_1.tour_id) 
        OR all_later_tours.next_tour_date IS NULL;

CREATE VIEW resolved_prep_starts AS
    SELECT food_prep.prep_id,
        food_prep.event_id,
        food_prep.recipe_id,
        food_prep.prep_date,
        COALESCE(food_prep.use_from, food_prep.prep_date) AS use_from,
        food_prep.use_until
    FROM food_prep;

CREATE VIEW event_ingredients_before_prep_time_resolve AS
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
        ingredient_sources.price * (recipe_multipliers.recipe_multiplier * resolved_recipes.weight / (ingredient_weight.weight * COALESCE(ingredient_sources.package_size, 1::numeric)))::numeric AS price,
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
    LEFT JOIN (
        SELECT event_meals_1.recipe_id,
            event_meals_1.event_id,
            event_meals_1.place_id,
            event_meals_1.start_time,
            event_meals_1.energy_per_serving * event_meals_1.servings::numeric / recipe_stats.energy AS recipe_multiplier
        FROM event_meals event_meals_1
        JOIN recipe_stats USING (recipe_id)
    ) recipe_multipliers USING (event_id, recipe_id, place_id, start_time);


CREATE VIEW prep_ingredients_with_duplicates AS
    SELECT resolved_prep_starts.event_id,
        resolved_prep_starts.prep_id,
        resolved_prep_starts.recipe_id,
        resolved_prep_starts.prep_date,
        event_ingredients_before_prep_time_resolve.ingredient_id,
        event_ingredients_before_prep_time_resolve.ingredient,
        event_ingredients_before_prep_time_resolve.weight,
        event_ingredients_before_prep_time_resolve.price,
        event_ingredients_before_prep_time_resolve.meal_id,
        event_ingredients_before_prep_time_resolve.subrecipe_hierarchy
   FROM resolved_prep_starts
   LEFT JOIN event_ingredients_before_prep_time_resolve USING (event_id)
   WHERE event_ingredients_before_prep_time_resolve.subrecipe_hierarchy ~ concat('^(.*\.)?', resolved_prep_starts.recipe_id, '(\..*)?$') 
   AND event_ingredients_before_prep_time_resolve.start_time >= resolved_prep_starts.use_from
   AND event_ingredients_before_prep_time_resolve.start_time <= resolved_prep_starts.use_until;

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

CREATE VIEW event_ingredients AS
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
    LEFT JOIN prep_ingredients USING (event_id, ingredient_id, subrecipe_hierarchy, meal_id);

CREATE VIEW event_recipes AS
    SELECT event_ingredients.event_id,
        event_ingredients.event,
        event_ingredients.recipe_id,
        event_ingredients.recipe,
        round(sum(weight), 2) AS weights,
        round(sum(energy), 2) AS energy,
        sum(price) AS price
    FROM event_ingredients
    GROUP BY event_ingredients.event_id, event_ingredients.event, event_ingredients.recipe_id, event_ingredients.recipe;

CREATE VIEW shopping_tour_ingredients AS
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

CREATE VIEW shopping_tour_ingredients_without_tour AS
    SELECT * FROM shopping_tour_ingredients 
    WHERE tour_id IS NULL;

CREATE VIEW shopping_list AS
    SELECT full_weight.event_id,
        full_weight.event_name,
        full_weight.ingredient_id,
        full_weight.ingredient,
        sum(full_weight.weight) AS weight,
        round(sum(full_weight.weight) * ingredients.energy * 1000::numeric, 2) AS energy,
        sum(full_weight.weight)::numeric * COALESCE(price_per_ingredient_weight.price, -1) AS price,
        full_weight.tour_id,
        category
    FROM (
        SELECT prefetch.event_id,
            prefetch.tour_id,
            prefetch.event_name,
            prefetch.ingredient_id,
            prefetch.ingredient,
            prefetch.store_id,
            prefetch.buy_by,
            round(GREATEST(prefetch.weight::numeric - COALESCE(event_storage.amount::numeric, 0::numeric), 0::numeric)::numeric, 2) AS weight
        FROM (
            SELECT shopping_tour_ingredients.event_id,
                shopping_tour_ingredients.tour_id,
                shopping_tour_ingredients.event AS event_name,
                shopping_tour_ingredients.ingredient_id,
                shopping_tour_ingredients.ingredient,
                shopping_tour_ingredients.store_id,
                shopping_tour_ingredients.buy_by,
                sum(shopping_tour_ingredients.weight) AS weight
            FROM shopping_tour_ingredients
            GROUP BY 
                shopping_tour_ingredients.event_id,
                shopping_tour_ingredients.tour_id,
                shopping_tour_ingredients.event,
                shopping_tour_ingredients.ingredient_id,
                shopping_tour_ingredients.ingredient,
                shopping_tour_ingredients.buy_by,
                shopping_tour_ingredients.store_id
        ) prefetch
        LEFT JOIN event_storage USING (event_id, ingredient_id)
    ) full_weight
    LEFT JOIN best_event_ingredient_sources price_per_ingredient_weight USING (ingredient_id, event_id)
    LEFT JOIN metro_categories USING (ingredient_source_id)
    LEFT JOIN ingredients USING (ingredient_id)
    GROUP BY 
        full_weight.event_id,
        full_weight.tour_id,
        full_weight.event_name,
        full_weight.ingredient_id,
        full_weight.ingredient,
        ingredients.energy,
        price_per_ingredient_weight.price,
        category;

CREATE VIEW shopping_list_per_day AS
    SELECT event_ingredients.event_id,
        event_ingredients.event AS event_name,
        event_ingredients.ingredient_id,
        event_ingredients.ingredient,
        round(sum(event_ingredients.weight), 2) AS weight,
        round(sum(event_ingredients.energy), 2) AS energy,
        sum(event_ingredients.price) AS price,
        date_trunc('day', event_ingredients.buy_by) AS day
    FROM event_ingredients
    GROUP BY 
        event_ingredients.event_id,
        event_ingredients.event,
        event_ingredients.ingredient_id,
        event_ingredients.ingredient,
        date_trunc('day', event_ingredients.buy_by);

CREATE VIEW shopping_list_assumptions AS
    SELECT prefetch.event_id,
        prefetch.event_name,
        prefetch.ingredient_id,
        prefetch.ingredient_name,
        round(LEAST(prefetch.weight::numeric, event_storage.amount::numeric)::numeric, 2) AS round
    FROM (
        SELECT event_ingredients.event_id,
            event_ingredients.event AS event_name,
            event_ingredients.ingredient_id,
            event_ingredients.ingredient AS ingredient_name,
            sum(event_ingredients.weight) AS weight
        FROM event_ingredients
        GROUP BY 
            event_ingredients.event_id,
            event_ingredients.event,
            event_ingredients.ingredient_id,
            event_ingredients.ingredient
    ) prefetch
    JOIN event_storage USING (event_id, ingredient_id);


CREATE VIEW recipe_ingredients_view AS
    SELECT recipes.name AS recipe,
        ingredients.name AS ingredient,
        recipe_ingredients.amount,
        units.name AS unit
    FROM units,
        recipes,
        recipe_ingredients,
        ingredients
    WHERE recipes.recipe_id = recipe_ingredients.recipe_id 
        AND ingredients.ingredient_id = recipe_ingredients.ingredient_id 
        AND units.unit_id = recipe_ingredients.unit_id;

CREATE VIEW ingredient_properties_view AS
    SELECT ingredients.name AS ingredient,
        food_properties.name AS property
    FROM ingredient_properties,
        ingredients,
        food_properties
    WHERE ingredients.ingredient_id = ingredient_properties.ingredient_id 
        AND food_properties.property_id = ingredient_properties.property_id;

CREATE VIEW ingredients_without_weight AS
    SELECT ingredients.ingredient_id,
        ingredients.name AS ingredient,
        ingredients.comment,
        recipe_ingredients.recipe_id,
        units.unit_id,
        units.name AS unit
    FROM recipe_ingredients
        LEFT JOIN ingredient_weight USING (unit_id, ingredient_id)
        LEFT JOIN ingredients USING (ingredient_id)
        LEFT JOIN units USING (unit_id)
    WHERE ingredient_weight.weight IS NULL;

CREATE VIEW ingredients_without_sources AS
    SELECT ingredients.ingredient_id,
        ingredients.name AS ingredient,
        ingredients.comment,
        recipe_ingredients.recipe_id
    FROM recipe_ingredients
        LEFT JOIN ingredient_sources USING (ingredient_id)
        LEFT JOIN ingredients USING (ingredient_id)
    WHERE ingredient_sources.store_id IS NULL;

CREATE VIEW subrecipes AS
    SELECT bar.recipe,
        bar.recipe_id,
        bar.ingredient,
        bar.weight,
        bar.subrecipe,
        bar.subrecipe_id,
        bar.is_subrecipe
    FROM (
        SELECT rr.recipe_id,
            rr.recipe,
            rr.ingredient,
            sum(rr.weight / recipe_weight.weight) AS weight,
            rr.subrecipe_id,
            recipes_1.name AS subrecipe,
            false AS is_subrecipe
        FROM resolved_recipes rr
            JOIN recipe_weight USING (recipe_id)
            JOIN recipes recipes_1 ON rr.subrecipe_id = recipes_1.recipe_id
        GROUP BY 
            rr.recipe_id,
            rr.subrecipe_id,
            rr.recipe,
            rr.ingredient_id,
            rr.ingredient,
            recipes_1.name
        UNION ALL
        SELECT resolved_meta.recipe_id,
            resolved_meta.recipe,
            resolved_meta.subrecipe AS ingredient,
            sum(resolved_meta.weight / recipe_weight.weight) AS weight,
            resolved_meta.parent_id,
            resolved_meta.parent,
            true AS is_subrecipe
        FROM resolved_meta
            JOIN recipe_weight ON recipe_weight.recipe_id = resolved_meta.recipe_id
        GROUP BY 
            resolved_meta.recipe_id,
            resolved_meta.recipe,
            resolved_meta.subrecipe,
            resolved_meta.subrecipe_id,
            resolved_meta.parent_id,
            resolved_meta.parent
    ) bar
    JOIN recipes USING (recipe_id)
    ORDER BY bar.recipe, bar.subrecipe_id, bar.is_subrecipe DESC;

-- Re-enable triggers
SET session_replication_role = 'origin';

COMMIT;
