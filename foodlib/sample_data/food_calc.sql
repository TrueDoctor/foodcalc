--
-- PostgreSQL database dump
--

-- Dumped from database version 16.1
-- Dumped by pg_dump version 16.1

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
-- Data for Name: _sqlx_migrations; Type: TABLE DATA; Schema: public; Owner: dennis
--

COPY public._sqlx_migrations (version, description, installed_on, success, checksum, execution_time) FROM stdin;
20230719115112	init	2024-01-31 00:36:39.295512+01	t	\\xacddfcb4d0e946d0a661e71b2ec5a946ed51851ba0e998332c34da39b2d2598b17dd11d3131becf250172425f8de6f9a	90997298
20230719120414	add users	2024-01-31 00:36:39.38785+01	t	\\xf2a37a1f8fc198e4257b16a22f23f22d226447214c5f4a1869cd36139b226eb1793a53fc99af5d483b4ea32b9f3bf483	18497476
20231120140016	add inventory	2024-01-31 00:36:39.407231+01	t	\\xf921704bbc1b4d98d99872390256f101930649f93fe0155e9b1ba0b79f6af5c94141cda22ab5d6589300056819be86b3	9113629
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
-- Data for Name: base_conversions; Type: TABLE DATA; Schema: public; Owner: kuechenteam
--

COPY public.base_conversions (from_unit, to_unit, from_amount, to_amount) FROM stdin;
\.


--
-- Data for Name: event_inventories; Type: TABLE DATA; Schema: public; Owner: dennis
--

COPY public.event_inventories (event_id, inventory_id) FROM stdin;
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
42	Nico Chili	\N	\N
43	Fachschafsseminar	\N	300,00 €
44	Seminar WS23	\N	400,00 €
45	Seminar WS23 Test	\N	400,00 €
46	Seminar WS23/24	\N	400,00 €
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
4	Gemüsebrühe	\N
93	Kartoffelsuppe	FSMI-Kartoffelsuppe (TODO: Verhältnisse anpassen. Wurde eher Karottensuppe.)
94	Semmelknödel	https://emmikochteinfach.de/klassische-semmelknoedel/
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
267	Pilzrahmsoße (Vegan)	Wie normale aber vegan
270	Nudeln mit Pesto	\N
271	Bessere Gemüsebrühe	Ohne Fertigbrühe, aber mit Liebe (Gewürze noch nicht nachgewogen)
2	Pilzrahmsoße	Funktioniert auch gut mit Granatapfel
274	Semmelknödel (Vegan)	\N
96	Semmelknödel mit Soßen (Vegan)	
80	Seminarfrühstück	\N
273	Brühe mit Einlage	\N
280	Käse-Mix für Vesper	\N
58	Nudeln	\N
278	Rotkraut	\N
41	Rührei	\N
282	Kochendes Wasser	Wasser, aber gekocht :)
\.


--
-- Data for Name: event_meals; Type: TABLE DATA; Schema: public; Owner: kuechenteam
--

COPY public.event_meals (event_id, recipe_id, place_id, comment, energy_per_serving, servings, start_time, end_time) FROM stdin;
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
-- Data for Name: groups; Type: TABLE DATA; Schema: public; Owner: dennis
--

COPY public.groups (id, name) FROM stdin;
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
247	Pesto	14.9	\N
248	Pastinake	2.66	\N
249	Lauch	1.24	\N
250	Maultaschen	8.37	\N
251	Sojamilch	2.3	\N
252	Rotkohl	0.95	\N
253	Brie	15	\N
254	Marmelade	11	\N
96	Äpfel	2.7100	\N
255	Nelken	18.09	\N
256	Aqua Faber	0.57	\N
257	Zwiebelschmelz vegan	32.99	\N
258	MSG	12.26	\N
\.


--
-- Data for Name: ingredient_properties; Type: TABLE DATA; Schema: public; Owner: kuechenteam
--

COPY public.ingredient_properties (ingredient_id, property_id) FROM stdin;
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
-- Data for Name: ingredient_sources; Type: TABLE DATA; Schema: public; Owner: kuechenteam
--

COPY public.ingredient_sources (ingredient_id, store_id, package_size, unit_id, price, url, comment) FROM stdin;
\.


--
-- Data for Name: inventories; Type: TABLE DATA; Schema: public; Owner: dennis
--

COPY public.inventories (inventory_id, name) FROM stdin;
\.


--
-- Data for Name: inventory_ingredients; Type: TABLE DATA; Schema: public; Owner: dennis
--

COPY public.inventory_ingredients (inventory_id, ingredient_id, amount) FROM stdin;
\.


--
-- Data for Name: meta_recipes; Type: TABLE DATA; Schema: public; Owner: kuechenteam
--

COPY public.meta_recipes (parent_id, child_id, weight) FROM stdin;
\.


--
-- Data for Name: recipe_ingredients; Type: TABLE DATA; Schema: public; Owner: kuechenteam
--

COPY public.recipe_ingredients (recipe_id, ingredient_id, amount, unit_id) FROM stdin;
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
75	1	Schritt 1	Wasser kochen	00:00:00	00:00:00	4
76	2	Schritt 2	Brühepulver zugeben	00:00:00	00:00:00	4
249	1	Vorheizen	Den Ofen auf 250 - 275 °C Ober-/Unterhitze aufheizen.	00:20:00	00:00:00	251
250	2	Glasieren	Die Kanelbullar (Zimtschnecken) mit verschlagenem Ei bepinseln. Anschließend mit Zucker bestreuen.	00:02:00	00:02:00	251
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
278	1	Zusammenrühren	Gekochte Nudeln in vorbereitete Form mit Pesto tun, verrühren, Tomaten untermischen	00:01:00	00:00:03	270
295	1	Schritt 1	Zwiebeln und Champignons anbraten (am besten in mehreren Fuhren)	00:00:00	00:00:00	2
296	2	Schritt 2	Milch dazu, Schlagsahne dazu, Mehl dazu	00:00:00	00:00:00	2
297	3	Schritt 3	Abschmecken	00:00:00	00:00:00	2
301	1	Schnippeln	Suppengemüse fein würfeln	00:01:00	00:05:00	271
302	2	Andünsten	Gemüse + Gewürze in Öl (man könnte auch Butter nehmen) anschwitzen	00:05:00	00:01:24	271
303	3	Fertig machen	Mit Wasser ablöschen und so lang köcheln lassen wie gewünscht (Wird ziemlich lange immer intensiver)	00:00:00	00:03:00	271
304	1	Schneiden	Kartoffeln in 1-2cm-Würfel schneiden (evtl. vorher schälen)	00:01:00	00:02:00	273
305	2	Kochen	Kartoffeln in heiße Brühe werfen und kochen, Maultaschen ein paar Minuten vor Servieren dazu (Sollte zu dem Zeitpunkt sprudelnd kochen)	00:00:30	00:01:00	273
312	1	Kochen	Topf + Wasser + Warm	00:04:12	00:01:24	282
314	1	Schritt 1	Zwiebeln klein hacken	00:00:00	00:00:00	41
315	2	Schritt 2	Alles verquirlen	00:00:00	00:00:00	41
316	3	Schritt 3	Im Gasbräter braten. Ihr habt schonmal Rührei gemacht.	00:00:00	00:00:00	41
317	1	Schritt 1	Kocht Nudeln. Ihr wisst, wie man Nudeln kocht.	00:08:00	00:00:00	58
\.


--
-- Data for Name: users; Type: TABLE DATA; Schema: public; Owner: dennis
--

COPY public.users (id, username, email, password_hash, is_admin, created_at) FROM stdin;
\.


--
-- Data for Name: user_groups; Type: TABLE DATA; Schema: public; Owner: dennis
--

COPY public.user_groups (user_id, group_id) FROM stdin;
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

SELECT pg_catalog.setval('public.events_event_id_seq', 46, true);


--
-- Name: food_properties_property_id_seq; Type: SEQUENCE SET; Schema: public; Owner: kuechenteam
--

SELECT pg_catalog.setval('public.food_properties_property_id_seq', 48, true);


--
-- Name: groups_id_seq; Type: SEQUENCE SET; Schema: public; Owner: dennis
--

SELECT pg_catalog.setval('public.groups_id_seq', 1, false);


--
-- Name: ingredients_ingredient_id_seq; Type: SEQUENCE SET; Schema: public; Owner: kuechenteam
--

SELECT pg_catalog.setval('public.ingredients_ingredient_id_seq', 258, true);


--
-- Name: inventories_inventory_id_seq; Type: SEQUENCE SET; Schema: public; Owner: dennis
--

SELECT pg_catalog.setval('public.inventories_inventory_id_seq', 1, false);


--
-- Name: places_place_id_seq; Type: SEQUENCE SET; Schema: public; Owner: kuechenteam
--

SELECT pg_catalog.setval('public.places_place_id_seq', 8, true);


--
-- Name: recipes_recipe_id_seq; Type: SEQUENCE SET; Schema: public; Owner: kuechenteam
--

SELECT pg_catalog.setval('public.recipes_recipe_id_seq', 289, true);


--
-- Name: steps_step_id_seq; Type: SEQUENCE SET; Schema: public; Owner: kuechenteam
--

SELECT pg_catalog.setval('public.steps_step_id_seq', 317, true);


--
-- Name: stores_store_id_seq; Type: SEQUENCE SET; Schema: public; Owner: kuechenteam
--

SELECT pg_catalog.setval('public.stores_store_id_seq', 1, false);


--
-- Name: units_unit_id_seq; Type: SEQUENCE SET; Schema: public; Owner: kuechenteam
--

SELECT pg_catalog.setval('public.units_unit_id_seq', 12, true);


--
-- Name: users_id_seq; Type: SEQUENCE SET; Schema: public; Owner: dennis
--

SELECT pg_catalog.setval('public.users_id_seq', 1, false);


--
-- PostgreSQL database dump complete
--

REFRESH MATERIALIZED VIEW conversions;
