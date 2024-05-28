create or replace view public.shopping_list (event_id, event_name, ingredient_id, ingredient, weight, energy, price) as
SELECT full_weight.event_id,
       full_weight.event_name,
       full_weight.ingredient_id,
       full_weight.ingredient,
       full_weight.weight,
       round(full_weight.weight * ingredients.energy * 1000::numeric, 2) AS energy,
       (full_weight.weight / price_per_ingredient_weight.weight)::double precision *
       COALESCE(price_per_ingredient_weight.price, '-1,00 €'::money)     AS price
FROM (SELECT prefetch.event_id,
             prefetch.event_name,
             prefetch.ingredient_id,
             prefetch.ingredient,
             round(GREATEST(prefetch.weight::double precision -
                            COALESCE(event_storage.amount::double precision, 0::double precision),
                            0::double precision)::numeric, 2) AS weight
      FROM (SELECT event_ingredients.event_id,
                   event_ingredients.event       AS event_name,
                   event_ingredients.ingredient_id,
                   event_ingredients.ingredient,
                   sum(event_ingredients.weight) AS weight
            FROM event_ingredients
            GROUP BY event_ingredients.event_id, event_ingredients.event, event_ingredients.ingredient_id,
                     event_ingredients.ingredient) prefetch
               LEFT JOIN event_storage USING (event_id, ingredient_id)) full_weight
         LEFT JOIN (SELECT ingredient_sources.ingredient_id,
                           ingredient_sources.package_size * ingredient_weight.weight AS weight,
                           ingredient_sources.price
                    FROM ingredient_sources
                             LEFT JOIN ingredient_weight USING (unit_id, ingredient_id)) price_per_ingredient_weight
                   USING (ingredient_id)
         LEFT JOIN ingredients USING (ingredient_id);
