-- Add inventory table
CREATE TABLE public.inventories (
    inventory_id SERIAL NOT NULL,
    name character varying NOT NULL
);

-- Add table to associate ingredients with inventories
CREATE TABLE public.inventory_ingredients (
    inventory_id integer NOT NULL,
    ingredient_id integer NOT NULL,
    amount numeric
);

-- Add table to associate inventories with events
CREATE TABLE public.event_inventories (
    event_id integer NOT NULL,
    inventory_id integer NOT NULL
);

-- Add view to show storage per event
CREATE OR REPLACE VIEW public.event_storage AS
 SELECT event_inventories.event_id,
    inventory_ingredients.ingredient_id,
    sum(inventory_ingredients.amount) AS amount
   FROM (public.inventory_ingredients
     JOIN public.event_inventories USING (inventory_id))
  GROUP BY event_inventories.event_id, inventory_ingredients.ingredient_id;

-- Change shopping list to use inventories
CREATE OR REPLACE VIEW public.shopping_list AS
 SELECT full_weight.event_id,
    full_weight.event_name,
    full_weight.ingredient_id,
    full_weight.ingredient,
    full_weight.weight,
    round(full_weight.weight * ingredients.energy * 1000::numeric, 2) AS energy,
    ceil(full_weight.weight / price_per_ingredient_weight.weight)::double precision * COALESCE(price_per_ingredient_weight.price, '-1,00 €'::money) AS price
   FROM ( SELECT prefetch.event_id,
            prefetch.event_name,
            prefetch.ingredient_id,
            prefetch.ingredient,
            round(GREATEST(prefetch.weight::double precision - COALESCE(event_storage.amount, 0::double precision), 0::double precision)::numeric, 2) AS weight
           FROM ( SELECT event_ingredients.event_id,
                    event_ingredients.event AS event_name,
                    event_ingredients.ingredient_id,
                    event_ingredients.ingredient,
                    sum(event_ingredients.weight) AS weight
                   FROM event_ingredients
                  GROUP BY event_ingredients.event_id, event_ingredients.event, event_ingredients.ingredient_id, event_ingredients.ingredient) prefetch
             LEFT JOIN event_storage USING (event_id, ingredient_id)) full_weight
     LEFT JOIN ( SELECT ingredient_sources.ingredient_id,
            ingredient_sources.package_size * ingredient_weight.weight AS weight,
            ingredient_sources.price
           FROM ingredient_sources
             LEFT JOIN ingredient_weight USING (unit_id, ingredient_id)) price_per_ingredient_weight USING (ingredient_id)
     LEFT JOIN ingredients USING (ingredient_id);

-- Add view for inventory checklist
CREATE OR REPLACE VIEW public.shopping_list_assumptions AS
 SELECT prefetch.event_id,
    prefetch.event_name,
    prefetch.ingredient_id,
    prefetch.ingredient_name,
    round((LEAST((prefetch.weight)::double precision, event_storage.amount))::numeric, 2) AS round
   FROM (( SELECT event_ingredients.event_id,
            event_ingredients.event AS event_name,
            event_ingredients.ingredient_id,
            event_ingredients.ingredient AS ingredient_name,
            sum(event_ingredients.weight) AS weight
           FROM public.event_ingredients
          GROUP BY event_ingredients.event_id, event_ingredients.event, event_ingredients.ingredient_id, event_ingredients.ingredient) prefetch
     JOIN public.event_storage USING (event_id, ingredient_id));