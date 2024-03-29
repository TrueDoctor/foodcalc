--
-- PostgreSQL database dump
--

-- Dumped from database version 14.2
-- Dumped by pg_dump version 14.2

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
-- Name: recipes; Type: SCHEMA; Schema: -; Owner: kuechenteam
--

CREATE SCHEMA recipes;


ALTER SCHEMA recipes OWNER TO kuechenteam;

--
-- Name: pgcrypto; Type: EXTENSION; Schema: -; Owner: -
--

CREATE EXTENSION IF NOT EXISTS pgcrypto WITH SCHEMA public;


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
-- Name: conversions; Type: VIEW; Schema: public; Owner: kuechenteam
--

CREATE VIEW public.conversions AS
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
   FROM conversion_table;


ALTER TABLE public.conversions OWNER TO kuechenteam;

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
-- Name: ingredient_sources; Type: TABLE; Schema: public; Owner: kuechenteam
--

CREATE TABLE public.ingredient_sources (
    ingredient integer NOT NULL,
    store integer NOT NULL,
    package_size numeric NOT NULL,
    package_unit integer NOT NULL,
    price money,
    url character varying,
    comment text,
    CONSTRAINT comment_not_empty CHECK ((comment <> ''::text))
);


ALTER TABLE public.ingredient_sources OWNER TO kuechenteam;

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
-- Name: meta_recipes; Type: TABLE; Schema: public; Owner: kuechenteam
--

CREATE TABLE public.meta_recipes (
    parent_id integer NOT NULL,
    child_id integer NOT NULL,
    weight numeric NOT NULL
);


ALTER TABLE public.meta_recipes OWNER TO kuechenteam;

--
-- Name: COLUMN meta_recipes.weight; Type: COMMENT; Schema: public; Owner: kuechenteam
--

COMMENT ON COLUMN public.meta_recipes.weight IS 'in kg';


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
-- Name: recipe_weight; Type: VIEW; Schema: public; Owner: tobiaswiese
--

CREATE VIEW public.recipe_weight AS
 SELECT recipes.recipe_id,
    recipes.name,
    recipes.comment,
    (COALESCE(sum(meta_recipes.weight), (0)::numeric) + COALESCE(sum((weights.weight * recipe_ingredients.amount)), (0)::numeric)) AS weight
   FROM ((public.recipes
     LEFT JOIN public.meta_recipes ON ((recipes.recipe_id = meta_recipes.parent_id)))
     LEFT JOIN (public.recipe_ingredients
     JOIN public.ingredient_weight weights USING (unit_id, ingredient_id)) USING (recipe_id))
  GROUP BY recipes.recipe_id, recipes.name, recipes.comment;


ALTER TABLE public.recipe_weight OWNER TO tobiaswiese;

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
11	10
10	12
14	3
14	13
14	31
15	41
\.


--
-- Data for Name: ingredient_sources; Type: TABLE DATA; Schema: public; Owner: kuechenteam
--

COPY public.ingredient_sources (ingredient, store, package_size, package_unit, price, url, comment) FROM stdin;
\.


--
-- Data for Name: ingredients; Type: TABLE DATA; Schema: public; Owner: kuechenteam
--

COPY public.ingredients (ingredient_id, name, energy, comment) FROM stdin;
1	Mehl 405	14.590	\N
3	Reis	3.66	\N
2	Ei	0.65	\N
112	Preiselbeerkompott	0	Hilfe ich find nix für energie
7	Olivenöl	37	\N
8	Zwiebeln	1.27	\N
15	Milch	2.78	\N
30	Gouda	15.24	\N
11	Sojahack	16.77	\N
10	Suppengrün	1.29	\N
12	Tomatenmark	1.81	\N
13	Rotwein	2.86	\N
48	Aprikosen	2.96	\N
25	Cayennepfeffer	0.0	\N
26	Pfeffer	0.0	\N
29	Kräutermischung	0.0	\N
17	Lasagneplatten	15	\N
18	Butter	31.01	\N
19	Parmesan	16.58	\N
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
51	Curry	0.0	\N
52	Speck	13.4	\N
53	Paprikapulver	0.0	\N
54	Quark	5.91	\N
55	Zucker	16.97	\N
56	Backpulver	0.0	\N
57	Röstzwiebeln	14.55	\N
58	Schinken	5.38	\N
87	Knoblauchpulver	0.0	\N
90	Muskatpulver	0.0	\N
92	Bunter Pfeffer	0.0	\N
107	Schaschlikspieß	0.0	\N
130	Koriander	0.0	\N
59	Salami	15.72	\N
60	Schmand	8.59	\N
61	Kräuterbutter	12.66	\N
62	Aufbackbrötchen	10.32	\N
63	TK-Blattspinat	0.91	\N
66	Lauch	1.22	\N
67	Petersilie	2.53	\N
68	Zitronensaft	1.17	\N
65	Liebstöckl	2.01	\N
69	Toast	11.17	\N
70	Schinkenscheiben	5.83	\N
71	Senf	3.69	\N
75	Chilipaste	0.0	\N
76	Frühlingszwiebeln\n\n	7.37	\N
77	Karotten	1.63	\N
80	Dosenananas	2.818	\N
81	Sahneschmelzkäse	12.89	\N
82	Lorbeerblätter	0.0	\N
83	Staudensellerie	0.9	\N
84	Schäufele (Schweinefleisch mit Knochen)	9.56	\N
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
106	nö	0	\N
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
9	Emmentaler	15.81	\N
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
145	Marjoran	0.0	\N
146	Hokkaido-Kürbis	208	\N
147	Semmelbrösel	10.01	\N
148	Tellerlinsen	12.2	\N
149	Kichererbsen	11.422	\N
150	Ingwer	2.09	\N
151	Nutella	22.82	\N
152	Apfelmus	3.81	\N
153	NaOHC (Natron)	0.04	\N
40	Spinat	0.93	TK
72	Maccheroni	15.84	\N
74	Honig	12.83	\N
78	Bratwurst	12.11	\N
79	Soßenbinder	16	\N
157	test	4	\N
101	Erdbeeren	1.51	\N
102	Weintrauben	3.03	\N
103	Bananen	3.91	\N
115	Vanillezucker	16.762	\N
116	Butterschmalz	26.86	\N
121	Fleischwurst	12.57	\N
134	Rindergulasch	5.27	\N
135	Sauerteig	12.1	\N
136	Getrocknete Tomaten\n	7.31	\N
137	Kokosmilch	8.68	\N
138	Sojasoße	4.32	\N
139	Currypaste	5.33	\N
140	Zitrone	1.51	\N
141	Asia-Gemüse	1.507	\N
142	Kidneybohnen	4.27	\N
143	Dosenmais	3.52	\N
4	Wasser	0.0	\N
14	Brühepulver	7.35	\N
24	Basilikum	1.97	frisch
22	Crème fraîche	11.59	\N
64	Hefe	0.0	Frisch
144	Chilischoten	0.0	\N
154	TK-Zwetschgen	2.01	\N
155	Gewürzspekulatius	19.11	\N
156	Kardamom	0.0	\N
158	Spätzle	15.727	\N
16	Mozzarella	11	gerieben
159	Sojagranulat	12.9	\N
73	Dosentomaten	0.91	passiert
160	Mehl 550	0	\N
161	Chilipulver	0.0	\N
\.


--
-- Data for Name: meta_recipes; Type: TABLE DATA; Schema: public; Owner: kuechenteam
--

COPY public.meta_recipes (parent_id, child_id, weight) FROM stdin;
11	14	0.450
14	4	0.300
7	11	2.5
7	3	1.15
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
29	14	2
29	30	16
\.


--
-- Data for Name: recipe_ingredients; Type: TABLE DATA; Schema: public; Owner: kuechenteam
--

COPY public.recipe_ingredients (recipe_id, ingredient_id, amount, unit_id) FROM stdin;
6	157	4	0
8	158	2.5	0
8	8	9	5
8	2	18	5
8	9	900	1
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
13	40	900	1
13	41	12	4
13	42	450	1
13	8	2	5
13	2	6	5
13	7	6	4
2	33	0	0
2	32	2	1
2	8	0	0
2	15	2	0
2	1	0	0
3	1	75	1
3	18	75	1
3	15	1	2
3	5	1	11
3	26	1	11
3	90	1	11
4	4	1	2
4	14	4	3
12	20	6	7
14	159	100	1
15	1	1000	1
15	4	650	8
15	135	50	1
15	7	15	1
11	5	1	11
15	5	25	1
15	64	4	1
7	17	500	1
11	26	1	11
7	30	300	1
16	73	500	1
16	24	25	1
16	5	1	11
21	54	120	1
21	22	120	1
21	5	1	11
21	26	1	11
28	160	250	1
28	64	6.25	1
28	4	200	8
23	8	1	5
23	52	50	1
22	8	1.5	5
11	90	1	11
25	9	120	1
24	33	120	1
27	96	300	1
27	114	1	11
26	9	120	1
29	8	2	0
29	6	100	8
29	77	500	1
29	34	1.35	0
29	20	12	7
29	142	4	0
29	143	2.4	0
29	73	8	0
29	161	100	1
29	53	100	1
29	131	75	1
29	37	125	1
29	105	75	1
1	31	5	0
1	27	2	0
1	8	1	0
1	5	1	11
1	26	1	11
1	90	1	11
1	1	75	1
11	73	800	1
11	8	3	5
11	20	2	7
11	7	25	8
\.


--
-- Data for Name: recipes; Type: TABLE DATA; Schema: public; Owner: kuechenteam
--

COPY public.recipes (recipe_id, name, comment) FROM stdin;
5	Apfelmuß	\N
6	TestRezept	\N
8	Käsespätzle	\N
9	Gulasch	\N
10	Bratkartoffeln	\N
12	Kartoffel-Gemüse-Ecken	\N
13	Blätterteigecken	\N
1	Kaisersoße	\N
2	Pilzrahmsoße	\N
3	Bechamelsoße	\N
11	Sojabolognese	\N
4	Gemüsebrühe	\N
14	Sojahack	\N
7	Lasagne	\N
15	Pizzateig	\N
16	Pizzasoße	\N
17	Pizza	\N
18	Flammkuchen_p	Mit Pizzateig
20	Flammkuchen	\N
21	Flammkuchencreme	\N
22	Flammkuchen_v	Vegetarisch
23	Flammkuchen_s	Standard
24	Flammkuchen_f	Forèstier
25	Flammkuchen_k	Käse
26	Flammkuchen_fk	Forèstier+Käse
27	Flammkuchen_a	Apfel
28	Flammkuchenteig	\N
29	Chili sin carne	\N
30	Chili_base	\N
\.


--
-- Data for Name: stores; Type: TABLE DATA; Schema: public; Owner: kuechenteam
--

COPY public.stores (store_id, name) FROM stdin;
0	Metro
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
\.


--
-- Data for Name: weights; Type: TABLE DATA; Schema: public; Owner: kuechenteam
--

COPY public.weights (unit_id, ingredient_id, weight) FROM stdin;
5	2	0.058
\.


--
-- Name: food_properties_property_id_seq; Type: SEQUENCE SET; Schema: public; Owner: kuechenteam
--

SELECT pg_catalog.setval('public.food_properties_property_id_seq', 48, true);


--
-- Name: ingredients_ingredient_id_seq; Type: SEQUENCE SET; Schema: public; Owner: kuechenteam
--

SELECT pg_catalog.setval('public.ingredients_ingredient_id_seq', 161, true);


--
-- Name: recipes_recipe_id_seq; Type: SEQUENCE SET; Schema: public; Owner: kuechenteam
--

SELECT pg_catalog.setval('public.recipes_recipe_id_seq', 30, true);


--
-- Name: stores_store_id_seq; Type: SEQUENCE SET; Schema: public; Owner: kuechenteam
--

SELECT pg_catalog.setval('public.stores_store_id_seq', 1, false);


--
-- Name: units_unit_id_seq; Type: SEQUENCE SET; Schema: public; Owner: kuechenteam
--

SELECT pg_catalog.setval('public.units_unit_id_seq', 1, true);


--
-- Name: base_conversions conversion_pk; Type: CONSTRAINT; Schema: public; Owner: kuechenteam
--

ALTER TABLE ONLY public.base_conversions
    ADD CONSTRAINT conversion_pk PRIMARY KEY (from_unit, to_unit);


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
    ADD CONSTRAINT ingredient_source_pk PRIMARY KEY (ingredient, store, package_size, package_unit);


--
-- Name: ingredients ingredients_pk; Type: CONSTRAINT; Schema: public; Owner: kuechenteam
--

ALTER TABLE ONLY public.ingredients
    ADD CONSTRAINT ingredients_pk PRIMARY KEY (ingredient_id);


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
-- Name: meta_recipes child_fk; Type: FK CONSTRAINT; Schema: public; Owner: kuechenteam
--

ALTER TABLE ONLY public.meta_recipes
    ADD CONSTRAINT child_fk FOREIGN KEY (child_id) REFERENCES public.recipes(recipe_id);


--
-- Name: base_conversions from_unit_fk; Type: FK CONSTRAINT; Schema: public; Owner: kuechenteam
--

ALTER TABLE ONLY public.base_conversions
    ADD CONSTRAINT from_unit_fk FOREIGN KEY (from_unit) REFERENCES public.units(unit_id) MATCH FULL ON UPDATE CASCADE ON DELETE SET NULL;


--
-- Name: ingredient_sources ingredient_fk; Type: FK CONSTRAINT; Schema: public; Owner: kuechenteam
--

ALTER TABLE ONLY public.ingredient_sources
    ADD CONSTRAINT ingredient_fk FOREIGN KEY (ingredient) REFERENCES public.ingredients(ingredient_id);


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
    ADD CONSTRAINT package_unit_fk FOREIGN KEY (package_unit) REFERENCES public.units(unit_id);


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
-- Name: ingredient_sources store_fk; Type: FK CONSTRAINT; Schema: public; Owner: kuechenteam
--

ALTER TABLE ONLY public.ingredient_sources
    ADD CONSTRAINT store_fk FOREIGN KEY (store) REFERENCES public.stores(store_id);


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
-- PostgreSQL database dump complete
--

