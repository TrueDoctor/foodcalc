-- Add migration script here
--
-- PostgreSQL database dump
--

-- Dumped from database version 15.3
-- Dumped by pg_dump version 15.3

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

--
-- Name: public; Type: SCHEMA; Schema: -; Owner: postgres
--

-- *not* creating schema, since initdb creates it


ALTER SCHEMA public OWNER TO postgres;

SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: base_conversions; Type: TABLE; Schema: public; Owner: kuechenteam
--

CREATE TABLE public.base_conversions (
    from_unit integer NOT NULL,
    to_unit integer NOT NULL,
    from_amount numeric NOT NULL,
    to_amount numeric NOT NULL
);


ALTER TABLE public.base_conversions OWNER TO kuechenteam;

--
-- Name: conversions; Type: MATERIALIZED VIEW; Schema: public; Owner: kuechenteam
--

CREATE MATERIALIZED VIEW public.conversions AS
 WITH RECURSIVE conversion_table AS (
         SELECT base_conversions.from_unit,
            base_conversions.to_unit,
            base_conversions.from_amount,
            base_conversions.to_amount
           FROM public.base_conversions
        UNION
         SELECT conversion_table_1.from_unit,
            base_conversions.to_unit,
            conversion_table_1.from_amount,
            round((conversion_table_1.to_amount * (base_conversions.to_amount / base_conversions.from_amount)), 6) AS round
           FROM (conversion_table conversion_table_1
             JOIN public.base_conversions ON ((conversion_table_1.to_unit = base_conversions.from_unit)))
        )
 SELECT conversion_table.from_unit,
    conversion_table.to_unit,
    conversion_table.from_amount,
    conversion_table.to_amount
   FROM conversion_table
  WITH NO DATA;


ALTER TABLE public.conversions OWNER TO kuechenteam;

--
-- Name: event_meals; Type: TABLE; Schema: public; Owner: kuechenteam
--

CREATE TABLE public.event_meals (
    event_id integer NOT NULL,
    recipe_id integer NOT NULL,
    place_id integer NOT NULL,
    comment text,
    energy_per_serving numeric NOT NULL,
    servings integer NOT NULL,
    start_time timestamp without time zone NOT NULL,
    end_time timestamp without time zone NOT NULL
);


ALTER TABLE public.event_meals OWNER TO kuechenteam;

--
-- Name: events; Type: TABLE; Schema: public; Owner: kuechenteam
--

CREATE TABLE public.events (
    event_id integer NOT NULL,
    event_name character varying NOT NULL,
    comment text,
    budget money
);


ALTER TABLE public.events OWNER TO kuechenteam;

--
-- Name: ingredient_sources; Type: TABLE; Schema: public; Owner: kuechenteam
--

CREATE TABLE public.ingredient_sources (
    ingredient_id integer NOT NULL,
    store_id integer NOT NULL,
    package_size numeric NOT NULL,
    unit_id integer NOT NULL,
    price money NOT NULL,
    url character varying,
    comment text,
    CONSTRAINT comment_not_empty CHECK ((comment <> ''::text))
);


ALTER TABLE public.ingredient_sources OWNER TO kuechenteam;

--
-- Name: ingredients; Type: TABLE; Schema: public; Owner: kuechenteam
--

CREATE TABLE public.ingredients (
    ingredient_id integer NOT NULL,
    name character varying NOT NULL,
    energy numeric DEFAULT 0 NOT NULL,
    comment text
);


ALTER TABLE public.ingredients OWNER TO kuechenteam;

--
-- Name: COLUMN ingredients.energy; Type: COMMENT; Schema: public; Owner: kuechenteam
--

COMMENT ON COLUMN public.ingredients.energy IS 'energy in kj/g';


--
-- Name: weights; Type: TABLE; Schema: public; Owner: kuechenteam
--

CREATE TABLE public.weights (
    unit_id integer NOT NULL,
    ingredient_id integer NOT NULL,
    weight numeric DEFAULT 0 NOT NULL
);


ALTER TABLE public.weights OWNER TO kuechenteam;

--
-- Name: COLUMN weights.weight; Type: COMMENT; Schema: public; Owner: kuechenteam
--

COMMENT ON COLUMN public.weights.weight IS 'weight in kg';


--
-- Name: ingredient_weight; Type: VIEW; Schema: public; Owner: kuechenteam
--

CREATE VIEW public.ingredient_weight AS
 SELECT weights.ingredient_id,
    weights.unit_id,
    weights.weight
   FROM public.weights
UNION
 SELECT ingredients.ingredient_id,
    conversions.from_unit AS unit_id,
    (conversions.to_amount / conversions.from_amount) AS weight
   FROM public.ingredients,
    public.conversions
  WHERE (conversions.to_unit = 0);


ALTER TABLE public.ingredient_weight OWNER TO kuechenteam;

--
-- Name: meta_recipes; Type: TABLE; Schema: public; Owner: kuechenteam
--

CREATE TABLE public.meta_recipes (
    parent_id integer NOT NULL,
    child_id integer NOT NULL,
    weight numeric NOT NULL,
    CONSTRAINT recursive CHECK ((child_id <> parent_id))
);


ALTER TABLE public.meta_recipes OWNER TO kuechenteam;

--
-- Name: COLUMN meta_recipes.weight; Type: COMMENT; Schema: public; Owner: kuechenteam
--

COMMENT ON COLUMN public.meta_recipes.weight IS 'in kg';


--
-- Name: places; Type: TABLE; Schema: public; Owner: kuechenteam
--

CREATE TABLE public.places (
    place_id integer NOT NULL,
    name character varying NOT NULL,
    comment text
);


ALTER TABLE public.places OWNER TO kuechenteam;

--
-- Name: recipe_ingredients; Type: TABLE; Schema: public; Owner: kuechenteam
--

CREATE TABLE public.recipe_ingredients (
    recipe_id integer NOT NULL,
    ingredient_id integer NOT NULL,
    amount numeric NOT NULL,
    unit_id integer NOT NULL
);


ALTER TABLE public.recipe_ingredients OWNER TO kuechenteam;

--
-- Name: recipes; Type: TABLE; Schema: public; Owner: kuechenteam
--

CREATE TABLE public.recipes (
    recipe_id integer NOT NULL,
    name character varying NOT NULL,
    comment text
);


ALTER TABLE public.recipes OWNER TO kuechenteam;

--
-- Name: recipe_weight; Type: VIEW; Schema: public; Owner: kuechenteam
--

CREATE VIEW public.recipe_weight AS
 SELECT recipes.recipe_id,
    recipes.name,
    recipes.comment,
    sum(weights.weight) AS weight
   FROM (public.recipes
     JOIN ( SELECT meta_recipes.parent_id AS recipe_id,
            meta_recipes.weight
           FROM public.meta_recipes
        UNION ALL
         SELECT recipe_ingredients.recipe_id,
            (weights_1.weight * recipe_ingredients.amount)
           FROM (public.recipe_ingredients
             JOIN public.ingredient_weight weights_1 USING (unit_id, ingredient_id))) weights USING (recipe_id))
  GROUP BY recipes.recipe_id, recipes.name, recipes.comment;


ALTER TABLE public.recipe_weight OWNER TO kuechenteam;

--
-- Name: resolved_meta; Type: VIEW; Schema: public; Owner: kuechenteam
--

CREATE VIEW public.resolved_meta AS
 WITH RECURSIVE meta AS (
         SELECT meta_recipes.parent_id AS recipe_id,
            meta_recipes.child_id,
            meta_recipes.weight,
            (meta_recipes.child_id)::text AS acc,
            meta_recipes.parent_id
           FROM public.meta_recipes
        UNION
         SELECT meta_1.recipe_id,
            meta_recipes.child_id,
            (meta_recipes.weight * (meta_1.weight / recipe_weight.weight)) AS weight,
            ((meta_1.acc || '.'::text) || meta_recipes.child_id) AS acc,
            meta_1.child_id AS parent_id
           FROM ((meta meta_1
             JOIN public.meta_recipes ON ((meta_1.child_id = meta_recipes.parent_id)))
             JOIN public.recipe_weight ON ((recipe_weight.recipe_id = meta_1.child_id)))
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
     JOIN public.recipe_ingredients ON ((meta.child_id = recipe_ingredients.recipe_id)))
     JOIN public.recipes r ON ((r.recipe_id = meta.recipe_id)))
     JOIN public.recipes r2 ON ((r2.recipe_id = meta.parent_id)))
     JOIN public.recipes mr ON ((mr.recipe_id = meta.child_id)));


ALTER TABLE public.resolved_meta OWNER TO kuechenteam;

--
-- Name: resolved_recipes; Type: VIEW; Schema: public; Owner: kuechenteam
--

CREATE VIEW public.resolved_recipes AS
 SELECT recipes.recipe_id,
    recipes.name AS recipe,
    recipe_ingredients.ingredient_id,
    ingredients.name AS ingredient,
    ((resolved_meta.weight / recipe_weight.weight) * (ingredient_weight.weight * recipe_ingredients.amount)) AS weight,
    resolved_meta.subrecipe AS subrecipe_id,
    subrecipes.name AS subrecipe,
    resolved_meta.acc
   FROM ((((((public.recipes
     LEFT JOIN ( SELECT resolved_meta_1.recipe_id,
            resolved_meta_1.subrecipe_id AS subrecipe,
            resolved_meta_1.weight,
            resolved_meta_1.acc
           FROM public.resolved_meta resolved_meta_1
        UNION
         SELECT recipes_1.recipe_id,
            recipes_1.recipe_id,
            recipe_weight_1.weight,
            NULL::text AS acc
           FROM (public.recipes recipes_1
             JOIN public.recipe_weight recipe_weight_1 USING (recipe_id))) resolved_meta USING (recipe_id))
     JOIN public.recipe_ingredients ON ((recipe_ingredients.recipe_id = resolved_meta.subrecipe)))
     JOIN public.recipe_weight ON ((resolved_meta.subrecipe = recipe_weight.recipe_id)))
     LEFT JOIN public.ingredients USING (ingredient_id))
     LEFT JOIN public.ingredient_weight USING (ingredient_id, unit_id))
     LEFT JOIN public.recipes subrecipes ON ((resolved_meta.subrecipe = subrecipes.recipe_id)))
  ORDER BY recipes.recipe_id;


ALTER TABLE public.resolved_recipes OWNER TO kuechenteam;

--
-- Name: recipe_stats; Type: VIEW; Schema: public; Owner: kuechenteam
--

CREATE VIEW public.recipe_stats AS
 SELECT rr.recipe_id,
    rr.recipe,
    sum(rr.weight) AS weight,
    sum(((ingredients.energy * rr.weight) * 1000.0)) AS energy
   FROM (public.resolved_recipes rr
     JOIN public.ingredients USING (ingredient_id))
  GROUP BY rr.recipe_id, rr.recipe
  ORDER BY rr.recipe_id;


ALTER TABLE public.recipe_stats OWNER TO kuechenteam;

--
-- Name: resolved_recipe_ingredients; Type: VIEW; Schema: public; Owner: kuechenteam
--

CREATE VIEW public.resolved_recipe_ingredients AS
 SELECT resolved_recipes.recipe_id,
    resolved_recipes.recipe,
    resolved_recipes.ingredient_id,
    resolved_recipes.ingredient,
    sum(resolved_recipes.weight) AS weight
   FROM public.resolved_recipes
  GROUP BY resolved_recipes.recipe_id, resolved_recipes.recipe, resolved_recipes.ingredient_id, resolved_recipes.ingredient;


ALTER TABLE public.resolved_recipe_ingredients OWNER TO kuechenteam;

--
-- Name: event_ingredients; Type: VIEW; Schema: public; Owner: kuechenteam
--

CREATE VIEW public.event_ingredients AS
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
    (resolved_recipes.weight * recipe_multipliers.recipe_multiplier) AS weight,
    (((recipe_multipliers.recipe_multiplier * resolved_recipes.weight) * ingredients.energy) * (1000)::numeric) AS energy,
    min((ingredient_sources.price * (((recipe_multipliers.recipe_multiplier * resolved_recipes.weight) / (ingredient_weight.weight * COALESCE(ingredient_sources.package_size, (1)::numeric))))::double precision)) AS price,
    event_meals.servings
   FROM (((((((public.events
     LEFT JOIN public.event_meals USING (event_id))
     LEFT JOIN public.places USING (place_id))
     LEFT JOIN public.resolved_recipe_ingredients resolved_recipes USING (recipe_id))
     LEFT JOIN public.ingredients USING (ingredient_id))
     LEFT JOIN public.ingredient_sources USING (ingredient_id))
     LEFT JOIN public.ingredient_weight USING (ingredient_id, unit_id))
     LEFT JOIN ( SELECT event_meals_1.recipe_id,
            event_meals_1.event_id,
            event_meals_1.place_id,
            event_meals_1.start_time,
            ((event_meals_1.energy_per_serving * (event_meals_1.servings)::numeric) / recipe_stats.energy) AS recipe_multiplier
           FROM (public.event_meals event_meals_1
             JOIN public.recipe_stats USING (recipe_id))) recipe_multipliers USING (event_id, recipe_id, place_id, start_time))
  GROUP BY events.event_id, events.event_name, event_meals.recipe_id, resolved_recipes.recipe, event_meals.place_id, places.name, event_meals.start_time, event_meals.end_time, resolved_recipes.ingredient_id, resolved_recipes.ingredient, resolved_recipes.weight, ingredients.energy, recipe_multipliers.recipe_multiplier, event_meals.servings;


ALTER TABLE public.event_ingredients OWNER TO kuechenteam;

--
-- Name: event_recipes; Type: VIEW; Schema: public; Owner: kuechenteam
--

CREATE VIEW public.event_recipes AS
 SELECT event_ingredients.event_id,
    event_ingredients.event,
    event_ingredients.recipe_id,
    event_ingredients.recipe,
    round(sum(event_ingredients.weight), 2) AS weights,
    round(sum(event_ingredients.energy), 2) AS energy,
    sum(event_ingredients.price) AS price
   FROM public.event_ingredients
  GROUP BY event_ingredients.event_id, event_ingredients.event, event_ingredients.recipe_id, event_ingredients.recipe;


ALTER TABLE public.event_recipes OWNER TO kuechenteam;

--
-- Name: events_event_id_seq; Type: SEQUENCE; Schema: public; Owner: kuechenteam
--

ALTER TABLE public.events ALTER COLUMN event_id ADD GENERATED BY DEFAULT AS IDENTITY (
    SEQUENCE NAME public.events_event_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);


--
-- Name: food_properties; Type: TABLE; Schema: public; Owner: kuechenteam
--

CREATE TABLE public.food_properties (
    property_id integer NOT NULL,
    name character varying
);


ALTER TABLE public.food_properties OWNER TO kuechenteam;

--
-- Name: food_properties_property_id_seq; Type: SEQUENCE; Schema: public; Owner: kuechenteam
--

ALTER TABLE public.food_properties ALTER COLUMN property_id ADD GENERATED BY DEFAULT AS IDENTITY (
    SEQUENCE NAME public.food_properties_property_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);


--
-- Name: ingredient_properties; Type: TABLE; Schema: public; Owner: kuechenteam
--

CREATE TABLE public.ingredient_properties (
    ingredient_id integer NOT NULL,
    property_id integer NOT NULL
);


ALTER TABLE public.ingredient_properties OWNER TO kuechenteam;

--
-- Name: ingredient_properties_view; Type: VIEW; Schema: public; Owner: kuechenteam
--

CREATE VIEW public.ingredient_properties_view AS
 SELECT ingredients.name AS ingredient,
    food_properties.name AS property
   FROM public.ingredient_properties,
    public.ingredients,
    public.food_properties
  WHERE ((ingredients.ingredient_id = ingredient_properties.ingredient_id) AND (food_properties.property_id = ingredient_properties.property_id));


ALTER TABLE public.ingredient_properties_view OWNER TO kuechenteam;

--
-- Name: ingredients_ingredient_id_seq; Type: SEQUENCE; Schema: public; Owner: kuechenteam
--

ALTER TABLE public.ingredients ALTER COLUMN ingredient_id ADD GENERATED BY DEFAULT AS IDENTITY (
    SEQUENCE NAME public.ingredients_ingredient_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);


--
-- Name: ingredients_without_sources; Type: VIEW; Schema: public; Owner: kuechenteam
--

CREATE VIEW public.ingredients_without_sources AS
 SELECT ingredients.ingredient_id,
    ingredients.name AS ingredient,
    ingredients.comment,
    recipe_ingredients.recipe_id
   FROM ((public.recipe_ingredients
     LEFT JOIN public.ingredient_sources USING (ingredient_id))
     LEFT JOIN public.ingredients USING (ingredient_id))
  WHERE (ingredient_sources.store_id IS NULL);


ALTER TABLE public.ingredients_without_sources OWNER TO kuechenteam;

--
-- Name: units; Type: TABLE; Schema: public; Owner: kuechenteam
--

CREATE TABLE public.units (
    unit_id integer NOT NULL,
    name character varying NOT NULL
);


ALTER TABLE public.units OWNER TO kuechenteam;

--
-- Name: ingredients_without_weight; Type: VIEW; Schema: public; Owner: kuechenteam
--

CREATE VIEW public.ingredients_without_weight AS
 SELECT ingredients.ingredient_id,
    ingredients.name AS ingredient,
    ingredients.comment,
    recipe_ingredients.recipe_id,
    units.unit_id,
    units.name AS unit
   FROM (((public.recipe_ingredients
     LEFT JOIN public.ingredient_weight USING (unit_id, ingredient_id))
     LEFT JOIN public.ingredients USING (ingredient_id))
     LEFT JOIN public.units USING (unit_id))
  WHERE (ingredient_weight.weight IS NULL);


ALTER TABLE public.ingredients_without_weight OWNER TO kuechenteam;

--
-- Name: meta_with_names; Type: VIEW; Schema: public; Owner: kuechenteam
--

CREATE VIEW public.meta_with_names AS
 SELECT meta_recipes.parent_id,
    r1.name AS parent,
    meta_recipes.child_id,
    r2.name AS child,
    meta_recipes.weight
   FROM ((public.meta_recipes
     JOIN public.recipes r1 ON ((r1.recipe_id = meta_recipes.parent_id)))
     JOIN public.recipes r2 ON ((r2.recipe_id = meta_recipes.child_id)));


ALTER TABLE public.meta_with_names OWNER TO kuechenteam;

--
-- Name: places_place_id_seq; Type: SEQUENCE; Schema: public; Owner: kuechenteam
--

ALTER TABLE public.places ALTER COLUMN place_id ADD GENERATED BY DEFAULT AS IDENTITY (
    SEQUENCE NAME public.places_place_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);


--
-- Name: recipe_ingredients_view; Type: VIEW; Schema: public; Owner: kuechenteam
--

CREATE VIEW public.recipe_ingredients_view AS
 SELECT recipes.name AS recipe,
    ingredients.name AS ingredient,
    recipe_ingredients.amount,
    units.name AS unit
   FROM public.units,
    public.recipes,
    public.recipe_ingredients,
    public.ingredients
  WHERE ((recipes.recipe_id = recipe_ingredients.recipe_id) AND (ingredients.ingredient_id = recipe_ingredients.ingredient_id) AND (units.unit_id = recipe_ingredients.unit_id));


ALTER TABLE public.recipe_ingredients_view OWNER TO kuechenteam;

--
-- Name: recipes_recipe_id_seq; Type: SEQUENCE; Schema: public; Owner: kuechenteam
--

ALTER TABLE public.recipes ALTER COLUMN recipe_id ADD GENERATED BY DEFAULT AS IDENTITY (
    SEQUENCE NAME public.recipes_recipe_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);


--
-- Name: shopping_list; Type: VIEW; Schema: public; Owner: kuechenteam
--

CREATE VIEW public.shopping_list AS
 SELECT event_ingredients.event_id,
    event_ingredients.event AS event_name,
    event_ingredients.ingredient_id,
    event_ingredients.ingredient,
    round(sum(event_ingredients.weight), 2) AS weight,
    round(sum(event_ingredients.energy), 2) AS energy,
    sum(event_ingredients.price) AS price
   FROM public.event_ingredients
  GROUP BY event_ingredients.event_id, event_ingredients.event, event_ingredients.ingredient_id, event_ingredients.ingredient;


ALTER TABLE public.shopping_list OWNER TO kuechenteam;

--
-- Name: shopping_list_per_day; Type: VIEW; Schema: public; Owner: postgres
--

CREATE VIEW public.shopping_list_per_day AS
 SELECT event_ingredients.event_id,
    event_ingredients.event AS event_name,
    event_ingredients.ingredient_id,
    event_ingredients.ingredient,
    round(sum(event_ingredients.weight), 2) AS weight,
    round(sum(event_ingredients.energy), 2) AS energy,
    sum(event_ingredients.price) AS price,
    date_trunc('day'::text, event_ingredients.start_time) AS day
   FROM public.event_ingredients
  GROUP BY event_ingredients.event_id, event_ingredients.event, event_ingredients.ingredient_id, event_ingredients.ingredient, (date_trunc('day'::text, event_ingredients.start_time));


ALTER TABLE public.shopping_list_per_day OWNER TO postgres;

--
-- Name: steps; Type: TABLE; Schema: public; Owner: kuechenteam
--

CREATE TABLE public.steps (
    step_id integer NOT NULL,
    step_order double precision NOT NULL,
    step_name character varying(255) NOT NULL,
    step_description text NOT NULL,
    fixed_duration interval DEFAULT '00:00:00'::interval NOT NULL,
    duration_per_kg interval DEFAULT '00:00:00'::interval NOT NULL,
    recipe_id integer NOT NULL
);


ALTER TABLE public.steps OWNER TO kuechenteam;

--
-- Name: steps_step_id_seq; Type: SEQUENCE; Schema: public; Owner: kuechenteam
--

ALTER TABLE public.steps ALTER COLUMN step_id ADD GENERATED ALWAYS AS IDENTITY (
    SEQUENCE NAME public.steps_step_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);


--
-- Name: stores; Type: TABLE; Schema: public; Owner: kuechenteam
--

CREATE TABLE public.stores (
    store_id integer NOT NULL,
    name character varying NOT NULL
);


ALTER TABLE public.stores OWNER TO kuechenteam;

--
-- Name: stores_store_id_seq; Type: SEQUENCE; Schema: public; Owner: kuechenteam
--

ALTER TABLE public.stores ALTER COLUMN store_id ADD GENERATED BY DEFAULT AS IDENTITY (
    SEQUENCE NAME public.stores_store_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);


--
-- Name: subrecipes; Type: VIEW; Schema: public; Owner: kuechenteam
--

CREATE VIEW public.subrecipes AS
 SELECT bar.recipe,
    bar.recipe_id,
    bar.ingredient,
    bar.weight,
    bar.subrecipe,
    bar.subrecipe_id,
    bar.is_subrecipe
   FROM (( SELECT rr.recipe_id,
            rr.recipe,
            rr.ingredient,
            sum((rr.weight / recipe_weight.weight)) AS weight,
            rr.subrecipe_id,
            recipes_1.name AS subrecipe,
            false AS is_subrecipe
           FROM ((public.resolved_recipes rr
             JOIN public.recipe_weight USING (recipe_id))
             JOIN public.recipes recipes_1 ON ((rr.subrecipe_id = recipes_1.recipe_id)))
          GROUP BY rr.recipe_id, rr.subrecipe_id, rr.recipe, rr.ingredient_id, rr.ingredient, recipes_1.name
        UNION ALL
         SELECT resolved_meta.recipe_id,
            resolved_meta.recipe,
            resolved_meta.subrecipe AS ingredient,
            sum((resolved_meta.weight / recipe_weight.weight)) AS weight,
            resolved_meta.parent_id,
            resolved_meta.parent,
            true AS is_subrecipe
           FROM (public.resolved_meta
             JOIN public.recipe_weight ON ((recipe_weight.recipe_id = resolved_meta.recipe_id)))
          GROUP BY resolved_meta.recipe_id, resolved_meta.recipe, resolved_meta.subrecipe, resolved_meta.subrecipe_id, resolved_meta.parent_id, resolved_meta.parent) bar
     JOIN public.recipes USING (recipe_id))
  ORDER BY bar.recipe, bar.subrecipe_id, bar.is_subrecipe DESC;


ALTER TABLE public.subrecipes OWNER TO kuechenteam;

--
-- Name: units_unit_id_seq; Type: SEQUENCE; Schema: public; Owner: kuechenteam
--

ALTER TABLE public.units ALTER COLUMN unit_id ADD GENERATED BY DEFAULT AS IDENTITY (
    SEQUENCE NAME public.units_unit_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);


--
-- Name: base_conversions conversion_pk; Type: CONSTRAINT; Schema: public; Owner: kuechenteam
--

ALTER TABLE ONLY public.base_conversions
    ADD CONSTRAINT conversion_pk PRIMARY KEY (from_unit, to_unit);


--
-- Name: event_meals event_meals_pk; Type: CONSTRAINT; Schema: public; Owner: kuechenteam
--

ALTER TABLE ONLY public.event_meals
    ADD CONSTRAINT event_meals_pk PRIMARY KEY (event_id, recipe_id, start_time, place_id);


--
-- Name: events events_event_name_key; Type: CONSTRAINT; Schema: public; Owner: kuechenteam
--

ALTER TABLE ONLY public.events
    ADD CONSTRAINT events_event_name_key UNIQUE (event_name);


--
-- Name: events events_pkey; Type: CONSTRAINT; Schema: public; Owner: kuechenteam
--

ALTER TABLE ONLY public.events
    ADD CONSTRAINT events_pkey PRIMARY KEY (event_id);


--
-- Name: ingredient_properties food_properties_pk; Type: CONSTRAINT; Schema: public; Owner: kuechenteam
--

ALTER TABLE ONLY public.ingredient_properties
    ADD CONSTRAINT food_properties_pk PRIMARY KEY (ingredient_id, property_id);


--
-- Name: food_properties food_property_pk; Type: CONSTRAINT; Schema: public; Owner: kuechenteam
--

ALTER TABLE ONLY public.food_properties
    ADD CONSTRAINT food_property_pk PRIMARY KEY (property_id);


--
-- Name: ingredients ingredient_name_unique; Type: CONSTRAINT; Schema: public; Owner: kuechenteam
--

ALTER TABLE ONLY public.ingredients
    ADD CONSTRAINT ingredient_name_unique UNIQUE (name);


--
-- Name: ingredient_sources ingredient_source_pk; Type: CONSTRAINT; Schema: public; Owner: kuechenteam
--

ALTER TABLE ONLY public.ingredient_sources
    ADD CONSTRAINT ingredient_source_pk PRIMARY KEY (ingredient_id, store_id, package_size, unit_id);


--
-- Name: ingredients ingredients_pk; Type: CONSTRAINT; Schema: public; Owner: kuechenteam
--

ALTER TABLE ONLY public.ingredients
    ADD CONSTRAINT ingredients_pk PRIMARY KEY (ingredient_id);


--
-- Name: places places_pkey; Type: CONSTRAINT; Schema: public; Owner: kuechenteam
--

ALTER TABLE ONLY public.places
    ADD CONSTRAINT places_pkey PRIMARY KEY (place_id);


--
-- Name: recipe_ingredients recipe_ingredients_pk; Type: CONSTRAINT; Schema: public; Owner: kuechenteam
--

ALTER TABLE ONLY public.recipe_ingredients
    ADD CONSTRAINT recipe_ingredients_pk PRIMARY KEY (recipe_id, ingredient_id);


--
-- Name: recipes recipe_name_unique; Type: CONSTRAINT; Schema: public; Owner: kuechenteam
--

ALTER TABLE ONLY public.recipes
    ADD CONSTRAINT recipe_name_unique UNIQUE (name);


--
-- Name: recipes recipes_pk; Type: CONSTRAINT; Schema: public; Owner: kuechenteam
--

ALTER TABLE ONLY public.recipes
    ADD CONSTRAINT recipes_pk PRIMARY KEY (recipe_id);


--
-- Name: steps steps_pkey; Type: CONSTRAINT; Schema: public; Owner: kuechenteam
--

ALTER TABLE ONLY public.steps
    ADD CONSTRAINT steps_pkey PRIMARY KEY (step_id);


--
-- Name: stores store_pk; Type: CONSTRAINT; Schema: public; Owner: kuechenteam
--

ALTER TABLE ONLY public.stores
    ADD CONSTRAINT store_pk PRIMARY KEY (store_id);


--
-- Name: meta_recipes sub_recipe_pk; Type: CONSTRAINT; Schema: public; Owner: kuechenteam
--

ALTER TABLE ONLY public.meta_recipes
    ADD CONSTRAINT sub_recipe_pk PRIMARY KEY (parent_id, child_id);


--
-- Name: weights to_kg_pk; Type: CONSTRAINT; Schema: public; Owner: kuechenteam
--

ALTER TABLE ONLY public.weights
    ADD CONSTRAINT to_kg_pk PRIMARY KEY (unit_id, ingredient_id);


--
-- Name: units units_pk; Type: CONSTRAINT; Schema: public; Owner: kuechenteam
--

ALTER TABLE ONLY public.units
    ADD CONSTRAINT units_pk PRIMARY KEY (unit_id);


--
-- Name: fki_recipe_id_fk; Type: INDEX; Schema: public; Owner: kuechenteam
--

CREATE INDEX fki_recipe_id_fk ON public.steps USING btree (recipe_id);


--
-- Name: meta_recipes child_fk; Type: FK CONSTRAINT; Schema: public; Owner: kuechenteam
--

ALTER TABLE ONLY public.meta_recipes
    ADD CONSTRAINT child_fk FOREIGN KEY (child_id) REFERENCES public.recipes(recipe_id);


--
-- Name: event_meals event_meals_event_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: kuechenteam
--

ALTER TABLE ONLY public.event_meals
    ADD CONSTRAINT event_meals_event_id_fkey FOREIGN KEY (event_id) REFERENCES public.events(event_id);


--
-- Name: event_meals event_meals_place_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: kuechenteam
--

ALTER TABLE ONLY public.event_meals
    ADD CONSTRAINT event_meals_place_id_fkey FOREIGN KEY (place_id) REFERENCES public.places(place_id);


--
-- Name: event_meals event_meals_recipe_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: kuechenteam
--

ALTER TABLE ONLY public.event_meals
    ADD CONSTRAINT event_meals_recipe_id_fkey FOREIGN KEY (recipe_id) REFERENCES public.recipes(recipe_id);


--
-- Name: base_conversions from_unit_fk; Type: FK CONSTRAINT; Schema: public; Owner: kuechenteam
--

ALTER TABLE ONLY public.base_conversions
    ADD CONSTRAINT from_unit_fk FOREIGN KEY (from_unit) REFERENCES public.units(unit_id) MATCH FULL ON UPDATE CASCADE ON DELETE SET NULL;


--
-- Name: ingredient_sources ingredient_fk; Type: FK CONSTRAINT; Schema: public; Owner: kuechenteam
--

ALTER TABLE ONLY public.ingredient_sources
    ADD CONSTRAINT ingredient_fk FOREIGN KEY (ingredient_id) REFERENCES public.ingredients(ingredient_id);


--
-- Name: weights ingredient_fk; Type: FK CONSTRAINT; Schema: public; Owner: kuechenteam
--

ALTER TABLE ONLY public.weights
    ADD CONSTRAINT ingredient_fk FOREIGN KEY (ingredient_id) REFERENCES public.ingredients(ingredient_id) MATCH FULL ON UPDATE CASCADE ON DELETE SET NULL;


--
-- Name: recipe_ingredients ingredient_fk; Type: FK CONSTRAINT; Schema: public; Owner: kuechenteam
--

ALTER TABLE ONLY public.recipe_ingredients
    ADD CONSTRAINT ingredient_fk FOREIGN KEY (ingredient_id) REFERENCES public.ingredients(ingredient_id);


--
-- Name: ingredient_properties ingredient_id_fk; Type: FK CONSTRAINT; Schema: public; Owner: kuechenteam
--

ALTER TABLE ONLY public.ingredient_properties
    ADD CONSTRAINT ingredient_id_fk FOREIGN KEY (ingredient_id) REFERENCES public.ingredients(ingredient_id);


--
-- Name: ingredient_sources package_unit_fk; Type: FK CONSTRAINT; Schema: public; Owner: kuechenteam
--

ALTER TABLE ONLY public.ingredient_sources
    ADD CONSTRAINT package_unit_fk FOREIGN KEY (unit_id) REFERENCES public.units(unit_id);


--
-- Name: meta_recipes parent_fk; Type: FK CONSTRAINT; Schema: public; Owner: kuechenteam
--

ALTER TABLE ONLY public.meta_recipes
    ADD CONSTRAINT parent_fk FOREIGN KEY (parent_id) REFERENCES public.recipes(recipe_id);


--
-- Name: ingredient_properties property_id; Type: FK CONSTRAINT; Schema: public; Owner: kuechenteam
--

ALTER TABLE ONLY public.ingredient_properties
    ADD CONSTRAINT property_id FOREIGN KEY (property_id) REFERENCES public.food_properties(property_id);


--
-- Name: recipe_ingredients recipe_fk; Type: FK CONSTRAINT; Schema: public; Owner: kuechenteam
--

ALTER TABLE ONLY public.recipe_ingredients
    ADD CONSTRAINT recipe_fk FOREIGN KEY (recipe_id) REFERENCES public.recipes(recipe_id);


--
-- Name: steps recipe_id_fk; Type: FK CONSTRAINT; Schema: public; Owner: kuechenteam
--

ALTER TABLE ONLY public.steps
    ADD CONSTRAINT recipe_id_fk FOREIGN KEY (recipe_id) REFERENCES public.recipes(recipe_id) ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE;


--
-- Name: ingredient_sources store_fk; Type: FK CONSTRAINT; Schema: public; Owner: kuechenteam
--

ALTER TABLE ONLY public.ingredient_sources
    ADD CONSTRAINT store_fk FOREIGN KEY (store_id) REFERENCES public.stores(store_id);


--
-- Name: base_conversions to_unit_fk; Type: FK CONSTRAINT; Schema: public; Owner: kuechenteam
--

ALTER TABLE ONLY public.base_conversions
    ADD CONSTRAINT to_unit_fk FOREIGN KEY (to_unit) REFERENCES public.units(unit_id) MATCH FULL ON UPDATE CASCADE ON DELETE SET NULL;


--
-- Name: weights unit_fk; Type: FK CONSTRAINT; Schema: public; Owner: kuechenteam
--

ALTER TABLE ONLY public.weights
    ADD CONSTRAINT unit_fk FOREIGN KEY (unit_id) REFERENCES public.units(unit_id) MATCH FULL ON UPDATE CASCADE ON DELETE SET NULL;


--
-- Name: recipe_ingredients unit_fk; Type: FK CONSTRAINT; Schema: public; Owner: kuechenteam
--

ALTER TABLE ONLY public.recipe_ingredients
    ADD CONSTRAINT unit_fk FOREIGN KEY (unit_id) REFERENCES public.units(unit_id) MATCH FULL ON UPDATE CASCADE ON DELETE RESTRICT;


--
-- Name: SCHEMA public; Type: ACL; Schema: -; Owner: postgres
--

REVOKE USAGE ON SCHEMA public FROM PUBLIC;
GRANT ALL ON SCHEMA public TO PUBLIC;
