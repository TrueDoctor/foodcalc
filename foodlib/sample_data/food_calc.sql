--
-- PostgreSQL database dump
--

-- Dumped from database version 15.4
-- Dumped by pg_dump version 15.4

SET lc_monetary to "de_DE.utf8";
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

ALTER TABLE IF EXISTS ONLY public.recipe_ingredients DROP CONSTRAINT IF EXISTS unit_fk;
ALTER TABLE IF EXISTS ONLY public.weights DROP CONSTRAINT IF EXISTS unit_fk;
ALTER TABLE IF EXISTS ONLY public.base_conversions DROP CONSTRAINT IF EXISTS to_unit_fk;
ALTER TABLE IF EXISTS ONLY public.ingredient_sources DROP CONSTRAINT IF EXISTS store_fk;
ALTER TABLE IF EXISTS ONLY public.steps DROP CONSTRAINT IF EXISTS recipe_id_fk;
ALTER TABLE IF EXISTS ONLY public.recipe_ingredients DROP CONSTRAINT IF EXISTS recipe_fk;
ALTER TABLE IF EXISTS ONLY public.ingredient_properties DROP CONSTRAINT IF EXISTS property_id;
ALTER TABLE IF EXISTS ONLY public.meta_recipes DROP CONSTRAINT IF EXISTS parent_fk;
ALTER TABLE IF EXISTS ONLY public.ingredient_sources DROP CONSTRAINT IF EXISTS package_unit_fk;
ALTER TABLE IF EXISTS ONLY public.ingredient_properties DROP CONSTRAINT IF EXISTS ingredient_id_fk;
ALTER TABLE IF EXISTS ONLY public.recipe_ingredients DROP CONSTRAINT IF EXISTS ingredient_fk;
ALTER TABLE IF EXISTS ONLY public.weights DROP CONSTRAINT IF EXISTS ingredient_fk;
ALTER TABLE IF EXISTS ONLY public.ingredient_sources DROP CONSTRAINT IF EXISTS ingredient_fk;
ALTER TABLE IF EXISTS ONLY public.base_conversions DROP CONSTRAINT IF EXISTS from_unit_fk;
ALTER TABLE IF EXISTS ONLY public.event_meals DROP CONSTRAINT IF EXISTS event_meals_recipe_id_fkey;
ALTER TABLE IF EXISTS ONLY public.event_meals DROP CONSTRAINT IF EXISTS event_meals_place_id_fkey;
ALTER TABLE IF EXISTS ONLY public.event_meals DROP CONSTRAINT IF EXISTS event_meals_event_id_fkey;
ALTER TABLE IF EXISTS ONLY public.meta_recipes DROP CONSTRAINT IF EXISTS child_fk;
DROP INDEX IF EXISTS public.fki_recipe_id_fk;
ALTER TABLE IF EXISTS ONLY public.units DROP CONSTRAINT IF EXISTS units_pk;
ALTER TABLE IF EXISTS ONLY public.weights DROP CONSTRAINT IF EXISTS to_kg_pk;
ALTER TABLE IF EXISTS ONLY public.meta_recipes DROP CONSTRAINT IF EXISTS sub_recipe_pk;
ALTER TABLE IF EXISTS ONLY public.stores DROP CONSTRAINT IF EXISTS store_pk;
ALTER TABLE IF EXISTS ONLY public.steps DROP CONSTRAINT IF EXISTS steps_pkey;
ALTER TABLE IF EXISTS ONLY public.recipes DROP CONSTRAINT IF EXISTS recipes_pk;
ALTER TABLE IF EXISTS ONLY public.recipes DROP CONSTRAINT IF EXISTS recipe_name_unique;
ALTER TABLE IF EXISTS ONLY public.recipe_ingredients DROP CONSTRAINT IF EXISTS recipe_ingredients_pk;
ALTER TABLE IF EXISTS ONLY public.places DROP CONSTRAINT IF EXISTS places_pkey;
ALTER TABLE IF EXISTS ONLY public.ingredients DROP CONSTRAINT IF EXISTS ingredients_pk;
ALTER TABLE IF EXISTS ONLY public.ingredient_sources DROP CONSTRAINT IF EXISTS ingredient_source_pk;
ALTER TABLE IF EXISTS ONLY public.ingredients DROP CONSTRAINT IF EXISTS ingredient_name_unique;
ALTER TABLE IF EXISTS ONLY public.food_properties DROP CONSTRAINT IF EXISTS food_property_pk;
ALTER TABLE IF EXISTS ONLY public.ingredient_properties DROP CONSTRAINT IF EXISTS food_properties_pk;
ALTER TABLE IF EXISTS ONLY public.events DROP CONSTRAINT IF EXISTS events_pkey;
ALTER TABLE IF EXISTS ONLY public.events DROP CONSTRAINT IF EXISTS events_event_name_key;
ALTER TABLE IF EXISTS ONLY public.event_meals DROP CONSTRAINT IF EXISTS event_meals_pk;
ALTER TABLE IF EXISTS ONLY public.base_conversions DROP CONSTRAINT IF EXISTS conversion_pk;
DROP VIEW IF EXISTS public.subrecipes;
DROP TABLE IF EXISTS public.stores;
DROP TABLE IF EXISTS public.steps;
DROP VIEW IF EXISTS public.shopping_list_per_day;
DROP VIEW IF EXISTS public.shopping_list;
DROP VIEW IF EXISTS public.recipe_ingredients_view;
DROP VIEW IF EXISTS public.meta_with_names;
DROP VIEW IF EXISTS public.ingredients_without_weight;
DROP TABLE IF EXISTS public.units;
DROP VIEW IF EXISTS public.ingredients_without_sources;
DROP VIEW IF EXISTS public.ingredient_properties_view;
DROP TABLE IF EXISTS public.ingredient_properties;
DROP TABLE IF EXISTS public.food_properties;
DROP VIEW IF EXISTS public.event_recipes;
DROP VIEW IF EXISTS public.event_ingredients;
DROP VIEW IF EXISTS public.resolved_recipe_ingredients;
DROP VIEW IF EXISTS public.recipe_stats;
DROP VIEW IF EXISTS public.resolved_recipes;
DROP VIEW IF EXISTS public.resolved_meta;
DROP VIEW IF EXISTS public.recipe_weight;
DROP TABLE IF EXISTS public.recipes;
DROP TABLE IF EXISTS public.recipe_ingredients;
DROP TABLE IF EXISTS public.places;
DROP TABLE IF EXISTS public.meta_recipes;
DROP VIEW IF EXISTS public.ingredient_weight;
DROP TABLE IF EXISTS public.weights;
DROP TABLE IF EXISTS public.ingredients;
DROP TABLE IF EXISTS public.ingredient_sources;
DROP TABLE IF EXISTS public.events;
DROP TABLE IF EXISTS public.event_meals;
DROP MATERIALIZED VIEW IF EXISTS public.conversions;
DROP TABLE IF EXISTS public.base_conversions;
-- *not* dropping schema, since initdb creates it
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




--
-- Name: events; Type: TABLE; Schema: public; Owner: kuechenteam
--

CREATE TABLE public.events (
    event_id integer NOT NULL,
    event_name character varying NOT NULL,
    comment text,
    budget money
);




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




--
-- Name: ingredients; Type: TABLE; Schema: public; Owner: kuechenteam
--

CREATE TABLE public.ingredients (
    ingredient_id integer NOT NULL,
    name character varying NOT NULL,
    energy numeric DEFAULT 0 NOT NULL,
    comment text
);




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




--
-- Name: meta_recipes; Type: TABLE; Schema: public; Owner: kuechenteam
--

CREATE TABLE public.meta_recipes (
    parent_id integer NOT NULL,
    child_id integer NOT NULL,
    weight numeric NOT NULL,
    CONSTRAINT recursive CHECK ((child_id <> parent_id))
);




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




--
-- Name: recipe_ingredients; Type: TABLE; Schema: public; Owner: kuechenteam
--

CREATE TABLE public.recipe_ingredients (
    recipe_id integer NOT NULL,
    ingredient_id integer NOT NULL,
    amount numeric NOT NULL,
    unit_id integer NOT NULL
);




--
-- Name: recipes; Type: TABLE; Schema: public; Owner: kuechenteam
--

CREATE TABLE public.recipes (
    recipe_id integer NOT NULL,
    name character varying NOT NULL,
    comment text
);




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




--
-- Name: units; Type: TABLE; Schema: public; Owner: kuechenteam
--

CREATE TABLE public.units (
    unit_id integer NOT NULL,
    name character varying NOT NULL
);




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
-- Data for Name: base_conversions; Type: TABLE DATA; Schema: public; Owner: kuechenteam
--

COPY public.base_conversions (from_unit, to_unit, from_amount, to_amount) FROM stdin;
1	0	1000	1
2	0	1	1
3	8	1	5
4	3	1	3
8	1	1	1
0	0	1	1
\.


--
-- Data for Name: event_meals; Type: TABLE DATA; Schema: public; Owner: kuechenteam
--

COPY public.event_meals (event_id, recipe_id, place_id, comment, energy_per_serving, servings, start_time, end_time) FROM stdin;
16	29	2	\N	1200	120	2022-12-20 19:00:00	2022-12-20 20:00:00
17	93	4	\N	800	8	2022-01-03 19:00:00	2022-01-03 19:30:00
0	75	1	\N	6000	100	2022-06-18 12:30:00	2022-06-18 14:30:00
4	80	4	\N	1500	36	2022-07-02 08:00:00	2022-07-02 10:00:00
4	80	4	\N	1500	36	2022-07-03 08:00:00	2022-07-03 10:00:00
4	61	4	\N	4000	36	2022-07-01 19:00:00	2022-07-01 20:00:00
9	93	8	\N	2400	40	2022-11-01 19:30:00	2022-11-01 21:00:00
4	77	4	\N	4000	36	2022-07-02 13:00:00	2022-07-02 14:00:00
9	29	8		2600	80	2022-11-03 19:30:00	2022-11-03 20:30:00
4	78	4	\N	4500	30	2022-07-02 19:00:00	2022-07-02 23:00:00
0	8	1	\N	6000	120	2022-06-18 18:00:00	2022-06-19 00:00:00
10	80	4	\N	1600	40	2022-11-19 08:00:00	2022-11-19 09:00:00
0	7	1	\N	6000	120	2022-06-17 18:00:00	2022-06-18 00:00:00
10	100	4	\N	2600	40	2022-11-19 19:00:00	2022-11-19 20:00:00
0	38	1	\N	6000	120	2022-06-17 18:00:00	2022-06-18 00:00:00
0	39	1	\N	6000	80	2022-06-16 12:00:00	2022-06-16 14:00:00
5	86	5	\N	3000	60	2022-07-14 21:00:00	2022-07-15 03:00:00
0	51	1	\N	6000	40	2022-06-19 13:00:00	2022-06-19 14:00:00
5	87	5	\N	4000	80	2022-07-14 19:00:00	2022-07-15 03:00:00
0	62	1	\N	6000	200	2022-06-17 18:00:00	2022-06-18 00:00:00
0	63	1	\N	6000	200	2022-06-18 18:00:00	2022-06-19 00:00:00
0	64	1	\N	6000	80	2022-06-17 14:30:00	2022-06-17 18:00:00
0	64	1	\N	6000	80	2022-06-18 14:00:00	2022-06-18 18:00:00
5	88	5	\N	3000	25	2022-07-14 14:00:00	2022-07-14 19:00:00
5	89	5	\N	3000	15	2022-07-14 14:00:00	2022-07-14 19:00:00
0	66	1	\N	500	120	2022-06-18 18:00:00	2022-06-19 00:00:00
0	67	3	\N	6000	40	2022-06-19 13:00:00	2022-06-19 14:00:00
0	67	3	\N	6000	40	2022-06-19 18:00:00	2022-06-19 21:00:00
0	61	1	\N	4000	160	2022-06-18 18:00:00	2022-06-19 00:00:00
0	43	1	\N	3000	60	2022-06-17 08:00:00	2022-06-16 10:00:00
0	43	1	\N	3000	20	2022-06-18 08:00:00	2022-06-17 09:30:00
0	43	1	\N	3000	50	2022-06-19 08:00:00	2022-06-18 09:30:00
0	69	3	\N	6000	80	2022-06-16 16:15:00	2022-06-16 18:45:00
0	69	3	\N	6000	60	2022-06-16 18:45:00	2022-06-16 22:15:00
0	72	1	\N	2000	200	2022-06-17 10:00:00	2022-06-18 00:00:00
0	68	1	\N	6000	100	2022-06-16 13:00:00	2022-06-16 14:00:00
10	29	4	\N	2600	40	2022-11-18 18:30:00	2022-11-18 19:30:00
10	80	4	\N	1600	40	2022-11-20 08:00:00	2022-11-20 09:00:00
10	97	4	\N	2600	40	2022-11-19 12:30:00	2022-11-19 13:30:00
6	91	6	\N	1850	220	2022-08-02 11:30:00	2022-08-02 13:30:00
7	81	7	\N	4000	12	2022-08-06 09:00:00	2022-08-06 20:00:00
3	63	3	\N	0	100	2022-10-10 12:00:00	2022-10-10 13:00:00
8	9	5	\N	0	20	1970-01-01 00:00:00	1970-01-01 02:00:00
17	75	4	\N	2600	8	2022-01-04 20:00:00	2022-01-04 20:00:00
17	55	4	\N	400	8	2022-01-03 20:00:00	2022-01-03 21:00:00
17	8	4	\N	2000	8	2022-01-03 20:00:00	2022-01-03 21:00:00
17	101	4	\N	2800	8	2022-01-05 20:00:00	2022-01-05 21:00:00
17	86	4	\N	2600	8	2022-01-06 20:00:00	2022-01-06 21:00:00
17	7	4	\N	2700	9	2023-01-02 20:00:00	2023-01-02 21:00:00
18	106	2	\N	2400	60	2023-02-20 12:00:00	2023-02-20 18:00:00
20	106	6	\N	2400	12	2023-02-04 12:00:00	2023-02-04 12:00:00
8	29	5	\N	2400	1	2023-03-31 12:00:00	2023-03-31 12:00:00
19	106	3	(Zahlen nicht fix)	4000	120	2023-06-22 12:00:00	2023-06-22 22:00:00
21	139	8	\N	1800	60	2023-04-25 19:30:00	2023-04-25 20:30:00
19	80	1	(zahlen nicht fix, frühstück nicht fix)	2000	30	2023-06-23 08:00:00	2023-06-23 10:00:00
21	108	8	\N	1200	40	2023-04-25 19:30:00	2023-04-25 19:30:00
8	7	7	\N	2400	10	2023-04-26 12:00:00	2023-04-26 14:00:00
22	7	5	\N	2400	10	2023-04-26 12:00:00	2023-04-26 12:00:00
19	67	3	\N	2000	120	2023-06-25 14:00:00	2023-05-15 21:00:00
23	61	5	\N	2400	16	2023-05-12 19:00:00	2023-05-12 18:00:00
19	80	1	\N	2000	30	2023-06-24 08:00:00	2023-06-24 12:00:00
19	68	1	\N	4000	150	2023-06-23 12:00:00	2023-06-15 14:00:00
19	64	1	\N	3000	40	2023-06-23 14:00:00	2023-06-23 19:00:00
29	191	5	Die Vegetarische Alternative zum Grill	2400	330	2023-07-13 18:00:00	2023-05-31 23:00:00
31	227	4	\N	2400	42	2023-07-08 14:00:00	2023-07-08 15:00:00
19	7	1	\N	4000	70	2023-06-24 19:00:00	2023-06-24 22:00:00
19	63	1	\N	4000	80	2023-06-23 19:00:00	2023-06-23 22:00:00
19	186	1	\N	1600	60	2023-06-25 12:00:00	2023-06-25 14:00:00
19	77	1	\N	2800	200	2023-06-24 12:00:00	2023-06-24 14:00:00
19	200	3	\N	1200	30	2023-06-23 17:00:00	2023-06-23 21:00:00
3	139	1	\N	2400	1	2023-05-20 12:00:00	2023-05-20 12:00:00
19	62	1	\N	4000	80	2023-06-24 19:00:00	2023-06-24 22:00:00
19	64	2	\N	3000	50	2023-06-22 18:00:00	2023-06-22 21:00:00
19	64	1	\N	3000	40	2023-06-24 14:00:00	2023-06-24 19:00:00
19	8	1	\N	5000	50	2023-06-24 19:00:00	2023-06-24 22:00:00
19	69	3	(personenzahl nicht fix)	6000	70	2023-06-21 12:00:00	2023-06-21 15:00:00
19	61	2	\N	2400	50	2023-06-21 18:00:00	2023-06-21 21:00:00
19	7	1	\N	4000	70	2023-06-23 19:00:00	2023-06-23 22:00:00
34	230	4	\N	2400	11	2023-07-21 19:00:00	2023-07-21 20:00:00
31	80	4	\N	1800	42	2023-07-09 08:00:00	2023-07-09 09:00:00
30	75	1	\N	2800	100	2023-06-24 12:00:00	2023-06-24 14:00:00
30	8	1	\N	5000	50	2023-06-22 12:00:00	2023-06-22 12:00:00
30	214	1	\N	3500	50	2023-06-23 18:00:00	2023-06-23 22:00:00
30	214	1	\N	3500	50	2023-06-24 18:00:00	2023-06-24 22:00:00
30	8	1	\N	5000	50	2023-06-24 18:00:00	2023-06-22 22:00:00
30	139	1	\N	4000	150	2023-06-23 12:00:00	2023-06-23 14:00:00
19	8	1	\N	5000	50	2023-06-23 19:00:00	2023-06-23 22:00:00
31	100	4	\N	2600	42	2023-07-08 20:00:00	2023-07-08 23:59:00
31	223	4	\N	2400	42	2023-07-07 19:30:00	2023-07-07 21:00:00
31	80	4	\N	1800	42	2023-07-08 08:00:00	2023-07-08 09:00:00
32	77	5	\N	2400	200	2023-07-13 18:00:00	2023-07-08 21:00:00
32	88	8	\N	3000	60	2023-07-13 16:00:00	2023-07-08 20:00:00
32	89	8	\N	2400	20	2023-07-13 14:00:00	2023-07-13 20:00:00
8	200	6	\N	1800	4	2023-07-10 12:00:00	2023-07-10 12:00:00
33	61	8	\N	2400	30	2023-07-14 16:00:00	2023-07-14 18:00:00
34	40	4	\N	2800	11	2023-07-22 19:00:00	2023-07-22 20:00:00
38	69	3	\N	6000	70	2024-06-12 14:00:00	2024-06-12 18:00:00
36	265	1	\N	2600	33	2023-08-10 12:00:00	2023-08-10 12:00:00
36	251	1	\N	1000	33	2023-08-11 22:00:00	2023-08-11 23:00:00
38	61	2	\N	4000	60	2024-06-12 18:00:00	2024-06-12 20:00:00
38	69	3	\N	4000	40	2024-06-13 12:00:00	2023-06-13 15:00:00
38	64	1	\N	3000	150	2024-06-13 18:00:00	2024-06-13 20:00:00
38	106	3	\N	4000	80	2024-06-13 12:00:00	2024-06-13 15:00:00
39	29	1	\N	1800	50	2023-10-08 12:00:00	2023-10-08 12:00:00
39	266	1	\N	800	50	2023-09-09 12:00:00	2023-09-09 12:00:00
40	39	8	\N	1800	60	2023-10-23 12:00:00	2023-10-23 12:00:00
41	227	1	\N	2400	100	2023-10-25 12:00:00	2023-10-25 14:00:00
40	29	8	Brot	1800	60	2023-10-23 12:00:00	2023-10-23 12:00:00
\.


--
-- Data for Name: events; Type: TABLE DATA; Schema: public; Owner: kuechenteam
--

COPY public.events (event_id, event_name, comment, budget) FROM stdin;
4	Seminar SS22	\N	670,00 €
5	Eulenfest SS22	\N	300,00 €
6	Los Ämmerles 2022	\N	\N
7	Filmdreh HFG	\N	60,00 €
3	Dummy	Testcomment	2.000,00 €
9	SAT WS22	1600-1800kJpP + Baguette besser als 2400kJpP	300,00 €
16	Tour de FSK	\N	300,00 €
17	Skiurlaub	\N	200,00 €
18	Backwerkstand	Ausprobieren, wie viel wir in welcher Zeit backen können	100,00 €
0	Unifest22		7.000,00 €
30	Ùnifest 23 Nachkochen	\N	3.000,00 €
20	Pita+Hummus Probebacken	\N	\N
21	SAT SS23	\N	300,00 €
22	Game Jam	\N	60,00 €
10	Seminar WS22/23	\N	550,00 €
23	Squeak Hackathon	\N	100,00 €
31	Seminar SS23	\N	400,00 €
8	Test Event 2	\N	\N
29	Eulenfest SS23 (Externe Fütterung)	Immoment noch kein Inhalt hier	750,00 €
24	Eulenfest SS23 (Externe Fütterung	\N	\N
33	VS Wahlen	\N	50,00 €
32	Eulenfest 2023	\N	500,00 €
34	Minifreizeit	\N	100,00 €
19	Unifest 23	IST SCHON NÄCHSTE WOCHE PANIK!!!	8.500,00 €
36	Gasque	\N	\N
37	Unifest 24	\N	\N
38	Unifest SS24	\N	8.500,00 €
39	Lennart Luisa 50.	\N	80,00 €
41	Kob Naturfreunde Helferfest	\N	\N
40	SAT WS 23/24	\N	\N
\.


--
-- Data for Name: food_properties; Type: TABLE DATA; Schema: public; Owner: kuechenteam
--

COPY public.food_properties (property_id, name) FROM stdin;
2	tierische produkte
3	gluten
4	fleisch
5	fisch
6	schwein
7	krebstiere
8	eier
9	erdnüsse
10	soja
11	schalenfrüchte
12	sellerie
13	senf
15	schwefeldioxid & sulfite
16	lupine
17	weichtiere
14	sesamsamen
18	cashewnüsse
19	dinkel
20	gerste
21	hafer
22	haselnüsse
23	kamut
24	mandel
25	paranüsse
26	pekannüsse
27	pistazie
28	macadamiannüsse / queenslandnüsse
29	roggen
30	walnüsse
31	weizen
32	lab
33	gelatine
34	farbstoff
35	konservierungsstoff
36	antioxidationsmittel
37	geschmacksverstärker
38	geschwefelt
39	geschwärzt
40	phosphat
41	milcheiweiß
42	koffeinhaltig
43	chininhaltig
44	süßungsmittel
45	gewachst
48	gelatine
1	Milch
\.


--
-- Data for Name: ingredient_properties; Type: TABLE DATA; Schema: public; Owner: kuechenteam
--

COPY public.ingredient_properties (ingredient_id, property_id) FROM stdin;
1	3
1	31
2	8
9	1
15	1
16	1
18	1
19	1
22	1
27	1
32	1
39	1
41	1
42	1
54	1
69	1
62	1
71	1
9	32
10	12
14	3
14	13
14	31
15	41
\.


--
-- Data for Name: ingredient_sources; Type: TABLE DATA; Schema: public; Owner: kuechenteam
--

COPY public.ingredient_sources (ingredient_id, store_id, package_size, unit_id, price, url, comment) FROM stdin;
246	2	1	0	6,00 €	https://www.ikea.com/de/de/food/salesareas/swedish-food-market/07837723-cf51-45d1-bb48-67ebcf723b97/	\N
42	0	1	0	19,25 €	https://produkte.metro.de/shop/pv/BTY-X72206/0032/0021/Greco-Feta-48-Fett-i.-Tr.-ca.-2-kg-Blöcke	\N
44	0	1.086	0	15,83 €	https://produkte.metro.de/shop/pv/BTY-X397613/0032/0021/Artysan-Chorizo-BBQ-Pikant-1-kg-Packung	\N
21	0	7	0	17,11 €	https://produkte.metro.de/shop/pv/BTY-Z42/0032/0021/Tomaten-6kg	\N
45	0	2.005	0	8,55 €	https://produkte.metro.de/shop/pv/BTY-X329202/0032/0021/METRO-Chef-Risotto-Reis-2-00-kg-Beutel	\N
47	0	2.518	0	6,09 €	https://produkte.metro.de/shop/pv/BTY-X293560/0032/0021/METRO-Chef-Erbsen-fein-tiefgefroren-2-5-kg-Beutel	\N
10	0	1.01	0	1,82 €	https://produkte.metro.de/shop/pv/BTY-X315699/0032/0021/METRO-Chef-Suppengrün-tiefgefroren-1-kg-Beutel	\N
12	0	0.88	0	2,77 €	https://produkte.metro.de/shop/pv/BTY-X311483/0032/0021/METRO-Chef-Tomatenmark-2-fach-konzentriert-800-g-Dose	\N
58	0	1	0	12,83 €	https://produkte.metro.de/shop/pv/BTY-X349273/0032/0021/aro-Delikatess-Kochhinterschinken-ca.-2-5-kg	\N
59	0	3.386	0	55,15 €	https://produkte.metro.de/shop/pv/BTY-X173258/0032/0021/Henkelmann-Salami-1A-500-g-Packung	\N
13	0	1.16	0	3,99 €	https://produkte.metro.de/shop/pv/BTY-X569844/0032/0021/Ribeaupierre-Merlot-Rotwein-VDP-0-75-l-Flasche	\N
96	0	10.48	0	19,25 €	https://produkte.metro.de/shop/pv/BTY-Z133/0032/0021/Äpfel-Jonagold-10kg	\N
15	0	1.08	0	1,09 €	https://produkte.metro.de/shop/pv/BTY-X702948/0032/0021/aro-frische-Vollmilch-3-5-Fett-1-l-Packung	\N
183	0	3.45	0	16,57 €	https://produkte.metro.de/shop/pv/BTY-Z1218/0032/0021/Cocktailrispentomaten-3kg	\N
16	0	2.019	0	16,03 €	https://produkte.metro.de/shop/pv/BTY-X172241/0032/0021/aro-Mozzarella-gerieben-45-Fett-i.-Tr.-2-kg-Beutel	\N
17	0	0.531	0	2,56 €	https://produkte.metro.de/shop/pv/BTY-X840874/0032/0021/Barilla-Collezione-Lasagne-Italien-500-g-Packung	\N
22	0	5.21	0	26,16 €	https://produkte.metro.de/shop/pv/BTY-X311862/0032/0021/aro-QS-Crème-fraîche-38-Fett-5-00-kg-Eimer	\N
24	0	1.16	0	16,04 €	https://produkte.metro.de/shop/pv/BTY-Z1083/0032/0021/Basilikum-1kg	\N
26	0	1.12	0	18,71 €	https://produkte.metro.de/shop/pv/BTY-X341341/0032/0021/METRO-Chef-Pfeffer-schwarz-gemahlen-1-x-1-1-kg-Beutel	\N
212	0	3.012	0	8,98 €	https://produkte.metro.de/shop/pv/BTY-X29425/0032/0021/METRO-Chef-Klossteig-gek%C3%BChlt-3-kg	\N
218	0	7.1	0	27,35 €	https://produkte.metro.de/shop/pv/BTY-X531079/0032/0022/METRO-Chef-Ciabatta-XXL-500-g	\N
27	0	5.18	0	20,60 €	https://produkte.metro.de/shop/pv/BTY-X303665/0032/0021/aro-Frische-Sahne-33-Fett-5-kg-Eimer	\N
191	0	0.531	0	6,41 €	https://produkte.metro.de/shop/pv/BTY-X381408/0032/0022/METRO-Chef-Edelnussmischung-10-x-500-g-Karton	\N
192	0	4.8	0	38,52 €	https://produkte.metro.de/shop/pv/BTY-X170675/0032/0021/Horeca-Select-Waffelröllchen-200er-Karton	\N
28	0	305	0	224,38 €	https://produkte.metro.de/shop/pv/BTY-X5970/0032/0021/Speisekartoffeln-mehlig-kochend-übergroß-Deutschland-10-kg-Sack	\N
30	0	1	0	4,98 €	https://produkte.metro.de/shop/pv/BTY-X303333/0032/0021/aro-Gouda-48-Fett-i.-Tr.-ca.-15-kg-Block	\N
31	0	2.514	0	4,80 €	https://produkte.metro.de/shop/pv/BTY-X293497/0032/0021/METRO-Chef-Kaisergemüse-tiefgefroren-erntefrisch-2-5-kg-Beutel	\N
33	0	1.05	0	6,20 €	https://produkte.metro.de/shop/pv/BTY-Z2129/0032/0021/Champignon-braun-1kg	\N
34	0	9.9	0	27,22 €	https://produkte.metro.de/shop/pv/BTY-Z2214/0032/0021/Paprika-Mix-16x500g	\N
35	0	5.8	0	8,01 €	https://produkte.metro.de/shop/pv/BTY-Z53/0032/0021/Zucchini-5kg	\N
39	0	2.6	0	13,39 €	https://produkte.metro.de/shop/pv/BTY-X377589/0032/0023/	\N
20	0	1.05	0	3,95 €	https://produkte.metro.de/shop/pv/BTY-Z1144/0032/0021/Knoblauch-weiss-1kg	\N
43	0	1	0	7,27 €	https://produkte.metro.de/shop/pv/BTY-Z143/0032/0021/Hähnchenbrustfilet-ca.-2-5kg-vak.-verpackt-unkalibriert	\N
46	0	1.233	0	3,99 €	https://produkte.metro.de/shop/pv/BTY-X436629/0032/0021/Leoff-Riesling-Weißwein-QBA-Qualitätswein-0-75-l-Flasche	\N
50	0	4.228	0	13,23 €	https://produkte.metro.de/shop/pv/BTY-X505381/0032/0021/Märsch-Sultaninen-ungeschwefelt-1-kg-Beutel	\N
53	0	0.572	0	4,27 €	https://produkte.metro.de/shop/pv/BTY-X341358/0032/0021/METRO-Chef-Paprika-edelsüß-1-x-500-g-Dose	\N
54	0	5.21	0	16,04 €	https://produkte.metro.de/shop/pv/BTY-X311869/0032/0021/aro-Speisequark-40-Fett-5-kg-Eimer	\N
57	0	10.6	0	64,15 €	https://produkte.metro.de/shop/pv/BTY-X354089/0032/0021/METRO-Chef-Röstzwiebeln-2-kg-Beutel	\N
60	0	5.2	0	21,35 €	https://produkte.metro.de/shop/pv/BTY-X311863/0032/0021/aro-Schmand-24-Fett-5-kg-Eimer	\N
197	0	1.009	0	1,51 €	https://produkte.metro.de/shop/pv/BTY-X954836/0032/0021/K%C3%BCchenmeister-Dinkelmehl-Type-630-1-kg-Packung	\N
198	0	0.025	0	0,89 €	https://produkte.metro.de/shop/pv/BTY-X45379/0032/0021/Ruf-Trockenbackhefe-3-St%C3%BCck-%C3%A1-7-g-21-g-Packung	\N
199	0	10.32	0	29,85 €	https://produkte.metro.de/shop/pv/BTY-X237193/0032/0021/Tate-Lyle-Brauner-Rohrzucker-1-00-kg-Packung	\N
200	0	3.175	0	11,36 €	https://produkte.metro.de/shop/pv/BTY-X77586/0032/0021/Alpro-Soja-Joghurt-Natur-500-g-Becher	\N
201	0	7.002	0	29,69 €	https://produkte.metro.de/shop/pv/BTY-X172244/0032/0021/aro-Mozzarella-Multipack-6-St%C3%BCck-%C3%A0-125-g-45-Fett-750-g-Packung	\N
202	0	12.53	0	41,73 €	https://produkte.metro.de/shop/pv/BTY-X871254/0032/0021/Genuport-Trade-Wei%C3%9Fweinessig-Ponti-Aceto-di-Vino-Bianco-Italien-1-00-l-Flasche	\N
211	0	0.55	0	1,87 €	https://produkte.metro.de/shop/pv/BTY-Z57/0032/0021/Broccoli-500g	\N
203	0	0.2	0	4,06 €	https://produkte.metro.de/shop/pv/BTY-Z1043/0032/0021/Schnittlauch-200g	\N
8	0	2.513	0	4,79 €	https://produkte.metro.de/shop/pv/BTY-X293559/0032/0021/METRO-Chef-Zwiebel-Würfel-10-x-10-mm-tiefgefroren-2-5-kg-Beutel	\N
77	0	2.532	0	3,41 €	https://produkte.metro.de/shop/pv/BTY-X293499/0032/0021/METRO-Chef-Karotten-Würfel-tiefgefroren-2-5-kg-Beutel	\N
18	0	10.28	0	59,49 €	https://produkte.metro.de/shop/pv/BTY-X314169/0032/0021/aro-Butter-mild-ges%C3%A4uert-82-Fett-250-g-St%C3%BCck	\N
23	0	9.8	0	51,30 €	https://produkte.metro.de/shop/pv/BTY-X725471/0032/0021/Culinaria-Balsamico-Condimento-Bianco-1-l-Flasche	\N
37	0	2.8	0	32,04 €	https://produkte.metro.de/shop/pv/BTY-X341603/0032/0021/METRO-Chef-Thymian-gerebelt-1-x-440-g-Beutel	\N
49	0	65.844	0	263,22 €	https://produkte.metro.de/shop/pv/BTY-X361492/0032/0021/Rama-Pflanzenmargarine-Der-Klassiker-60-Fett-500-g-Becher	\N
51	0	1.013	0	9,08 €	https://produkte.metro.de/shop/pv/BTY-X615331/0032/0021/Fuchs-Currypulver-Goldelefant-1-x-1-kg-Beutel	\N
118	0	10.13	0	20,32 €	https://produkte.metro.de/shop/pv/BTY-X182282/0032/0021/Südzucker-Puderzucker-10-00-kg	\N
120	0	1	0	4,75 €	https://produkte.metro.de/shop/pv/BTY-X301063/0032/0021/aro-Edamer-40-Fett-i.-Tr.-ca.-3-kg-Block	\N
61	0	0.253	0	3,58 €	https://produkte.metro.de/shop/pv/BTY-X382025/0034/0023/Meggle-Kräuterbutter-62-Fett-250-g-Packung	\N
123	0	5.18	0	8,01 €	https://produkte.metro.de/shop/pv/BTY-X303670/0032/0021/aro-Joghurt-natur-3-5-Fett-im-Milchanteil-5-00-kg-Eimer	\N
78	0	1	0	9,08 €	https://produkte.metro.de/shop/pv/BTY-X91855/0032/0021/aro-Bratwurst-fein-gebrüht-gekühlt-30-Stück-à-ca.-120-g-ca.3-6-kg-Packung	\N
63	0	2.532	0	4,48 €	https://produkte.metro.de/shop/pv/BTY-X293569/0032/0021/METRO-Chef-Spinat-gehackt-tiefgefroren-2-5-kg-Beutel	\N
136	0	6.8	0	42,79 €	https://produkte.metro.de/shop/pv/BTY-X380810/0033/0021/Niklas-Tomaten-getrocknet-Tunesien-1-kg-Schachtel	\N
79	0	4.908	0	17,14 €	https://produkte.metro.de/shop/pv/BTY-X313262/0032/0022/aro-Sossenbinder-hell-18-x-250-g-Karton	\N
66	0	2.532	0	4,44 €	https://produkte.metro.de/shop/pv/BTY-X293557/0032/0021/METRO-Chef-Porree-Scheiben-tiefgefroren-2-5-kg-Beutel	\N
140	0	1.02	0	2,02 €	https://produkte.metro.de/shop/pv/BTY-Z70/0032/0021/Zitronen-1kg	\N
185	0	0.146	0	1,19 €	https://produkte.metro.de/shop/pv/BTY-X746226/0032/0022/aro-Butterkekse-mit-Vollmilchschokolade-16-x-125-g-Packungen	\N
71	0	1.519	0	9,08 €	https://produkte.metro.de/shop/pv/BTY-X10340/0032/0021/Culinaria-Dijon-Senf-extra-fein-1-00-kg	\N
72	0	2.013	0	4,91 €	https://produkte.metro.de/shop/pv/BTY-X216514/0032/0021/Ardo-Makkaroni-tiefgefroren-2-00-kg-Beutel	\N
186	0	0.419	0	1,89 €	https://produkte.metro.de/shop/pv/BTY-X94281/0032/0022/aro-Butterkeks-2-Stück-à-200-g-20-x-400-g-Packungen	\N
83	0	1.008	0	10,69 €	https://produkte.metro.de/shop/pv/BTY-X409374/0032/0021/Staudensellerie-Streifen-4-mm-küchenfertig-1-kg-Beutel	\N
84	0	1	0	7,69 €	https://produkte.metro.de/shop/pv/BTY-X724999/0032/0021/Schweineschäufele-je-kg	\N
86	0	5	0	7,48 €	https://produkte.metro.de/shop/pv/BTY-Z51/0032/0021/Gurken-Kiste-mind.-4-2kg	\N
87	0	1.015	0	19,25 €	https://produkte.metro.de/shop/pv/BTY-X615282/0032/0021/Fuchs-Knoblauchpulver-1-kg-Beutel	\N
88	0	2.623	0	9,86 €	https://produkte.metro.de/shop/pv/BTY-X293503/0032/0021/METRO-Chef-Maiskolben-halbiert-tiefgefroren-2-5-kg-Beutel	\N
89	0	1.02	0	3,20 €	https://produkte.metro.de/shop/pv/BTY-Z1103/0032/0021/Limetten-1kg	\N
90	0	1.008	0	35,83 €	https://produkte.metro.de/shop/pv/BTY-X341490/0032/0021/METRO-Chef-Bag-Muskatnuss-gemahlen-1-x-1-kg-Beutel	\N
116	0	10.615	0	106,96 €	https://produkte.metro.de/shop/pv/BTY-X345164/0032/0021/METRO-Chef-Butterschmalz-99-8-Fett-2-5-kg-Packung	\N
187	0	0.255	0	0,68 €	https://produkte.metro.de/shop/pv/BTY-X369282/0032/0022/aro-Salzstangen-28-x-250-g-Packungen	\N
188	0	0.102	0	1,23 €	https://produkte.metro.de/shop/pv/BTY-X220207/0037/0022/Tuc-Original-24-x-100-g-Tüten	\N
190	0	2.102	0	25,68 €	https://produkte.metro.de/shop/pv/BTY-X383744/0032/0022/Khao-Shong-Erdnüsse-mit-Wasabi-12-x-140-g-Karton	\N
204	0	0.276	0	4,59 €	https://produkte.metro.de/shop/pv/BTY-X690216/0032/0022/	\N
29	0	5.265	0	39,20 €	https://produkte.metro.de/shop/pv/BTY-X321844/0032/0021/	\N
4	-1	1	0	0,00 €	\N	\N
159	-1	9	0	49,75 €	https://www.amazon.de/vegetarisch-glutenfrei-Trockensoja-Vantastic-Foods/dp/B00UOXMUOM/ref=asc_df_B00UOXMUOM/?tag=googshopde-21&linkCode=df0&hvadid=380262930641&hvpos=&hvnetw=g&hvrand=6717087554966365523&hvpone=&hvptwo=&hvqmt=&hvdev=c&hvdvcmdl=&hvlocint=&hvlocphy=9041873&hvtargid=pla-835788141551&psc=1&th=1&psc=1&tag=&ref=&adgrpid=74384287821&hvpone=&hvptwo=&hvadid=380262930641&hvpos=&hvnetw=g&hvrand=6717087554966365523&hvqmt=&hvdev=c&hvdvcmdl=&hvlocint=&hvlocphy=9041873&hvtargid=pla-835788141551	\N
25	-1	1	0	11,96 €	https://www.amazon.de/FU-CAYENNE-PFEFFER-GEM-1000G/dp/B00JWQHFLU/ref=asc_df_B00JWQHFLU/?tag=googshopde-21&linkCode=df0&hvadid=447459217138&hvpos=&hvnetw=g&hvrand=11274143342335503808&hvpone=&hvptwo=&hvqmt=&hvdev=c&hvdvcmdl=&hvlocint=&hvlocphy=9041873&hvtargid=pla-925953624878&th=1	\N
161	-1	1	0	9,39 €	https://www.so-schmeckts.de/chilipulver-gewuerzzubereitung-fuchs-professional?gclid=CjwKCAjws8yUBhA1EiwAi_tpEWEKaPFdPmtcigxCBK3jdUmvRvQsX_muvasbGnXycbJUL0Hux2y0dBoC3ToQAvD_BwE	\N
135	-1	1	0	0,00 €	\N	\N
160	-1	1	0	1,20 €	Köber-Mühle Remchingen	\N
175	-1	6	0	42,40 €	https://www.amazon.de/-/en/dp/B082RHTK25/ref=sr_1_7?keywords=sojaschnetzel&qid=1653837891&s=grocery&sprefix=sojaschn%2Cgrocery%2C90&sr=1-7	\N
48	0	8.898	0	21,47 €	https://produkte.metro.de/shop/pv/BTY-X367582/0032/0021/aro-Aprikosen-2650-ml-1-Dose	\N
3	0	15.28	0	32,07 €	https://produkte.metro.de/shop/pv/BTY-X329222/0032/0021/METRO-Chef-Langkorn-Parboiled-Reis-1-x-5-kg-Beutel	\N
173	0	5.5	0	8,55 €	https://produkte.metro.de/shop/pv/BTY-Z54/0032/0021/Auberginen-5kg	\N
38	0	2.8	0	27,54 €	https://produkte.metro.de/shop/pv/BTY-X5710/0032/0021/Fuchs-Rosmarin-500-g-Beutel	\N
176	0	5.1	0	6,94 €	https://produkte.metro.de/shop/pv/BTY-Z87/0032/0021/Zwiebel-rot-5kg	\N
177	0	3.81	0	9,61 €	https://produkte.metro.de/shop/pv/BTY-Z62/0032/0021/Salat-mix-Kiste	\N
36	0	0.518	0	9,08 €	https://produkte.metro.de/shop/pv/BTY-X917878/0032/0021/Fuchs-Oregano-gerebelt-1-x-500-g-Beutel	\N
178	0	0.505	0	1,74 €	https://produkte.metro.de/shop/pv/BTY-X125444/0032/0021/GAB-Fladenbrot-Pide-500-g-Beutel	\N
40	0	0.51	0	2,56 €	https://produkte.metro.de/shop/pv/BTY-Z2099/0032/0021/Blattspinat-500g	\N
64	0	3.137	0	32,04 €	https://produkte.metro.de/shop/pv/BTY-X67250/0032/0021/Friessinger-Mühle-Trockenbackhefe-500-g	\N
65	0	1.1	0	17,97 €	https://produkte.metro.de/shop/pv/BTY-X1447/0032/0021/METRO-Chef-Liebstöckel-Deutschland-100-g	\N
67	0	0.35	0	3,20 €	https://produkte.metro.de/shop/pv/BTY-Z1202/0032/0021/Petersilie-kraus-300g	\N
68	0	1.14	0	2,25 €	https://produkte.metro.de/shop/pv/BTY-X491355/0033/0021/RIOBA-Zitronensaft-0-75-l-Flasche	\N
55	0	5.21	0	17,11 €	https://produkte.metro.de/shop/pv/BTY-X322462/0032/0021/	\N
74	0	7.42	0	31,89 €	https://produkte.metro.de/shop/pv/BTY-X588564/0032/0021/aro-Blütenhonig-flüssig-500-g-Glas	\N
125	0	14.04	0	25,57 €	https://produkte.metro.de/shop/pv/BTY-X380028/0034/0021/Radieschen-Niederlande-1-kg-Beutel	\N
129	0	1	0	11,76 €	https://produkte.metro.de/shop/pv/BTY-X3509/0032/0021/Salakis-Schafskäse-48-Fett-ca.-2-kg-Packung	\N
69	0	0.35	0	2,66 €	https://produkte.metro.de/shop/pv/BTY-X458215/0032/0021/GAB-Buttertoast-verzehrfertig-geschnitten-Buttertoastbrot-mit-2-8-Fett-500-g-Beutel	\N
130	0	1.6	0	13,90 €	https://produkte.metro.de/shop/pv/BTY-Z2134/0032/0021/Koriander-1kg	\N
133	0	1.063	0	42,79 €	https://produkte.metro.de/shop/pv/BTY-X431300/0032/0021/aro-Pinienkerne-China-1-kg-Beutel	\N
134	0	1	0	11,44 €	https://produkte.metro.de/shop/pv/BTY-X293071/0035/0021/Rinder-Gulasch-aus-der-Keule-3-x-3-cm-vak.-verpackt-3-x-3-kg-9-kg-auf-Vorbestellung	\N
142	0	3	0	3,95 €	https://produkte.metro.de/shop/pv/BTY-X300784/0032/0021/aro-Kidneybohnen-Rot-2-5-kg-Dose	\N
146	0	1.163	0	7,16 €	https://produkte.metro.de/shop/pv/BTY-X409014/0032/0021/Kürbis-Hokkaido-Würfel-15-x-15-mm-1-kg-Beutel	\N
147	0	5.483	0	10,69 €	https://produkte.metro.de/shop/pv/BTY-X287434/0032/0021/aro-Semmelbrösel-aus-Weissbrot-5-kg-Sack	\N
148	0	5.026	0	19,04 €	https://produkte.metro.de/shop/pv/BTY-X917025/0032/0021/Müller´s-Mühle-Linsen-6-7-mm-erlesen-Qualität-5-kg-Sack	\N
149	0	2.983	0	4,16 €	https://produkte.metro.de/shop/pv/BTY-X300781/0032/0021/aro-Kichererbsen-in-Lake-2-5-kg-Dose	\N
151	0	8.788	0	32,68 €	https://produkte.metro.de/shop/pv/BTY-X904076/0032/0021/Nutella-Nuss-Nougat-Creme-cremig-1-00-kg-Tiegel	\N
154	0	4.8	0	7,59 €	https://produkte.metro.de/shop/pv/BTY-X414960/0032/0021/METRO-Chef-Bio-Zwetschken-tiefgefroren-1-5-kg-Packung	\N
168	0	2.53	0	11,00 €	https://produkte.metro.de/shop/pv/BTY-X117075/0032/0021/Burgis-Eierspätzle-frisch-pfannenfertig-2-5-kg-Packung	\N
169	0	0.525	0	0,79 €	https://produkte.metro.de/shop/pv/BTY-X442778/0032/0022/aro-Haferflocken-zart-20-x-500-g-Packungen	\N
171	0	4.23	0	16,36 €	https://produkte.metro.de/shop/pv/BTY-X733507/0032/0021/Edna-Korn-Spezialbrot-tiefgefroren-fertig-gebacken-8-Stück-à-500-g-4-kg-Karton	\N
172	0	5.82	0	14,25 €	https://produkte.metro.de/shop/pv/BTY-X393215/0032/0022/aro-Baked-Beans-12-x-400-g-Tray	\N
179	0	2.514	0	7,37 €	https://produkte.metro.de/shop/pv/BTY-X300394/0032/0021/METRO-Chef-Paprika-Mix-tiefgefroren-2-50-kg-Beutel	\N
184	0	0.038	0	0,68 €	https://produkte.metro.de/shop/pv/BTY-X950305/0033/0022/Mars-Balisto-Müsli-Mix-Vollkornkeks-(36-)-Milchschokolade-(39-)-Haselnusscremegeschmack-Rosinen-20-Stück-à-37-g-20-x-37-g-Riegel	\N
194	0	5.1	0	5,34 €	https://produkte.metro.de/shop/pv/BTY-Z1274/0032/0021/Zwiebeln-5kg	\N
127	0	5.015	0	5,35 €	https://produkte.metro.de/shop/pv/BTY-X35881/0032/0021/Sabarot-Couscous-grob-5-kg-Beutel	\N
174	0	11	0	67,38 €	https://produkte.metro.de/shop/pv/BTY-X187585/0032/0021/Vergina-Kalamata-Oliven-in-Salzlake-mit-Stein-schwarz-3-3-l-Kanister	\N
6	0	10.48	0	33,16 €	https://produkte.metro.de/shop/pv/BTY-X131381/0032/0021/Schell-Sonnenblumenöl-10-l-Kanister	\N
14	0	0.946	0	12,29 €	https://produkte.metro.de/shop/pv/BTY-X962780/0032/0021/Maggi-Klare-Gemüsebrühe-900-g-Packung	\N
104	0	0.7	0	2,66 €	https://produkte.metro.de/shop/pv/BTY-X51975/0032/0021/Wawi-Blockschokolade-48-Kakao-200-g	\N
111	0	1.533	0	0,56 €	https://produkte.metro.de/shop/pv/BTY-X351949/0032/0021/aro-Orangensaft-100-Fruchtgehalt-Pfandfrei-1-x-1-5-l-Packung	\N
52	0	1	0	10,26 €	https://produkte.metro.de/shop/pv/BTY-X83646/0032/0021/aro-Gelderländer-Bauchspeck-gewürfelt-gepökelt-mild-geräuchert-ca.-1-2-kg-Packung	\N
70	0	0.523	0	8,55 €	https://produkte.metro.de/shop/pv/BTY-X352342/0032/0021/METRO-Chef-Delikatess-Kochhinterschinken-500-g-Packung	\N
56	0	1.067	0	4,80 €	https://produkte.metro.de/shop/pv/BTY-X313261/0032/0021/aro-Backpulver-1-00-kg-Packung	\N
73	0	12.512	0	19,90 €	https://produkte.metro.de/shop/pv/BTY-X310022/0032/0021/aro-Passierte-Tomaten-1-l-Packung	\N
76	0	2.52	0	11,22 €	https://produkte.metro.de/shop/pv/BTY-Z66/0032/0021/Lauchzwiebel-14er	\N
80	0	0.909	0	1,98 €	https://produkte.metro.de/shop/pv/BTY-X999445/0032/0021/aro-Ananas-leicht-gezuckert-in-Scheiben-850-ml-Dose	\N
81	0	0.266	0	1,79 €	https://produkte.metro.de/shop/pv/BTY-X537898/0033/0021/aro-Schmelzkäse-Holländer-Scheiben-10-Scheiben-à-25-g-45-Fett-24-x-250-g-Packungen	\N
196	0	0.14	0	2,35 €	https://produkte.metro.de/shop/pv/BTY-X350350/0032/0021/Sanotact-Bierhefe-Flocken-100-g-Schachtel	\N
82	0	3.67	0	80,14 €	https://produkte.metro.de/shop/pv/BTY-X959862/0032/0021/Fuchs-Lorbeerblätter-ganze-Blätter-schonend-veredelt-und-getrocknet-250-g-Beutel	\N
91	0	1.028	0	14,97 €	https://produkte.metro.de/shop/pv/BTY-X821219/0032/0021/Frischpack-Bergjausen-Käse-50-Fett-1-kg-Packung	\N
95	0	2.5	0	9,61 €	https://produkte.metro.de/shop/pv/BTY-Z46/0032/0021/Rucola-Salat-1kg	\N
119	0	10.99	0	11,22 €	https://produkte.metro.de/shop/pv/BTY-X288593/0032/0021/METRO-Chef-Gewürzgurken-knackig-würzig-55-60-Stück-10-2-l-Dose	\N
121	0	0.658	0	3,79 €	https://produkte.metro.de/shop/pv/BTY-X75831/0032/0021/aro-Schinkenfleischwurst-im-Ring-vak.-verpackt-650-g-Packung	\N
124	0	0.513	0	0,59 €	https://produkte.metro.de/shop/pv/BTY-X352049/0032/0021/aro-Reine-Buttermilch-1-x-500-g-Stück	\N
85	0	1	0	27,81 €	https://produkte.metro.de/shop/pv/BTY-X539706/0032/0021/Rinderfilet-vorgereift-vak.-verpackt-je-kg	\N
126	0	0.3	0	2,64 €	https://produkte.metro.de/shop/pv/BTY-X321844/0032/0021/METRO-Chef-8-Kräuter-tiefgefroren-300-g-Beutel	\N
132	0	0.563	0	9,08 €	https://produkte.metro.de/shop/pv/BTY-X9878/0032/0021/Ubena-Kümmel-Ganz-500-g	\N
128	0	1.002	0	2,99 €	https://produkte.metro.de/shop/pv/BTY-X371047/0032/0021/Frießinger-Mühle-Duru-Bulgur-mittelgrob-Weizengrütze-1-x-1-kg-Packung	\N
137	0	1	0	3,20 €	https://produkte.metro.de/shop/pv/BTY-X276135/0032/0021/METRO-Chef-Kokosmilch-17-Fett-ungesüßt-extra-cremig-1-l-Packung	\N
138	0	0.491	0	4,48 €	https://produkte.metro.de/shop/pv/BTY-X303016/0032/0021/Kikkoman-Sojasoße-glutenfrei-250-ml-Flasche	\N
143	0	2.5	0	7,48 €	https://produkte.metro.de/shop/pv/BTY-X931838/0032/0021/Bonduelle-Gemüsemais-zart-jung-2-65-l-Dose	\N
131	0	0.563	0	16,04 €	https://produkte.metro.de/shop/pv/BTY-X170128/0032/0021/Ubena-Cumin-(Kreuzkümmel)-gemahlen-500-g-Dose	\N
150	0	0.476	0	6,94 €	https://produkte.metro.de/shop/pv/BTY-X409800/0032/0021/450G-INGWER-PUEREE	\N
152	0	4.725	0	8,01 €	https://produkte.metro.de/shop/pv/BTY-X301716/0032/0021/aro-Apfelmus-4-25-l-Dose	\N
155	0	0.01	0	1,77 €	https://produkte.metro.de/shop/pv/BTY-X287653/0032/0021/Borggreve-Gewürz-Spekulatius-500-g-Beutel	\N
139	0	0.45	0	3,95 €	https://produkte.metro.de/shop/pv/BTY-X461033/0032/0021/Cock-Currypaste-Rot-1-x-400-g-Becher	\N
162	0	0.383	0	2,88 €	https://produkte.metro.de/shop/pv/BTY-X5600/0032/0021/Berief-Bio-Tofu-geräuchert-2-x-175-g-Packung	\N
165	0	1	0	1,49 €	https://produkte.metro.de/shop/pv/BTY-Z1242/0032/0021/Sellerie-Stück	\N
167	0	1	0	7,49 €	https://produkte.metro.de/shop/pv/BTY-X539038/0032/0021/aro-Wiener-Würstchen-gekühlt-20-Stück-a-50g-ca.-1-kg-Packung	\N
170	0	1.062	0	2,49 €	https://produkte.metro.de/shop/pv/BTY-X389189/0032/0021/Alpro-Barista-Hafermilch-1-l	\N
144	0	1	0	12,83 €	https://produkte.metro.de/shop/pv/BTY-X380234/0032/0021/Chili-Mix-Niederlande-50-g-Schachtel	\N
205	0	2.5	0	3,73 €	https://produkte.metro.de/shop/pv/BTY-X293576/0032/0021/METRO-Chef-Kohlrabi-Streifen-tiefgefroren-2-5-kg-Beutel	\N
145	0	0.535	0	7,48 €	https://produkte.metro.de/shop/pv/BTY-X268103/0032/0021/Fuchs-Majoran-gerebelt-1-x-500-g-Beutel	\N
163	0	0.187	0	2,27 €	https://produkte.metro.de/shop/pv/BTY-X350475/0033/0021/Popp-Hummus-Natur-1-x-150-g-Becher	\N
180	0	1.008	0	1,49 €	https://produkte.metro.de/shop/pv/BTY-X322462/0032/0021/aro-Raffinade-Zucker-1-00-kg-Packung	\N
181	0	0.1	0	1,00 €	https://produkte.metro.de/shop/pv/BTY-X338642/0032/0021/M%C3%A4rsch-Import-Mandeln-gestiftelt-blanchiert-100-g-Beutel	\N
195	0	4.22	0	42,76 €	https://produkte.metro.de/shop/pv/BTY-X381395/0032/0021/METRO-Chef-Cashewkerne-natur-Deutschland-1-kg-Beutel	\N
92	0	1.013	0	25,67 €	https://produkte.metro.de/shop/pv/BTY-X143052/0032/0021/Fuchs-Bunter-Pfeffer-Gewürzmischung-Mix-Schwarzer-Weißer-Rosa-Grünen-Pfeffer-geschrotet-1-kg-Beutel	\N
94	0	5.04	0	7,37 €	https://produkte.metro.de/shop/pv/BTY-X414847/0032/0021/aro-Fusilli-Spirelli-Nudeln-1-x-5-kg-Sack	\N
213	0	1.012	0	11,76 €	https://produkte.metro.de/shop/pv/BTY-X449935/0032/0021/METRO-Chef-Kakao-Pulver-1-kg-Packung	\N
210	0	3.03	0	2,77 €	https://produkte.metro.de/shop/pv/BTY-Z79/0032/0021/M%C3%B6hren-3kg	\N
214	0	2.015	0	8,01 €	https://produkte.metro.de/shop/pv/BTY-X615324/0032/0021/FUCHS-Pommes-Frites-W%C3%BCrzsalz-Rot-1-x-2-kg-Beutel	\N
158	0	5.355	0	14,54 €	https://produkte.metro.de/shop/pv/BTY-X209293/0032/0021/aro-Schwäbische-Eierspätzle-aus-reinem-Hartweizengrieß-und-Vollei-5-kg-Karton	\N
219	0	1.009	0	1,79 €	https://produkte.metro.de/shop/pv/BTY-X113546/0032/0021/K%C3%BCchenmeister-Bio-Dinkelvollkornmehl-1-kg-Beutel	\N
220	0	0.525	0	4,79 €	https://produkte.metro.de/shop/pv/BTY-X77229/0032/0021/Doyal-Tahini-wei%C3%9Fe-Sesampaste-300-g-Tiegel	\N
98	0	2.855	0	8,01 €	https://produkte.metro.de/shop/pv/BTY-X214564/0032/0021/aro-Mandarin-Orangen-2-65-kg-Dose	\N
1	0	1.009	0	0,91 €	https://produkte.metro.de/shop/pv/BTY-X446653/0032/0021/Mühlen-König-Weizenmehl-Type-405-1-kg-Packung	\N
2	0	0.727	0	2,29 €	https://produkte.metro.de/shop/pv/BTY-X388415/0032/0021/aro-Eier-10er-Gr.-L-Braun-Bodenhaltung-10-Stück	\N
5	0	10.441	0	7,38 €	https://produkte.metro.de/shop/pv/BTY-X298854/0032/0021/Safrisalz-Speisesalz-grobkörnig-1-kg-Paket	\N
7	0	13.9	0	128,27 €	https://produkte.metro.de/shop/pv/BTY-X408155/0032/0022/METRO-Chef-Bio-Olivenöl-12-x-0-75-l-Flaschen	\N
221	0	0.53	0	0,68 €	https://produkte.metro.de/shop/pv/BTY-X437307/0032/0021/METRO-Chef-Weinsauerkraut-520-g-Packung	\N
9	0	1.01	0	14,39 €	https://produkte.metro.de/shop/pv/BTY-X97373/0032/0021/Goldsteig-Emmentaler-gerieben-nussig-fein-45-Fett-1-kg	\N
19	0	1	0	23,81 €	https://produkte.metro.de/shop/pv/BTY-X796440/0032/0021/METRO-Chef-Parmigiano-Reggiano-DOP-32-Fett-i.-Tr.-12-Monate-gereift-ca.-1-kg	\N
32	0	1.038	0	4,59 €	https://produkte.metro.de/shop/pv/BTY-X315451/0032/0021/aro-Schlagsahne-33-Fett-1-00-l-Packung	\N
223	0	18	0	28,83 €	https://produkte.metro.de/shop/pv/BTY-X12670/0032/0021/aro-Gesch%C3%A4lte-Tomaten-2-5-kg-Dose	\N
41	0	1	0	11,76 €	https://produkte.metro.de/shop/pv/BTY-X203887/0032/0021/Kerrygold-Cheddar-Block-Cheddar-Käse-1-Block-à-ca.-2-5-kg-mit-32-Fett-mit-essbarer-Rinde-2-5-kg	\N
226	0	1.612	0	1,93 €	https://produkte.metro.de/shop/pv/BTY-X351949/0032/0021/aro-Orangensaft-100-Fruchtgehalt-Pfandfrei-1-x-1-5-l-Packung	\N
62	0	7.177	0	44,29 €	https://produkte.metro.de/shop/pv/BTY-X733639/0032/0021/Edna-Brötchenkiste-5-fach-sortiert-tiefgefroren-vorgebacken-175-Stück-à-40-g-7-kg-Karton	\N
97	0	0.525	0	2,66 €	https://produkte.metro.de/shop/pv/BTY-X402289/0032/0021/Bio-Birne-Packham-Argentinien-500-g-Schachtel	\N
99	0	2.513	0	23,53 €	https://produkte.metro.de/shop/pv/BTY-X293588/0032/0021/METRO-Chef-Himbeeren-tiefgefroren-2-50-kg-Beutel	\N
100	0	1.03	0	8,23 €	https://produkte.metro.de/shop/pv/BTY-X300525/0032/0021/METRO-Chef-Heidelbeeren-tiefgefroren-1-kg-Beutel	\N
101	0	2.513	0	10,58 €	https://produkte.metro.de/shop/pv/BTY-X293592/0032/0021/METRO-Chef-Erdbeeren-tiefgefroren-erntefrisch-2-5-kg-Beutel	\N
102	0	0.7	0	2,13 €	https://produkte.metro.de/shop/pv/BTY-Z1260/0032/0021/Trauben-hell-kernlos-500g	\N
107	0	0.929	0	8,56 €	https://produkte.metro.de/shop/pv/BTY-X440382/0032/0021/Papstar-Pure-Schaschlikspieße-Holz-pure-Ø-3-mm-Länge-20-cm	\N
108	0	2.62	0	19,78 €	https://produkte.metro.de/shop/pv/BTY-X700788/0032/0021/Landfrisch-Frischkäse-Natur-Eimer-2-5-kg-2-50-kg-Eimer	\N
109	0	1.722	0	9,94 €	https://produkte.metro.de/shop/pv/BTY-X144826/0037/0021/Kölln-Müsli-Bircher-Frucht-1-7-kg-Beutel	\N
110	0	797.8	0	275,89 €	https://produkte.metro.de/shop/pv/BTY-X19249/0032/0021/aro-Natürliches-Mineralwasser-Classic-6-x-1-5-l-Flaschen	\N
112	0	2.1	0	11,76 €	https://produkte.metro.de/shop/pv/BTY-X308621/0032/0021/METRO-Chef-Wild-Preiselbeeren-2-kg-Eimer	\N
113	0	1.751	0	10,69 €	https://produkte.metro.de/shop/pv/BTY-X282573/0032/0021/METRO-Chef-Premium-Bourbon-Vanille-Eiscreme-tiefgefroren-3-l-Packung	\N
103	0	1	0	1,70 €	https://produkte.metro.de/shop/pv/BTY-Z1068/0032/0021/Bananen-Gep-(in-kg)	\N
114	0	1.015	0	13,36 €	https://produkte.metro.de/shop/pv/BTY-X615286/0032/0021/Fuchs-Zimt-gemahlen-1-kg-Beutel	\N
115	0	1.067	0	5,34 €	https://produkte.metro.de/shop/pv/BTY-X313259/0032/0021/aro-Vanillin-Zucker-1-00-kg-Packung	\N
207	0	0.57	0	4,91 €	https://produkte.metro.de/shop/pv/BTY-X332435/0032/0021/METRO-Chef-Gouda-500-g-Packung	\N
122	0	5.21	0	12,79 €	https://produkte.metro.de/shop/pv/BTY-X311868/0032/0021/aro-QS-Saure-Sahne-10-Fett-5-00-kg-Eimer	\N
206	0	0.505	0	1,19 €	https://produkte.metro.de/shop/pv/BTY-X458215/0032/0021/GAB-Buttertoast-verzehrfertig-geschnitten-Buttertoastbrot-mit-2-8-Fett-500-g-Beutel	\N
105	0	0.204	0	0,27 €	https://produkte.metro.de/shop/pv/BTY-X51975/0032/0021/Wawi-Blockschokolade-48-Kakao-200-g	\N
222	0	2.2	0	9,19 €	https://produkte.metro.de/shop/pv/BTY-X34043/0032/0021/Berief-Natur-Bio-Tofu-natur-2-kg-Packung	\N
224	0	0.06	0	1,70 €	https://produkte.metro.de/shop/pv/BTY-X663/0033/0021/METRO-Chef-Salbei-Deutschland-40-g	\N
225	0	0.804	0	4,27 €	https://produkte.metro.de/shop/pv/BTY-X205109/0032/0021/Palapa-Weizentortilla-18-St%C3%BCck-%C3%98-20-cm-800-g-Beutel	\N
227	0	0.264	0	1,70 €	https://produkte.metro.de/shop/pv/BTY-X431349/0032/0021/Oatly-Germany-BIO-Hafer-Cuisine-250-ml-Flasche	\N
\.


--
-- Data for Name: ingredients; Type: TABLE DATA; Schema: public; Owner: kuechenteam
--

COPY public.ingredients (ingredient_id, name, energy, comment) FROM stdin;
112	Preiselbeerkompott	0	Hilfe ich find nix für energie
7	Olivenöl	37	\N
15	Milch	2.78	\N
30	Gouda	15.24	\N
10	Suppengrün	1.29	\N
12	Tomatenmark	1.81	\N
13	Rotwein	2.86	\N
48	Aprikosen	2.96	\N
25	Cayennepfeffer	0.0	\N
26	Pfeffer	0.0	\N
29	Kräutermischung	0.0	\N
17	Lasagneplatten	15	\N
18	Butter	31.01	\N
20	Knoblauch	6.07	\N
21	Tomaten	0.84	\N
23	Balsamico	3.92	\N
27	Sahne	12.69	\N
31	Kaisergemüse	1.27	\N
32	Schlagsahne	12.69	\N
33	Champignons	1.48	\N
34	Paprika	0.94	\N
35	Zucchini	0.95	\N
36	Oregano	3	\N
37	Thymian	2.19	\N
38	Rosmarin	2.16	\N
39	TK-Blätterteig	16	\N
41	Cheddar	16.28	\N
42	Feta	11.9	\N
43	Hähnchenbrust	5.56	\N
44	Chorizo	13.16	\N
45	Risottoreis	14.92	\N
46	Weißwein	3.02	\N
47	Erbsen	4.2	\N
49	Margarine	29.7	\N
50	Rosinen	13.13	\N
52	Speck	13.4	\N
53	Paprikapulver	0.0	\N
54	Quark	5.91	\N
55	Zucker	16.97	\N
56	Backpulver	0.0	\N
57	Röstzwiebeln	14.55	\N
87	Knoblauchpulver	0.0	\N
90	Muskatpulver	0.0	\N
92	Bunter Pfeffer	0.0	\N
107	Schaschlikspieß	0.0	\N
59	Salami	15.72	\N
60	Schmand	8.59	\N
61	Kräuterbutter	12.66	\N
62	Aufbackbrötchen	10.32	\N
63	TK-Blattspinat	0.91	\N
67	Petersilie	2.53	\N
68	Zitronensaft	1.17	\N
65	Liebstöckl	2.01	\N
69	Toast	11.17	\N
70	Schinkenscheiben	5.83	\N
71	Senf	3.69	\N
75	Chilipaste	0.0	\N
80	Dosenananas	2.818	\N
81	Sahneschmelzkäse	12.89	\N
82	Lorbeerblätter	0.0	\N
83	Staudensellerie	0.9	\N
85	Rinderfilet	6.36	\N
86	Salatgurken	0.59	\N
88	Maiskolben	4.47	\N
89	Limetten	2.03	\N
91	Bergkäse	16.07	\N
93	Sonnenblumenkerne	20.54	\N
94	Spirelli	15.375	\N
95	Rucola	1.24	\N
96	Äpfel	2.71	\N
97	Birnen	2.41	\N
98	Dosenmandarinen	2.519	\N
99	Himbeeren	1.8	\N
100	Heidelbeeren	1.93	\N
104	Vollmilchschokolade	22.66	\N
105	Zartbitterschokolade	21.54	\N
108	Frischkäse	14.09	\N
109	Müsli	15.26	\N
110	Mineralwasser	0.0	\N
111	O-Saft	1.85	\N
5	Salz	0.0	\N
113	Vanilleeis\n	8.591	\N
114	Zimt	0.0	\N
117	Stollen Dresdner Art	16.57	\N
118	Puderzucker	16.97	\N
119	Gewürzgurken	0.93	\N
120	Edamer	14.82	\N
6	Sonnenblumenöl	37	\N
122	Saure Sahne	4.883	\N
123	Naturjoghurt	3.08	\N
124	Buttermilch	1.56	\N
125	Radieschen	0.73	\N
126	TK-8-Kräuter	2.1075	\N
127	Couscous	14.85	\N
128	Bulgur	14.8	\N
129	Schafskäse	11.9	\N
28	Kartoffeln	2.9	\N
131	Kreuzkümmel	0.0	\N
132	Kümmel	0.0	\N
133	Pinienkerne	14.69	\N
146	Hokkaido-Kürbis	208	\N
148	Tellerlinsen	12.2	\N
149	Kichererbsen	11.422	\N
150	Ingwer	2.09	\N
151	Nutella	22.82	\N
152	Apfelmus	3.81	\N
72	Maccheroni	15.84	\N
74	Honig	12.83	\N
78	Bratwurst	12.11	\N
79	Soßenbinder	16	\N
101	Erdbeeren	1.51	\N
102	Weintrauben	3.03	\N
103	Bananen	3.91	\N
115	Vanillezucker	16.762	\N
116	Butterschmalz	26.86	\N
121	Fleischwurst	12.57	\N
134	Rindergulasch	5.27	\N
135	Sauerteig	12.1	\N
137	Kokosmilch	8.68	\N
140	Zitrone	1.51	\N
141	Asia-Gemüse	1.507	\N
142	Kidneybohnen	4.27	\N
143	Dosenmais	3.52	\N
4	Wasser	0.0	\N
14	Brühepulver	7.35	\N
22	Crème fraîche	11.59	\N
76	Frühlingszwiebeln	7.37	\N
19	Parmesan	16.58	gerieben
145	Marjoran	0.0	\N
58	Schinken	5.38	ganz
144	Chilischoten	0.0	\N
154	TK-Zwetschgen	2.01	\N
155	Gewürzspekulatius	19.11	\N
156	Kardamom	0.0	\N
158	Spätzle	15.727	\N
2	Ei	6.49	\N
3	Reis	14.8114	\N
77	TK-Karotten	1.63	\N
66	TK-Lauch	1.22	\N
139	Currypaste	5.33	vegan!!
64	Frischhefe	0.0	Frisch
40	Spinat	0.9300	Frisch
24	Basilikum	1.9700	frisch
51	Currypulver	0	\N
84	Schäufele (Schweinefleisch mit Knochen)	9.5600	\N
9	Emmentaler	15.8100	\N
159	Sojagranulat	12.9	\N
161	Chilipulver	0.0	\N
130	Koriander	0	(gewürz)
138	Sojasoße (glutenfrei)	4.3200	\N
160	Mehl 550	14.235	\N
162	Räuchertofu	6.84	\N
168	Spätzle (frisch)	6.55	\N
169	Haferflocken	15.57	\N
170	Hafermilch	1.88	\N
172	Baked Beans	3.3	\N
173	Aubergine	1.04	\N
174	Schwarze Oliven	4.98	\N
176	Rote Zwiebeln	1.17	\N
177	Gemischter Salat	1.51	\N
178	Fladenbrot	10.67	\N
165	Sellerie	0.7	Knolle
167	Wiener	9.21	\N
179	TK-Paprika	1.1991	\N
171	Brot	10	\N
73	Dosentomaten	1.09	passiert
180	Hagelzucker	16.97	\N
181	Mandeln (gestiftet)	25.92	\N
183	Cocktailtomaten	0.8	\N
184	Müsliriegel	15.57	\N
185	Schokokekse	19.51	\N
186	Kekse	13.49	\N
187	Salzstangen	14.57	\N
189	Reiswaffeln	16.20	\N
191	Nussmix	26.24	\N
192	Waffelröllchen	22.30	\N
193	Mandarinen	0	\N
190	Wasabi-Nüsse	19.24	\N
194	Zwiebeln (Frisch)	1.27	\N
196	Hefeflocken	0.0	\N
195	Cashewnüsse	25.46	\N
198	Trockenhefe	0.0	\N
199	Rohrzucker	16.7	\N
136	Getrocknete Tomaten	7.31	\N
16	Mozzarella (gerieben)	11	gerieben
201	Mozzarella	11	\N
202	Weißweinessig	0.21	\N
203	Schnittlauch	1.67	\N
200	Sojajoghurt	2.29	Vegan
204	Halloumi	12	\N
206	Toastbrot	10	\N
207	Gouda-Scheibe	15	\N
8	TK-Zwiebeln	1.2700	TK
188	Tuc Cracker	20.0100	\N
205	TK-Kohlrabi	1.2000	\N
197	Dinkelmehl 630er	14.0300	\N
208	Sauerteig Anstellgut	0	\N
209	Altbrot	10.01	\N
210	Karotten (Frisch)	1.63	\N
211	Brokkoli	1.42	\N
212	Kloßteig	4.55	\N
214	Pommessalz	0	\N
213	Kakaopulver	1.5	\N
218	Baguette	10.32	\N
175	Sojaschnetzel (trocken)	15.1980	\N
215	Test	10.2000	\N
219	Dinkelvollkornmehl	14.69	~5-10% des Mehlgewichts für elastischeren Teig
220	Tahini	3.56	Sesampaste
163	Hummus (fertig)	8.5000	\N
221	Sauerkraut	1.09	\N
222	Tofu	4.97	Natur und seidenfest
223	Dosentomaten (ganz,geschält)	1.09	\N
224	Salbei	0	\N
1	Mehl 405	14.5900	\N
225	Wrap	13.99	Artikelnummer: 117665
226	Orangensaft	1.6	\N
227	Vegane Sahne	12	vegan
228	VeganesHuhnGewürz	0	\N
229	Schupfnudeln	4.17	\N
231	Kurkuma	0	(gemahlen)
233	Garam masala	0	\N
234	Ahornsirup	12	\N
230	Rote Linsen	15.3000	\N
232	Korianderblätter (frisch)	0	\N
235	Limettensaft	0	\N
237	Sojamedallions	14	\N
147	Semmelbrösel	10.0100	Paniermehl
238	Annanas	0	\N
239	Kichererbsenmehl	13.2	\N
241	Grapefruitsaft	0	\N
242	Hänchenfond	0	\N
244	Bratöl	37	\N
245	Rinderfond (vegan)	0	\N
243	Ingwerpulver	0	\N
246	Kötbullar (vegan)	7.0	\N
\.


--
-- Data for Name: meta_recipes; Type: TABLE DATA; Schema: public; Owner: kuechenteam
--

COPY public.meta_recipes (parent_id, child_id, weight) FROM stdin;
17	15	1.7
17	16	0.5
18	15	1.7
20	28	0.45
20	21	0.240
18	21	0.24
22	20	0.7
23	20	0.7
27	20	0.7
24	22	0.8
25	22	0.8
26	24	0.8
200	4	8
31	32	0.275
43	41	1
43	42	1
46	17	2.2
47	17	2.2
44	46	2.35
45	46	2.35
48	46	2.35
49	46	2.35
33	31	0.514
51	4	0.35
56	52	14
57	53	33
56	53	33
60	58	3
60	11	2
61	59	3.5
61	29	5
64	2	2
64	1	1
64	11	2
63	44	3
63	45	3
63	46	5
63	48	3
63	49	3
225	4	0.1000
43	70	2
97	2	3
97	98	0.5000
64	58	5.5
64	59	2.5
63	47	3
97	58	0.5
72	71	3
51	73	0.1
53	4	2
91	16	0.3000
14	4	0.3000
29	14	2
75	59	1
75	74	1
77	76	1
81	58	1.05
82	4	0.35
77	74	2
91	90	1.1370
36	34	0.2500
35	34	0.0700
37	34	0.0700
29	30	16
7	3	0.6000
7	11	1.5000
87	11	1
86	4	0.25
86	3	0.15
87	58	1.2
90	92	0.152
93	4	7.5000
96	94	1
96	1	0.15
96	2	0.25
98	4	0.1000
100	56	2
100	99	3
80	41	0.0800
97	1	1
101	2	1.5
67	33	3
108	4	1
67	50	2
251	231	0.9300
251	250	0.0950
191	139	0.2500
265	101	1
69	35	1
69	36	2
69	37	1
68	56	1.5000
68	57	2.5000
68	59	1
66	54	1
66	55	2
106	102	1
106	103	1
50	31	0.5140
11	14	0.4500
62	23	3
62	24	1
62	25	1
62	26	2
62	27	3
203	136	2.5
39	4	0.8000
204	136	2.5000
204	213	0.5000
223	200	0.5
224	4	0.5000
78	79	0.0800
78	81	0.0800
78	82	0.0800
78	83	0.0200
78	84	0.1400
78	85	0.0800
139	59	1
139	203	1.5000
230	58	1.5
230	229	1
230	228	1
227	59	0.5000
227	224	1
227	225	0.2300
\.


--
-- Data for Name: places; Type: TABLE DATA; Schema: public; Owner: kuechenteam
--

COPY public.places (place_id, name, comment) FROM stdin;
1	Akk Halle	\N
2	Akk Tribüne	\N
3	Unifest Gelände	\N
4	Seminarhütte	\N
5	-118 Infobau	\N
6	Mensa GS Ammerbuch	\N
7	ZKM	\N
8	Infobau Draußen	Unter dem Dach
\.


--
-- Data for Name: recipe_ingredients; Type: TABLE DATA; Schema: public; Owner: kuechenteam
--

COPY public.recipe_ingredients (recipe_id, ingredient_id, amount, unit_id) FROM stdin;
9	134	6	0
9	8	3	0
9	21	2	0
9	34	1	0
9	1	0.6	0
9	12	800	1
9	6	50	8
9	5	150	1
10	28	5	0
10	8	6	5
10	52	500	1
10	5	100	1
10	26	100	1
10	53	200	1
10	25	75	1
10	2	20	5
12	34	6	5
12	28	3	0
12	35	4	5
12	8	4	5
12	7	8	4
12	36	50	1
12	24	50	1
12	37	50	1
12	38	50	1
12	5	50	1
12	26	50	1
13	39	3	10
13	41	12	4
13	42	450	1
13	8	2	5
13	2	6	5
13	7	6	4
12	20	6	7
102	1	920	1
102	4	600	1
102	5	20	1
102	7	30	1
102	64	8	1
102	199	1	4
102	219	80	1
229	12	100	1
16	73	500	1
16	24	25	1
21	54	120	1
21	22	120	1
229	73	800	1
229	194	2	5
229	20	3	7
23	52	50	1
80	18	25	1
25	9	120	1
24	33	120	1
27	96	300	1
26	9	120	1
80	49	5	1
80	70	5	1
80	108	10	1
80	151	30	1
80	163	8	1
80	171	80	1
80	172	30	1
14	6	5	1
14	138	7	8
14	159	100	1
30	5	92	1
30	6	100	8
13	63	900	1
30	8	2	0
32	39	275	1
31	2	3	5
31	30	50	1
31	26	10	1
31	90	5	1
30	14	15	1
33	8	1	5
214	229	1	0
38	28	2	0
38	77	0.75	0
38	66	0.75	0
38	165	0.5	0
38	8	0.3	0
38	82	10	5
38	24	10	1
38	4	8	2
38	53	10	1
38	12	10	1
38	38	5	1
38	26	5	1
9	28	6	0
30	20	12	7
30	26	8	1
40	168	500	1
38	145	10	1
33	63	450	1
38	148	2.25	0
40	9	200	1
40	8	2	5
40	90	1.5	1
40	5	7.5	1
40	26	10	1
30	34	1.3500	0
21	5	4.75	1
16	5	5	1
27	114	3.1418	1
30	37	10	1
30	51	7	1
8	27	400	8
8	8	1	5
8	9	200	1
8	158	1	0
33	183	10	5
22	194	1.5	5
23	194	1	5
42	169	100	1
214	152	500	1
214	114	5	1
214	55	20	1
42	170	500	1
43	52	150	1
43	33	300	1
43	18	250	1
43	172	1500	1
44	33	200	1
49	59	200	1
48	34	100	1
48	173	100	1
47	35	150	1
47	174	50	1
89	70	15	1
46	16	150	1
33	42	200	1
89	206	50	1
89	207	20	1
229	210	150	1
229	38	2	1
51	5	10	1
51	26	15	1
51	20	1	7
51	8	1	5
229	37	2	1
51	46	150	8
51	45	300	1
51	7	2	4
51	33	250	1
51	66	1	5
229	145	2	1
229	36	2	1
229	5	2	1
54	140	1	5
54	176	2	5
54	20	1	7
54	34	2	5
54	21	500	1
54	86	500	1
54	42	200	1
54	174	100	1
54	7	4	4
54	5	5	1
54	26	5	1
50	8	6	5
50	52	250	1
50	66	2	5
50	132	0.5000	3
28	4	520	8
28	64	0.2500	1
28	160	1	0
229	26	2	1
213	51	2	1
57	134	9	0
59	3	100	1
59	4	250	1
58	94	500	1
58	4	1	2
45	70	200	1
30	53	16	1
30	68	25	1
74	6	0.010	2
213	159	200	1
21	26	6	1
70	1	1	0
70	55	125	1
70	18	150	1
70	64	1	5
70	2	2	5
70	15	500	8
70	50	50	1
70	180	50	1
70	181	50	1
43	171	3	0
17	95	5	1
71	96	1	0
71	103	1	0
72	184	0.5	0
72	185	0.2	0
72	186	0.2	0
72	188	0.2	0
72	187	0.2	0
72	189	0.2	0
72	190	0.2	0
72	191	0.3	0
72	192	0.15	0
47	194	50	1
73	195	0.15	0
73	196	4	4
73	5	1	3
73	87	0.5	3
74	8	0.5	0
74	179	0.5	0
74	35	0.6	0
74	77	0.5	0
74	150	0.005	0
74	73	2	0
74	20	4	7
81	55	1	3
74	137	1	2
74	139	0.02	0
213	228	5	8
213	4	600	1
223	218	0.25	0
225	244	60	8
81	73	150	1
81	23	4	4
81	7	4	4
81	5	0.4	3
81	26	0.4	3
81	136	100	1
81	95	40	1
81	183	250	1
81	201	125	1
82	28	1	0
82	194	120	1
82	6	4	4
82	71	1	4
82	202	5	4
82	26	5	1
82	5	5	1
82	203	1	9
82	55	5	1
79	160	400	1
79	198	7	1
79	5	2	3
79	55	0.5	3
79	7	3	4
79	4	230	8
83	194	4	5
83	21	8	5
83	35	2	5
83	20	4	7
83	7	8	4
83	38	20	1
83	36	20	1
83	145	20	1
83	5	10	1
83	34	4	5
97	4	5	2
84	103	1	5
84	104	18	1
84	105	18	1
85	21	4	5
85	201	125	1
85	23	4	4
85	7	4	4
85	5	5	1
85	24	5	1
97	5	5	4
225	6	36	1
225	51	1	3
225	131	1	3
225	138	2	8
225	147	10	1
97	158	2	0
30	73	8	0
30	142	4	0
30	143	2.4000	0
30	161	21	1
30	210	500	1
30	213	20	1
99	212	1	0
88	206	50	1
30	214	8	1
86	28	250	1
86	77	125	1
86	205	125	1
86	194	20	1
99	4	2	2
87	30	150	1
11	5	10	1
90	160	640	1
90	4	380	1
90	5	12	1
92	160	50	1
92	4	50	1
92	208	2	1
80	207	20	1
80	121	5	1
4	4	1	2
4	14	12	1
93	6	15	4
93	28	5	0
93	65	15	5
93	66	750	1
93	67	0.5	9
93	77	1	0
93	145	15	4
93	165	750	1
93	194	5	5
94	209	240	1
94	2	2	5
94	15	150	8
94	194	1	5
94	67	30	1
94	18	2	4
94	5	1	3
94	26	0.5	3
94	147	60	1
98	1	10	1
98	26	0.3000	1
98	90	0.2000	1
98	147	50	1
98	194	1.5000	5
98	211	500	1
98	68	5	8
101	168	1	0
103	4	500	1
103	5	0.5000	3
103	7	30	8
103	20	1	7
103	26	2	1
103	68	10	8
103	131	3	1
103	149	500	1
103	220	80	1
11	7	25	8
11	8	3	5
11	20	2	7
11	26	1	1
11	73	800	1
11	90	0.1	1
88	207	20	1
136	8	0.1000	0
136	103	0.5000	0
15	1	1000	1
15	4	600	8
15	5	25	1
15	7	15	1
15	64	4	1
15	135	50	1
136	131	1.5000	1
136	137	400	1
108	18	25	1
108	28	1	0
108	194	0.2000	0
108	221	0.5000	0
108	5	3	1
108	90	1	1
108	132	4	1
200	5	23	1
200	49	8	4
200	51	16	1
200	55	8	4
200	76	4	9
200	77	4	0
200	130	3.7700	1
200	150	120	1
200	161	0.4800	1
200	226	2	2
200	227	0.8000	0
200	235	183	1
191	225	40	1
68	158	4	0
76	4	100	1
76	5	0.5000	3
76	49	1	4
76	197	0.3000	0
76	198	0.5000	1
76	199	1	3
76	200	50	1
186	5	0	0
186	7	50	1
186	8	150	1
186	12	100	1
186	20	4	7
186	26	0	0
186	34	700	1
186	35	500	1
186	37	0	0
186	38	0	0
186	55	0	0
186	173	400	1
186	223	500	1
186	224	0	0
55	5	5	1
55	7	5	4
55	23	3	4
55	26	5	1
55	33	150	1
55	34	3	5
55	76	4	5
55	96	1	5
55	125	6	5
55	176	1	5
55	177	300	1
106	177	0.5000	0
106	183	0.3000	0
106	194	0.0500	0
228	173	0.2	0
228	35	0.2	0
228	28	0.4	0
228	194	0.2	0
228	34	0.2	0
228	20	4	7
228	210	0.2	0
203	222	0.2	0
228	38	2	1
228	7	50	8
1	1	75	1
1	5	50	1
1	26	40.5000	1
1	27	2	0
1	31	5	0
1	90	2	1
1	194	1	0
228	5	1	1
228	26	1	1
225	170	7	1
225	237	100	1
225	239	7	1
225	241	0.2100	0
225	242	200	1
225	243	14	1
225	26	3	1
225	233	2.7	1
225	196	1.48	1
225	132	1	1
225	231	1	1
225	137	175	1
52	245	548	1
52	4	6.6000	0
52	175	2	0
52	90	26	1
52	130	8	1
52	235	260	1
52	26	8	1
224	5	1.5000	1
224	6	3	4
224	20	4	7
224	26	3	1
224	53	1	3
224	73	200	1
224	130	1	3
224	131	1	3
224	137	200	8
224	150	25	1
224	194	2	5
224	230	250	1
224	231	1	3
224	232	1	9
224	233	1	3
224	234	1	3
53	13	2	2
53	20	100	1
78	204	140	1
78	218	80	1
53	26	10	1
53	28	8	0
53	34	7.5000	0
53	36	50	1
53	37	17	1
53	53	32	1
53	73	8	2
53	145	50	1
53	194	2.5000	0
53	210	2.5000	0
53	214	10	1
53	5	22	1
53	82	4	1
53	38	2	1
91	16	240	1
34	49	9	1
34	178	50	1
36	16	70	1
36	21	1	5
35	9	1	12
35	70	1	12
37	21	0.1000	5
37	162	20	1
37	163	20	1
7	17	200	1
7	9	300	1
41	2	0.5000	0
41	5	10	1
41	15	80	1
41	26	20	1
41	53	20	1
41	194	3	5
3	1	100	1
3	5	15	1
3	15	1	2
3	18	100	1
3	26	12	1
3	90	10	1
231	18	75	1
231	15	250	1
231	64	25	1
231	55	75	1
231	5	0.5	3
231	156	0.5	3
231	1	500	1
250	18	37.5	1
250	55	50	1
250	114	0.5	4
251	2	0.5000	5
251	180	30	1
2	1	0.0100	0
2	5	2	1
2	8	0.4000	0
2	15	1.000	0
2	26	2	1
2	27	0.5000	0
2	33	1.2000	0
265	47	0.2000	0
265	246	0.3000	0
265	112	0.01	0
266	218	1	0
39	5	10	1
39	6	50	1
39	8	4	5
39	20	3	7
39	26	10	1
39	28	0.8000	0
39	34	700	1
39	53	2	3
39	78	200	1
39	132	1.5000	3
39	161	0.5000	3
\.


--
-- Data for Name: recipes; Type: TABLE DATA; Schema: public; Owner: kuechenteam
--

COPY public.recipes (recipe_id, name, comment) FROM stdin;
8	Käsespätzle	\N
9	Gulasch	\N
10	Bratkartoffeln	\N
12	Kartoffel-Gemüse-Ecken	\N
13	Blätterteigecken	\N
16	Pizzasoße	\N
17	Pizza	\N
18	Flammkuchen_p	Mit Pizzateig
20	Flammkuchen	\N
21	Flammkuchencreme	\N
31	Quiche	\N
32	Quicheteig	\N
38	Linsensuppe	\N
22	Flammkuchen Vegetarisch	\N
23	Flammkuchen Standard	\N
24	Flammkuchen Forèstier	\N
25	Flammkuchen Käse	\N
26	Flammkuchen Forèstier + Käse	\N
27	Flammkuchen Apfel	\N
40	Kässpätzle	\N
42	Veganer Porridge	\N
43	Frühstück	\N
44	Pizza Funghi	\N
45	Pizza Prosciutto	\N
46	Pizza Margherita	\N
47	Pizza Vegana	\N
48	Pizza Vegetariana	\N
49	Pizza Salame	\N
33	Spinat-Tomaten-Quiche	Nachschauen, ob das nicht zu viel Tomaten sind
51	Pilzrisotto	\N
54	Griechischer Bauernsalat	\N
56	GPN-Gulasch Vegan	\N
57	GPN-Gulasch	\N
58	Nudeln	\N
59	Reis	\N
60	Pasta bolognese	\N
61	Chili con Reis	Vegan
63	Pizza mix	\N
64	Reis-Nudel-Buffet	\N
36	Tomaten-Mozzarella-Fladenbrot	\N
70	Hefezopf	\N
71	Obst (Snack)	\N
72	Snacks	\N
73	Veganer Parmesan	\N
74	Curry	\N
75	Curry mit Reis	\N
77	Curry mit Naan	\N
82	Kartoffelsalat	\N
81	Nudelsalat	https://kochkarussell.com/italienischer-nudelsalat-einfach/
79	Stockbrot	https://www.einfachbacken.de/rezepte/stockbrot-schnelles-grundrezept
83	Grillgemüse	\N
84	Schokobananen	\N
85	Caprese	\N
86	Kartoffeln In Bechamelsauce	https://docs.google.com/spreadsheets/d/12qS3gSCdPiNNX9lGgZVcZmNQXt8Z8E9XrmAiDaIz07w/edit?usp=sharing
87	Nudeln mit Bolognese	\N
90	Spielstadt-Pizzateig	\N
92	Sauerteig	\N
80	Seminarfrühstück	\N
4	Gemüsebrühe	\N
93	Kartoffelsuppe	FSMI-Kartoffelsuppe (TODO: Verhältnisse anpassen. Wurde eher Karottensuppe.)
94	Semmelknödel	https://emmikochteinfach.de/klassische-semmelknoedel/
96	Semmelknödel mit Soßen	TODO: Kaisersoße durch Knödelkonformes Curry ersetzen
97	Spätzle mit Soßen	
98	Brokkolisoße	vegan
14	Sojahack	\N
99	Kartoffelklöße	\N
100	Gulasch mit Klößen	\N
29	Chili sin carne	Beim Seminar (WS22\\23) würden abens 2166 kj pro person + 600kj pp baguette gegessen
30	Chili_base	kalibriert (Gewürze)
101	Spätzle mit Pilzrahmsoße	\N
103	Hummus	Gewürzmengen unkalibriert
1	Kaisersoße	\N
62	Flammkuchen mix	\N
191	Bananencurry-Wrap mit Reis	Erstellt für das Eulenfest 2023
227	Crispy-Nugget-Dal mit Reis	\N
15	Pizzateig	\N
35	Schinken-Käse-Fladenbrot	\N
76	Naan	https://www.eat-this.org/veganes-naan-brot/
213	Fake-Curry-Hähnchen	\N
216	Knödelbase (Vegan)	\N
78	Grillen	\N
186	Ratatouille	\N
66	Salat Mix	\N
37	Tofu-Hummus-Fladenbrot	Hälfte mit, Hälfte ohne Tomaten
55	Gemischter Salat + Dressing	\N
68	Gulasch mit Beilagen	Spätzle und Reis
52	Sojaschnetzel	\N
136	Bananencurry	\N
106	Hummus-Pita	\N
203	Bananencurry mit Tofu	\N
108	Sauerkrauteintopf	ggf. Würstchen als Beilage ergänzen
50	Quiche Lorraine	\N
7	Lasagne	\N
11	Sojabolognese	\N
250	Zimtschneckenfüllung	\N
41	Rührei	\N
102	Pita	
53	GPN-Gulasch_base	\N
28	Flammkuchenteig	\N
89	Sandwich Toast Schinken-Käse	\N
88	Sandwich Toast Käse	\N
3	Bechamelsoße	\N
91	Spielstadt-Pizza	\N
34	Fladenbrot (beschmiert)	\N
251	Zimtschnecken	\N
224	Rote-Linsen-Dal	\N
67	Quiche mix	\N
2	Pilzrahmsoße	Funktioniert auch gut mit Granatapfel
69	Fladenbrot Mix	\N
39	Kartoffelgulasch	\N
228	Ofengemüse	\N
204	Bananencurry mit Fake-Hähnchen	\N
225	Sojanuggets (Crispy)	Paniert und Crispy, mit orientalischen Gewürzen. DIE NOCH AUSGEWOGEN WERDEN MÜSSEN @SEMINAR (auch der andere shit)
231	Zimtschneckenteig	\N
229	Tomatensoße	\N
214	Schupfnudeln mit Apfelmus	\N
139	Bananencurry mit Reis	\N
230	Nudeln mit Tomatensoße und Ofengemüse	\N
200	Möhren-Orangen-Suppe (Vegan)	Vegane Margarine!!! (abgewogen)
265	Spätzle mit Köttbullar	\N
215	Vegane Knödel base	\N
266	Baguettescheiben	\N
223	Möhren-Orangen-Suppe mit Baguette (vegan)	\N
\.


--
-- Data for Name: steps; Type: TABLE DATA; Schema: public; Owner: kuechenteam
--

COPY public.steps (step_id, step_order, step_name, step_description, fixed_duration, duration_per_kg, recipe_id) FROM stdin;
237	1	Schnippeln und Anbraten	Frühlingszwiebeln in  Ringe schneiden, weiße Teile mit Karotten in Margarine anbraten. Mit Currypulver bestäuben.	00:01:00	00:03:00	200
238	2	Ablöschen	Mit Limettensaft ablöschen, dann Brühe dazu	00:00:00	00:03:00	200
239	3	Abschmecken	Sahneersatz, Orangensaft, Salz zugeben, abschmecken	00:00:00	00:05:00	200
92	1	Schritt 1	Linsen einweichen (viele Linsen brauchen sonst sehr lange zum Durchwerden)	00:00:00	00:00:00	38
93	2	Schritt 2	Zwiebeln anbraten	00:00:00	00:00:00	38
94	3	Schritt 3	Karotten, Lauch und Sellerie dazu, auch anbraten	00:00:00	00:00:00	38
95	4	Schritt 4	Tomatenmark kurz rösten und mit Wasser ablöschen	00:00:00	00:00:00	38
96	5	Schritt 5	Linsen und Restwasser dazugeben und kochen. DIESER SCHRITT BRAUCHT ZEIT	00:00:00	00:00:00	38
97	6	Schritt 6	Würzen und abschmecken	00:00:00	00:00:00	38
116	1	Schritt 1	Brühe ansetzen	00:00:00	00:00:00	14
117	2	Schritt 2	Sojagranulat in heißer Brühe  einweichen	00:00:00	00:00:00	14
118	3	Würzen	Öl und sojasoße hinzugeben	00:00:00	00:00:00	14
119	4	Abgießen	Überschüssiges Wasser Abgießen	00:00:00	00:00:00	14
127	1	Gemüse schneiden	Paprika in ca. 1 cm² große Stücke schneiden. Karotten waschen und würfeln, Knoblauch hacken	00:02:00	00:02:00	30
128	2	Zwiebeln anbraten	Sonnenblumenöl in den Topf geben und zwiebeln nach und nach anbraten	00:02:00	00:01:00	30
129	3	Gemüse anbraten	Paprika zu den Zwiebeln geben und anbraten. Dann die restlichen Zutaten hinzufügen	00:02:00	00:01:00	30
130	4	Würzen	Nun noch das Chili würzen. Die Gewürzmengen sind tatsächlich kalibriert und sollten ungefähr passen.	00:01:00	00:00:30	30
240	4	Pürieren	Suppe durchpürieren, bis sie sämig ist. Mit Frühlingszwiebelgrün bestreut servieren.	00:00:00	00:05:00	200
244	1	Schritt 1	Zwiebeln klein hacken	00:00:00	00:00:00	41
245	2	Schritt 2	Alles verquirlen	00:00:00	00:00:00	41
75	1	Schritt 1	Wasser kochen	00:00:00	00:00:00	4
76	2	Schritt 2	Brühepulver zugeben	00:00:00	00:00:00	4
246	3	Schritt 3	Im Gasbräter braten. Ihr habt schonmal Rührei gemacht.	00:00:00	00:00:00	41
249	1	Vorheizen	Den Ofen auf 250 - 275 °C Ober-/Unterhitze aufheizen.	00:20:00	00:00:00	251
250	2	Glasieren	Die Kanelbullar (Zimtschnecken) mit verschlagenem Ei bepinseln. Anschließend mit Zucker bestreuen.	00:02:00	00:02:00	251
84	1	Schritt 1	Kocht Nudeln. Ihr wisst, wie man Nudeln kocht.	00:00:00	00:00:00	58
85	1	Schritt 1	Kocht Reis. Ihr wisst, wie man Reis kocht.	00:00:00	00:00:00	59
86	1	Schritt 1	Soßen und Nudeln und Reis kochen	00:00:00	00:00:00	64
251	3	Backen	Auf der mittleren Stufe im vorgeheizten Backofen etwa 5 - 8 Minuten backen. Unter einem Handtuch abkühlen lassen.	00:02:00	00:06:00	251
101	1	Schritt 1	Zutaten in einem passenden Topf vermischen	00:00:00	00:00:00	42
102	2	Schritt 2	Unter ständigem Rühren auf niedriger Stufe kochen, bis der Brei breiig ist	00:00:00	00:00:00	42
103	1	Schritt 1	Hefezopf bei 180°C backen	00:00:00	00:00:00	43
104	2	Schritt 2	Porridge und Rührei ansetzen	00:00:00	00:00:00	43
105	3	Schritt 3	Speck und Pilze in Butter anbraten (getrennt), Baked Beans aufwärmen	00:00:00	00:00:00	43
106	4	Schritt 4	Brote und Aufschnitt auslegen	00:00:00	00:00:00	43
107	1	Schritt 1	Zwiebeln rösten	00:00:00	00:00:00	8
108	2	Schritt 2	Spätzle, Sahne und Käae in Gasbräter geben	00:00:00	00:00:00	8
109	3	Schritt 3	Unter ständigem Rühren erwärmen. Kurz vor Ende Zwiebeln hinzufügen	00:00:00	00:00:00	8
110	4	Schritt 4	Würzen	00:00:00	00:00:00	8
111	1	Schritt 1	Kartoffeln waschen, schälen und schneiden (1-2cm Würfel oder Scheiben)	00:00:00	00:00:00	10
112	3	Schritt 3	Kartoffeln in Öl mit geschlossenem Deckel anbraten	00:00:00	00:00:00	10
113	2	Schritt 2	Kartoffeln kurz in kaltes Wasser lesen, damit Stärke austreten kann	00:00:00	00:00:00	10
114	4	Schritt 4	10 Minuten vor Ende gewürfelte Zwiebeln hinzugeben	00:00:00	00:00:00	10
115	5	Schritt 5	5 Minuten vor Ende offen braten und würzen	00:00:00	00:00:00	10
120	1	Chilli Kochen	Chilli halb zubereiten, aber noch nicht würzen 	00:00:00	00:00:00	29
121	2	Sojahack hinzugben	Sojahack in den Topf geben	00:03:00	00:00:12	29
122	3	Abschmecken	Die restlichen Gewürze aus dem chilli base rezept hinzugen	00:05:00	00:00:06	29
141	1	Kichererbsen kochen	Kichererbsen mit etwas Backpulver kochen, bis sie weich sind	00:00:00	00:30:00	103
142	2	Hummus machen	Tahini mit Zitronensaft und ein bisschen Wasser vom Kichererbsenkochen im Mixer schaumig pürieren. Die restlichen Zutaten dazugeben und pürieren, bis eine cremige Masse entsteht. Der Knoblauch sollte vorher etwas kleingeschnitten werden.	00:10:00	00:10:00	103
241	1	Schritt 1	Sojabolognese kochen	00:00:00	00:00:00	7
242	2	Schritt 2	Abwechselnd Bolognese, Lasagneplatten und Gouda schichten. Währenddessen Bechamelsoße ansetzen.	00:00:00	00:00:00	7
243	3	Schritt 3	Bechamelsoße als oberste Schicht ausgießen, bei 180°C Umluft backen	00:00:00	00:00:00	7
247	1	Schritt 1	Butter schmelzen, Mehl darin anbräunen, Gewürze hinzufügen	00:00:00	00:00:00	3
248	2	Schritt 2	Milch unter Rühren zugeben. Dabei immer warten, bis die Milch völlig eingezogen ist, bevor mehr dazugegeben wird	00:00:00	00:00:00	3
252	1	Schritt 1	Zwiebeln und Champignons anbraten (am besten in mehreren Fuhren)	00:00:00	00:00:00	2
253	2	Schritt 2	Milch dazu, Schlagsahne dazu, Mehl dazu	00:00:00	00:00:00	2
254	3	Schritt 3	Abschmecken	00:00:00	00:00:00	2
155	5	Pizza Boden vorbacken	E Grill auf 230 Grad stellen und den Boden für ca. 30s - 1m braten	00:00:00	00:05:00	15
261	1	Gemüse schneiden	Zwiebeln in dünne Scheiben schneiden, Paprika in 2cm-Quadrate, Kartoffeln in 1.5cm-Würfel, Knoblauch fein würfeln	00:00:00	00:10:00	39
158	1	Zwiebeln anschwitzen	Zwiebeln in Butter anschwitzen. Dann alles außer Brühe und Sauerkraut dazu, kurz anbraten	00:05:00	00:10:00	108
159	2	Fertig kochen	Mit Sauerkraut ablöschen, Brühe dazu, köcheln lassen bis durch	00:05:00	00:20:00	108
160	3	Abschmecken	Wild würzen oder Dennis rufen	00:10:00	00:01:00	108
262	2	Gemüse anbraten	Erst Zwiebeln, später Knoblauch, Paprika und dann Kartoffeln andünsten. Kümmel mit den Zwiebeln anrösten	00:00:00	00:02:00	39
263	3	Ablöschen	Mit Brühe ablöschen	00:03:00	00:00:00	39
264	4	Abschmecken	Beim Würzen nicht fest an die Gewichte halten	00:00:00	00:01:00	39
265	5	Bratwurst braten	Bratwurst in 1-1.5cm-Scheiben anbraten und zum Gulasch reichen	00:00:00	00:03:00	39
193	1	Gemüse vorbereiten	Waschen, putzen, mundgerecht kleinschneiden	00:00:00	00:00:00	186
194	2	Gemüse scharf anbraten	Zwiebeln + Zuccini, dan Paprika, dann Aubergine. Dann mit Tomatenmark und Tomaten ablöschen, würzen.	00:00:00	00:00:00	186
195	3	köcheln	köcheln	00:10:00	00:05:00	186
196	1	Variante 1	Pita aufschneiden, Hummus darin verteilen, evtl. Rohkost und andere Zutaten in die Tasche füllen	00:00:00	00:03:00	106
197	2	Variante 2	Hummus in Schüssel füllen, mit Pita servieren	00:00:00	00:01:00	106
198	1	Schritt 1	Zwiebeln andünsten, später Knoblauch dazugeben, parallel dazu Sojahack ansetzen	00:00:00	00:00:00	11
199	2	Schritt 2	Sojahack anbraten, Zwiebeln dazugeben, mit Tomaten ablöschen	00:00:00	00:00:00	11
200	3	Schritt 3	Würzen, köchlen lassen, abschmecken	00:00:00	00:00:00	11
208	1	Anbraten	Zwiebeln anbraten. Bananen in Scheiben dazu (das gibt Matsch, das soll so). \\\\Tofu in etwa 1cm dicken Scheiben knusprig ausbraten	00:15:00	00:02:00	136
209	2	Kochen	Kokosmilch dazu, köcheln	00:15:00	00:05:00	136
210	3	Würzen	Gewürze sind nicht abgewogen. Nicht blind dem Rezept folgen!	00:10:00	00:02:00	136
211	4	Anrichten	Tofu in Zentimeterwürfel zerschneiden und unterrühren	00:04:00	00:00:15	136
215	1	Schritt 1	Zwiebeln in Topf anbraten, Gemüse dazu, Sahne dazu, Mehl dazu	00:00:00	00:00:00	1
216	2	Schritt 2	Köcheln lassen	00:00:00	00:00:00	1
217	3	Schritt 3	Mit Salz, Pfeffer und Muskat abschmecken	00:00:00	00:00:00	1
229	1	Hefe aufschwämmen	Hefe, Honig mit Schluck Wasser ~30°C und etwas Mehl aufschwämmen, viertel- bis halbe Stunde stehen lassen, bis sich Blasen bilden	00:25:00	00:00:30	102
230	2	Autolyseteig ansetzen	Mehl, Wasser ~18°C, Öl verkneten, bis ein glatter Teig entsteht. In Teigwanne ruhen lassen, bis der Hefesponge fertig ist	00:00:00	00:05:00	102
231	3	Teig fertig machen und portionieren	Nacheinander Hefesponge und Salz in den Autolyseteig einkneten. In Portionen von ~120g einteilen, rundschleifen und bedeckt ruhen lassen. Bei größeren Mengen kann auch erst nur ein Teil portioniert werden, um Platz zu sparen	00:05:00	00:15:00	102
232	4	Fladen backen	Vorportionierten Teig auf 0.5-1cm Dicke auswellen und sofort in Ofen mit 250°C einschießen. Backen, bis sich eine Tasche bildet und beide Seiten gebräunt sind $\\rightarrow$ ggf. wenden. Beim Auswellen großzügig mehlen. Fixzeit ist für Ofen vorheizen angedacht.	00:40:00	00:20:00	102
\.


--
-- Data for Name: stores; Type: TABLE DATA; Schema: public; Owner: kuechenteam
--

COPY public.stores (store_id, name) FROM stdin;
0	Metro
-1	Dummy
2	IKEA
\.


--
-- Data for Name: units; Type: TABLE DATA; Schema: public; Owner: kuechenteam
--

COPY public.units (unit_id, name) FROM stdin;
0	kg
1	g
2	l
3	TL
4	EL
5	Stk
6	Knolle
7	Zehe
8	ml
9	Bund
10	Pck
11	Prise
12	Scheibe
\.


--
-- Data for Name: weights; Type: TABLE DATA; Schema: public; Owner: kuechenteam
--

COPY public.weights (unit_id, ingredient_id, weight) FROM stdin;
5	2	0.058
5	35	0.21
5	34	0.155
5	8	0.05
7	20	0.003
10	39	0.450
12	9	0.0175
5	21	0.099
12	69	0.025
12	70	0.033
5	76	0.035
5	176	0.05
5	167	0.075
5	66	0.14
5	96	0.135
5	140	0.08
5	125	0.004
2	6	0.9
2	7	0.9
5	165	0.8
5	64	0.042
5	183	0.042
5	194	0.05
5	82	0.0005
9	203	0.03
5	103	0.115
9	67	0.06
5	65	0.01
5	89	0.06
9	76	0.176
9	232	0.025
\.


--
-- Name: events_event_id_seq; Type: SEQUENCE SET; Schema: public; Owner: kuechenteam
--

SELECT pg_catalog.setval('public.events_event_id_seq', 41, true);


--
-- Name: food_properties_property_id_seq; Type: SEQUENCE SET; Schema: public; Owner: kuechenteam
--

SELECT pg_catalog.setval('public.food_properties_property_id_seq', 48, true);


--
-- Name: ingredients_ingredient_id_seq; Type: SEQUENCE SET; Schema: public; Owner: kuechenteam
--

SELECT pg_catalog.setval('public.ingredients_ingredient_id_seq', 246, true);


--
-- Name: places_place_id_seq; Type: SEQUENCE SET; Schema: public; Owner: kuechenteam
--

SELECT pg_catalog.setval('public.places_place_id_seq', 8, true);


--
-- Name: recipes_recipe_id_seq; Type: SEQUENCE SET; Schema: public; Owner: kuechenteam
--

SELECT pg_catalog.setval('public.recipes_recipe_id_seq', 266, true);


--
-- Name: steps_step_id_seq; Type: SEQUENCE SET; Schema: public; Owner: kuechenteam
--

SELECT pg_catalog.setval('public.steps_step_id_seq', 265, true);


--
-- Name: stores_store_id_seq; Type: SEQUENCE SET; Schema: public; Owner: kuechenteam
--

SELECT pg_catalog.setval('public.stores_store_id_seq', 1, false);


--
-- Name: units_unit_id_seq; Type: SEQUENCE SET; Schema: public; Owner: kuechenteam
--

SELECT pg_catalog.setval('public.units_unit_id_seq', 12, true);


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


--
-- Name: conversions; Type: MATERIALIZED VIEW DATA; Schema: public; Owner: kuechenteam
--

REFRESH MATERIALIZED VIEW public.conversions;


--
-- PostgreSQL database dump complete
--

