ALTER TABLE public.ingredient_sources 
    DROP CONSTRAINT ingredient_source_pk,
    ADD ingredient_source_id SERIAL PRIMARY KEY;

CREATE TABLE public.event_source_overrides (
    event_id INTEGER NOT NULL REFERENCES events(event_id),
    ingredient_source_id INTEGER NOT NULL REFERENCES ingredient_sources(ingredient_source_id),
    PRIMARY KEY (event_id, ingredient_source_id)
    -- // TODO: add constraint to prevent multiple overrides for the same ingredient in one event
);

CREATE TABLE public.food_prep(
    prep_id SERIAL PRIMARY KEY,
    event_id INTEGER NOT NULL REFERENCES events(event_id),
    recipe_id INTEGER NOT NULL REFERENCES recipes(recipe_id),
    prep_date TIMESTAMP NOT NULL,
    use_from TIMESTAMP,
    use_until TIMESTAMP NOT NULL
);

CREATE TABLE public.shopping_tours (
    tour_id SERIAL PRIMARY KEY,
    event_id INTEGER NOT NULL REFERENCES events(event_id),
    tour_date TIMESTAMP NOT NULL,
    store_id INTEGER NOT NULL REFERENCES stores(store_id)
);
