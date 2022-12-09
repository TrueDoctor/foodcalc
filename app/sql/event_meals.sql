CREATE TABLE event_meals (
    event_id integer NOT NULL  REFERENCES events(event_id),
    recipe_id integer NOT NULL  REFERENCES recipes(recipe_id),
    place_id integer NOT NULL REFERENCES places(place_id),
    comment text,
    energy_per_serving numeric NOT NULL,
    servings integer NOT NULL,
    start_time timestamp NOT NULL,
    end_time timestamp NOT NULL,
    CONSTRAINT event_meals_pk PRIMARY KEY (event_id, recipe_id, start_time, place_id)
);
