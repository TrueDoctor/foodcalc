--
-- PostgreSQL database dump
--

-- Dumped from database version 16.3
-- Dumped by pg_dump version 16.3

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

SET session_replication_role = replica;

--
-- Data for Name: units; Type: TABLE DATA; Schema: public; Owner: dennis
--

INSERT INTO public.units VALUES (0, 'kg');
INSERT INTO public.units VALUES (1, 'g');
INSERT INTO public.units VALUES (2, 'l');
INSERT INTO public.units VALUES (3, 'TL');
INSERT INTO public.units VALUES (4, 'EL');
INSERT INTO public.units VALUES (5, 'Stk');
INSERT INTO public.units VALUES (6, 'Knolle');
INSERT INTO public.units VALUES (7, 'Zehe');
INSERT INTO public.units VALUES (8, 'ml');
INSERT INTO public.units VALUES (9, 'Bund');
INSERT INTO public.units VALUES (10, 'Pck');
INSERT INTO public.units VALUES (11, 'Prise');
INSERT INTO public.units VALUES (12, 'Scheibe');


--
-- Data for Name: base_conversions; Type: TABLE DATA; Schema: public; Owner: dennis
--

INSERT INTO public.base_conversions VALUES (1, 0, 1000, 1);
INSERT INTO public.base_conversions VALUES (2, 0, 1, 1);
INSERT INTO public.base_conversions VALUES (3, 8, 1, 5);
INSERT INTO public.base_conversions VALUES (4, 3, 1, 3);
INSERT INTO public.base_conversions VALUES (8, 1, 1, 1);
INSERT INTO public.base_conversions VALUES (0, 0, 1, 1);


--
-- Data for Name: event_inventories; Type: TABLE DATA; Schema: public; Owner: dennis
--

INSERT INTO public.event_inventories VALUES (52, 0);
INSERT INTO public.event_inventories VALUES (52, 1);


--
-- Data for Name: events; Type: TABLE DATA; Schema: public; Owner: dennis
--

INSERT INTO public.events VALUES (4, 'Seminar SS22', NULL, 670.00);
INSERT INTO public.events VALUES (5, 'Eulenfest SS22', NULL, 300.00);
INSERT INTO public.events VALUES (6, 'Los Ämmerles 2022', NULL, NULL);
INSERT INTO public.events VALUES (7, 'Filmdreh HFG', NULL, 60.00);
INSERT INTO public.events VALUES (3, 'Dummy', 'Testcomment', 2000.00);
INSERT INTO public.events VALUES (9, 'SAT WS22', '1600-1800kJpP + Baguette besser als 2400kJpP', 300.00);
INSERT INTO public.events VALUES (16, 'Tour de FSK', NULL, 300.00);
INSERT INTO public.events VALUES (17, 'Skiurlaub', NULL, 200.00);
INSERT INTO public.events VALUES (18, 'Backwerkstand', 'Ausprobieren, wie viel wir in welcher Zeit backen können', 100.00);
INSERT INTO public.events VALUES (0, 'Unifest22', '', 7000.00);
INSERT INTO public.events VALUES (30, 'Ùnifest 23 Nachkochen', NULL, 3000.00);
INSERT INTO public.events VALUES (20, 'Pita+Hummus Probebacken', NULL, NULL);
INSERT INTO public.events VALUES (21, 'SAT SS23', NULL, 300.00);
INSERT INTO public.events VALUES (22, 'Game Jam', NULL, 60.00);
INSERT INTO public.events VALUES (10, 'Seminar WS22/23', NULL, 550.00);
INSERT INTO public.events VALUES (23, 'Squeak Hackathon', NULL, 100.00);
INSERT INTO public.events VALUES (31, 'Seminar SS23', NULL, 400.00);
INSERT INTO public.events VALUES (8, 'Test Event 2', NULL, NULL);
INSERT INTO public.events VALUES (29, 'Eulenfest SS23 (Externe Fütterung)', 'Immoment noch kein Inhalt hier', 750.00);
INSERT INTO public.events VALUES (33, 'VS Wahlen', NULL, 50.00);
INSERT INTO public.events VALUES (32, 'Eulenfest 2023', NULL, 500.00);
INSERT INTO public.events VALUES (34, 'Minifreizeit', NULL, 100.00);
INSERT INTO public.events VALUES (19, 'Unifest 23', 'IST SCHON NÄCHSTE WOCHE PANIK!!!', 8500.00);
INSERT INTO public.events VALUES (36, 'Gasque', NULL, NULL);
INSERT INTO public.events VALUES (39, 'Lennart Luisa 50.', NULL, 80.00);
INSERT INTO public.events VALUES (41, 'Kob Naturfreunde Helferfest', NULL, NULL);
INSERT INTO public.events VALUES (40, 'SAT WS 23/24', NULL, NULL);
INSERT INTO public.events VALUES (46, 'Seminar WS23/24', NULL, 400.00);
INSERT INTO public.events VALUES (48, 'StuKo-Treffen', NULL, NULL);
INSERT INTO public.events VALUES (42, 'Nico Chili', NULL, NULL);
INSERT INTO public.events VALUES (50, 'SAT SS24', NULL, 140.00);
INSERT INTO public.events VALUES (49, 'Lila Pause Generationentreffen', NULL, 100.00);
INSERT INTO public.events VALUES (38, 'Unifest SS24', NULL, 8500.00);
INSERT INTO public.events VALUES (51, 'Eulenfest24-E&T', NULL, NULL);
INSERT INTO public.events VALUES (53, 'Seminar SS24', '', 350.00);
INSERT INTO public.events VALUES (52, 'Eulenfest Catering 24', 'Budget geraten', 460.00);
INSERT INTO public.events VALUES (47, 'Eulenfest24-Extern - TEST EVENT', 'Wichtig: Gäste-Essen, nicht Helfer-Essen (', 1.00);
INSERT INTO public.events VALUES (54, 'Montagsgrillen', '', 8.00);
INSERT INTO public.events VALUES (55, 'SAT WS24/25', '', 2.00);
INSERT INTO public.events VALUES (56, 'Seminar WS24', NULL, 50.00);
INSERT INTO public.events VALUES (58, 'Seminar vorkochen 2', 'Menu Mexican (Französich aussprechen)', 0.50);
INSERT INTO public.events VALUES (57, 'Seminar vorkochen 1', 'Suppe+Risotto', 50.00);


--
-- Data for Name: places; Type: TABLE DATA; Schema: public; Owner: dennis
--

INSERT INTO public.places VALUES (1, 'Akk Halle', NULL);
INSERT INTO public.places VALUES (2, 'Akk Tribüne', NULL);
INSERT INTO public.places VALUES (3, 'Unifest Gelände', NULL);
INSERT INTO public.places VALUES (4, 'Seminarhütte', NULL);
INSERT INTO public.places VALUES (5, '-118 Infobau', NULL);
INSERT INTO public.places VALUES (6, 'Mensa GS Ammerbuch', NULL);
INSERT INTO public.places VALUES (7, 'ZKM', NULL);
INSERT INTO public.places VALUES (8, 'Infobau Draußen', 'Unter dem Dach');
INSERT INTO public.places VALUES (9, 'Lila Haus', NULL);


--
-- Data for Name: recipes; Type: TABLE DATA; Schema: public; Owner: dennis
--

INSERT INTO public.recipes VALUES (9, 'Gulasch', NULL);
INSERT INTO public.recipes VALUES (10, 'Bratkartoffeln', NULL);
INSERT INTO public.recipes VALUES (12, 'Kartoffel-Gemüse-Ecken', NULL);
INSERT INTO public.recipes VALUES (13, 'Blätterteigecken', NULL);
INSERT INTO public.recipes VALUES (16, 'Pizzasoße', NULL);
INSERT INTO public.recipes VALUES (17, 'Pizza', NULL);
INSERT INTO public.recipes VALUES (18, 'Flammkuchen_p', 'Mit Pizzateig');
INSERT INTO public.recipes VALUES (20, 'Flammkuchen', NULL);
INSERT INTO public.recipes VALUES (21, 'Flammkuchencreme', NULL);
INSERT INTO public.recipes VALUES (31, 'Quiche', NULL);
INSERT INTO public.recipes VALUES (32, 'Quicheteig', NULL);
INSERT INTO public.recipes VALUES (38, 'Linsensuppe', NULL);
INSERT INTO public.recipes VALUES (22, 'Flammkuchen Vegetarisch', NULL);
INSERT INTO public.recipes VALUES (23, 'Flammkuchen Standard', NULL);
INSERT INTO public.recipes VALUES (24, 'Flammkuchen Forèstier', NULL);
INSERT INTO public.recipes VALUES (25, 'Flammkuchen Käse', NULL);
INSERT INTO public.recipes VALUES (26, 'Flammkuchen Forèstier + Käse', NULL);
INSERT INTO public.recipes VALUES (27, 'Flammkuchen Apfel', NULL);
INSERT INTO public.recipes VALUES (42, 'Veganer Porridge', NULL);
INSERT INTO public.recipes VALUES (43, 'Frühstück', NULL);
INSERT INTO public.recipes VALUES (44, 'Pizza Funghi', NULL);
INSERT INTO public.recipes VALUES (45, 'Pizza Prosciutto', NULL);
INSERT INTO public.recipes VALUES (46, 'Pizza Margherita', NULL);
INSERT INTO public.recipes VALUES (47, 'Pizza Vegana', NULL);
INSERT INTO public.recipes VALUES (48, 'Pizza Vegetariana', NULL);
INSERT INTO public.recipes VALUES (49, 'Pizza Salame', NULL);
INSERT INTO public.recipes VALUES (33, 'Spinat-Tomaten-Quiche', 'Nachschauen, ob das nicht zu viel Tomaten sind');
INSERT INTO public.recipes VALUES (51, 'Pilzrisotto', NULL);
INSERT INTO public.recipes VALUES (54, 'Griechischer Bauernsalat', NULL);
INSERT INTO public.recipes VALUES (56, 'GPN-Gulasch Vegan', NULL);
INSERT INTO public.recipes VALUES (57, 'GPN-Gulasch', NULL);
INSERT INTO public.recipes VALUES (59, 'Reis', NULL);
INSERT INTO public.recipes VALUES (60, 'Pasta bolognese', NULL);
INSERT INTO public.recipes VALUES (61, 'Chili con Reis', 'Vegan');
INSERT INTO public.recipes VALUES (63, 'Pizza mix', NULL);
INSERT INTO public.recipes VALUES (64, 'Reis-Nudel-Buffet', NULL);
INSERT INTO public.recipes VALUES (36, 'Tomaten-Mozzarella-Fladenbrot', NULL);
INSERT INTO public.recipes VALUES (70, 'Hefezopf', NULL);
INSERT INTO public.recipes VALUES (71, 'Obst (Snack)', NULL);
INSERT INTO public.recipes VALUES (72, 'Snacks', NULL);
INSERT INTO public.recipes VALUES (73, 'Veganer Parmesan', NULL);
INSERT INTO public.recipes VALUES (74, 'Curry', NULL);
INSERT INTO public.recipes VALUES (75, 'Curry mit Reis', NULL);
INSERT INTO public.recipes VALUES (81, 'Nudelsalat', 'https://kochkarussell.com/italienischer-nudelsalat-einfach/');
INSERT INTO public.recipes VALUES (79, 'Stockbrot', 'https://www.einfachbacken.de/rezepte/stockbrot-schnelles-grundrezept');
INSERT INTO public.recipes VALUES (83, 'Grillgemüse', NULL);
INSERT INTO public.recipes VALUES (84, 'Schokobananen', NULL);
INSERT INTO public.recipes VALUES (85, 'Caprese', NULL);
INSERT INTO public.recipes VALUES (86, 'Kartoffeln In Bechamelsauce', 'https://docs.google.com/spreadsheets/d/12qS3gSCdPiNNX9lGgZVcZmNQXt8Z8E9XrmAiDaIz07w/edit?usp=sharing');
INSERT INTO public.recipes VALUES (87, 'Nudeln mit Bolognese', NULL);
INSERT INTO public.recipes VALUES (90, 'Spielstadt-Pizzateig', NULL);
INSERT INTO public.recipes VALUES (92, 'Sauerteig', NULL);
INSERT INTO public.recipes VALUES (4, 'Gemüsebrühe', NULL);
INSERT INTO public.recipes VALUES (93, 'Kartoffelsuppe', 'FSMI-Kartoffelsuppe (TODO: Verhältnisse anpassen. Wurde eher Karottensuppe.)');
INSERT INTO public.recipes VALUES (94, 'Semmelknödel', 'https://emmikochteinfach.de/klassische-semmelknoedel/');
INSERT INTO public.recipes VALUES (97, 'Spätzle mit Soßen', '');
INSERT INTO public.recipes VALUES (98, 'Brokkolisoße', 'vegan');
INSERT INTO public.recipes VALUES (99, 'Kartoffelklöße', NULL);
INSERT INTO public.recipes VALUES (100, 'Gulasch mit Klößen', NULL);
INSERT INTO public.recipes VALUES (29, 'Chili sin carne', 'Beim Seminar (WS22\23) würden abens 2166 kj pro person + 600kj pp baguette gegessen');
INSERT INTO public.recipes VALUES (30, 'Chili_base', 'kalibriert (Gewürze)');
INSERT INTO public.recipes VALUES (101, 'Spätzle mit Pilzrahmsoße', NULL);
INSERT INTO public.recipes VALUES (103, 'Hummus', 'Gewürzmengen unkalibriert');
INSERT INTO public.recipes VALUES (1, 'Kaisersoße', NULL);
INSERT INTO public.recipes VALUES (62, 'Flammkuchen mix', NULL);
INSERT INTO public.recipes VALUES (191, 'Bananencurry-Wrap mit Reis', 'Erstellt für das Eulenfest 2023');
INSERT INTO public.recipes VALUES (227, 'Crispy-Nugget-Dal mit Reis', NULL);
INSERT INTO public.recipes VALUES (15, 'Pizzateig', NULL);
INSERT INTO public.recipes VALUES (35, 'Schinken-Käse-Fladenbrot', NULL);
INSERT INTO public.recipes VALUES (76, 'Naan', 'https://www.eat-this.org/veganes-naan-brot/');
INSERT INTO public.recipes VALUES (213, 'Fake-Curry-Hähnchen', NULL);
INSERT INTO public.recipes VALUES (216, 'Knödelbase (Vegan)', NULL);
INSERT INTO public.recipes VALUES (78, 'Grillen', NULL);
INSERT INTO public.recipes VALUES (186, 'Ratatouille', NULL);
INSERT INTO public.recipes VALUES (66, 'Salat Mix', NULL);
INSERT INTO public.recipes VALUES (37, 'Tofu-Hummus-Fladenbrot', 'Hälfte mit, Hälfte ohne Tomaten');
INSERT INTO public.recipes VALUES (55, 'Gemischter Salat + Dressing', NULL);
INSERT INTO public.recipes VALUES (68, 'Gulasch mit Beilagen', 'Spätzle und Reis');
INSERT INTO public.recipes VALUES (52, 'Sojaschnetzel', NULL);
INSERT INTO public.recipes VALUES (136, 'Bananencurry', NULL);
INSERT INTO public.recipes VALUES (203, 'Bananencurry mit Tofu', NULL);
INSERT INTO public.recipes VALUES (108, 'Sauerkrauteintopf', 'ggf. Würstchen als Beilage ergänzen');
INSERT INTO public.recipes VALUES (50, 'Quiche Lorraine', NULL);
INSERT INTO public.recipes VALUES (7, 'Lasagne', NULL);
INSERT INTO public.recipes VALUES (250, 'Zimtschneckenfüllung', NULL);
INSERT INTO public.recipes VALUES (102, 'Pita', '');
INSERT INTO public.recipes VALUES (53, 'GPN-Gulasch_base', NULL);
INSERT INTO public.recipes VALUES (28, 'Flammkuchenteig', NULL);
INSERT INTO public.recipes VALUES (89, 'Sandwich Toast Schinken-Käse', NULL);
INSERT INTO public.recipes VALUES (88, 'Sandwich Toast Käse', NULL);
INSERT INTO public.recipes VALUES (3, 'Bechamelsoße', NULL);
INSERT INTO public.recipes VALUES (91, 'Spielstadt-Pizza', NULL);
INSERT INTO public.recipes VALUES (34, 'Fladenbrot (beschmiert)', NULL);
INSERT INTO public.recipes VALUES (251, 'Zimtschnecken', NULL);
INSERT INTO public.recipes VALUES (224, 'Rote-Linsen-Dal', NULL);
INSERT INTO public.recipes VALUES (67, 'Quiche mix', NULL);
INSERT INTO public.recipes VALUES (69, 'Fladenbrot Mix', NULL);
INSERT INTO public.recipes VALUES (39, 'Kartoffelgulasch', NULL);
INSERT INTO public.recipes VALUES (228, 'Ofengemüse', NULL);
INSERT INTO public.recipes VALUES (204, 'Bananencurry mit Fake-Hähnchen', NULL);
INSERT INTO public.recipes VALUES (225, 'Sojanuggets (Crispy)', 'Paniert und Crispy, mit orientalischen Gewürzen. DIE NOCH AUSGEWOGEN WERDEN MÜSSEN @SEMINAR (auch der andere shit)');
INSERT INTO public.recipes VALUES (231, 'Zimtschneckenteig', NULL);
INSERT INTO public.recipes VALUES (229, 'Tomatensoße', NULL);
INSERT INTO public.recipes VALUES (214, 'Schupfnudeln mit Apfelmus', NULL);
INSERT INTO public.recipes VALUES (139, 'Bananencurry mit Reis', NULL);
INSERT INTO public.recipes VALUES (230, 'Nudeln mit Tomatensoße und Ofengemüse', NULL);
INSERT INTO public.recipes VALUES (200, 'Möhren-Orangen-Suppe (Vegan)', 'Vegane Margarine!!! (abgewogen)');
INSERT INTO public.recipes VALUES (265, 'Spätzle mit Köttbullar', NULL);
INSERT INTO public.recipes VALUES (215, 'Vegane Knödel base', NULL);
INSERT INTO public.recipes VALUES (266, 'Baguettescheiben', NULL);
INSERT INTO public.recipes VALUES (223, 'Möhren-Orangen-Suppe mit Baguette (vegan)', NULL);
INSERT INTO public.recipes VALUES (267, 'Pilzrahmsoße (Vegan)', 'Wie normale aber vegan');
INSERT INTO public.recipes VALUES (270, 'Nudeln mit Pesto', NULL);
INSERT INTO public.recipes VALUES (271, 'Bessere Gemüsebrühe', 'Ohne Fertigbrühe, aber mit Liebe (Gewürze noch nicht nachgewogen)');
INSERT INTO public.recipes VALUES (274, 'Semmelknödel (Vegan)', NULL);
INSERT INTO public.recipes VALUES (96, 'Semmelknödel mit Soßen (Vegan)', '');
INSERT INTO public.recipes VALUES (80, 'Seminarfrühstück', NULL);
INSERT INTO public.recipes VALUES (273, 'Brühe mit Einlage', NULL);
INSERT INTO public.recipes VALUES (280, 'Käse-Mix für Vesper', NULL);
INSERT INTO public.recipes VALUES (58, 'Nudeln', NULL);
INSERT INTO public.recipes VALUES (41, 'Rührei', NULL);
INSERT INTO public.recipes VALUES (282, 'Kochendes Wasser', 'Wasser, aber gekocht :)');
INSERT INTO public.recipes VALUES (278, 'Rotkraut', NULL);
INSERT INTO public.recipes VALUES (14, 'Sojahack', NULL);
INSERT INTO public.recipes VALUES (290, 'Sojahack Bolognese', NULL);
INSERT INTO public.recipes VALUES (11, 'Sojabolognese', NULL);
INSERT INTO public.recipes VALUES (293, 'Kokos Korma', '');
INSERT INTO public.recipes VALUES (106, 'Hummus-Pita', NULL);
INSERT INTO public.recipes VALUES (294, 'Kokos Korma mit Reis', '');
INSERT INTO public.recipes VALUES (295, 'Süßkartoffel Kichererbsen Spinat Matsch', '');
INSERT INTO public.recipes VALUES (296, 'Pflaumenkompott', '');
INSERT INTO public.recipes VALUES (297, 'Süßkartoffel Teigtaschen', '');
INSERT INTO public.recipes VALUES (82, 'Kartoffelsalat', 'der Südliche/~Falsche~/Nasse');
INSERT INTO public.recipes VALUES (298, 'Süße Teigtaschen', '');
INSERT INTO public.recipes VALUES (300, 'Tofu (gebraten)', 'gebraten');
INSERT INTO public.recipes VALUES (307, 'Teigtaschen mix', '');
INSERT INTO public.recipes VALUES (308, 'Müsli', '');
INSERT INTO public.recipes VALUES (309, 'Unifest Frühstück', '');
INSERT INTO public.recipes VALUES (292, 'Chili in Wrap', NULL);
INSERT INTO public.recipes VALUES (2, 'Pilzrahmsoße', 'Hat auch mal gut mit Granatapfel funktioniert');
INSERT INTO public.recipes VALUES (310, 'Crepeteig', 'FSMI Rezept, angelegt für das Eulenfest 2024');
INSERT INTO public.recipes VALUES (311, 'Souvlaki', 'Noch nicht eingewohen, Erstellt für Eulenfest 24 Externe Fütterung');
INSERT INTO public.recipes VALUES (312, 'Souvlaki-Vegetarisch', 'Noch nicht eingewohen, Erstellt für Eulenfest 24 Externe Fütterung');
INSERT INTO public.recipes VALUES (77, 'Curry mit Nudeln und Reis', 'pls ändern eigentlich mit naan');
INSERT INTO public.recipes VALUES (313, 'Erbsenmus', 'Dummy');
INSERT INTO public.recipes VALUES (316, 'Erdnussbuttersoße', '');
INSERT INTO public.recipes VALUES (314, 'Nudeln mit Erbsenmus', '');
INSERT INTO public.recipes VALUES (315, 'Blumenkohl Erdnuss nudeln', 'Chili, salz geraten');
INSERT INTO public.recipes VALUES (317, 'Dampfnudeln', '');
INSERT INTO public.recipes VALUES (318, 'Dampfnudelnteig', '');
INSERT INTO public.recipes VALUES (319, 'Salatwraps', '');
INSERT INTO public.recipes VALUES (320, 'Süßkartoffel Wraps', '');
INSERT INTO public.recipes VALUES (321, 'Mexikanisches Menü', '');
INSERT INTO public.recipes VALUES (322, 'Rote-Beete-Gerstenrisotto mit Kräuterricotta', '');
INSERT INTO public.recipes VALUES (324, 'Lauchsuppe', '');
INSERT INTO public.recipes VALUES (323, 'Nudelsuppe', 'Gerne mit Buchstabennudeln für Junggebliebene');
INSERT INTO public.recipes VALUES (8, 'Käsespätzle (Badisch)', 'Badisch = Mit Sahne');
INSERT INTO public.recipes VALUES (40, 'Käsespätzle (Schwäbisch)', 'Schwäbisch = Ohne Sahne');
INSERT INTO public.recipes VALUES (326, 'Kräuterricotta', NULL);
INSERT INTO public.recipes VALUES (327, 'Rote-Beete-Gerstenrisotto', 'Ein bisschen langweilig ohne Kräuterricotta');
INSERT INTO public.recipes VALUES (329, 'Test', 'cool');
INSERT INTO public.recipes VALUES (331, 'Test Recipe', 'Test Comment');


--
-- Data for Name: event_meals; Type: TABLE DATA; Schema: public; Owner: dennis
--

INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (16, 29, 2, NULL, 1200, 120, 1, '2022-12-20 19:00:00+00', '2022-12-20 20:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (17, 93, 4, NULL, 800, 8, 2, '2022-01-03 19:00:00+00', '2022-01-03 19:30:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (0, 75, 1, NULL, 6000, 100, 3, '2022-06-18 12:30:00+00', '2022-06-18 14:30:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (4, 80, 4, NULL, 1500, 36, 4, '2022-07-02 08:00:00+00', '2022-07-02 10:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (4, 80, 4, NULL, 1500, 36, 5, '2022-07-03 08:00:00+00', '2022-07-03 10:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (4, 61, 4, NULL, 4000, 36, 6, '2022-07-01 19:00:00+00', '2022-07-01 20:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (9, 93, 8, NULL, 2400, 40, 7, '2022-11-01 19:30:00+00', '2022-11-01 21:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (4, 77, 4, NULL, 4000, 36, 8, '2022-07-02 13:00:00+00', '2022-07-02 14:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (9, 29, 8, '', 2600, 80, 9, '2022-11-03 19:30:00+00', '2022-11-03 20:30:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (4, 78, 4, NULL, 4500, 30, 10, '2022-07-02 19:00:00+00', '2022-07-02 23:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (0, 8, 1, NULL, 6000, 120, 11, '2022-06-18 18:00:00+00', '2022-06-19 00:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (10, 80, 4, NULL, 1600, 40, 12, '2022-11-19 08:00:00+00', '2022-11-19 09:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (0, 7, 1, NULL, 6000, 120, 13, '2022-06-17 18:00:00+00', '2022-06-18 00:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (10, 100, 4, NULL, 2600, 40, 14, '2022-11-19 19:00:00+00', '2022-11-19 20:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (0, 38, 1, NULL, 6000, 120, 15, '2022-06-17 18:00:00+00', '2022-06-18 00:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (0, 39, 1, NULL, 6000, 80, 16, '2022-06-16 12:00:00+00', '2022-06-16 14:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (5, 86, 5, NULL, 3000, 60, 17, '2022-07-14 21:00:00+00', '2022-07-15 03:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (0, 51, 1, NULL, 6000, 40, 18, '2022-06-19 13:00:00+00', '2022-06-19 14:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (5, 87, 5, NULL, 4000, 80, 19, '2022-07-14 19:00:00+00', '2022-07-15 03:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (0, 62, 1, NULL, 6000, 200, 20, '2022-06-17 18:00:00+00', '2022-06-18 00:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (0, 63, 1, NULL, 6000, 200, 21, '2022-06-18 18:00:00+00', '2022-06-19 00:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (0, 64, 1, NULL, 6000, 80, 22, '2022-06-17 14:30:00+00', '2022-06-17 18:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (0, 64, 1, NULL, 6000, 80, 23, '2022-06-18 14:00:00+00', '2022-06-18 18:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (5, 88, 5, NULL, 3000, 25, 24, '2022-07-14 14:00:00+00', '2022-07-14 19:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (5, 89, 5, NULL, 3000, 15, 25, '2022-07-14 14:00:00+00', '2022-07-14 19:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (0, 66, 1, NULL, 500, 120, 26, '2022-06-18 18:00:00+00', '2022-06-19 00:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (0, 67, 3, NULL, 6000, 40, 27, '2022-06-19 13:00:00+00', '2022-06-19 14:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (0, 67, 3, NULL, 6000, 40, 28, '2022-06-19 18:00:00+00', '2022-06-19 21:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (0, 61, 1, NULL, 4000, 160, 29, '2022-06-18 18:00:00+00', '2022-06-19 00:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (0, 43, 1, NULL, 3000, 60, 30, '2022-06-17 08:00:00+00', '2022-06-16 10:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (0, 43, 1, NULL, 3000, 20, 31, '2022-06-18 08:00:00+00', '2022-06-17 09:30:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (0, 43, 1, NULL, 3000, 50, 32, '2022-06-19 08:00:00+00', '2022-06-18 09:30:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (0, 69, 3, NULL, 6000, 80, 33, '2022-06-16 16:15:00+00', '2022-06-16 18:45:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (0, 69, 3, NULL, 6000, 60, 34, '2022-06-16 18:45:00+00', '2022-06-16 22:15:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (0, 72, 1, NULL, 2000, 200, 35, '2022-06-17 10:00:00+00', '2022-06-18 00:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (0, 68, 1, NULL, 6000, 100, 36, '2022-06-16 13:00:00+00', '2022-06-16 14:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (10, 29, 4, NULL, 2600, 40, 37, '2022-11-18 18:30:00+00', '2022-11-18 19:30:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (10, 80, 4, NULL, 1600, 40, 38, '2022-11-20 08:00:00+00', '2022-11-20 09:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (10, 97, 4, NULL, 2600, 40, 39, '2022-11-19 12:30:00+00', '2022-11-19 13:30:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (6, 91, 6, NULL, 1850, 220, 40, '2022-08-02 11:30:00+00', '2022-08-02 13:30:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (7, 81, 7, NULL, 4000, 12, 41, '2022-08-06 09:00:00+00', '2022-08-06 20:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (3, 63, 3, NULL, 0, 100, 42, '2022-10-10 12:00:00+00', '2022-10-10 13:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (8, 9, 5, NULL, 0, 20, 43, '1970-01-01 00:00:00+00', '1970-01-01 02:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (17, 75, 4, NULL, 2600, 8, 44, '2022-01-04 20:00:00+00', '2022-01-04 20:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (17, 55, 4, NULL, 400, 8, 45, '2022-01-03 20:00:00+00', '2022-01-03 21:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (17, 8, 4, NULL, 2000, 8, 46, '2022-01-03 20:00:00+00', '2022-01-03 21:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (17, 101, 4, NULL, 2800, 8, 47, '2022-01-05 20:00:00+00', '2022-01-05 21:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (17, 86, 4, NULL, 2600, 8, 48, '2022-01-06 20:00:00+00', '2022-01-06 21:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (17, 7, 4, NULL, 2700, 9, 49, '2023-01-02 20:00:00+00', '2023-01-02 21:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (18, 106, 2, NULL, 2400, 60, 50, '2023-02-20 12:00:00+00', '2023-02-20 18:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (20, 106, 6, NULL, 2400, 12, 51, '2023-02-04 12:00:00+00', '2023-02-04 12:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (19, 106, 3, '(Zahlen nicht fix)', 4000, 120, 52, '2023-06-22 12:00:00+00', '2023-06-22 22:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (21, 139, 8, NULL, 1800, 60, 53, '2023-04-25 19:30:00+00', '2023-04-25 20:30:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (19, 80, 1, '(zahlen nicht fix, frühstück nicht fix)', 2000, 30, 54, '2023-06-23 08:00:00+00', '2023-06-23 10:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (21, 108, 8, NULL, 1200, 40, 55, '2023-04-25 19:30:00+00', '2023-04-25 19:30:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (8, 7, 7, NULL, 2400, 10, 56, '2023-04-26 12:00:00+00', '2023-04-26 14:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (22, 7, 5, NULL, 2400, 10, 57, '2023-04-26 12:00:00+00', '2023-04-26 12:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (19, 67, 3, NULL, 2000, 120, 58, '2023-06-25 14:00:00+00', '2023-05-15 21:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (23, 61, 5, NULL, 2400, 16, 59, '2023-05-12 19:00:00+00', '2023-05-12 18:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (19, 80, 1, NULL, 2000, 30, 60, '2023-06-24 08:00:00+00', '2023-06-24 12:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (19, 68, 1, NULL, 4000, 150, 61, '2023-06-23 12:00:00+00', '2023-06-15 14:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (19, 64, 1, NULL, 3000, 40, 62, '2023-06-23 14:00:00+00', '2023-06-23 19:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (29, 191, 5, 'Die Vegetarische Alternative zum Grill', 2400, 330, 63, '2023-07-13 18:00:00+00', '2023-05-31 23:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (31, 227, 4, NULL, 2400, 42, 64, '2023-07-08 14:00:00+00', '2023-07-08 15:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (19, 7, 1, NULL, 4000, 70, 65, '2023-06-24 19:00:00+00', '2023-06-24 22:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (19, 63, 1, NULL, 4000, 80, 66, '2023-06-23 19:00:00+00', '2023-06-23 22:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (19, 186, 1, NULL, 1600, 60, 67, '2023-06-25 12:00:00+00', '2023-06-25 14:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (19, 77, 1, NULL, 2800, 200, 68, '2023-06-24 12:00:00+00', '2023-06-24 14:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (19, 200, 3, NULL, 1200, 30, 69, '2023-06-23 17:00:00+00', '2023-06-23 21:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (3, 139, 1, NULL, 2400, 1, 70, '2023-05-20 12:00:00+00', '2023-05-20 12:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (19, 62, 1, NULL, 4000, 80, 71, '2023-06-24 19:00:00+00', '2023-06-24 22:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (19, 64, 2, NULL, 3000, 50, 72, '2023-06-22 18:00:00+00', '2023-06-22 21:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (19, 64, 1, NULL, 3000, 40, 73, '2023-06-24 14:00:00+00', '2023-06-24 19:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (19, 8, 1, NULL, 5000, 50, 74, '2023-06-24 19:00:00+00', '2023-06-24 22:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (19, 69, 3, '(personenzahl nicht fix)', 6000, 70, 75, '2023-06-21 12:00:00+00', '2023-06-21 15:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (19, 61, 2, NULL, 2400, 50, 76, '2023-06-21 18:00:00+00', '2023-06-21 21:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (19, 7, 1, NULL, 4000, 70, 77, '2023-06-23 19:00:00+00', '2023-06-23 22:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (34, 230, 4, NULL, 2400, 11, 78, '2023-07-21 19:00:00+00', '2023-07-21 20:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (31, 80, 4, NULL, 1800, 42, 79, '2023-07-09 08:00:00+00', '2023-07-09 09:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (30, 75, 1, NULL, 2800, 100, 80, '2023-06-24 12:00:00+00', '2023-06-24 14:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (30, 8, 1, NULL, 5000, 50, 81, '2023-06-22 12:00:00+00', '2023-06-22 12:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (30, 214, 1, NULL, 3500, 50, 82, '2023-06-23 18:00:00+00', '2023-06-23 22:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (30, 214, 1, NULL, 3500, 50, 83, '2023-06-24 18:00:00+00', '2023-06-24 22:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (30, 8, 1, NULL, 5000, 50, 84, '2023-06-24 18:00:00+00', '2023-06-22 22:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (30, 139, 1, NULL, 4000, 150, 85, '2023-06-23 12:00:00+00', '2023-06-23 14:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (19, 8, 1, NULL, 5000, 50, 86, '2023-06-23 19:00:00+00', '2023-06-23 22:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (31, 100, 4, NULL, 2600, 42, 87, '2023-07-08 20:00:00+00', '2023-07-08 23:59:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (31, 223, 4, NULL, 2400, 42, 88, '2023-07-07 19:30:00+00', '2023-07-07 21:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (31, 80, 4, NULL, 1800, 42, 89, '2023-07-08 08:00:00+00', '2023-07-08 09:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (32, 77, 5, NULL, 2400, 200, 90, '2023-07-13 18:00:00+00', '2023-07-08 21:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (32, 88, 8, NULL, 3000, 60, 91, '2023-07-13 16:00:00+00', '2023-07-08 20:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (32, 89, 8, NULL, 2400, 20, 92, '2023-07-13 14:00:00+00', '2023-07-13 20:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (8, 200, 6, NULL, 1800, 4, 93, '2023-07-10 12:00:00+00', '2023-07-10 12:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (33, 61, 8, NULL, 2400, 30, 94, '2023-07-14 16:00:00+00', '2023-07-14 18:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (34, 40, 4, NULL, 2800, 11, 95, '2023-07-22 19:00:00+00', '2023-07-22 20:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (36, 265, 1, NULL, 2600, 33, 96, '2023-08-10 12:00:00+00', '2023-08-10 12:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (36, 251, 1, NULL, 1000, 33, 97, '2023-08-11 22:00:00+00', '2023-08-11 23:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (39, 29, 1, NULL, 1800, 50, 98, '2023-10-08 12:00:00+00', '2023-10-08 12:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (39, 266, 1, NULL, 800, 50, 99, '2023-09-09 12:00:00+00', '2023-09-09 12:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (40, 39, 8, NULL, 1800, 60, 100, '2023-10-23 12:00:00+00', '2023-10-23 12:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (41, 227, 1, NULL, 2400, 100, 101, '2023-10-25 12:00:00+00', '2023-10-25 14:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (40, 29, 8, 'Brot', 1800, 60, 102, '2023-10-23 12:00:00+00', '2023-10-23 12:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (8, 29, 5, NULL, 1000, 17, 103, '2023-03-31 12:00:00+00', '2023-03-31 12:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (46, 270, 4, NULL, 2600, 37, 104, '2024-01-13 13:00:00+00', '2024-01-13 14:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (46, 96, 4, NULL, 2400, 37, 105, '2024-01-13 19:00:00+00', '2024-01-13 20:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (46, 80, 4, NULL, 1800, 37, 106, '2024-01-13 08:00:00+00', '2024-01-08 09:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (46, 80, 4, NULL, 1800, 37, 107, '2024-01-14 08:00:00+00', '2024-01-14 09:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (46, 273, 4, NULL, 2000, 37, 108, '2024-01-12 19:00:00+00', '2024-01-12 20:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (47, 29, 8, NULL, 1450, 750, 109, '2024-06-12 17:30:00+00', '2024-06-12 22:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (42, 29, 1, NULL, 2400, 30, 111, '2023-11-08 12:00:00+00', '2023-11-08 12:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (38, 64, 1, NULL, 3000, 100, 115, '2024-06-14 14:00:00+00', '2024-06-14 18:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (50, 292, 8, NULL, 2400, 100, 118, '2024-04-25 19:00:00+00', '2024-04-25 21:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (49, 30, 1, NULL, 2400, 30, 119, '2024-04-17 12:00:00+00', '2024-04-17 12:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (38, 63, 1, NULL, 3000, 60, 126, '2024-06-14 18:00:00+00', '2024-06-14 21:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (38, 7, 1, NULL, 3000, 40, 129, '2024-06-14 21:00:00+00', '2024-06-15 00:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (38, 63, 1, NULL, 3000, 50, 130, '2024-06-14 21:00:00+00', '2024-06-15 00:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (38, 7, 1, NULL, 3000, 40, 144, '2024-06-15 21:00:00+00', '2024-06-16 00:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (38, 63, 1, NULL, 3000, 50, 145, '2024-06-15 21:00:00+00', '2024-06-16 00:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (48, 292, 1, NULL, 2800, 20, 117, '2024-05-24 17:30:00+00', '2024-05-24 18:30:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (48, 139, 1, NULL, 2600, 20, 112, '2024-05-25 10:30:00+00', '2024-05-25 12:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (38, 214, 1, NULL, 2400, 30, 147, '2024-06-15 21:00:00+00', '2024-06-16 00:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (38, 309, 1, NULL, 3000, 100, 122, '2024-06-14 08:00:00+00', '2024-06-14 10:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (38, 7, 1, NULL, 3000, 50, 133, '2024-06-15 00:00:00+00', '2024-06-15 02:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (38, 294, 2, NULL, 3200, 150, 150, '2024-06-12 18:00:00+00', '2024-06-12 20:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (38, 106, 3, NULL, 3200, 80, 116, '2024-06-13 12:00:00+00', '2024-06-13 14:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (38, 69, 3, NULL, 3200, 130, 113, '2024-06-12 14:00:00+00', '2024-06-12 17:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (38, 7, 1, NULL, 3000, 50, 148, '2024-06-16 00:00:00+00', '2024-06-16 02:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (38, 67, 3, NULL, 3000, 160, 139, '2024-06-16 11:00:00+00', '2024-06-16 18:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (38, 69, 3, NULL, 3200, 50, 121, '2024-06-13 14:00:00+00', '2024-06-13 18:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (38, 307, 3, NULL, 3000, 80, 151, '2024-06-16 11:00:00+00', '2024-06-16 18:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (38, 214, 1, NULL, 2400, 20, 132, '2024-06-14 21:00:00+00', '2024-06-15 00:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (38, 106, 3, NULL, 3200, 80, 120, '2024-06-13 14:00:00+00', '2024-06-13 18:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (38, 68, 1, NULL, 4000, 300, 124, '2024-06-14 12:00:00+00', '2024-06-14 14:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (38, 40, 1, NULL, 3000, 60, 127, '2024-06-14 18:00:00+00', '2024-06-14 21:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (38, 40, 1, NULL, 3000, 30, 134, '2024-06-15 00:00:00+00', '2024-06-15 02:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (38, 40, 1, NULL, 3000, 40, 131, '2024-06-14 21:00:00+00', '2024-06-15 00:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (38, 40, 1, NULL, 3000, 40, 146, '2024-06-15 21:00:00+00', '2024-06-16 00:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (38, 40, 1, NULL, 3000, 50, 149, '2024-06-16 00:00:00+00', '2024-06-16 02:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (38, 69, 3, NULL, 3200, 60, 114, '2024-06-13 12:00:00+00', '2024-06-13 15:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (38, 292, 2, '', 3600, 150, 110, '2024-06-13 18:00:00+00', '2024-06-13 21:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (51, 292, 8, NULL, 1600, 330, 152, '2024-07-12 18:00:00+00', '2024-07-13 00:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (38, 309, 1, '', 3000, 40, 138, '2024-06-16 08:00:00+00', '2024-06-16 13:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (51, 310, 8, NULL, 750, 525, 153, '2024-07-12 18:00:00+00', '2024-07-13 00:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (38, 77, 1, '', 3000, 300, 136, '2024-06-15 13:00:00+00', '2024-06-15 15:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (38, 309, 1, '', 3000, 70, 135, '2024-06-15 08:00:00+00', '2024-06-15 13:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (38, 64, 1, '', 3000, 110, 137, '2024-06-15 14:00:00+00', '2024-06-15 19:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (38, 214, 1, '', 2400, 50, 128, '2024-06-14 18:30:00+00', '2024-06-14 21:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (38, 63, 1, '', 3000, 60, 141, '2024-06-15 18:30:00+00', '2024-06-15 21:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (38, 7, 1, '', 3000, 80, 125, '2024-06-14 19:00:00+00', '2024-06-14 21:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (38, 7, 1, '', 3000, 80, 140, '2024-06-15 19:00:00+00', '2024-06-15 21:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (38, 40, 1, '', 3000, 60, 142, '2024-06-15 18:30:00+00', '2024-06-15 21:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (38, 214, 1, '', 2400, 50, 143, '2024-06-15 18:30:00+00', '2024-06-15 21:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (52, 314, 8, NULL, 2400, 200, 155, '2024-07-12 16:00:00+00', '2024-07-12 22:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (52, 88, 5, '', 2400, 80, 165, '2024-07-12 15:00:00+00', '2024-07-12 18:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (51, 311, 8, NULL, 100, 700, 154, '2024-07-12 18:00:00+00', '2024-07-13 00:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (52, 89, 5, '', 2400, 40, 170, '2024-07-12 15:00:00+00', '2024-07-12 18:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (53, 81, 4, '', 1800, 34, 175, '2024-07-06 18:00:00+00', '2024-07-06 20:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (53, 82, 4, '', 1200, 34, 176, '2024-07-06 18:00:00+00', '2024-07-06 20:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (53, 139, 4, '', 3000, 34, 172, '2024-07-05 18:00:00+00', '2024-07-05 20:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (53, 80, 4, '', 2400, 34, 173, '2024-07-06 08:00:00+00', '2024-07-06 09:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (53, 315, 4, '', 3400, 34, 174, '2024-07-06 11:00:00+00', '2024-07-06 13:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (53, 85, 4, '', 600, 34, 179, '2024-07-06 18:00:00+00', '2024-07-06 20:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (53, 80, 4, '', 2400, 34, 180, '2024-07-07 08:00:00+00', '2024-07-07 10:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (53, 319, 4, '', 1600, 34, 181, '2024-07-06 18:00:00+00', '2024-07-06 20:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (52, 317, 5, '', 1400, 200, 171, '2024-07-12 20:00:00+00', '2024-07-12 20:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (56, 324, 4, '', 3600, 30, 185, '2024-12-13 19:00:00+00', '2024-12-13 20:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (56, 322, 4, '', 3400, 40, 187, '2024-12-14 13:00:00+00', '2024-12-14 14:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (54, 292, 8, '', 3000, 600, 182, '2024-10-14 17:00:00+00', '2024-10-14 22:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (55, 320, 8, '', 2400, 120, 184, '2024-10-29 18:00:00+00', '2024-10-30 12:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (56, 323, 4, '', 2400, 15, 186, '2024-12-13 19:00:00+00', '2024-12-13 20:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (56, 80, 4, '', 2400, 40, 189, '2024-12-14 08:00:00+00', '2024-12-14 08:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (56, 321, 4, '', 2800, 40, 188, '2024-12-14 19:00:00+00', '2024-12-14 20:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (56, 80, 4, '', 2400, 40, 190, '2024-12-15 08:00:00+00', '2024-12-15 09:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (58, 321, 9, '', 3000, 15, 193, '2024-11-29 19:00:00+00', '2024-11-29 20:00:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (57, 324, 9, '', 1200, 15, 191, '2024-11-22 19:00:00+00', '2024-11-22 19:30:00+00');
INSERT INTO public.event_meals OVERRIDING SYSTEM VALUE VALUES (57, 322, 9, '', 1800, 15, 192, '2024-11-22 19:30:00+00', '2024-11-22 20:00:00+00');


--
-- Data for Name: ingredients; Type: TABLE DATA; Schema: public; Owner: dennis
--

INSERT INTO public.ingredients VALUES (112, 'Preiselbeerkompott', 0, 'Hilfe ich find nix für energie');
INSERT INTO public.ingredients VALUES (7, 'Olivenöl', 37, NULL);
INSERT INTO public.ingredients VALUES (15, 'Milch', 2.78, NULL);
INSERT INTO public.ingredients VALUES (30, 'Gouda', 15.24, NULL);
INSERT INTO public.ingredients VALUES (10, 'Suppengrün', 1.29, NULL);
INSERT INTO public.ingredients VALUES (12, 'Tomatenmark', 1.81, NULL);
INSERT INTO public.ingredients VALUES (13, 'Rotwein', 2.86, NULL);
INSERT INTO public.ingredients VALUES (48, 'Aprikosen', 2.96, NULL);
INSERT INTO public.ingredients VALUES (25, 'Cayennepfeffer', 0.0, NULL);
INSERT INTO public.ingredients VALUES (26, 'Pfeffer', 0.0, NULL);
INSERT INTO public.ingredients VALUES (29, 'Kräutermischung', 0.0, NULL);
INSERT INTO public.ingredients VALUES (17, 'Lasagneplatten', 15, NULL);
INSERT INTO public.ingredients VALUES (18, 'Butter', 31.01, NULL);
INSERT INTO public.ingredients VALUES (20, 'Knoblauch', 6.07, NULL);
INSERT INTO public.ingredients VALUES (21, 'Tomaten', 0.84, NULL);
INSERT INTO public.ingredients VALUES (23, 'Balsamico', 3.92, NULL);
INSERT INTO public.ingredients VALUES (27, 'Sahne', 12.69, NULL);
INSERT INTO public.ingredients VALUES (31, 'Kaisergemüse', 1.27, NULL);
INSERT INTO public.ingredients VALUES (32, 'Schlagsahne', 12.69, NULL);
INSERT INTO public.ingredients VALUES (33, 'Champignons', 1.48, NULL);
INSERT INTO public.ingredients VALUES (34, 'Paprika', 0.94, NULL);
INSERT INTO public.ingredients VALUES (35, 'Zucchini', 0.95, NULL);
INSERT INTO public.ingredients VALUES (36, 'Oregano', 3, NULL);
INSERT INTO public.ingredients VALUES (37, 'Thymian', 2.19, NULL);
INSERT INTO public.ingredients VALUES (38, 'Rosmarin', 2.16, NULL);
INSERT INTO public.ingredients VALUES (41, 'Cheddar', 16.28, NULL);
INSERT INTO public.ingredients VALUES (42, 'Feta', 11.9, NULL);
INSERT INTO public.ingredients VALUES (43, 'Hähnchenbrust', 5.56, NULL);
INSERT INTO public.ingredients VALUES (44, 'Chorizo', 13.16, NULL);
INSERT INTO public.ingredients VALUES (45, 'Risottoreis', 14.92, NULL);
INSERT INTO public.ingredients VALUES (46, 'Weißwein', 3.02, NULL);
INSERT INTO public.ingredients VALUES (47, 'Erbsen', 4.2, NULL);
INSERT INTO public.ingredients VALUES (49, 'Margarine', 29.7, NULL);
INSERT INTO public.ingredients VALUES (50, 'Rosinen', 13.13, NULL);
INSERT INTO public.ingredients VALUES (52, 'Speck', 13.4, NULL);
INSERT INTO public.ingredients VALUES (53, 'Paprikapulver', 0.0, NULL);
INSERT INTO public.ingredients VALUES (54, 'Quark', 5.91, NULL);
INSERT INTO public.ingredients VALUES (55, 'Zucker', 16.97, NULL);
INSERT INTO public.ingredients VALUES (56, 'Backpulver', 0.0, NULL);
INSERT INTO public.ingredients VALUES (57, 'Röstzwiebeln', 14.55, NULL);
INSERT INTO public.ingredients VALUES (87, 'Knoblauchpulver', 0.0, NULL);
INSERT INTO public.ingredients VALUES (90, 'Muskatpulver', 0.0, NULL);
INSERT INTO public.ingredients VALUES (92, 'Bunter Pfeffer', 0.0, NULL);
INSERT INTO public.ingredients VALUES (107, 'Schaschlikspieß', 0.0, NULL);
INSERT INTO public.ingredients VALUES (59, 'Salami', 15.72, NULL);
INSERT INTO public.ingredients VALUES (60, 'Schmand', 8.59, NULL);
INSERT INTO public.ingredients VALUES (61, 'Kräuterbutter', 12.66, NULL);
INSERT INTO public.ingredients VALUES (62, 'Aufbackbrötchen', 10.32, NULL);
INSERT INTO public.ingredients VALUES (63, 'TK-Blattspinat', 0.91, NULL);
INSERT INTO public.ingredients VALUES (67, 'Petersilie', 2.53, NULL);
INSERT INTO public.ingredients VALUES (68, 'Zitronensaft', 1.17, NULL);
INSERT INTO public.ingredients VALUES (65, 'Liebstöckl', 2.01, NULL);
INSERT INTO public.ingredients VALUES (69, 'Toast', 11.17, NULL);
INSERT INTO public.ingredients VALUES (70, 'Schinkenscheiben', 5.83, NULL);
INSERT INTO public.ingredients VALUES (71, 'Senf', 3.69, NULL);
INSERT INTO public.ingredients VALUES (75, 'Chilipaste', 0.0, NULL);
INSERT INTO public.ingredients VALUES (80, 'Dosenananas', 2.818, NULL);
INSERT INTO public.ingredients VALUES (81, 'Sahneschmelzkäse', 12.89, NULL);
INSERT INTO public.ingredients VALUES (82, 'Lorbeerblätter', 0.0, NULL);
INSERT INTO public.ingredients VALUES (83, 'Staudensellerie', 0.9, NULL);
INSERT INTO public.ingredients VALUES (85, 'Rinderfilet', 6.36, NULL);
INSERT INTO public.ingredients VALUES (86, 'Salatgurken', 0.59, NULL);
INSERT INTO public.ingredients VALUES (88, 'Maiskolben', 4.47, NULL);
INSERT INTO public.ingredients VALUES (89, 'Limetten', 2.03, NULL);
INSERT INTO public.ingredients VALUES (91, 'Bergkäse', 16.07, NULL);
INSERT INTO public.ingredients VALUES (93, 'Sonnenblumenkerne', 20.54, NULL);
INSERT INTO public.ingredients VALUES (94, 'Spirelli', 15.375, NULL);
INSERT INTO public.ingredients VALUES (95, 'Rucola', 1.24, NULL);
INSERT INTO public.ingredients VALUES (97, 'Birnen', 2.41, NULL);
INSERT INTO public.ingredients VALUES (98, 'Dosenmandarinen', 2.519, NULL);
INSERT INTO public.ingredients VALUES (99, 'Himbeeren', 1.8, NULL);
INSERT INTO public.ingredients VALUES (100, 'Heidelbeeren', 1.93, NULL);
INSERT INTO public.ingredients VALUES (104, 'Vollmilchschokolade', 22.66, NULL);
INSERT INTO public.ingredients VALUES (105, 'Zartbitterschokolade', 21.54, NULL);
INSERT INTO public.ingredients VALUES (108, 'Frischkäse', 14.09, NULL);
INSERT INTO public.ingredients VALUES (109, 'Müsli', 15.26, NULL);
INSERT INTO public.ingredients VALUES (110, 'Mineralwasser', 0.0, NULL);
INSERT INTO public.ingredients VALUES (111, 'O-Saft', 1.85, NULL);
INSERT INTO public.ingredients VALUES (5, 'Salz', 0.0, NULL);
INSERT INTO public.ingredients VALUES (113, 'Vanilleeis
', 8.591, NULL);
INSERT INTO public.ingredients VALUES (114, 'Zimt', 0.0, NULL);
INSERT INTO public.ingredients VALUES (117, 'Stollen Dresdner Art', 16.57, NULL);
INSERT INTO public.ingredients VALUES (118, 'Puderzucker', 16.97, NULL);
INSERT INTO public.ingredients VALUES (119, 'Gewürzgurken', 0.93, NULL);
INSERT INTO public.ingredients VALUES (120, 'Edamer', 14.82, NULL);
INSERT INTO public.ingredients VALUES (6, 'Sonnenblumenöl', 37, NULL);
INSERT INTO public.ingredients VALUES (122, 'Saure Sahne', 4.883, NULL);
INSERT INTO public.ingredients VALUES (123, 'Naturjoghurt', 3.08, NULL);
INSERT INTO public.ingredients VALUES (124, 'Buttermilch', 1.56, NULL);
INSERT INTO public.ingredients VALUES (125, 'Radieschen', 0.73, NULL);
INSERT INTO public.ingredients VALUES (126, 'TK-8-Kräuter', 2.1075, NULL);
INSERT INTO public.ingredients VALUES (127, 'Couscous', 14.85, NULL);
INSERT INTO public.ingredients VALUES (128, 'Bulgur', 14.8, NULL);
INSERT INTO public.ingredients VALUES (129, 'Schafskäse', 11.9, NULL);
INSERT INTO public.ingredients VALUES (28, 'Kartoffeln', 2.9, NULL);
INSERT INTO public.ingredients VALUES (131, 'Kreuzkümmel', 0.0, NULL);
INSERT INTO public.ingredients VALUES (132, 'Kümmel', 0.0, NULL);
INSERT INTO public.ingredients VALUES (133, 'Pinienkerne', 14.69, NULL);
INSERT INTO public.ingredients VALUES (146, 'Hokkaido-Kürbis', 208, NULL);
INSERT INTO public.ingredients VALUES (148, 'Tellerlinsen', 12.2, NULL);
INSERT INTO public.ingredients VALUES (149, 'Kichererbsen', 11.422, NULL);
INSERT INTO public.ingredients VALUES (150, 'Ingwer', 2.09, NULL);
INSERT INTO public.ingredients VALUES (151, 'Nutella', 22.82, NULL);
INSERT INTO public.ingredients VALUES (152, 'Apfelmus', 3.81, NULL);
INSERT INTO public.ingredients VALUES (72, 'Maccheroni', 15.84, NULL);
INSERT INTO public.ingredients VALUES (74, 'Honig', 12.83, NULL);
INSERT INTO public.ingredients VALUES (78, 'Bratwurst', 12.11, NULL);
INSERT INTO public.ingredients VALUES (79, 'Soßenbinder', 16, NULL);
INSERT INTO public.ingredients VALUES (101, 'Erdbeeren', 1.51, NULL);
INSERT INTO public.ingredients VALUES (102, 'Weintrauben', 3.03, NULL);
INSERT INTO public.ingredients VALUES (103, 'Bananen', 3.91, NULL);
INSERT INTO public.ingredients VALUES (115, 'Vanillezucker', 16.762, NULL);
INSERT INTO public.ingredients VALUES (116, 'Butterschmalz', 26.86, NULL);
INSERT INTO public.ingredients VALUES (121, 'Fleischwurst', 12.57, NULL);
INSERT INTO public.ingredients VALUES (134, 'Rindergulasch', 5.27, NULL);
INSERT INTO public.ingredients VALUES (135, 'Sauerteig', 12.1, NULL);
INSERT INTO public.ingredients VALUES (137, 'Kokosmilch', 8.68, NULL);
INSERT INTO public.ingredients VALUES (140, 'Zitrone', 1.51, NULL);
INSERT INTO public.ingredients VALUES (141, 'Asia-Gemüse', 1.507, NULL);
INSERT INTO public.ingredients VALUES (142, 'Kidneybohnen', 4.27, NULL);
INSERT INTO public.ingredients VALUES (143, 'Dosenmais', 3.52, NULL);
INSERT INTO public.ingredients VALUES (4, 'Wasser', 0.0, NULL);
INSERT INTO public.ingredients VALUES (14, 'Brühepulver', 7.35, NULL);
INSERT INTO public.ingredients VALUES (22, 'Crème fraîche', 11.59, NULL);
INSERT INTO public.ingredients VALUES (76, 'Frühlingszwiebeln', 7.37, NULL);
INSERT INTO public.ingredients VALUES (19, 'Parmesan', 16.58, 'gerieben');
INSERT INTO public.ingredients VALUES (145, 'Marjoran', 0.0, NULL);
INSERT INTO public.ingredients VALUES (58, 'Schinken', 5.38, 'ganz');
INSERT INTO public.ingredients VALUES (144, 'Chilischoten', 0.0, NULL);
INSERT INTO public.ingredients VALUES (155, 'Gewürzspekulatius', 19.11, NULL);
INSERT INTO public.ingredients VALUES (156, 'Kardamom', 0.0, NULL);
INSERT INTO public.ingredients VALUES (158, 'Spätzle', 15.727, NULL);
INSERT INTO public.ingredients VALUES (3, 'Reis', 14.8114, NULL);
INSERT INTO public.ingredients VALUES (77, 'TK-Karotten', 1.63, NULL);
INSERT INTO public.ingredients VALUES (66, 'TK-Lauch', 1.22, NULL);
INSERT INTO public.ingredients VALUES (139, 'Currypaste', 5.33, 'vegan!!');
INSERT INTO public.ingredients VALUES (64, 'Frischhefe', 0.0, 'Frisch');
INSERT INTO public.ingredients VALUES (40, 'Spinat', 0.9300, 'Frisch');
INSERT INTO public.ingredients VALUES (24, 'Basilikum', 1.9700, 'frisch');
INSERT INTO public.ingredients VALUES (51, 'Currypulver', 0, NULL);
INSERT INTO public.ingredients VALUES (84, 'Schäufele (Schweinefleisch mit Knochen)', 9.5600, NULL);
INSERT INTO public.ingredients VALUES (9, 'Emmentaler', 15.8100, NULL);
INSERT INTO public.ingredients VALUES (159, 'Sojagranulat', 12.9, NULL);
INSERT INTO public.ingredients VALUES (161, 'Chilipulver', 0.0, NULL);
INSERT INTO public.ingredients VALUES (130, 'Koriander', 0, '(gewürz)');
INSERT INTO public.ingredients VALUES (138, 'Sojasoße (glutenfrei)', 4.3200, NULL);
INSERT INTO public.ingredients VALUES (160, 'Mehl 550', 14.235, NULL);
INSERT INTO public.ingredients VALUES (162, 'Räuchertofu', 6.84, NULL);
INSERT INTO public.ingredients VALUES (168, 'Spätzle (frisch)', 6.55, NULL);
INSERT INTO public.ingredients VALUES (169, 'Haferflocken', 15.57, NULL);
INSERT INTO public.ingredients VALUES (154, 'Zwetschgen', 2.0100, '');
INSERT INTO public.ingredients VALUES (170, 'Hafermilch', 1.88, NULL);
INSERT INTO public.ingredients VALUES (172, 'Baked Beans', 3.3, NULL);
INSERT INTO public.ingredients VALUES (173, 'Aubergine', 1.04, NULL);
INSERT INTO public.ingredients VALUES (174, 'Schwarze Oliven', 4.98, NULL);
INSERT INTO public.ingredients VALUES (176, 'Rote Zwiebeln', 1.17, NULL);
INSERT INTO public.ingredients VALUES (177, 'Gemischter Salat', 1.51, NULL);
INSERT INTO public.ingredients VALUES (178, 'Fladenbrot', 10.67, NULL);
INSERT INTO public.ingredients VALUES (165, 'Sellerie', 0.7, 'Knolle');
INSERT INTO public.ingredients VALUES (167, 'Wiener', 9.21, NULL);
INSERT INTO public.ingredients VALUES (179, 'TK-Paprika', 1.1991, NULL);
INSERT INTO public.ingredients VALUES (171, 'Brot', 10, NULL);
INSERT INTO public.ingredients VALUES (73, 'Dosentomaten', 1.09, 'passiert');
INSERT INTO public.ingredients VALUES (180, 'Hagelzucker', 16.97, NULL);
INSERT INTO public.ingredients VALUES (181, 'Mandeln (gestiftet)', 25.92, NULL);
INSERT INTO public.ingredients VALUES (183, 'Cocktailtomaten', 0.8, NULL);
INSERT INTO public.ingredients VALUES (184, 'Müsliriegel', 15.57, NULL);
INSERT INTO public.ingredients VALUES (185, 'Schokokekse', 19.51, NULL);
INSERT INTO public.ingredients VALUES (186, 'Kekse', 13.49, NULL);
INSERT INTO public.ingredients VALUES (187, 'Salzstangen', 14.57, NULL);
INSERT INTO public.ingredients VALUES (189, 'Reiswaffeln', 16.20, NULL);
INSERT INTO public.ingredients VALUES (191, 'Nussmix', 26.24, NULL);
INSERT INTO public.ingredients VALUES (192, 'Waffelröllchen', 22.30, NULL);
INSERT INTO public.ingredients VALUES (193, 'Mandarinen', 0, NULL);
INSERT INTO public.ingredients VALUES (190, 'Wasabi-Nüsse', 19.24, NULL);
INSERT INTO public.ingredients VALUES (194, 'Zwiebeln (Frisch)', 1.27, NULL);
INSERT INTO public.ingredients VALUES (196, 'Hefeflocken', 0.0, NULL);
INSERT INTO public.ingredients VALUES (195, 'Cashewnüsse', 25.46, NULL);
INSERT INTO public.ingredients VALUES (198, 'Trockenhefe', 0.0, NULL);
INSERT INTO public.ingredients VALUES (199, 'Rohrzucker', 16.7, NULL);
INSERT INTO public.ingredients VALUES (136, 'Getrocknete Tomaten', 7.31, NULL);
INSERT INTO public.ingredients VALUES (16, 'Mozzarella (gerieben)', 11, 'gerieben');
INSERT INTO public.ingredients VALUES (201, 'Mozzarella', 11, NULL);
INSERT INTO public.ingredients VALUES (202, 'Weißweinessig', 0.21, NULL);
INSERT INTO public.ingredients VALUES (203, 'Schnittlauch', 1.67, NULL);
INSERT INTO public.ingredients VALUES (200, 'Sojajoghurt', 2.29, 'Vegan');
INSERT INTO public.ingredients VALUES (204, 'Halloumi', 12, NULL);
INSERT INTO public.ingredients VALUES (206, 'Toastbrot', 10, NULL);
INSERT INTO public.ingredients VALUES (207, 'Gouda-Scheibe', 15, NULL);
INSERT INTO public.ingredients VALUES (188, 'Tuc Cracker', 20.0100, NULL);
INSERT INTO public.ingredients VALUES (197, 'Dinkelmehl 630er', 14.0300, NULL);
INSERT INTO public.ingredients VALUES (208, 'Sauerteig Anstellgut', 0, NULL);
INSERT INTO public.ingredients VALUES (209, 'Altbrot', 10.01, NULL);
INSERT INTO public.ingredients VALUES (210, 'Karotten (Frisch)', 1.63, NULL);
INSERT INTO public.ingredients VALUES (211, 'Brokkoli', 1.42, NULL);
INSERT INTO public.ingredients VALUES (212, 'Kloßteig', 4.55, NULL);
INSERT INTO public.ingredients VALUES (214, 'Pommessalz', 0, NULL);
INSERT INTO public.ingredients VALUES (213, 'Kakaopulver', 1.5, NULL);
INSERT INTO public.ingredients VALUES (218, 'Baguette', 10.32, NULL);
INSERT INTO public.ingredients VALUES (175, 'Sojaschnetzel (trocken)', 15.1980, NULL);
INSERT INTO public.ingredients VALUES (215, 'Test', 10.2000, NULL);
INSERT INTO public.ingredients VALUES (219, 'Dinkelvollkornmehl', 14.69, '~5-10% des Mehlgewichts für elastischeren Teig');
INSERT INTO public.ingredients VALUES (220, 'Tahini', 3.56, 'Sesampaste');
INSERT INTO public.ingredients VALUES (163, 'Hummus (fertig)', 8.5000, NULL);
INSERT INTO public.ingredients VALUES (221, 'Sauerkraut', 1.09, NULL);
INSERT INTO public.ingredients VALUES (222, 'Tofu', 4.97, 'Natur und seidenfest');
INSERT INTO public.ingredients VALUES (223, 'Dosentomaten (ganz,geschält)', 1.09, NULL);
INSERT INTO public.ingredients VALUES (224, 'Salbei', 0, NULL);
INSERT INTO public.ingredients VALUES (225, 'Wrap', 13.99, 'Artikelnummer: 117665');
INSERT INTO public.ingredients VALUES (226, 'Orangensaft', 1.6, NULL);
INSERT INTO public.ingredients VALUES (227, 'Vegane Sahne', 12, 'vegan');
INSERT INTO public.ingredients VALUES (229, 'Schupfnudeln', 4.17, NULL);
INSERT INTO public.ingredients VALUES (231, 'Kurkuma', 0, '(gemahlen)');
INSERT INTO public.ingredients VALUES (233, 'Garam masala', 0, NULL);
INSERT INTO public.ingredients VALUES (234, 'Ahornsirup', 12, NULL);
INSERT INTO public.ingredients VALUES (230, 'Rote Linsen', 15.3000, NULL);
INSERT INTO public.ingredients VALUES (232, 'Korianderblätter (frisch)', 0, NULL);
INSERT INTO public.ingredients VALUES (235, 'Limettensaft', 0, NULL);
INSERT INTO public.ingredients VALUES (237, 'Sojamedallions', 14, NULL);
INSERT INTO public.ingredients VALUES (147, 'Semmelbrösel', 10.0100, 'Paniermehl');
INSERT INTO public.ingredients VALUES (238, 'Annanas', 0, NULL);
INSERT INTO public.ingredients VALUES (239, 'Kichererbsenmehl', 13.2, NULL);
INSERT INTO public.ingredients VALUES (241, 'Grapefruitsaft', 0, NULL);
INSERT INTO public.ingredients VALUES (244, 'Bratöl', 37, NULL);
INSERT INTO public.ingredients VALUES (245, 'Rinderfond (vegan)', 0, NULL);
INSERT INTO public.ingredients VALUES (243, 'Ingwerpulver', 0, NULL);
INSERT INTO public.ingredients VALUES (246, 'Kötbullar (vegan)', 7.0, NULL);
INSERT INTO public.ingredients VALUES (247, 'Pesto', 14.9, NULL);
INSERT INTO public.ingredients VALUES (248, 'Pastinake', 2.66, NULL);
INSERT INTO public.ingredients VALUES (249, 'Lauch', 1.24, NULL);
INSERT INTO public.ingredients VALUES (250, 'Maultaschen', 8.37, NULL);
INSERT INTO public.ingredients VALUES (251, 'Sojamilch', 2.3, NULL);
INSERT INTO public.ingredients VALUES (252, 'Rotkohl', 0.95, NULL);
INSERT INTO public.ingredients VALUES (253, 'Brie', 15, NULL);
INSERT INTO public.ingredients VALUES (254, 'Marmelade', 11, NULL);
INSERT INTO public.ingredients VALUES (96, 'Äpfel', 2.7100, NULL);
INSERT INTO public.ingredients VALUES (255, 'Nelken', 18.09, NULL);
INSERT INTO public.ingredients VALUES (256, 'Aqua Faber', 0.57, NULL);
INSERT INTO public.ingredients VALUES (257, 'Zwiebelschmelz vegan', 32.99, NULL);
INSERT INTO public.ingredients VALUES (258, 'MSG', 12.26, NULL);
INSERT INTO public.ingredients VALUES (259, 'Apfelessig', 3, '');
INSERT INTO public.ingredients VALUES (260, 'Mandeln (gehobelt)', 25.92, '');
INSERT INTO public.ingredients VALUES (261, 'Süßkartoffel', 4.75, '');
INSERT INTO public.ingredients VALUES (262, 'Nektarine', 2.25, NULL);
INSERT INTO public.ingredients VALUES (263, 'Pflaume', 10.1, NULL);
INSERT INTO public.ingredients VALUES (264, 'Edna Partymischkiste 5-fach sortiert', 14.88, '');
INSERT INTO public.ingredients VALUES (265, 'Früchtemüsli', 14.68, '');
INSERT INTO public.ingredients VALUES (266, 'Schokomüsli', 14.65, '');
INSERT INTO public.ingredients VALUES (228, 'Hähnchenfond (vegan)', 0, 'null');
INSERT INTO public.ingredients VALUES (2, 'Ei', 6.4900, 'Hi');
INSERT INTO public.ingredients VALUES (268, 'Vollei', 6.49, '');
INSERT INTO public.ingredients VALUES (269, 'Schweinefleisch', 1.04, '');
INSERT INTO public.ingredients VALUES (270, 'Farfalle', 3.5, '');
INSERT INTO public.ingredients VALUES (271, 'Blumenkohl', 1.04, '');
INSERT INTO public.ingredients VALUES (272, 'Erdnussbutter', 25, '');
INSERT INTO public.ingredients VALUES (273, 'Sambal Oelek', 0, '');
INSERT INTO public.ingredients VALUES (274, 'Erdnüsse gesalzen', 25, '');
INSERT INTO public.ingredients VALUES (275, 'eingelegte Pflaumen', 2, '');
INSERT INTO public.ingredients VALUES (276, 'Majoran', 0, '');
INSERT INTO public.ingredients VALUES (277, 'Zwiebelpulver', 0, '');
INSERT INTO public.ingredients VALUES (278, 'Cardamon', 0, '');
INSERT INTO public.ingredients VALUES (279, 'Basilikum Gewürz', 0, 'Gerebelt');
INSERT INTO public.ingredients VALUES (8, 'Zwiebeln', 1.27, '(Vorgeschnitten)');
INSERT INTO public.ingredients VALUES (1, 'Mehl 405', 14.5900, '');
INSERT INTO public.ingredients VALUES (280, 'dummy', 1, '');
INSERT INTO public.ingredients VALUES (39, 'Fertig Blätterteig', 16, '');
INSERT INTO public.ingredients VALUES (205, 'Kohlrabi', 1.2000, '');
INSERT INTO public.ingredients VALUES (281, 'Ricotta', 7, '');
INSERT INTO public.ingredients VALUES (283, 'Rote Beete', 1.75, '');
INSERT INTO public.ingredients VALUES (282, 'Gerstengraupen', 14.6600, 'Körner, auch als Rollgerste bekannt');


--
-- Data for Name: stores; Type: TABLE DATA; Schema: public; Owner: dennis
--

INSERT INTO public.stores VALUES (0, 'Metro');
INSERT INTO public.stores VALUES (-1, 'Dummy');
INSERT INTO public.stores VALUES (2, 'IKEA');
INSERT INTO public.stores VALUES (3, 'Gemüse Schenk');
INSERT INTO public.stores VALUES (4, 'Mensa');
INSERT INTO public.stores VALUES (5, 'Edna');
INSERT INTO public.stores VALUES (6, 'privat');
INSERT INTO public.stores VALUES (7, 'Köbermühle');


--
-- Data for Name: ingredient_sources; Type: TABLE DATA; Schema: public; Owner: dennis
--

INSERT INTO public.ingredient_sources VALUES (219, 7, 5, 0, '', NULL, 260, 8.00);
INSERT INTO public.ingredient_sources VALUES (246, 2, 1, 0, 'https://www.ikea.com/de/de/food/salesareas/swedish-food-market/07837723-cf51-45d1-bb48-67ebcf723b97/', NULL, 1, 6.00);
INSERT INTO public.ingredient_sources VALUES (20, 0, 1.05, 0, 'https://produkte.metro.de/shop/pv/BTY-Z1144/0032/0021/Knoblauch-weiss-1kg', NULL, 2, 3.95);
INSERT INTO public.ingredient_sources VALUES (200, 0, 3.175, 0, 'https://produkte.metro.de/shop/pv/BTY-X77586/0032/0021/Alpro-Soja-Joghurt-Natur-500-g-Becher', NULL, 37, 11.36);
INSERT INTO public.ingredient_sources VALUES (140, 0, 1.02, 0, 'https://produkte.metro.de/shop/pv/BTY-Z70/0032/0021/Zitronen-1kg', NULL, 49, 2.02);
INSERT INTO public.ingredient_sources VALUES (197, 7, 25, 0, '', NULL, 259, 30.00);
INSERT INTO public.ingredient_sources VALUES (29, 0, 5.265, 0, 'https://produkte.metro.de/shop/pv/BTY-X321844/0032/0021/', NULL, 51, 39.20);
INSERT INTO public.ingredient_sources VALUES (4, -1, 1, 0, NULL, NULL, 52, 0.00);
INSERT INTO public.ingredient_sources VALUES (15, 0, 1.08, 0, 'https://produkte.metro.de/shop/pv/BTY-X702948/0032/0021/aro-frische-Vollmilch-3-5-Fett-1-l-Packung', NULL, 6, 1.09);
INSERT INTO public.ingredient_sources VALUES (194, 3, 2.5000, 0, '', NULL, 227, 2.40);
INSERT INTO public.ingredient_sources VALUES (17, 0, 0.531, 0, 'https://produkte.metro.de/shop/pv/BTY-X840874/0032/0021/Barilla-Collezione-Lasagne-Italien-500-g-Packung', NULL, 7, 2.79);
INSERT INTO public.ingredient_sources VALUES (18, 0, 0.252, 0, 'https://produkte.metro.de/shop/pv/BTY-X314169/0032/0021/aro-Butter-mild-ges%C3%A4uert-82-Fett-250-g-St%C3%BCck', NULL, 8, 1.99);
INSERT INTO public.ingredient_sources VALUES (21, 0, 7, 0, 'https://produkte.metro.de/shop/pv/BTY-Z42/0032/0021/Tomaten-6kg', NULL, 9, 8.01);
INSERT INTO public.ingredient_sources VALUES (22, 0, 5.21, 0, 'https://produkte.metro.de/shop/pv/BTY-X311862/0032/0021/aro-QS-Crème-fraîche-38-Fett-5-00-kg-Eimer', NULL, 10, 30.98);
INSERT INTO public.ingredient_sources VALUES (24, 0, 1.16, 0, 'https://produkte.metro.de/shop/pv/BTY-Z1083/0032/0021/Basilikum-1kg', NULL, 11, 16.04);
INSERT INTO public.ingredient_sources VALUES (27, 0, 5.18, 0, 'https://produkte.metro.de/shop/pv/BTY-X303665/0032/0021/aro-Frische-Sahne-33-Fett-5-kg-Eimer', NULL, 12, 22.42);
INSERT INTO public.ingredient_sources VALUES (28, 0, 10.15, 0, 'https://produkte.metro.de/shop/pv/BTY-X5970/0032/0021/Speisekartoffeln-mehlig-kochend-übergroß-Deutschland-10-kg-Sack', NULL, 13, 11.76);
INSERT INTO public.ingredient_sources VALUES (30, 0, 1, 0, 'https://produkte.metro.de/shop/pv/BTY-X303333/0032/0021/aro-Gouda-48-Fett-i.-Tr.-ca.-15-kg-Block', NULL, 14, 5.34);
INSERT INTO public.ingredient_sources VALUES (31, 0, 2.514, 0, 'https://produkte.metro.de/shop/pv/BTY-X293497/0032/0021/METRO-Chef-Kaisergemüse-tiefgefroren-erntefrisch-2-5-kg-Beutel', NULL, 15, 5.66);
INSERT INTO public.ingredient_sources VALUES (33, 0, 1.05, 0, 'https://produkte.metro.de/shop/pv/BTY-Z2129/0032/0021/Champignon-braun-1kg', NULL, 16, 5.34);
INSERT INTO public.ingredient_sources VALUES (34, 0, 9.9, 0, 'https://produkte.metro.de/shop/pv/BTY-Z2214/0032/0021/Paprika-Mix-16x500g', NULL, 17, 32.36);
INSERT INTO public.ingredient_sources VALUES (35, 0, 5.8, 0, 'https://produkte.metro.de/shop/pv/BTY-Z53/0032/0021/Zucchini-5kg', NULL, 18, 9.61);
INSERT INTO public.ingredient_sources VALUES (39, 0, 0.277, 0, 'https://produkte.metro.de/shop/pv/BTY-X377589/0032/0023/', NULL, 19, 1.49);
INSERT INTO public.ingredient_sources VALUES (42, 0, 1, 0, 'https://produkte.metro.de/shop/pv/BTY-X72206/0032/0021/Greco-Feta-48-Fett-i.-Tr.-ca.-2-kg-Blöcke', NULL, 20, 19.25);
INSERT INTO public.ingredient_sources VALUES (43, 0, 1, 0, 'https://produkte.metro.de/shop/pv/BTY-Z143/0032/0021/Hähnchenbrustfilet-ca.-2-5kg-vak.-verpackt-unkalibriert', NULL, 3, 7.69);
INSERT INTO public.ingredient_sources VALUES (44, 0, 1.086, 0, 'https://produkte.metro.de/shop/pv/BTY-X397613/0032/0021/Artysan-Chorizo-BBQ-Pikant-1-kg-Packung', NULL, 21, 16.36);
INSERT INTO public.ingredient_sources VALUES (45, 0, 2.005, 0, 'https://produkte.metro.de/shop/pv/BTY-X329202/0032/0021/METRO-Chef-Risotto-Reis-2-00-kg-Beutel', NULL, 22, 7.27);
INSERT INTO public.ingredient_sources VALUES (46, 0, 1.233, 0, 'https://produkte.metro.de/shop/pv/BTY-X436629/0032/0021/Leoff-Riesling-Weißwein-QBA-Qualitätswein-0-75-l-Flasche', NULL, 23, 3.99);
INSERT INTO public.ingredient_sources VALUES (47, 0, 2.518, 0, 'https://produkte.metro.de/shop/pv/BTY-X293560/0032/0021/METRO-Chef-Erbsen-fein-tiefgefroren-2-5-kg-Beutel', NULL, 24, 7.76);
INSERT INTO public.ingredient_sources VALUES (50, 0, 1.05, 0, 'https://produkte.metro.de/shop/pv/BTY-X505381/0032/0021/Märsch-Sultaninen-ungeschwefelt-1-kg-Beutel', NULL, 25, 5.34);
INSERT INTO public.ingredient_sources VALUES (53, 0, 0.572, 0, 'https://produkte.metro.de/shop/pv/BTY-X341358/0032/0021/METRO-Chef-Paprika-edelsüß-1-x-500-g-Dose', NULL, 26, 4.27);
INSERT INTO public.ingredient_sources VALUES (54, 0, 5.21, 0, 'https://produkte.metro.de/shop/pv/BTY-X311869/0032/0021/aro-Speisequark-40-Fett-5-kg-Eimer', NULL, 27, 16.04);
INSERT INTO public.ingredient_sources VALUES (57, 0, 2.02, 0, 'https://produkte.metro.de/shop/pv/BTY-X354089/0032/0021/METRO-Chef-Röstzwiebeln-2-kg-Beutel', NULL, 28, 12.83);
INSERT INTO public.ingredient_sources VALUES (59, 0, 0.531, 0, 'https://produkte.metro.de/shop/pv/BTY-X173258/0032/0021/Henkelmann-Salami-1A-500-g-Packung', NULL, 29, 8.76);
INSERT INTO public.ingredient_sources VALUES (60, 0, 5.2, 0, 'https://produkte.metro.de/shop/pv/BTY-X311863/0032/0021/aro-Schmand-24-Fett-5-kg-Eimer', NULL, 44, 21.60);
INSERT INTO public.ingredient_sources VALUES (65, 0, 0.15, 0, 'https://produkte.metro.de/shop/pv/BTY-X1447/0032/0021/METRO-Chef-Liebstöckel-Deutschland-100-g', NULL, 30, 3.20);
INSERT INTO public.ingredient_sources VALUES (77, 0, 2.532, 0, 'https://produkte.metro.de/shop/pv/BTY-X293499/0032/0021/METRO-Chef-Karotten-Würfel-tiefgefroren-2-5-kg-Beutel', NULL, 45, 3.48);
INSERT INTO public.ingredient_sources VALUES (89, 0, 1.02, 0, 'https://produkte.metro.de/shop/pv/BTY-Z1103/0032/0021/Limetten-1kg', NULL, 50, 2.99);
INSERT INTO public.ingredient_sources VALUES (96, 0, 10.48, 0, 'https://produkte.metro.de/shop/pv/BTY-Z133/0032/0021/Äpfel-Jonagold-10kg', NULL, 31, 18.71);
INSERT INTO public.ingredient_sources VALUES (183, 0, 3.45, 0, 'https://produkte.metro.de/shop/pv/BTY-Z1218/0032/0021/Cocktailrispentomaten-3kg', NULL, 32, 10.69);
INSERT INTO public.ingredient_sources VALUES (191, 0, 0.531, 0, 'https://produkte.metro.de/shop/pv/BTY-X381408/0032/0022/METRO-Chef-Edelnussmischung-10-x-500-g-Karton', NULL, 33, 6.41);
INSERT INTO public.ingredient_sources VALUES (192, 0, 0.75, 0, 'https://produkte.metro.de/shop/pv/BTY-X170675/0032/0021/Horeca-Select-Waffelröllchen-200er-Karton', NULL, 34, 6.42);
INSERT INTO public.ingredient_sources VALUES (197, 0, 1.009, 0, 'https://produkte.metro.de/shop/pv/BTY-X954836/0032/0021/K%C3%BCchenmeister-Dinkelmehl-Type-630-1-kg-Packung', NULL, 46, 1.49);
INSERT INTO public.ingredient_sources VALUES (198, 0, 0.025, 0, 'https://produkte.metro.de/shop/pv/BTY-X45379/0032/0021/Ruf-Trockenbackhefe-3-St%C3%BCck-%C3%A1-7-g-21-g-Packung', NULL, 35, 0.52);
INSERT INTO public.ingredient_sources VALUES (199, 0, 1.016, 0, 'https://produkte.metro.de/shop/pv/BTY-X237193/0032/0021/Tate-Lyle-Brauner-Rohrzucker-1-00-kg-Packung', NULL, 36, 4.80);
INSERT INTO public.ingredient_sources VALUES (201, 0, 1.346, 0, 'https://produkte.metro.de/shop/pv/BTY-X172244/0032/0021/aro-Mozzarella-Multipack-6-St%C3%BCck-%C3%A0-125-g-45-Fett-750-g-Packung', NULL, 47, 5.94);
INSERT INTO public.ingredient_sources VALUES (203, 0, 0.2, 0, 'https://produkte.metro.de/shop/pv/BTY-Z1043/0032/0021/Schnittlauch-200g', NULL, 48, 4.06);
INSERT INTO public.ingredient_sources VALUES (211, 0, 0.55, 0, 'https://produkte.metro.de/shop/pv/BTY-Z57/0032/0021/Broccoli-500g', NULL, 38, 2.02);
INSERT INTO public.ingredient_sources VALUES (212, 0, 3.012, 0, 'https://produkte.metro.de/shop/pv/BTY-X29425/0032/0021/METRO-Chef-Klossteig-gek%C3%BChlt-3-kg', NULL, 39, 8.98);
INSERT INTO public.ingredient_sources VALUES (218, 0, 0.5, 0, 'https://produkte.metro.de/shop/pv/BTY-X531079/0032/0022/METRO-Chef-Ciabatta-XXL-500-g', NULL, 40, 2.02);
INSERT INTO public.ingredient_sources VALUES (13, 0, 1.16, 0, 'https://produkte.metro.de/shop/pv/BTY-X569844/0032/0021/Ribeaupierre-Merlot-Rotwein-VDP-0-75-l-Flasche', NULL, 5, 3.99);
INSERT INTO public.ingredient_sources VALUES (25, -1, 1, 0, 'https://www.amazon.de/FU-CAYENNE-PFEFFER-GEM-1000G/dp/B00JWQHFLU/ref=asc_df_B00JWQHFLU/?tag=googshopde-21&linkCode=df0&hvadid=447459217138&hvpos=&hvnetw=g&hvrand=11274143342335503808&hvpone=&hvptwo=&hvqmt=&hvdev=c&hvdvcmdl=&hvlocint=&hvlocphy=9041873&hvtargid=pla-925953624878&th=1', NULL, 54, 11.96);
INSERT INTO public.ingredient_sources VALUES (135, -1, 1, 0, NULL, NULL, 56, 0.00);
INSERT INTO public.ingredient_sources VALUES (38, 0, 2.8, 0, 'https://produkte.metro.de/shop/pv/BTY-X5710/0032/0021/Fuchs-Rosmarin-500-g-Beutel', NULL, 59, 27.54);
INSERT INTO public.ingredient_sources VALUES (55, 0, 5.21, 0, 'https://produkte.metro.de/shop/pv/BTY-X322462/0032/0021/', NULL, 95, 17.11);
INSERT INTO public.ingredient_sources VALUES (69, 0, 0.35, 0, 'https://produkte.metro.de/shop/pv/BTY-X458215/0032/0021/GAB-Buttertoast-verzehrfertig-geschnitten-Buttertoastbrot-mit-2-8-Fett-500-g-Beutel', NULL, 96, 2.66);
INSERT INTO public.ingredient_sources VALUES (127, 0, 5.015, 0, 'https://produkte.metro.de/shop/pv/BTY-X35881/0032/0021/Sabarot-Couscous-grob-5-kg-Beutel', NULL, 97, 5.35);
INSERT INTO public.ingredient_sources VALUES (160, 7, 25, 0, 'Köber-Mühle Remchingen', NULL, 57, 25.00);
INSERT INTO public.ingredient_sources VALUES (175, 0, 1.5, 0, '', NULL, 58, 12.00);
INSERT INTO public.ingredient_sources VALUES (104, 0, 0.7, 0, 'https://produkte.metro.de/shop/pv/BTY-X51975/0032/0021/Wawi-Blockschokolade-48-Kakao-200-g', NULL, 98, 2.66);
INSERT INTO public.ingredient_sources VALUES (111, 0, 1.533, 0, 'https://produkte.metro.de/shop/pv/BTY-X351949/0032/0021/aro-Orangensaft-100-Fruchtgehalt-Pfandfrei-1-x-1-5-l-Packung', NULL, 99, 0.56);
INSERT INTO public.ingredient_sources VALUES (196, 0, 0.14, 0, 'https://produkte.metro.de/shop/pv/BTY-X350350/0032/0021/Sanotact-Bierhefe-Flocken-100-g-Schachtel', NULL, 100, 2.35);
INSERT INTO public.ingredient_sources VALUES (124, 0, 0.513, 0, 'https://produkte.metro.de/shop/pv/BTY-X352049/0032/0021/aro-Reine-Buttermilch-1-x-500-g-Stück', NULL, 116, 0.59);
INSERT INTO public.ingredient_sources VALUES (146, 0, 4.65, 0, 'https://produkte.metro.de/shop/pv/BTY-X409014/0032/0021/Kürbis-Hokkaido-Würfel-15-x-15-mm-1-kg-Beutel', NULL, 123, 28.63);
INSERT INTO public.ingredient_sources VALUES (103, 0, 1, 0, 'https://produkte.metro.de/shop/pv/BTY-Z1068/0032/0021/Bananen-Gep-(in-kg)', NULL, 187, 1.70);
INSERT INTO public.ingredient_sources VALUES (105, 0, 0.204, 0, 'https://produkte.metro.de/shop/pv/BTY-X51975/0032/0021/Wawi-Blockschokolade-48-Kakao-200-g', NULL, 188, 0.27);
INSERT INTO public.ingredient_sources VALUES (40, 0, 0.51, 0, 'https://produkte.metro.de/shop/pv/BTY-Z2099/0032/0021/Blattspinat-500g', NULL, 92, 2.34);
INSERT INTO public.ingredient_sources VALUES (48, 0, 2.966, 0, 'https://produkte.metro.de/shop/pv/BTY-X367582/0032/0021/aro-Aprikosen-2650-ml-1-Dose', NULL, 62, 6.41);
INSERT INTO public.ingredient_sources VALUES (51, 0, 1.013, 0, 'https://produkte.metro.de/shop/pv/BTY-X615331/0032/0021/Fuchs-Currypulver-Goldelefant-1-x-1-kg-Beutel', NULL, 64, 10.15);
INSERT INTO public.ingredient_sources VALUES (52, 0, 1, 0, 'https://produkte.metro.de/shop/pv/BTY-X83646/0032/0021/aro-Gelderländer-Bauchspeck-gewürfelt-gepökelt-mild-geräuchert-ca.-1-2-kg-Packung', NULL, 101, 10.69);
INSERT INTO public.ingredient_sources VALUES (61, 0, 0.253, 0, 'https://produkte.metro.de/shop/pv/BTY-X382025/0034/0023/Meggle-Kräuterbutter-62-Fett-250-g-Packung', NULL, 65, 3.69);
INSERT INTO public.ingredient_sources VALUES (63, 0, 2.532, 0, 'https://produkte.metro.de/shop/pv/BTY-X293569/0032/0021/METRO-Chef-Spinat-gehackt-tiefgefroren-2-5-kg-Beutel', NULL, 66, 4.78);
INSERT INTO public.ingredient_sources VALUES (64, 0, 0.512, 0, 'https://produkte.metro.de/shop/pv/BTY-X67250/0032/0021/Friessinger-Mühle-Trockenbackhefe-500-g', NULL, 67, 5.83);
INSERT INTO public.ingredient_sources VALUES (66, 0, 2.532, 0, 'https://produkte.metro.de/shop/pv/BTY-X293557/0032/0021/METRO-Chef-Porree-Scheiben-tiefgefroren-2-5-kg-Beutel', NULL, 93, 4.59);
INSERT INTO public.ingredient_sources VALUES (71, 0, 1.519, 0, 'https://produkte.metro.de/shop/pv/BTY-X10340/0032/0021/Culinaria-Dijon-Senf-extra-fein-1-00-kg', NULL, 68, 7.48);
INSERT INTO public.ingredient_sources VALUES (72, 0, 2.013, 0, 'https://produkte.metro.de/shop/pv/BTY-X216514/0032/0021/Ardo-Makkaroni-tiefgefroren-2-00-kg-Beutel', NULL, 69, 5.08);
INSERT INTO public.ingredient_sources VALUES (79, 0, 4.908, 0, 'https://produkte.metro.de/shop/pv/BTY-X313262/0032/0022/aro-Sossenbinder-hell-18-x-250-g-Karton', NULL, 71, 14.25);
INSERT INTO public.ingredient_sources VALUES (83, 0, 1.008, 0, 'https://produkte.metro.de/shop/pv/BTY-X409374/0032/0021/Staudensellerie-Streifen-4-mm-küchenfertig-1-kg-Beutel', NULL, 72, 10.69);
INSERT INTO public.ingredient_sources VALUES (84, 0, 1, 0, 'https://produkte.metro.de/shop/pv/BTY-X724999/0032/0021/Schweineschäufele-je-kg', NULL, 73, 8.34);
INSERT INTO public.ingredient_sources VALUES (86, 0, 5, 0, 'https://produkte.metro.de/shop/pv/BTY-Z51/0032/0021/Gurken-Kiste-mind.-4-2kg', NULL, 74, 7.80);
INSERT INTO public.ingredient_sources VALUES (87, 0, 1.015, 0, 'https://produkte.metro.de/shop/pv/BTY-X615282/0032/0021/Fuchs-Knoblauchpulver-1-kg-Beutel', NULL, 75, 20.85);
INSERT INTO public.ingredient_sources VALUES (88, 0, 2.623, 0, 'https://produkte.metro.de/shop/pv/BTY-X293503/0032/0021/METRO-Chef-Maiskolben-halbiert-tiefgefroren-2-5-kg-Beutel', NULL, 76, 10.69);
INSERT INTO public.ingredient_sources VALUES (90, 0, 1.008, 0, 'https://produkte.metro.de/shop/pv/BTY-X341490/0032/0021/METRO-Chef-Bag-Muskatnuss-gemahlen-1-x-1-kg-Beutel', NULL, 77, 35.83);
INSERT INTO public.ingredient_sources VALUES (116, 0, 2.601, 0, 'https://produkte.metro.de/shop/pv/BTY-X345164/0032/0021/METRO-Chef-Butterschmalz-99-8-Fett-2-5-kg-Packung', NULL, 78, 35.94);
INSERT INTO public.ingredient_sources VALUES (118, 0, 10.13, 0, 'https://produkte.metro.de/shop/pv/BTY-X182282/0032/0021/Südzucker-Puderzucker-10-00-kg', NULL, 94, 20.32);
INSERT INTO public.ingredient_sources VALUES (120, 0, 1, 0, 'https://produkte.metro.de/shop/pv/BTY-X301063/0032/0021/aro-Edamer-40-Fett-i.-Tr.-ca.-3-kg-Block', NULL, 79, 5.55);
INSERT INTO public.ingredient_sources VALUES (123, 0, 5.18, 0, 'https://produkte.metro.de/shop/pv/BTY-X303670/0032/0021/aro-Joghurt-natur-3-5-Fett-im-Milchanteil-5-00-kg-Eimer', NULL, 80, 7.48);
INSERT INTO public.ingredient_sources VALUES (136, 0, 1.045, 0, 'https://produkte.metro.de/shop/pv/BTY-X380810/0033/0021/Niklas-Tomaten-getrocknet-Tunesien-1-kg-Schachtel', NULL, 81, 8.55);
INSERT INTO public.ingredient_sources VALUES (161, 0, 0.563, 0, 'https://produkte.metro.de/shop/pv/BTY-X427567/0032/0021/METRO-Chef-Chilipulver-gemahlen-390-g-Dose', NULL, 55, 5.34);
INSERT INTO public.ingredient_sources VALUES (173, 0, 5.5, 0, 'https://produkte.metro.de/shop/pv/BTY-Z54/0032/0021/Auberginen-5kg', NULL, 82, 10.69);
INSERT INTO public.ingredient_sources VALUES (176, 0, 5.1, 0, 'https://produkte.metro.de/shop/pv/BTY-Z87/0032/0021/Zwiebel-rot-5kg', NULL, 83, 9.41);
INSERT INTO public.ingredient_sources VALUES (177, 0, 4.5, 0, 'https://produkte.metro.de/shop/pv/BTY-Z62/0032/0021/Salat-mix-Kiste', NULL, 84, 8.01);
INSERT INTO public.ingredient_sources VALUES (178, 0, 0.505, 0, 'https://produkte.metro.de/shop/pv/BTY-X125444/0032/0021/GAB-Fladenbrot-Pide-500-g-Beutel', NULL, 85, 1.74);
INSERT INTO public.ingredient_sources VALUES (186, 0, 0.419, 0, 'https://produkte.metro.de/shop/pv/BTY-X94281/0032/0022/aro-Butterkeks-2-Stück-à-200-g-20-x-400-g-Packungen', NULL, 86, 1.89);
INSERT INTO public.ingredient_sources VALUES (187, 0, 0.255, 0, 'https://produkte.metro.de/shop/pv/BTY-X369282/0032/0022/aro-Salzstangen-28-x-250-g-Packungen', NULL, 87, 0.71);
INSERT INTO public.ingredient_sources VALUES (188, 0, 0.102, 0, 'https://produkte.metro.de/shop/pv/BTY-X220207/0037/0022/Tuc-Original-24-x-100-g-Tüten', NULL, 88, 1.27);
INSERT INTO public.ingredient_sources VALUES (190, 0, 2.102, 0, 'https://produkte.metro.de/shop/pv/BTY-X383744/0032/0022/Khao-Shong-Erdnüsse-mit-Wasabi-12-x-140-g-Karton', NULL, 89, 25.68);
INSERT INTO public.ingredient_sources VALUES (204, 0, 0.276, 0, 'https://produkte.metro.de/shop/pv/BTY-X690216/0032/0022/', NULL, 90, 3.69);
INSERT INTO public.ingredient_sources VALUES (37, 0, 0.452, 0, 'https://produkte.metro.de/shop/pv/BTY-X341603/0032/0021/METRO-Chef-Thymian-gerebelt-1-x-440-g-Beutel', NULL, 61, 5.51);
INSERT INTO public.ingredient_sources VALUES (159, 0, 2.5000, 0, '', NULL, 53, 16.00);
INSERT INTO public.ingredient_sources VALUES (14, 0, 0.946, 0, 'https://produkte.metro.de/shop/pv/BTY-X962780/0032/0021/Maggi-Klare-Gemüsebrühe-900-g-Packung', NULL, 135, 11.22);
INSERT INTO public.ingredient_sources VALUES (32, 0, 1.038, 0, 'https://produkte.metro.de/shop/pv/BTY-X315451/0032/0021/aro-Schlagsahne-33-Fett-1-00-l-Packung', NULL, 145, 4.59);
INSERT INTO public.ingredient_sources VALUES (56, 0, 1.067, 0, 'https://produkte.metro.de/shop/pv/BTY-X313261/0032/0021/aro-Backpulver-1-00-kg-Packung', NULL, 102, 3.95);
INSERT INTO public.ingredient_sources VALUES (67, 0, 0.35, 0, 'https://produkte.metro.de/shop/pv/BTY-Z1202/0032/0021/Petersilie-kraus-300g', NULL, 103, 3.20);
INSERT INTO public.ingredient_sources VALUES (68, 0, 1.14, 0, 'https://produkte.metro.de/shop/pv/BTY-X491355/0033/0021/RIOBA-Zitronensaft-0-75-l-Flasche', NULL, 104, 2.01);
INSERT INTO public.ingredient_sources VALUES (70, 0, 0.523, 0, 'https://produkte.metro.de/shop/pv/BTY-X352342/0032/0021/METRO-Chef-Delikatess-Kochhinterschinken-500-g-Packung', NULL, 105, 7.48);
INSERT INTO public.ingredient_sources VALUES (73, 0, 1.1, 0, 'https://produkte.metro.de/shop/pv/BTY-X310022/0032/0021/aro-Passierte-Tomaten-1-l-Packung', NULL, 106, 1.59);
INSERT INTO public.ingredient_sources VALUES (74, 0, 0.742, 0, 'https://produkte.metro.de/shop/pv/BTY-X588564/0032/0021/aro-Blütenhonig-flüssig-500-g-Glas', NULL, 107, 3.19);
INSERT INTO public.ingredient_sources VALUES (76, 0, 2.52, 0, 'https://produkte.metro.de/shop/pv/BTY-Z66/0032/0021/Lauchzwiebel-14er', NULL, 108, 10.69);
INSERT INTO public.ingredient_sources VALUES (80, 0, 0.909, 0, 'https://produkte.metro.de/shop/pv/BTY-X999445/0032/0021/aro-Ananas-leicht-gezuckert-in-Scheiben-850-ml-Dose', NULL, 109, 1.92);
INSERT INTO public.ingredient_sources VALUES (82, 0, 0.285, 0, 'https://produkte.metro.de/shop/pv/BTY-X959862/0032/0021/Fuchs-Lorbeerblätter-ganze-Blätter-schonend-veredelt-und-getrocknet-250-g-Beutel', NULL, 111, 8.55);
INSERT INTO public.ingredient_sources VALUES (85, 0, 1, 0, 'https://produkte.metro.de/shop/pv/BTY-X539706/0032/0021/Rinderfilet-vorgereift-vak.-verpackt-je-kg', NULL, 136, 33.16);
INSERT INTO public.ingredient_sources VALUES (91, 0, 1.028, 0, 'https://produkte.metro.de/shop/pv/BTY-X821219/0032/0021/Frischpack-Bergjausen-Käse-50-Fett-1-kg-Packung', NULL, 112, 14.97);
INSERT INTO public.ingredient_sources VALUES (92, 0, 1.013, 0, 'https://produkte.metro.de/shop/pv/BTY-X143052/0032/0021/Fuchs-Bunter-Pfeffer-Gewürzmischung-Mix-Schwarzer-Weißer-Rosa-Grünen-Pfeffer-geschrotet-1-kg-Beutel', NULL, 148, 25.67);
INSERT INTO public.ingredient_sources VALUES (94, 0, 5.04, 0, 'https://produkte.metro.de/shop/pv/BTY-X414847/0032/0021/aro-Fusilli-Spirelli-Nudeln-1-x-5-kg-Sack', NULL, 149, 7.37);
INSERT INTO public.ingredient_sources VALUES (95, 0, 2.5, 0, 'https://produkte.metro.de/shop/pv/BTY-Z46/0032/0021/Rucola-Salat-1kg', NULL, 113, 9.61);
INSERT INTO public.ingredient_sources VALUES (97, 0, 0.525, 0, 'https://produkte.metro.de/shop/pv/BTY-X402289/0032/0021/Bio-Birne-Packham-Argentinien-500-g-Schachtel', NULL, 150, 2.66);
INSERT INTO public.ingredient_sources VALUES (98, 0, 2.855, 0, 'https://produkte.metro.de/shop/pv/BTY-X214564/0032/0021/aro-Mandarin-Orangen-2-65-kg-Dose', NULL, 151, 6.69);
INSERT INTO public.ingredient_sources VALUES (99, 0, 2.513, 0, 'https://produkte.metro.de/shop/pv/BTY-X293588/0032/0021/METRO-Chef-Himbeeren-tiefgefroren-2-50-kg-Beutel', NULL, 152, 18.71);
INSERT INTO public.ingredient_sources VALUES (100, 0, 1.03, 0, 'https://produkte.metro.de/shop/pv/BTY-X300525/0032/0021/METRO-Chef-Heidelbeeren-tiefgefroren-1-kg-Beutel', NULL, 153, 8.23);
INSERT INTO public.ingredient_sources VALUES (101, 0, 2.513, 0, 'https://produkte.metro.de/shop/pv/BTY-X293592/0032/0021/METRO-Chef-Erdbeeren-tiefgefroren-erntefrisch-2-5-kg-Beutel', NULL, 154, 8.86);
INSERT INTO public.ingredient_sources VALUES (102, 0, 0.7, 0, 'https://produkte.metro.de/shop/pv/BTY-Z1260/0032/0021/Trauben-hell-kernlos-500g', NULL, 155, 2.34);
INSERT INTO public.ingredient_sources VALUES (119, 0, 10.99, 0, 'https://produkte.metro.de/shop/pv/BTY-X288593/0032/0021/METRO-Chef-Gewürzgurken-knackig-würzig-55-60-Stück-10-2-l-Dose', NULL, 114, 11.22);
INSERT INTO public.ingredient_sources VALUES (121, 0, 0.658, 0, 'https://produkte.metro.de/shop/pv/BTY-X75831/0032/0021/aro-Schinkenfleischwurst-im-Ring-vak.-verpackt-650-g-Packung', NULL, 115, 3.49);
INSERT INTO public.ingredient_sources VALUES (125, 0, 1.02, 0, 'https://produkte.metro.de/shop/pv/BTY-X380028/0034/0021/Radieschen-Niederlande-1-kg-Beutel', NULL, 137, 2.13);
INSERT INTO public.ingredient_sources VALUES (126, 0, 0.3, 0, 'https://produkte.metro.de/shop/pv/BTY-X321844/0032/0021/METRO-Chef-8-Kräuter-tiefgefroren-300-g-Beutel', NULL, 117, 2.34);
INSERT INTO public.ingredient_sources VALUES (128, 0, 1.012, 0, 'https://produkte.metro.de/shop/pv/BTY-X371047/0032/0021/Frießinger-Mühle-Duru-Bulgur-mittelgrob-Weizengrütze-1-x-1-kg-Packung', NULL, 118, 2.99);
INSERT INTO public.ingredient_sources VALUES (129, 0, 1, 0, 'https://produkte.metro.de/shop/pv/BTY-X3509/0032/0021/Salakis-Schafskäse-48-Fett-ca.-2-kg-Packung', NULL, 138, 15.78);
INSERT INTO public.ingredient_sources VALUES (130, 0, 1.6, 0, 'https://produkte.metro.de/shop/pv/BTY-Z2134/0032/0021/Koriander-1kg', NULL, 119, 13.90);
INSERT INTO public.ingredient_sources VALUES (132, 0, 0.563, 0, 'https://produkte.metro.de/shop/pv/BTY-X9878/0032/0021/Ubena-Kümmel-Ganz-500-g', NULL, 120, 9.61);
INSERT INTO public.ingredient_sources VALUES (133, 0, 1.063, 0, 'https://produkte.metro.de/shop/pv/BTY-X431300/0032/0021/aro-Pinienkerne-China-1-kg-Beutel', NULL, 121, 42.79);
INSERT INTO public.ingredient_sources VALUES (142, 0, 3, 0, 'https://produkte.metro.de/shop/pv/BTY-X300784/0032/0021/aro-Kidneybohnen-Rot-2-5-kg-Dose', NULL, 139, 3.95);
INSERT INTO public.ingredient_sources VALUES (147, 0, 5.483, 0, 'https://produkte.metro.de/shop/pv/BTY-X287434/0032/0021/aro-Semmelbrösel-aus-Weissbrot-5-kg-Sack', NULL, 122, 10.69);
INSERT INTO public.ingredient_sources VALUES (148, 0, 5.026, 0, 'https://produkte.metro.de/shop/pv/BTY-X917025/0032/0021/Müller´s-Mühle-Linsen-6-7-mm-erlesen-Qualität-5-kg-Sack', NULL, 124, 18.18);
INSERT INTO public.ingredient_sources VALUES (149, 0, 2.983, 0, 'https://produkte.metro.de/shop/pv/BTY-X300781/0032/0021/aro-Kichererbsen-in-Lake-2-5-kg-Dose', NULL, 125, 3.63);
INSERT INTO public.ingredient_sources VALUES (151, 0, 1.463, 0, 'https://produkte.metro.de/shop/pv/BTY-X904076/0032/0021/Nutella-Nuss-Nougat-Creme-cremig-1-00-kg-Tiegel', NULL, 126, 5.87);
INSERT INTO public.ingredient_sources VALUES (168, 0, 2.53, 0, 'https://produkte.metro.de/shop/pv/BTY-X117075/0032/0021/Burgis-Eierspätzle-frisch-pfannenfertig-2-5-kg-Packung', NULL, 128, 8.86);
INSERT INTO public.ingredient_sources VALUES (169, 0, 0.525, 0, 'https://produkte.metro.de/shop/pv/BTY-X442778/0032/0022/aro-Haferflocken-zart-20-x-500-g-Packungen', NULL, 129, 0.79);
INSERT INTO public.ingredient_sources VALUES (171, 0, 4.23, 0, 'https://produkte.metro.de/shop/pv/BTY-X733507/0032/0021/Edna-Korn-Spezialbrot-tiefgefroren-fertig-gebacken-8-Stück-à-500-g-4-kg-Karton', NULL, 130, 16.36);
INSERT INTO public.ingredient_sources VALUES (172, 0, 5.82, 0, 'https://produkte.metro.de/shop/pv/BTY-X393215/0032/0022/aro-Baked-Beans-12-x-400-g-Tray', NULL, 131, 16.56);
INSERT INTO public.ingredient_sources VALUES (174, 0, 3.642, 0, 'https://produkte.metro.de/shop/pv/BTY-X187585/0032/0021/Vergina-Kalamata-Oliven-in-Salzlake-mit-Stein-schwarz-3-3-l-Kanister', NULL, 132, 21.71);
INSERT INTO public.ingredient_sources VALUES (179, 0, 2.514, 0, 'https://produkte.metro.de/shop/pv/BTY-X300394/0032/0021/METRO-Chef-Paprika-Mix-tiefgefroren-2-50-kg-Beutel', NULL, 140, 7.37);
INSERT INTO public.ingredient_sources VALUES (184, 0, 0.848, 0, 'https://produkte.metro.de/shop/pv/BTY-X950305/0033/0022/Mars-Balisto-Müsli-Mix-Vollkornkeks-(36-)-Milchschokolade-(39-)-Haselnusscremegeschmack-Rosinen-20-Stück-à-37-g-20-x-37-g-Riegel', NULL, 133, 10.49);
INSERT INTO public.ingredient_sources VALUES (194, 0, 5.1, 0, 'https://produkte.metro.de/shop/pv/BTY-Z1274/0032/0021/Zwiebeln-5kg', NULL, 134, 10.15);
INSERT INTO public.ingredient_sources VALUES (9, 0, 1.01, 0, 'https://produkte.metro.de/shop/pv/BTY-X97373/0032/0021/Goldsteig-Emmentaler-gerieben-nussig-fein-45-Fett-1-kg', NULL, 143, 14.39);
INSERT INTO public.ingredient_sources VALUES (21, 3, 1, 0, '', NULL, 263, 3.20);
INSERT INTO public.ingredient_sources VALUES (10, 3, 2.5, 0, NULL, NULL, 212, 3.05);
INSERT INTO public.ingredient_sources VALUES (28, 3, 10, 0, NULL, NULL, 213, 1.10);
INSERT INTO public.ingredient_sources VALUES (34, 3, 2.5, 0, NULL, NULL, 214, 4.90);
INSERT INTO public.ingredient_sources VALUES (35, 3, 2.5, 0, NULL, NULL, 215, 2.90);
INSERT INTO public.ingredient_sources VALUES (66, 3, 2.5, 0, NULL, NULL, 216, 2.90);
INSERT INTO public.ingredient_sources VALUES (67, 3, 0.1, 0, NULL, NULL, 217, 9.40);
INSERT INTO public.ingredient_sources VALUES (77, 3, 2.5, 0, NULL, NULL, 218, 1.85);
INSERT INTO public.ingredient_sources VALUES (83, 3, 2.5, 0, NULL, NULL, 219, 4.00);
INSERT INTO public.ingredient_sources VALUES (86, 3, 2.5, 0, NULL, NULL, 220, 2.75);
INSERT INTO public.ingredient_sources VALUES (125, 3, 2.5, 0, NULL, NULL, 221, 4.09);
INSERT INTO public.ingredient_sources VALUES (165, 3, 2.5, 0, NULL, NULL, 222, 2.90);
INSERT INTO public.ingredient_sources VALUES (173, 3, 2.5, 0, NULL, NULL, 223, 4.45);
INSERT INTO public.ingredient_sources VALUES (176, 3, 2.5, 0, NULL, NULL, 224, 3.10);
INSERT INTO public.ingredient_sources VALUES (177, 3, 2.5, 0, NULL, NULL, 225, 4.00);
INSERT INTO public.ingredient_sources VALUES (179, 3, 2.5, 0, NULL, NULL, 226, 4.90);
INSERT INTO public.ingredient_sources VALUES (109, 0, 1.722, 0, 'https://produkte.metro.de/shop/pv/BTY-X144826/0037/0021/Kölln-Müsli-Bircher-Frucht-1-7-kg-Beutel', NULL, 158, 7.48);
INSERT INTO public.ingredient_sources VALUES (113, 0, 1.751, 0, 'https://produkte.metro.de/shop/pv/BTY-X282573/0032/0021/METRO-Chef-Premium-Bourbon-Vanille-Eiscreme-tiefgefroren-3-l-Packung', NULL, 189, 14.97);
INSERT INTO public.ingredient_sources VALUES (114, 0, 1.015, 0, 'https://produkte.metro.de/shop/pv/BTY-X615286/0032/0021/Fuchs-Zimt-gemahlen-1-kg-Beutel', NULL, 190, 4.28);
INSERT INTO public.ingredient_sources VALUES (131, 0, 0.563, 0, 'https://produkte.metro.de/shop/pv/BTY-X170128/0032/0021/Ubena-Cumin-(Kreuzkümmel)-gemahlen-500-g-Dose', NULL, 159, 22.46);
INSERT INTO public.ingredient_sources VALUES (137, 0, 1, 0, 'https://produkte.metro.de/shop/pv/BTY-X276135/0032/0021/METRO-Chef-Kokosmilch-17-Fett-ungesüßt-extra-cremig-1-l-Packung', NULL, 201, 2.45);
INSERT INTO public.ingredient_sources VALUES (138, 0, 0.491, 0, 'https://produkte.metro.de/shop/pv/BTY-X303016/0032/0021/Kikkoman-Sojasoße-glutenfrei-250-ml-Flasche', NULL, 160, 4.48);
INSERT INTO public.ingredient_sources VALUES (139, 0, 0.45, 0, 'https://produkte.metro.de/shop/pv/BTY-X461033/0032/0021/Cock-Currypaste-Rot-1-x-400-g-Becher', NULL, 161, 3.95);
INSERT INTO public.ingredient_sources VALUES (143, 0, 2.5, 0, 'https://produkte.metro.de/shop/pv/BTY-X931838/0032/0021/Bonduelle-Gemüsemais-zart-jung-2-65-l-Dose', NULL, 162, 9.94);
INSERT INTO public.ingredient_sources VALUES (144, 0, 0.5, 0, 'https://produkte.metro.de/shop/pv/BTY-X380234/0032/0021/Chili-Mix-Niederlande-50-g-Schachtel', NULL, 163, 1.49);
INSERT INTO public.ingredient_sources VALUES (145, 0, 0.535, 0, 'https://produkte.metro.de/shop/pv/BTY-X268103/0032/0021/Fuchs-Majoran-gerebelt-1-x-500-g-Beutel', NULL, 182, 7.48);
INSERT INTO public.ingredient_sources VALUES (150, 0, 0.476, 0, 'https://produkte.metro.de/shop/pv/BTY-X409800/0032/0021/450G-INGWER-PUEREE', NULL, 164, 6.09);
INSERT INTO public.ingredient_sources VALUES (152, 0, 4.725, 0, 'https://produkte.metro.de/shop/pv/BTY-X301716/0032/0021/aro-Apfelmus-4-25-l-Dose', NULL, 165, 7.97);
INSERT INTO public.ingredient_sources VALUES (155, 0, 0.509, 0, 'https://produkte.metro.de/shop/pv/BTY-X287653/0032/0021/Borggreve-Gewürz-Spekulatius-500-g-Beutel', NULL, 166, 0.01);
INSERT INTO public.ingredient_sources VALUES (162, 0, 0.383, 0, 'https://produkte.metro.de/shop/pv/BTY-X5600/0032/0021/Berief-Bio-Tofu-geräuchert-2-x-175-g-Packung', NULL, 167, 3.19);
INSERT INTO public.ingredient_sources VALUES (163, 0, 0.187, 0, 'https://produkte.metro.de/shop/pv/BTY-X350475/0033/0021/Popp-Hummus-Natur-1-x-150-g-Becher', NULL, 168, 2.27);
INSERT INTO public.ingredient_sources VALUES (165, 0, 1, 0, 'https://produkte.metro.de/shop/pv/BTY-Z1242/0032/0021/Sellerie-Stück', NULL, 169, 3.09);
INSERT INTO public.ingredient_sources VALUES (170, 0, 1.063, 0, 'https://produkte.metro.de/shop/pv/BTY-X389189/0032/0021/Alpro-Barista-Hafermilch-1-l', NULL, 171, 2.49);
INSERT INTO public.ingredient_sources VALUES (180, 0, 1.008, 0, 'https://produkte.metro.de/shop/pv/BTY-X322462/0032/0021/aro-Raffinade-Zucker-1-00-kg-Packung', NULL, 184, 1.49);
INSERT INTO public.ingredient_sources VALUES (181, 0, 0.1, 0, 'https://produkte.metro.de/shop/pv/BTY-X338642/0032/0021/M%C3%A4rsch-Import-Mandeln-gestiftelt-blanchiert-100-g-Beutel', NULL, 172, 1.00);
INSERT INTO public.ingredient_sources VALUES (185, 0, 0.146, 0, 'https://produkte.metro.de/shop/pv/BTY-X746226/0032/0022/aro-Butterkekse-mit-Vollmilchschokolade-16-x-125-g-Packungen', NULL, 202, 1.49);
INSERT INTO public.ingredient_sources VALUES (195, 0, 1.055, 0, 'https://produkte.metro.de/shop/pv/BTY-X381395/0032/0021/METRO-Chef-Cashewkerne-natur-Deutschland-1-kg-Beutel', NULL, 173, 13.90);
INSERT INTO public.ingredient_sources VALUES (202, 0, 1.032, 0, 'https://produkte.metro.de/shop/pv/BTY-X871254/0032/0021/Genuport-Trade-Wei%C3%9Fweinessig-Ponti-Aceto-di-Vino-Bianco-Italien-1-00-l-Flasche', NULL, 203, 3.09);
INSERT INTO public.ingredient_sources VALUES (205, 0, 2.5, 0, 'https://produkte.metro.de/shop/pv/BTY-X293576/0032/0021/METRO-Chef-Kohlrabi-Streifen-tiefgefroren-2-5-kg-Beutel', NULL, 185, 3.73);
INSERT INTO public.ingredient_sources VALUES (206, 0, 0.505, 0, 'https://produkte.metro.de/shop/pv/BTY-X458215/0032/0021/GAB-Buttertoast-verzehrfertig-geschnitten-Buttertoastbrot-mit-2-8-Fett-500-g-Beutel', NULL, 204, 1.49);
INSERT INTO public.ingredient_sources VALUES (207, 0, 0.57, 0, 'https://produkte.metro.de/shop/pv/BTY-X332435/0032/0021/METRO-Chef-Gouda-500-g-Packung', NULL, 205, 4.27);
INSERT INTO public.ingredient_sources VALUES (210, 0, 3.03, 0, 'https://produkte.metro.de/shop/pv/BTY-Z79/0032/0021/M%C3%B6hren-3kg', NULL, 174, 3.52);
INSERT INTO public.ingredient_sources VALUES (213, 0, 1.012, 0, 'https://produkte.metro.de/shop/pv/BTY-X449935/0032/0021/METRO-Chef-Kakao-Pulver-1-kg-Packung', NULL, 175, 11.76);
INSERT INTO public.ingredient_sources VALUES (214, 0, 2.015, 0, 'https://produkte.metro.de/shop/pv/BTY-X615324/0032/0021/FUCHS-Pommes-Frites-W%C3%BCrzsalz-Rot-1-x-2-kg-Beutel', NULL, 176, 8.01);
INSERT INTO public.ingredient_sources VALUES (219, 0, 1.009, 0, 'https://produkte.metro.de/shop/pv/BTY-X113546/0032/0021/K%C3%BCchenmeister-Bio-Dinkelvollkornmehl-1-kg-Beutel', NULL, 177, 1.77);
INSERT INTO public.ingredient_sources VALUES (220, 0, 0.525, 0, 'https://produkte.metro.de/shop/pv/BTY-X77229/0032/0021/Doyal-Tahini-wei%C3%9Fe-Sesampaste-300-g-Tiegel', NULL, 178, 4.80);
INSERT INTO public.ingredient_sources VALUES (221, 0, 0.53, 0, 'https://produkte.metro.de/shop/pv/BTY-X437307/0032/0021/METRO-Chef-Weinsauerkraut-520-g-Packung', NULL, 179, 0.79);
INSERT INTO public.ingredient_sources VALUES (222, 0, 2.2, 0, 'https://produkte.metro.de/shop/pv/BTY-X34043/0032/0021/Berief-Natur-Bio-Tofu-natur-2-kg-Packung', NULL, 206, 12.79);
INSERT INTO public.ingredient_sources VALUES (223, 0, 3, 0, 'https://produkte.metro.de/shop/pv/BTY-X12670/0032/0021/aro-Gesch%C3%A4lte-Tomaten-2-5-kg-Dose', NULL, 180, 4.80);
INSERT INTO public.ingredient_sources VALUES (224, 0, 0.06, 0, 'https://produkte.metro.de/shop/pv/BTY-X663/0033/0021/METRO-Chef-Salbei-Deutschland-40-g', NULL, 207, 1.70);
INSERT INTO public.ingredient_sources VALUES (225, 0, 1.82, 0, 'https://produkte.metro.de/shop/pv/BTY-X293719/0032/0021/METRO-Chef-Weizen-Wraps-Mexican-Style-%C3%98-30-cm-18-St%C3%BCck-1-62-kg-Packung', NULL, 208, 5.77);
INSERT INTO public.ingredient_sources VALUES (226, 0, 1.612, 0, 'https://produkte.metro.de/shop/pv/BTY-X351949/0032/0021/aro-Orangensaft-100-Fruchtgehalt-Pfandfrei-1-x-1-5-l-Packung', NULL, 186, 2.68);
INSERT INTO public.ingredient_sources VALUES (227, 0, 0.264, 0, 'https://produkte.metro.de/shop/pv/BTY-X431349/0032/0021/Oatly-Germany-BIO-Hafer-Cuisine-250-ml-Flasche', NULL, 209, 1.70);
INSERT INTO public.ingredient_sources VALUES (250, 0, 0.366, 0, 'https://produkte.metro.de/shop/pv/BTY-X211508/0032/0021/Bürger-Gemüsemaultaschen-360-g-Beutel', NULL, 210, 2.19);
INSERT INTO public.ingredient_sources VALUES (108, 0, 2.62, 0, 'https://produkte.metro.de/shop/pv/BTY-X700788/0032/0021/Landfrisch-Frischkäse-Natur-Eimer-2-5-kg-2-50-kg-Eimer', NULL, 157, 21.92);
INSERT INTO public.ingredient_sources VALUES (205, 3, 2.5, 0, NULL, NULL, 228, 2.95);
INSERT INTO public.ingredient_sources VALUES (210, 3, 2.5, 0, NULL, NULL, 229, 1.85);
INSERT INTO public.ingredient_sources VALUES (249, 3, 2.5, 0, NULL, NULL, 230, 2.90);
INSERT INTO public.ingredient_sources VALUES (252, 3, 2.5, 0, NULL, NULL, 231, 1.65);
INSERT INTO public.ingredient_sources VALUES (94, 4, 1, 0, NULL, NULL, 232, 4.20);
INSERT INTO public.ingredient_sources VALUES (264, 5, 6, 0, NULL, NULL, 233, 50.00);
INSERT INTO public.ingredient_sources VALUES (265, 0, 2.5, 0, NULL, NULL, 234, 9.96);
INSERT INTO public.ingredient_sources VALUES (266, 0, 2.50, 0, NULL, NULL, 235, 9.44);
INSERT INTO public.ingredient_sources VALUES (208, 6, 1, 0, NULL, NULL, 239, 0.00);
INSERT INTO public.ingredient_sources VALUES (258, 6, 1, 0, NULL, NULL, 253, 0.00);
INSERT INTO public.ingredient_sources VALUES (1, 7, 2.5000, 0, '', NULL, 211, 2.35);
INSERT INTO public.ingredient_sources VALUES (268, 0, 1, 0, NULL, NULL, 264, 5.99);
INSERT INTO public.ingredient_sources VALUES (8, 3, 2.5, 0, NULL, NULL, 265, 2.75);
INSERT INTO public.ingredient_sources VALUES (263, 0, 1, 0, NULL, NULL, 257, 5.00);
INSERT INTO public.ingredient_sources VALUES (1, 0, 1.009, 0, 'https://produkte.metro.de/shop/pv/BTY-X446653/0032/0021/Mühlen-König-Weizenmehl-Type-405-1-kg-Packung', NULL, 191, 0.89);
INSERT INTO public.ingredient_sources VALUES (6, 0, 10.48, 0, 'https://produkte.metro.de/shop/pv/BTY-X131381/0032/0021/Schell-Sonnenblumenöl-10-l-Kanister', NULL, 193, 34.23);
INSERT INTO public.ingredient_sources VALUES (7, 0, 13.9, 0, 'https://produkte.metro.de/shop/pv/BTY-X408155/0032/0022/METRO-Chef-Bio-Olivenöl-12-x-0-75-l-Flaschen', NULL, 142, 128.27);
INSERT INTO public.ingredient_sources VALUES (8, 0, 2.513, 0, 'https://produkte.metro.de/shop/pv/BTY-X293559/0032/0021/METRO-Chef-Zwiebel-Würfel-10-x-10-mm-tiefgefroren-2-5-kg-Beutel', NULL, 194, 4.70);
INSERT INTO public.ingredient_sources VALUES (10, 0, 1.01, 0, 'https://produkte.metro.de/shop/pv/BTY-X315699/0032/0021/METRO-Chef-Suppengrün-tiefgefroren-1-kg-Beutel', NULL, 41, 1.87);
INSERT INTO public.ingredient_sources VALUES (16, 0, 2.019, 0, 'https://produkte.metro.de/shop/pv/BTY-X172241/0032/0021/aro-Mozzarella-gerieben-45-Fett-i.-Tr.-2-kg-Beutel', NULL, 42, 17.11);
INSERT INTO public.ingredient_sources VALUES (23, 0, 1.6, 0, 'https://produkte.metro.de/shop/pv/BTY-X725471/0032/0021/Culinaria-Balsamico-Condimento-Bianco-1-l-Flasche', NULL, 91, 8.01);
INSERT INTO public.ingredient_sources VALUES (26, 0, 1.12, 0, 'https://produkte.metro.de/shop/pv/BTY-X341341/0032/0021/METRO-Chef-Pfeffer-schwarz-gemahlen-1-x-1-1-kg-Beutel', NULL, 195, 14.97);
INSERT INTO public.ingredient_sources VALUES (41, 0, 1, 0, 'https://produkte.metro.de/shop/pv/BTY-X203887/0032/0021/Kerrygold-Cheddar-Block-Cheddar-Käse-1-Block-à-ca.-2-5-kg-mit-32-Fett-mit-essbarer-Rinde-2-5-kg', NULL, 146, 14.97);
INSERT INTO public.ingredient_sources VALUES (58, 0, 1, 0, 'https://produkte.metro.de/shop/pv/BTY-X349273/0032/0021/aro-Delikatess-Kochhinterschinken-ca.-2-5-kg', NULL, 43, 11.76);
INSERT INTO public.ingredient_sources VALUES (78, 0, 1, 0, 'https://produkte.metro.de/shop/pv/BTY-X91855/0032/0021/aro-Bratwurst-fein-gebrüht-gekühlt-30-Stück-à-ca.-120-g-ca.3-6-kg-Packung', NULL, 70, 6.41);
INSERT INTO public.ingredient_sources VALUES (81, 0, 0.266, 0, 'https://produkte.metro.de/shop/pv/BTY-X537898/0033/0021/aro-Schmelzkäse-Holländer-Scheiben-10-Scheiben-à-25-g-45-Fett-24-x-250-g-Packungen', NULL, 110, 1.69);
INSERT INTO public.ingredient_sources VALUES (110, 0, 1.533, 0, 'https://produkte.metro.de/shop/pv/BTY-X19249/0032/0021/aro-Natürliches-Mineralwasser-Classic-6-x-1-5-l-Flaschen', NULL, 196, 0.58);
INSERT INTO public.ingredient_sources VALUES (112, 0, 2.1, 0, 'https://produkte.metro.de/shop/pv/BTY-X308621/0032/0021/METRO-Chef-Wild-Preiselbeeren-2-kg-Eimer', NULL, 197, 11.76);
INSERT INTO public.ingredient_sources VALUES (115, 0, 1.067, 0, 'https://produkte.metro.de/shop/pv/BTY-X313259/0032/0021/aro-Vanillin-Zucker-1-00-kg-Packung', NULL, 198, 5.34);
INSERT INTO public.ingredient_sources VALUES (122, 0, 5.21, 0, 'https://produkte.metro.de/shop/pv/BTY-X311868/0032/0021/aro-QS-Saure-Sahne-10-Fett-5-00-kg-Eimer', NULL, 199, 12.79);
INSERT INTO public.ingredient_sources VALUES (134, 0, 1, 0, 'https://produkte.metro.de/shop/pv/BTY-X293071/0035/0021/Rinder-Gulasch-aus-der-Keule-3-x-3-cm-vak.-verpackt-3-x-3-kg-9-kg-auf-Vorbestellung', NULL, 200, 11.44);
INSERT INTO public.ingredient_sources VALUES (156, 0, 0.251, 0, 'https://produkte.metro.de/shop/pv/BTY-X146034/0032/0021/Wiberg-Kardamon-ganz-200-g-Dose', NULL, 248, 32.09);
INSERT INTO public.ingredient_sources VALUES (158, 0, 5.355, 0, 'https://produkte.metro.de/shop/pv/BTY-X209293/0032/0021/aro-Schwäbische-Eierspätzle-aus-reinem-Hartweizengrieß-und-Vollei-5-kg-Karton', NULL, 183, 14.54);
INSERT INTO public.ingredient_sources VALUES (189, 0, 0.108, 0, 'https://produkte.metro.de/shop/pv/BTY-X8885/0032/0021/bio-ZENTRALE-Reiswaffeln-Natur-100-g', NULL, 237, 1.49);
INSERT INTO public.ingredient_sources VALUES (209, 0, 0.758, 0, 'https://produkte.metro.de/shop/pv/BTY-X753309/0032/0021/Leimer-Semmelw%C3%BCrfel-Kn%C3%B6delbrot-laktosefrei-vegan-750-g-Beutel', NULL, 240, 4.98);
INSERT INTO public.ingredient_sources VALUES (228, 0, 1.198, 0, 'https://produkte.metro.de/shop/pv/BTY-X251233/0032/0021/Chef-Liquid-Fond-Vegan-Konzentrat-Like-Chicken-W%C3%BCrzso%C3%9Fe-1-l-Flasche', NULL, 238, 26.74);
INSERT INTO public.ingredient_sources VALUES (229, 0, 2.021, 0, 'https://produkte.metro.de/shop/pv/BTY-X432606/0032/0021/METRO-Chef-Schupfnudeln-2-kg-Beutel', NULL, 236, 8.23);
INSERT INTO public.ingredient_sources VALUES (230, 0, 1.005, 0, 'https://produkte.metro.de/shop/pv/BTY-X716202/0032/0021/M%C3%BCller''s-M%C3%BChle-Rote-Linsen-1-kg-Beutel', NULL, 243, 4.12);
INSERT INTO public.ingredient_sources VALUES (231, 0, 0.934, 0, 'https://produkte.metro.de/shop/pv/BTY-X341570/0032/0021/METRO-Chef-Kurkuma-gemahlen-1-x-910-g-Beutel', NULL, 244, 15.50);
INSERT INTO public.ingredient_sources VALUES (232, 0, 0.15, 0, 'https://produkte.metro.de/shop/pv/BTY-Z126/0032/0021/Koriander-gr%C3%BCn-100g', NULL, 245, 3.20);
INSERT INTO public.ingredient_sources VALUES (233, 0, 0.255, 0, 'https://produkte.metro.de/shop/pv/BTY-X304205/0032/0021/Wiberg-Garam-Masala-gemahlen-200-g-Dose', NULL, 246, 8.01);
INSERT INTO public.ingredient_sources VALUES (234, 0, 0.586, 0, 'https://produkte.metro.de/shop/pv/BTY-X130850/0032/0021/La-Comtesse-Bio-Ahorn-Sirup-Kanada-Grad-A-250-ml-Flasche', NULL, 247, 6.05);
INSERT INTO public.ingredient_sources VALUES (235, 0, 0.35, 0, 'https://produkte.metro.de/shop/pv/BTY-X930941/0032/0021/Hitchcock-Limettensaft-100-Direktsaft-12-x-0-2-l-Flaschen', NULL, 241, 1.18);
INSERT INTO public.ingredient_sources VALUES (245, 0, 1.248, 0, 'https://produkte.metro.de/shop/pv/BTY-X250572/0032/0021/Chef-Fond-Rind-Konzentrat-1-l-Flasche', NULL, 242, 26.74);
INSERT INTO public.ingredient_sources VALUES (247, 0, 0.835, 0, 'https://produkte.metro.de/shop/pv/BTY-X419066/0032/0021/METRO-Chef-Pesto-alla-Genovese-500-g-Glas', NULL, 249, 3.95);
INSERT INTO public.ingredient_sources VALUES (251, 0, 1.053, 0, 'https://produkte.metro.de/shop/pv/BTY-X458826/0032/0021/METRO-Chef-Bio-Barista-Soja-Drink-1-l-St%C3%BCck', NULL, 252, 1.54);
INSERT INTO public.ingredient_sources VALUES (253, 0, 0.205, 0, 'https://produkte.metro.de/shop/pv/BTY-X240241/0032/0021/aro-Franz%C3%B6sische-Brie-Ecke-60-Fett-1-x-200-g-Packung', NULL, 250, 1.99);
INSERT INTO public.ingredient_sources VALUES (254, 0, 0.678, 0, 'https://produkte.metro.de/shop/pv/BTY-X13788/0032/0021/aro-Konfit%C3%BCre-Erdbeere-450-g-Glas', NULL, 251, 1.39);
INSERT INTO public.ingredient_sources VALUES (259, 0, 1.069, 0, 'https://produkte.metro.de/shop/pv/BTY-X929780/0032/0021/K%C3%BChne-Apfelessig-750-ml-Flasche', NULL, 255, 2.45);
INSERT INTO public.ingredient_sources VALUES (260, 0, 0.509, 0, 'https://produkte.metro.de/shop/pv/BTY-X375649/0032/0021/METRO-Chef-Mandeln-gehobelt-500-g-Beutel', NULL, 258, 4.98);
INSERT INTO public.ingredient_sources VALUES (261, 0, 1.05, 0, 'https://produkte.metro.de/shop/pv/BTY-Z2243/0032/0021/S%C3%BC%C3%9Fkartoffeln-1kg', NULL, 254, 4.15);
INSERT INTO public.ingredient_sources VALUES (262, 0, 1.02, 0, 'https://produkte.metro.de/shop/pv/BTY-Z115/0032/0021/Nektarinen-gelb-1kg', NULL, 256, 1.81);
INSERT INTO public.ingredient_sources VALUES (5, 0, 1.037, 0, 'https://produkte.metro.de/shop/pv/BTY-X298854/0032/0021/Safrisalz-Speisesalz-grobkörnig-1-kg-Paket', NULL, 141, 0.91);
INSERT INTO public.ingredient_sources VALUES (154, 0, 4.8, 0, 'https://produkte.metro.de/shop/pv/BTY-X414960/0032/0021/METRO-Chef-Bio-Zwetschken-tiefgefroren-1-5-kg-Packung', NULL, 127, 7.59);
INSERT INTO public.ingredient_sources VALUES (2, 0, 0.727, 0, 'https://produkte.metro.de/shop/pv/BTY-X388415/0032/0021/aro-Eier-10er-Gr.-L-Braun-Bodenhaltung-10-Stück', NULL, 181, 2.30);
INSERT INTO public.ingredient_sources VALUES (3, 0, 5.023, 0, 'https://produkte.metro.de/shop/pv/BTY-X329222/0032/0021/METRO-Chef-Langkorn-Parboiled-Reis-1-x-5-kg-Beutel', NULL, 192, 7.44);
INSERT INTO public.ingredient_sources VALUES (12, 0, 0.88, 0, 'https://produkte.metro.de/shop/pv/BTY-X311483/0032/0021/METRO-Chef-Tomatenmark-2-fach-konzentriert-800-g-Dose', NULL, 4, 2.77);
INSERT INTO public.ingredient_sources VALUES (19, 0, 1, 0, 'https://produkte.metro.de/shop/pv/BTY-X796440/0032/0021/METRO-Chef-Parmigiano-Reggiano-DOP-32-Fett-i.-Tr.-12-Monate-gereift-ca.-1-kg', NULL, 144, 25.25);
INSERT INTO public.ingredient_sources VALUES (36, 0, 0.518, 0, 'https://produkte.metro.de/shop/pv/BTY-X917878/0032/0021/Fuchs-Oregano-gerebelt-1-x-500-g-Beutel', NULL, 60, 9.61);
INSERT INTO public.ingredient_sources VALUES (49, 0, 0.516, 0, 'https://produkte.metro.de/shop/pv/BTY-X361492/0032/0021/Rama-Pflanzenmargarine-Der-Klassiker-60-Fett-500-g-Becher', NULL, 63, 2.19);
INSERT INTO public.ingredient_sources VALUES (62, 0, 7.177, 0, 'https://produkte.metro.de/shop/pv/BTY-X733639/0032/0021/Edna-Brötchenkiste-5-fach-sortiert-tiefgefroren-vorgebacken-175-Stück-à-40-g-7-kg-Karton', NULL, 147, 44.29);
INSERT INTO public.ingredient_sources VALUES (107, 0, 0.929, 0, 'https://produkte.metro.de/shop/pv/BTY-X440382/0032/0021/Papstar-Pure-Schaschlikspieße-Holz-pure-Ø-3-mm-Länge-20-cm', NULL, 156, 9.51);
INSERT INTO public.ingredient_sources VALUES (167, 0, 1, 0, 'https://produkte.metro.de/shop/pv/BTY-X539038/0032/0021/aro-Wiener-Würstchen-gekühlt-20-Stück-a-50g-ca.-1-kg-Packung', NULL, 170, 6.99);
INSERT INTO public.ingredient_sources VALUES (269, 0, 1, 0, 'https://produkte.metro.de/shop/pv/BTY-X144987/0032/0021/QS-S%C3%BCddeutsche-Schweineschulter-schier-vak.-verpackt-je-kg', 'Temorär, wird noch geändert (Eulenfest 24)', 266, 5.55);
INSERT INTO public.ingredient_sources VALUES (271, 0, 9.2, 0, 'https://produkte.metro.de/shop/pv/BTY-Z56/0032/0021/Blumenkohl-weiss-Kiste', NULL, 271, 18.71);
INSERT INTO public.ingredient_sources VALUES (272, 0, 0.8, 0, 'https://produkte.metro.de/shop/pv/BTY-X151841/0032/0021/Swartberg-Erdnusspaste-1-x-500-g-Glas', NULL, 268, 3.95);
INSERT INTO public.ingredient_sources VALUES (273, 0, 1.04, 0, 'https://produkte.metro.de/shop/pv/BTY-X24378/0032/0021/Chi-Chi-Sambal-Oelek-extra-scharf-725-g-Tiegel', NULL, 270, 4.33);
INSERT INTO public.ingredient_sources VALUES (274, 0, 1.011, 0, 'https://produkte.metro.de/shop/pv/BTY-X345376/0032/0021/METRO-Chef-Erdn%C3%BCsse-Ger%C3%B6stet-Gesalzen-1-x-1-kg-Packung', NULL, 269, 6.94);
INSERT INTO public.ingredient_sources VALUES (275, 0, 1.01, 0, 'https://produkte.metro.de/shop/pv/BTY-X241600/0032/0021/aro-Pflaumen-halbe-Frucht-720-ml-Glas', NULL, 267, 1.79);


--
-- Data for Name: event_source_overrides; Type: TABLE DATA; Schema: public; Owner: dennis
--

INSERT INTO public.event_source_overrides VALUES (38, 212);
INSERT INTO public.event_source_overrides VALUES (38, 214);
INSERT INTO public.event_source_overrides VALUES (38, 215);
INSERT INTO public.event_source_overrides VALUES (38, 216);
INSERT INTO public.event_source_overrides VALUES (38, 217);
INSERT INTO public.event_source_overrides VALUES (38, 218);
INSERT INTO public.event_source_overrides VALUES (38, 219);
INSERT INTO public.event_source_overrides VALUES (38, 220);
INSERT INTO public.event_source_overrides VALUES (38, 221);
INSERT INTO public.event_source_overrides VALUES (38, 222);
INSERT INTO public.event_source_overrides VALUES (38, 223);
INSERT INTO public.event_source_overrides VALUES (38, 224);
INSERT INTO public.event_source_overrides VALUES (38, 225);
INSERT INTO public.event_source_overrides VALUES (38, 226);
INSERT INTO public.event_source_overrides VALUES (38, 227);
INSERT INTO public.event_source_overrides VALUES (38, 228);
INSERT INTO public.event_source_overrides VALUES (38, 229);
INSERT INTO public.event_source_overrides VALUES (38, 230);
INSERT INTO public.event_source_overrides VALUES (38, 231);
INSERT INTO public.event_source_overrides VALUES (38, 211);
INSERT INTO public.event_source_overrides VALUES (38, 57);
INSERT INTO public.event_source_overrides VALUES (38, 259);
INSERT INTO public.event_source_overrides VALUES (38, 260);
INSERT INTO public.event_source_overrides VALUES (38, 232);
INSERT INTO public.event_source_overrides VALUES (38, 263);
INSERT INTO public.event_source_overrides VALUES (38, 13);
INSERT INTO public.event_source_overrides VALUES (54, 265);
INSERT INTO public.event_source_overrides VALUES (54, 229);
INSERT INTO public.event_source_overrides VALUES (54, 214);


--
-- Data for Name: food_prep; Type: TABLE DATA; Schema: public; Owner: dennis
--

INSERT INTO public.food_prep VALUES (1, 38, 290, '2024-06-11 13:00:00+00', NULL, '2024-06-17 00:00:00+00');
INSERT INTO public.food_prep VALUES (2, 38, 2, '2024-06-11 13:00:00+00', NULL, '2024-06-17 00:00:00+00');
INSERT INTO public.food_prep VALUES (3, 38, 69, '2024-06-11 13:00:00+00', NULL, '2024-06-17 00:00:00+00');
INSERT INTO public.food_prep VALUES (4, 38, 293, '2024-06-11 16:00:00+00', NULL, '2024-06-17 00:00:00+00');
INSERT INTO public.food_prep VALUES (5, 38, 103, '2024-06-12 09:00:00+00', NULL, '2024-06-17 00:00:00+00');
INSERT INTO public.food_prep VALUES (6, 38, 15, '2024-06-12 09:00:00+00', NULL, '2024-06-17 00:00:00+00');
INSERT INTO public.food_prep VALUES (8, 38, 56, '2024-06-12 10:00:00+00', NULL, '2024-06-17 00:00:00+00');
INSERT INTO public.food_prep VALUES (9, 38, 102, '2024-06-12 17:00:00+00', NULL, '2024-06-17 00:00:00+00');
INSERT INTO public.food_prep VALUES (10, 38, 29, '2024-06-12 17:00:00+00', NULL, '2024-06-17 00:00:00+00');
INSERT INTO public.food_prep VALUES (7, 38, 57, '2024-06-12 10:00:00+00', NULL, '2024-06-17 00:00:00+00');


--
-- Data for Name: food_properties; Type: TABLE DATA; Schema: public; Owner: dennis
--

INSERT INTO public.food_properties VALUES (2, 'tierische produkte');
INSERT INTO public.food_properties VALUES (3, 'gluten');
INSERT INTO public.food_properties VALUES (4, 'fleisch');
INSERT INTO public.food_properties VALUES (5, 'fisch');
INSERT INTO public.food_properties VALUES (6, 'schwein');
INSERT INTO public.food_properties VALUES (7, 'krebstiere');
INSERT INTO public.food_properties VALUES (8, 'eier');
INSERT INTO public.food_properties VALUES (9, 'erdnüsse');
INSERT INTO public.food_properties VALUES (10, 'soja');
INSERT INTO public.food_properties VALUES (11, 'schalenfrüchte');
INSERT INTO public.food_properties VALUES (12, 'sellerie');
INSERT INTO public.food_properties VALUES (13, 'senf');
INSERT INTO public.food_properties VALUES (15, 'schwefeldioxid & sulfite');
INSERT INTO public.food_properties VALUES (16, 'lupine');
INSERT INTO public.food_properties VALUES (17, 'weichtiere');
INSERT INTO public.food_properties VALUES (14, 'sesamsamen');
INSERT INTO public.food_properties VALUES (18, 'cashewnüsse');
INSERT INTO public.food_properties VALUES (19, 'dinkel');
INSERT INTO public.food_properties VALUES (20, 'gerste');
INSERT INTO public.food_properties VALUES (21, 'hafer');
INSERT INTO public.food_properties VALUES (22, 'haselnüsse');
INSERT INTO public.food_properties VALUES (23, 'kamut');
INSERT INTO public.food_properties VALUES (24, 'mandel');
INSERT INTO public.food_properties VALUES (25, 'paranüsse');
INSERT INTO public.food_properties VALUES (26, 'pekannüsse');
INSERT INTO public.food_properties VALUES (27, 'pistazie');
INSERT INTO public.food_properties VALUES (28, 'macadamiannüsse / queenslandnüsse');
INSERT INTO public.food_properties VALUES (29, 'roggen');
INSERT INTO public.food_properties VALUES (30, 'walnüsse');
INSERT INTO public.food_properties VALUES (31, 'weizen');
INSERT INTO public.food_properties VALUES (32, 'lab');
INSERT INTO public.food_properties VALUES (33, 'gelatine');
INSERT INTO public.food_properties VALUES (34, 'farbstoff');
INSERT INTO public.food_properties VALUES (35, 'konservierungsstoff');
INSERT INTO public.food_properties VALUES (36, 'antioxidationsmittel');
INSERT INTO public.food_properties VALUES (37, 'geschmacksverstärker');
INSERT INTO public.food_properties VALUES (38, 'geschwefelt');
INSERT INTO public.food_properties VALUES (39, 'geschwärzt');
INSERT INTO public.food_properties VALUES (40, 'phosphat');
INSERT INTO public.food_properties VALUES (41, 'milcheiweiß');
INSERT INTO public.food_properties VALUES (42, 'koffeinhaltig');
INSERT INTO public.food_properties VALUES (43, 'chininhaltig');
INSERT INTO public.food_properties VALUES (44, 'süßungsmittel');
INSERT INTO public.food_properties VALUES (45, 'gewachst');
INSERT INTO public.food_properties VALUES (48, 'gelatine');
INSERT INTO public.food_properties VALUES (1, 'Milch');


--
-- Data for Name: groups; Type: TABLE DATA; Schema: public; Owner: dennis
--



--
-- Data for Name: ingredient_properties; Type: TABLE DATA; Schema: public; Owner: dennis
--

INSERT INTO public.ingredient_properties VALUES (1, 3);
INSERT INTO public.ingredient_properties VALUES (1, 31);
INSERT INTO public.ingredient_properties VALUES (2, 8);
INSERT INTO public.ingredient_properties VALUES (9, 1);
INSERT INTO public.ingredient_properties VALUES (15, 1);
INSERT INTO public.ingredient_properties VALUES (16, 1);
INSERT INTO public.ingredient_properties VALUES (18, 1);
INSERT INTO public.ingredient_properties VALUES (19, 1);
INSERT INTO public.ingredient_properties VALUES (22, 1);
INSERT INTO public.ingredient_properties VALUES (27, 1);
INSERT INTO public.ingredient_properties VALUES (32, 1);
INSERT INTO public.ingredient_properties VALUES (39, 1);
INSERT INTO public.ingredient_properties VALUES (41, 1);
INSERT INTO public.ingredient_properties VALUES (42, 1);
INSERT INTO public.ingredient_properties VALUES (54, 1);
INSERT INTO public.ingredient_properties VALUES (69, 1);
INSERT INTO public.ingredient_properties VALUES (62, 1);
INSERT INTO public.ingredient_properties VALUES (71, 1);
INSERT INTO public.ingredient_properties VALUES (9, 32);
INSERT INTO public.ingredient_properties VALUES (10, 12);
INSERT INTO public.ingredient_properties VALUES (14, 3);
INSERT INTO public.ingredient_properties VALUES (14, 13);
INSERT INTO public.ingredient_properties VALUES (14, 31);
INSERT INTO public.ingredient_properties VALUES (15, 41);


--
-- Data for Name: inventories; Type: TABLE DATA; Schema: public; Owner: dennis
--

INSERT INTO public.inventories VALUES (1, 'FSI Gewürzkiste');
INSERT INTO public.inventories VALUES (2, 'FSI Nebenraum');


--
-- Data for Name: inventory_ingredients; Type: TABLE DATA; Schema: public; Owner: dennis
--

INSERT INTO public.inventory_ingredients VALUES (2, 73, 10);
INSERT INTO public.inventory_ingredients VALUES (2, 159, 1);
INSERT INTO public.inventory_ingredients VALUES (2, 94, 5);
INSERT INTO public.inventory_ingredients VALUES (2, 158, 2.5);
INSERT INTO public.inventory_ingredients VALUES (2, 137, 1);
INSERT INTO public.inventory_ingredients VALUES (2, 221, 0.8);
INSERT INTO public.inventory_ingredients VALUES (2, 12, 1.600);
INSERT INTO public.inventory_ingredients VALUES (2, 172, 0.400);
INSERT INTO public.inventory_ingredients VALUES (2, 3, 10);
INSERT INTO public.inventory_ingredients VALUES (2, 174, 2);
INSERT INTO public.inventory_ingredients VALUES (2, 202, 2);
INSERT INTO public.inventory_ingredients VALUES (2, 244, 1);
INSERT INTO public.inventory_ingredients VALUES (2, 23, 0.5);
INSERT INTO public.inventory_ingredients VALUES (1, 131, 0.460);
INSERT INTO public.inventory_ingredients VALUES (1, 12, 1.180);
INSERT INTO public.inventory_ingredients VALUES (1, 38, 166);
INSERT INTO public.inventory_ingredients VALUES (1, 276, 0.108);
INSERT INTO public.inventory_ingredients VALUES (1, 36, 0.020);
INSERT INTO public.inventory_ingredients VALUES (1, 90, 0.250);
INSERT INTO public.inventory_ingredients VALUES (1, 233, 0.045);
INSERT INTO public.inventory_ingredients VALUES (1, 231, 0.4);
INSERT INTO public.inventory_ingredients VALUES (1, 277, 0.037);
INSERT INTO public.inventory_ingredients VALUES (1, 199, 0.660);
INSERT INTO public.inventory_ingredients VALUES (1, 29, 0.258);
INSERT INTO public.inventory_ingredients VALUES (1, 56, 0.015);
INSERT INTO public.inventory_ingredients VALUES (1, 255, 0.003);
INSERT INTO public.inventory_ingredients VALUES (1, 278, 0.033);
INSERT INTO public.inventory_ingredients VALUES (1, 147, 0.270);
INSERT INTO public.inventory_ingredients VALUES (1, 197, 0.740);
INSERT INTO public.inventory_ingredients VALUES (1, 196, 0.136);
INSERT INTO public.inventory_ingredients VALUES (1, 65, 0.2);
INSERT INTO public.inventory_ingredients VALUES (1, 239, 0.219);
INSERT INTO public.inventory_ingredients VALUES (1, 130, 0.004);
INSERT INTO public.inventory_ingredients VALUES (1, 132, 0.033);
INSERT INTO public.inventory_ingredients VALUES (1, 279, 0.010);
INSERT INTO public.inventory_ingredients VALUES (1, 25, 0.1540);
INSERT INTO public.inventory_ingredients VALUES (1, 68, 0.0000);
INSERT INTO public.inventory_ingredients VALUES (1, 213, 0.000);
INSERT INTO public.inventory_ingredients VALUES (1, 259, 0.2000);
INSERT INTO public.inventory_ingredients VALUES (1, 214, 0.8080);
INSERT INTO public.inventory_ingredients VALUES (1, 5, 0.000);
INSERT INTO public.inventory_ingredients VALUES (1, 14, 0.7700);
INSERT INTO public.inventory_ingredients VALUES (1, 51, 0.3300);
INSERT INTO public.inventory_ingredients VALUES (1, 53, 1.0400);
INSERT INTO public.inventory_ingredients VALUES (1, 161, 0.1600);
INSERT INTO public.inventory_ingredients VALUES (1, 26, 0.7800);
INSERT INTO public.inventory_ingredients VALUES (1, 37, 0.1570);


--
-- Data for Name: meta_recipes; Type: TABLE DATA; Schema: public; Owner: dennis
--

INSERT INTO public.meta_recipes VALUES (17, 15, 1.7);
INSERT INTO public.meta_recipes VALUES (17, 16, 0.5);
INSERT INTO public.meta_recipes VALUES (18, 15, 1.7);
INSERT INTO public.meta_recipes VALUES (20, 28, 0.45);
INSERT INTO public.meta_recipes VALUES (20, 21, 0.240);
INSERT INTO public.meta_recipes VALUES (18, 21, 0.24);
INSERT INTO public.meta_recipes VALUES (22, 20, 0.7);
INSERT INTO public.meta_recipes VALUES (23, 20, 0.7);
INSERT INTO public.meta_recipes VALUES (27, 20, 0.7);
INSERT INTO public.meta_recipes VALUES (24, 22, 0.8);
INSERT INTO public.meta_recipes VALUES (25, 22, 0.8);
INSERT INTO public.meta_recipes VALUES (26, 24, 0.8);
INSERT INTO public.meta_recipes VALUES (200, 4, 8);
INSERT INTO public.meta_recipes VALUES (31, 32, 0.275);
INSERT INTO public.meta_recipes VALUES (43, 41, 1);
INSERT INTO public.meta_recipes VALUES (43, 42, 1);
INSERT INTO public.meta_recipes VALUES (46, 17, 2.2);
INSERT INTO public.meta_recipes VALUES (47, 17, 2.2);
INSERT INTO public.meta_recipes VALUES (44, 46, 2.35);
INSERT INTO public.meta_recipes VALUES (45, 46, 2.35);
INSERT INTO public.meta_recipes VALUES (48, 46, 2.35);
INSERT INTO public.meta_recipes VALUES (49, 46, 2.35);
INSERT INTO public.meta_recipes VALUES (33, 31, 0.514);
INSERT INTO public.meta_recipes VALUES (51, 4, 0.35);
INSERT INTO public.meta_recipes VALUES (56, 52, 14);
INSERT INTO public.meta_recipes VALUES (57, 53, 33);
INSERT INTO public.meta_recipes VALUES (56, 53, 33);
INSERT INTO public.meta_recipes VALUES (60, 58, 3);
INSERT INTO public.meta_recipes VALUES (60, 11, 2);
INSERT INTO public.meta_recipes VALUES (61, 59, 3.5);
INSERT INTO public.meta_recipes VALUES (61, 29, 5);
INSERT INTO public.meta_recipes VALUES (64, 11, 2);
INSERT INTO public.meta_recipes VALUES (63, 44, 3);
INSERT INTO public.meta_recipes VALUES (63, 45, 3);
INSERT INTO public.meta_recipes VALUES (63, 46, 5);
INSERT INTO public.meta_recipes VALUES (63, 48, 3);
INSERT INTO public.meta_recipes VALUES (63, 49, 3);
INSERT INTO public.meta_recipes VALUES (225, 4, 0.1000);
INSERT INTO public.meta_recipes VALUES (43, 70, 2);
INSERT INTO public.meta_recipes VALUES (97, 2, 3);
INSERT INTO public.meta_recipes VALUES (97, 98, 0.5000);
INSERT INTO public.meta_recipes VALUES (64, 58, 5.5);
INSERT INTO public.meta_recipes VALUES (64, 59, 2.5);
INSERT INTO public.meta_recipes VALUES (63, 47, 3);
INSERT INTO public.meta_recipes VALUES (97, 58, 0.5);
INSERT INTO public.meta_recipes VALUES (72, 71, 3);
INSERT INTO public.meta_recipes VALUES (51, 73, 0.1);
INSERT INTO public.meta_recipes VALUES (53, 4, 2);
INSERT INTO public.meta_recipes VALUES (91, 16, 0.3000);
INSERT INTO public.meta_recipes VALUES (29, 14, 2);
INSERT INTO public.meta_recipes VALUES (75, 59, 1);
INSERT INTO public.meta_recipes VALUES (75, 74, 1);
INSERT INTO public.meta_recipes VALUES (77, 76, 1);
INSERT INTO public.meta_recipes VALUES (81, 58, 1.05);
INSERT INTO public.meta_recipes VALUES (82, 4, 0.35);
INSERT INTO public.meta_recipes VALUES (77, 74, 2);
INSERT INTO public.meta_recipes VALUES (91, 90, 1.1370);
INSERT INTO public.meta_recipes VALUES (36, 34, 0.2500);
INSERT INTO public.meta_recipes VALUES (35, 34, 0.0700);
INSERT INTO public.meta_recipes VALUES (37, 34, 0.0700);
INSERT INTO public.meta_recipes VALUES (29, 30, 16);
INSERT INTO public.meta_recipes VALUES (87, 11, 1);
INSERT INTO public.meta_recipes VALUES (86, 4, 0.25);
INSERT INTO public.meta_recipes VALUES (86, 3, 0.15);
INSERT INTO public.meta_recipes VALUES (87, 58, 1.2);
INSERT INTO public.meta_recipes VALUES (90, 92, 0.152);
INSERT INTO public.meta_recipes VALUES (93, 4, 7.5000);
INSERT INTO public.meta_recipes VALUES (98, 4, 0.1000);
INSERT INTO public.meta_recipes VALUES (100, 56, 2);
INSERT INTO public.meta_recipes VALUES (100, 99, 3);
INSERT INTO public.meta_recipes VALUES (97, 1, 1);
INSERT INTO public.meta_recipes VALUES (101, 2, 1.5);
INSERT INTO public.meta_recipes VALUES (67, 33, 3);
INSERT INTO public.meta_recipes VALUES (108, 4, 1);
INSERT INTO public.meta_recipes VALUES (67, 50, 2);
INSERT INTO public.meta_recipes VALUES (251, 231, 0.9300);
INSERT INTO public.meta_recipes VALUES (251, 250, 0.0950);
INSERT INTO public.meta_recipes VALUES (191, 139, 0.2500);
INSERT INTO public.meta_recipes VALUES (265, 101, 1);
INSERT INTO public.meta_recipes VALUES (69, 35, 1);
INSERT INTO public.meta_recipes VALUES (69, 36, 2);
INSERT INTO public.meta_recipes VALUES (69, 37, 1);
INSERT INTO public.meta_recipes VALUES (68, 56, 1.5000);
INSERT INTO public.meta_recipes VALUES (68, 57, 2.5000);
INSERT INTO public.meta_recipes VALUES (68, 59, 1);
INSERT INTO public.meta_recipes VALUES (66, 54, 1);
INSERT INTO public.meta_recipes VALUES (66, 55, 2);
INSERT INTO public.meta_recipes VALUES (50, 31, 0.5140);
INSERT INTO public.meta_recipes VALUES (62, 23, 3);
INSERT INTO public.meta_recipes VALUES (62, 24, 1);
INSERT INTO public.meta_recipes VALUES (62, 25, 1);
INSERT INTO public.meta_recipes VALUES (62, 26, 2);
INSERT INTO public.meta_recipes VALUES (62, 27, 3);
INSERT INTO public.meta_recipes VALUES (39, 4, 0.8000);
INSERT INTO public.meta_recipes VALUES (204, 136, 2.5000);
INSERT INTO public.meta_recipes VALUES (204, 213, 0.5000);
INSERT INTO public.meta_recipes VALUES (223, 200, 0.5);
INSERT INTO public.meta_recipes VALUES (224, 4, 0.5000);
INSERT INTO public.meta_recipes VALUES (78, 79, 0.0800);
INSERT INTO public.meta_recipes VALUES (78, 81, 0.0800);
INSERT INTO public.meta_recipes VALUES (78, 82, 0.0800);
INSERT INTO public.meta_recipes VALUES (78, 83, 0.0200);
INSERT INTO public.meta_recipes VALUES (78, 84, 0.1400);
INSERT INTO public.meta_recipes VALUES (78, 85, 0.0800);
INSERT INTO public.meta_recipes VALUES (139, 59, 1);
INSERT INTO public.meta_recipes VALUES (139, 203, 1.5000);
INSERT INTO public.meta_recipes VALUES (230, 58, 1.5);
INSERT INTO public.meta_recipes VALUES (230, 229, 1);
INSERT INTO public.meta_recipes VALUES (230, 228, 1);
INSERT INTO public.meta_recipes VALUES (227, 59, 0.5000);
INSERT INTO public.meta_recipes VALUES (227, 224, 1);
INSERT INTO public.meta_recipes VALUES (227, 225, 0.2300);
INSERT INTO public.meta_recipes VALUES (270, 58, 3);
INSERT INTO public.meta_recipes VALUES (273, 271, 10);
INSERT INTO public.meta_recipes VALUES (80, 41, 0.0800);
INSERT INTO public.meta_recipes VALUES (80, 280, 0.0400);
INSERT INTO public.meta_recipes VALUES (96, 267, 0.5000);
INSERT INTO public.meta_recipes VALUES (96, 274, 1);
INSERT INTO public.meta_recipes VALUES (96, 282, 2);
INSERT INTO public.meta_recipes VALUES (58, 282, 1);
INSERT INTO public.meta_recipes VALUES (290, 282, 0.5000);
INSERT INTO public.meta_recipes VALUES (11, 290, 0.666);
INSERT INTO public.meta_recipes VALUES (7, 11, 1.877);
INSERT INTO public.meta_recipes VALUES (7, 3, 0.669);
INSERT INTO public.meta_recipes VALUES (14, 282, 0.3000);
INSERT INTO public.meta_recipes VALUES (294, 293, 1.7377);
INSERT INTO public.meta_recipes VALUES (294, 59, 1.508);
INSERT INTO public.meta_recipes VALUES (297, 295, 1.716);
INSERT INTO public.meta_recipes VALUES (297, 15, 1.3);
INSERT INTO public.meta_recipes VALUES (298, 296, 1.099);
INSERT INTO public.meta_recipes VALUES (298, 15, 0.8);
INSERT INTO public.meta_recipes VALUES (203, 300, 1.142);
INSERT INTO public.meta_recipes VALUES (203, 136, 5);
INSERT INTO public.meta_recipes VALUES (307, 297, 2);
INSERT INTO public.meta_recipes VALUES (307, 298, 1);
INSERT INTO public.meta_recipes VALUES (106, 102, 1);
INSERT INTO public.meta_recipes VALUES (106, 103, 1);
INSERT INTO public.meta_recipes VALUES (309, 308, 1);
INSERT INTO public.meta_recipes VALUES (309, 103, 80);
INSERT INTO public.meta_recipes VALUES (292, 29, 0.1700);
INSERT INTO public.meta_recipes VALUES (292, 59, 0.0760);
INSERT INTO public.meta_recipes VALUES (64, 2, 3);
INSERT INTO public.meta_recipes VALUES (37, 103, 0.02);
INSERT INTO public.meta_recipes VALUES (103, 282, 1);
INSERT INTO public.meta_recipes VALUES (317, 318, 1);
INSERT INTO public.meta_recipes VALUES (319, 103, 0.3000);
INSERT INTO public.meta_recipes VALUES (320, 59, 0.2);
INSERT INTO public.meta_recipes VALUES (322, 326, 0.1);
INSERT INTO public.meta_recipes VALUES (322, 327, 0.5);
INSERT INTO public.meta_recipes VALUES (327, 4, 0.5);


--
-- Data for Name: metro_categories; Type: TABLE DATA; Schema: public; Owner: dennis
--

INSERT INTO public.metro_categories VALUES (191, '/Food / Trockensortiment / Backen, Zucker, Mehl, Salz, Desserts');
INSERT INTO public.metro_categories VALUES (6, '/Food / Molkereiprodukte / Milch');
INSERT INTO public.metro_categories VALUES (193, '/Food / Trockensortiment / Essig & Öle');
INSERT INTO public.metro_categories VALUES (142, '/Food / Trockensortiment / Essig & Öle');
INSERT INTO public.metro_categories VALUES (194, '/Food / Tiefkühl / Gemüse');
INSERT INTO public.metro_categories VALUES (143, '/Food / Käse');
INSERT INTO public.metro_categories VALUES (41, '/Food / Tiefkühl / Gemüse');
INSERT INTO public.metro_categories VALUES (4, '/Food / Trockensortiment / Konserven');
INSERT INTO public.metro_categories VALUES (5, '/Food / Alkoholische Getränke / Wein');
INSERT INTO public.metro_categories VALUES (135, '');
INSERT INTO public.metro_categories VALUES (42, '/Food / Käse');
INSERT INTO public.metro_categories VALUES (7, '/Food / Trockensortiment / Nudeln & Teigwaren');
INSERT INTO public.metro_categories VALUES (8, '/Food / Molkereiprodukte / Butter, Aufstrich, Fette');
INSERT INTO public.metro_categories VALUES (144, '/Food / Käse');
INSERT INTO public.metro_categories VALUES (9, '');
INSERT INTO public.metro_categories VALUES (10, '/Food / Molkereiprodukte / Sahne');
INSERT INTO public.metro_categories VALUES (91, '/Food / Trockensortiment / Essig & Öle');
INSERT INTO public.metro_categories VALUES (11, '/Food / Gemüse / Pilze & Kräuter');
INSERT INTO public.metro_categories VALUES (195, '/Food / Trockensortiment / Gewürze');
INSERT INTO public.metro_categories VALUES (12, '/Food / Molkereiprodukte / Sahne');
INSERT INTO public.metro_categories VALUES (13, '/Food / Gemüse / Kartoffeln');
INSERT INTO public.metro_categories VALUES (14, '/Food / Käse');
INSERT INTO public.metro_categories VALUES (15, '/Food / Tiefkühl / Gemüse');
INSERT INTO public.metro_categories VALUES (145, '');
INSERT INTO public.metro_categories VALUES (16, '/Food / Gemüse / Pilze & Kräuter / Champignons/Food / Gemüse / Pilze & Kräuter');
INSERT INTO public.metro_categories VALUES (17, '/Food / Gemüse / Fruchtgemüse / Paprika & Peperoni');
INSERT INTO public.metro_categories VALUES (18, '/Food / Gemüse / Fruchtgemüse / Zucchini');
INSERT INTO public.metro_categories VALUES (60, '/Food / Trockensortiment / Gewürze');
INSERT INTO public.metro_categories VALUES (61, '/Food / Trockensortiment / Gewürze');
INSERT INTO public.metro_categories VALUES (19, '/Food / Convenience / Pizza');
INSERT INTO public.metro_categories VALUES (92, '/Food / Gemüse / Salat/Food / Gemüse / Salat / Spinat');
INSERT INTO public.metro_categories VALUES (146, '');
INSERT INTO public.metro_categories VALUES (20, '/Food / Käse');
INSERT INTO public.metro_categories VALUES (22, '/Food / Trockensortiment / Reis & Hülsenfrüchte');
INSERT INTO public.metro_categories VALUES (23, '/Food / Alkoholische Getränke / Wein');
INSERT INTO public.metro_categories VALUES (24, '/Food / Tiefkühl / Gemüse');
INSERT INTO public.metro_categories VALUES (62, '/Food / Trockensortiment / Konserven');
INSERT INTO public.metro_categories VALUES (63, '/Food / Molkereiprodukte / Butter, Aufstrich, Fette');
INSERT INTO public.metro_categories VALUES (25, '');
INSERT INTO public.metro_categories VALUES (64, '/Food / Trockensortiment / Gewürze');
INSERT INTO public.metro_categories VALUES (101, '');
INSERT INTO public.metro_categories VALUES (26, '/Food / Trockensortiment / Gewürze');
INSERT INTO public.metro_categories VALUES (27, '/Food / Molkereiprodukte / Joghurt & Quark & Desserts');
INSERT INTO public.metro_categories VALUES (102, '/Food / Trockensortiment / Backen, Zucker, Mehl, Salz, Desserts');
INSERT INTO public.metro_categories VALUES (28, '/Food / Trockensortiment / Gewürze');
INSERT INTO public.metro_categories VALUES (43, '/Food / Fleisch & Wurstwaren / Wurst & Schinken / Schinken & Speck / Kochschinken & Bratenaufschnitt');
INSERT INTO public.metro_categories VALUES (29, '/Food / Fleisch & Wurstwaren / Wurst & Schinken / Rohwurst');
INSERT INTO public.metro_categories VALUES (44, '/Food / Molkereiprodukte / Sahne');
INSERT INTO public.metro_categories VALUES (65, '/Food / Molkereiprodukte / Butter, Aufstrich, Fette');
INSERT INTO public.metro_categories VALUES (147, '/Food / Tiefkühl / Torten, Kuchen & Desserts');
INSERT INTO public.metro_categories VALUES (66, '/Food / Tiefkühl / Gemüse');
INSERT INTO public.metro_categories VALUES (67, '/Food / Trockensortiment / Backen, Zucker, Mehl, Salz, Desserts');
INSERT INTO public.metro_categories VALUES (30, '/Food / Gemüse / Pilze & Kräuter');
INSERT INTO public.metro_categories VALUES (93, '/Food / Tiefkühl / Gemüse');
INSERT INTO public.metro_categories VALUES (103, '/Food / Gemüse / Pilze & Kräuter');
INSERT INTO public.metro_categories VALUES (104, '/Food / Frühstück, Kaffee & Tee / Tee, Kaffee & Kakao / Barsirup & Kaffeesirup');
INSERT INTO public.metro_categories VALUES (105, '/Food / Fleisch & Wurstwaren / Wurst & Schinken / Schinken & Speck / Kochschinken & Bratenaufschnitt');
INSERT INTO public.metro_categories VALUES (68, '/Food / Trockensortiment / Ketchup, Saucen, Mayonnaise, Senf');
INSERT INTO public.metro_categories VALUES (69, '/Food / Tiefkühl / Fertiggerichte & Fingerfood');
INSERT INTO public.metro_categories VALUES (106, '/Food / Trockensortiment / Konserven');
INSERT INTO public.metro_categories VALUES (107, '');
INSERT INTO public.metro_categories VALUES (108, '/Food / Gemüse / Zwiebeln & Knoblauch / Zwiebeln');
INSERT INTO public.metro_categories VALUES (45, '/Food / Tiefkühl / Gemüse');
INSERT INTO public.metro_categories VALUES (70, '/Food / Fleisch & Wurstwaren / Wurst & Schinken / Brühwurst / Bratwurst');
INSERT INTO public.metro_categories VALUES (71, '/Food / Trockensortiment / Gewürze/Food / Trockensortiment / Backen, Zucker, Mehl, Salz, Desserts');
INSERT INTO public.metro_categories VALUES (109, '/Food / Trockensortiment / Konserven');
INSERT INTO public.metro_categories VALUES (110, '/Food / Käse');
INSERT INTO public.metro_categories VALUES (111, '/Food / Trockensortiment / Gewürze');
INSERT INTO public.metro_categories VALUES (72, '/Food / Gemüse / Convenience');
INSERT INTO public.metro_categories VALUES (73, '/Food / Fleisch & Wurstwaren / Fleisch / Schwein');
INSERT INTO public.metro_categories VALUES (136, '/Food / Fleisch & Wurstwaren / Fleisch / Rind / Irish Beef/Food / Fleisch & Wurstwaren / Fleisch / Rind');
INSERT INTO public.metro_categories VALUES (74, '/Food / Gemüse / Fruchtgemüse / Gurken');
INSERT INTO public.metro_categories VALUES (75, '/Food / Trockensortiment / Gewürze');
INSERT INTO public.metro_categories VALUES (76, '/Food / Tiefkühl / Gemüse');
INSERT INTO public.metro_categories VALUES (50, '');
INSERT INTO public.metro_categories VALUES (77, '/Food / Trockensortiment / Gewürze');
INSERT INTO public.metro_categories VALUES (112, '/Food / Käse');
INSERT INTO public.metro_categories VALUES (148, '/Food / Trockensortiment / Ketchup, Saucen, Mayonnaise, Senf/Food / Trockensortiment / Gewürze');
INSERT INTO public.metro_categories VALUES (149, '/Food / Trockensortiment / Nudeln & Teigwaren');
INSERT INTO public.metro_categories VALUES (113, '/Food / Gemüse / Salat/Food / Gemüse / Salat / Rucola');
INSERT INTO public.metro_categories VALUES (31, '/Food / Obst / Äpfel');
INSERT INTO public.metro_categories VALUES (150, '/Food / Obst / Birnen');
INSERT INTO public.metro_categories VALUES (151, '/Food / Trockensortiment / Konserven');
INSERT INTO public.metro_categories VALUES (152, '/Food / Tiefkühl / Obst');
INSERT INTO public.metro_categories VALUES (153, '/Food / Tiefkühl / Obst');
INSERT INTO public.metro_categories VALUES (154, '/Food / Tiefkühl / Obst');
INSERT INTO public.metro_categories VALUES (155, '/Food / Obst / Trauben');
INSERT INTO public.metro_categories VALUES (156, '/Non-Food / Gastro & Haushalt / Geschirr, Besteck & Gläser / Einweggeschirr');
INSERT INTO public.metro_categories VALUES (157, '/Food / Käse');
INSERT INTO public.metro_categories VALUES (158, '/Food / Frühstück, Kaffee & Tee / Frühstück');
INSERT INTO public.metro_categories VALUES (196, '/Food / Alkoholfreie Getränke / Wasser & Wasserfilter');
INSERT INTO public.metro_categories VALUES (197, '/Food / Trockensortiment / Konserven');
INSERT INTO public.metro_categories VALUES (189, '/Food / Tiefkühl / Speiseeis');
INSERT INTO public.metro_categories VALUES (190, '/Food / Trockensortiment / Gewürze');
INSERT INTO public.metro_categories VALUES (198, '/Food / Trockensortiment / Backen, Zucker, Mehl, Salz, Desserts');
INSERT INTO public.metro_categories VALUES (78, '/Food / Molkereiprodukte / Butter, Aufstrich, Fette');
INSERT INTO public.metro_categories VALUES (94, '/Food / Trockensortiment / Backen, Zucker, Mehl, Salz, Desserts');
INSERT INTO public.metro_categories VALUES (141, '/Food / Trockensortiment / Gewürze');
INSERT INTO public.metro_categories VALUES (127, '/Food / Tiefkühl / Obst');
INSERT INTO public.metro_categories VALUES (199, '/Food / Molkereiprodukte / Sahne');
INSERT INTO public.metro_categories VALUES (80, '/Food / Molkereiprodukte / Joghurt & Quark & Desserts');
INSERT INTO public.metro_categories VALUES (137, '/Food / Gemüse / Wurzelgemüse / Radieschen/Food / Gemüse / Wurzelgemüse');
INSERT INTO public.metro_categories VALUES (117, '/Food / Tiefkühl / Gemüse');
INSERT INTO public.metro_categories VALUES (118, '/Food / Trockensortiment / Reis & Hülsenfrüchte');
INSERT INTO public.metro_categories VALUES (138, '/Food / Käse');
INSERT INTO public.metro_categories VALUES (119, '/Food / Gemüse / Pilze & Kräuter');
INSERT INTO public.metro_categories VALUES (159, '/Food / Trockensortiment / Gewürze');
INSERT INTO public.metro_categories VALUES (120, '/Food / Trockensortiment / Gewürze');
INSERT INTO public.metro_categories VALUES (121, '/Food / Obst / Nüsse und getrocknete Früchte');
INSERT INTO public.metro_categories VALUES (200, '/Food / Fleisch & Wurstwaren / Fleisch / Rind / Irish Beef/Food / Fleisch & Wurstwaren / Fleisch / Rind');
INSERT INTO public.metro_categories VALUES (81, '/Food / Feinkost / Marinaden/Food / Gemüse / Fruchtgemüse / Tomate');
INSERT INTO public.metro_categories VALUES (201, '/Food / Trockensortiment / Asia Food');
INSERT INTO public.metro_categories VALUES (160, '/Food / Trockensortiment / Asia Food');
INSERT INTO public.metro_categories VALUES (161, '');
INSERT INTO public.metro_categories VALUES (139, '/Food / Trockensortiment / Konserven');
INSERT INTO public.metro_categories VALUES (162, '/Food / Trockensortiment / Konserven');
INSERT INTO public.metro_categories VALUES (163, '/Food / Gemüse / Fruchtgemüse / Paprika & Peperoni');
INSERT INTO public.metro_categories VALUES (182, '/Food / Trockensortiment / Gewürze');
INSERT INTO public.metro_categories VALUES (122, '/Food / Trockensortiment / Backen, Zucker, Mehl, Salz, Desserts');
INSERT INTO public.metro_categories VALUES (124, '/Food / Trockensortiment / Reis & Hülsenfrüchte');
INSERT INTO public.metro_categories VALUES (125, '');
INSERT INTO public.metro_categories VALUES (164, '/Food / Gemüse / Convenience');
INSERT INTO public.metro_categories VALUES (126, '/Food / Frühstück, Kaffee & Tee / Frühstück');
INSERT INTO public.metro_categories VALUES (165, '/Food / Trockensortiment / Konserven');
INSERT INTO public.metro_categories VALUES (166, '/Food / Süßes & Salziges / Saisonales / Weihnachten');
INSERT INTO public.metro_categories VALUES (248, '/Food / Trockensortiment / Gewürze');
INSERT INTO public.metro_categories VALUES (183, '/Food / Trockensortiment / Nudeln & Teigwaren');
INSERT INTO public.metro_categories VALUES (55, '');
INSERT INTO public.metro_categories VALUES (167, '/Food / Feinkost / Glutenfreie/Vegane Produkte');
INSERT INTO public.metro_categories VALUES (168, '/Food / Feinkost / Sandwiches & Aufstriche');
INSERT INTO public.metro_categories VALUES (169, '/Food / Gemüse / Wurzelgemüse');
INSERT INTO public.metro_categories VALUES (170, '/Food / Fleisch & Wurstwaren / Wurst & Schinken / Brühwurst / Würstchen');
INSERT INTO public.metro_categories VALUES (128, '/Food / Convenience / Pasta & Fertiggerichte');
INSERT INTO public.metro_categories VALUES (129, '/Food / Frühstück, Kaffee & Tee / Frühstück');
INSERT INTO public.metro_categories VALUES (171, '/Food / Molkereiprodukte / Milch');
INSERT INTO public.metro_categories VALUES (130, '/Food / Tiefkühl / Kartoffelprodukte und Backwaren');
INSERT INTO public.metro_categories VALUES (131, '/Food / Trockensortiment / Konserven');
INSERT INTO public.metro_categories VALUES (82, '/Food / Gemüse / Fruchtgemüse / Aubergine');
INSERT INTO public.metro_categories VALUES (132, '/Food / Trockensortiment / Konserven');
INSERT INTO public.metro_categories VALUES (83, '/Food / Gemüse / Zwiebeln & Knoblauch / Zwiebeln');
INSERT INTO public.metro_categories VALUES (84, '/Food / Gemüse / Salat');
INSERT INTO public.metro_categories VALUES (85, '/Food / Backwaren & Aufbacken / Zwieback & Trockenbrot/Food / Backwaren & Aufbacken / Brot');
INSERT INTO public.metro_categories VALUES (140, '/Food / Tiefkühl / Gemüse');
INSERT INTO public.metro_categories VALUES (184, '/Food / Trockensortiment / Backen, Zucker, Mehl, Salz, Desserts');
INSERT INTO public.metro_categories VALUES (172, '/Food / Trockensortiment / Backen, Zucker, Mehl, Salz, Desserts');
INSERT INTO public.metro_categories VALUES (32, '');
INSERT INTO public.metro_categories VALUES (133, '/Food / Süßes & Salziges / Schokolade');
INSERT INTO public.metro_categories VALUES (202, '/Food / Süßes & Salziges / Gebäck & Kekse');
INSERT INTO public.metro_categories VALUES (86, '/Food / Süßes & Salziges / Gebäck & Kekse');
INSERT INTO public.metro_categories VALUES (87, '/Food / Süßes & Salziges / Snack, Chips & Dips / Chips & Snacks');
INSERT INTO public.metro_categories VALUES (88, '');
INSERT INTO public.metro_categories VALUES (237, '');
INSERT INTO public.metro_categories VALUES (89, '/Food / Trockensortiment / Asia Food');
INSERT INTO public.metro_categories VALUES (33, '/Food / Obst / Nüsse und getrocknete Früchte');
INSERT INTO public.metro_categories VALUES (34, '/Food / Süßes & Salziges / Gebäck & Kekse');
INSERT INTO public.metro_categories VALUES (134, '/Food / Gemüse / Zwiebeln & Knoblauch / Zwiebeln');
INSERT INTO public.metro_categories VALUES (173, '/Food / Obst / Nüsse und getrocknete Früchte');
INSERT INTO public.metro_categories VALUES (46, '/Food / Trockensortiment / Backen, Zucker, Mehl, Salz, Desserts');
INSERT INTO public.metro_categories VALUES (35, '/Food / Trockensortiment / Backen, Zucker, Mehl, Salz, Desserts');
INSERT INTO public.metro_categories VALUES (36, '/Food / Trockensortiment / Backen, Zucker, Mehl, Salz, Desserts');
INSERT INTO public.metro_categories VALUES (47, '/Food / Käse');
INSERT INTO public.metro_categories VALUES (203, '/Food / Trockensortiment / Essig & Öle');
INSERT INTO public.metro_categories VALUES (48, '/Food / Gemüse / Pilze & Kräuter');
INSERT INTO public.metro_categories VALUES (90, '/Food / Käse');
INSERT INTO public.metro_categories VALUES (185, '/Food / Tiefkühl / Gemüse');
INSERT INTO public.metro_categories VALUES (204, '/Food / Backwaren & Aufbacken / Zwieback & Trockenbrot/Food / Backwaren & Aufbacken / Toastbrot');
INSERT INTO public.metro_categories VALUES (205, '/Food / Käse');
INSERT INTO public.metro_categories VALUES (240, '/Food / Trockensortiment / Backen, Zucker, Mehl, Salz, Desserts');
INSERT INTO public.metro_categories VALUES (174, '/Food / Gemüse / Wurzelgemüse');
INSERT INTO public.metro_categories VALUES (38, '/Food / Gemüse / Kohlgemüse');
INSERT INTO public.metro_categories VALUES (39, '/Food / Convenience / Pizza');
INSERT INTO public.metro_categories VALUES (175, '/Food / Frühstück, Kaffee & Tee / Tee, Kaffee & Kakao / Kakao');
INSERT INTO public.metro_categories VALUES (176, '/Food / Trockensortiment / Gewürze');
INSERT INTO public.metro_categories VALUES (40, '/Food / Backwaren & Aufbacken / Brot/Food / Backwaren & Aufbacken / Baguette, Ciabatta & Fladenbrot');
INSERT INTO public.metro_categories VALUES (177, '/Food / Trockensortiment / Backen, Zucker, Mehl, Salz, Desserts');
INSERT INTO public.metro_categories VALUES (178, '/Food / Trockensortiment / Asia Food');
INSERT INTO public.metro_categories VALUES (179, '/Food / Trockensortiment / Konserven');
INSERT INTO public.metro_categories VALUES (206, '/Food / Feinkost / Glutenfreie/Vegane Produkte');
INSERT INTO public.metro_categories VALUES (180, '');
INSERT INTO public.metro_categories VALUES (207, '/Food / Gemüse / Pilze & Kräuter');
INSERT INTO public.metro_categories VALUES (208, '/Food / Backwaren & Aufbacken / Zwieback & Trockenbrot');
INSERT INTO public.metro_categories VALUES (186, '/Food / Alkoholfreie Getränke / Säfte & Saftgetränke');
INSERT INTO public.metro_categories VALUES (209, '/Food / Molkereiprodukte / Milch');
INSERT INTO public.metro_categories VALUES (238, '/Food / Trockensortiment / Konserven');
INSERT INTO public.metro_categories VALUES (236, '');
INSERT INTO public.metro_categories VALUES (243, '/Food / Trockensortiment / Reis & Hülsenfrüchte');
INSERT INTO public.metro_categories VALUES (244, '/Food / Trockensortiment / Gewürze');
INSERT INTO public.metro_categories VALUES (245, '/Food / Gemüse / Pilze & Kräuter');
INSERT INTO public.metro_categories VALUES (246, '');
INSERT INTO public.metro_categories VALUES (247, '/Food / Frühstück, Kaffee & Tee / Frühstück/Food / Trockensortiment / Backen, Zucker, Mehl, Salz, Desserts');
INSERT INTO public.metro_categories VALUES (241, '/Food / Frühstück, Kaffee & Tee / Tee, Kaffee & Kakao / Barsirup & Kaffeesirup');
INSERT INTO public.metro_categories VALUES (79, '/Food / Käse');
INSERT INTO public.metro_categories VALUES (271, '');
INSERT INTO public.metro_categories VALUES (181, '/Food / Molkereiprodukte / Eier');
INSERT INTO public.metro_categories VALUES (192, '/Food / Trockensortiment / Reis & Hülsenfrüchte');
INSERT INTO public.metro_categories VALUES (3, '/Food / Fleisch & Wurstwaren / Fleisch / Geflügel / Hähnchen');
INSERT INTO public.metro_categories VALUES (21, '/Food / Fleisch & Wurstwaren / Wurst & Schinken / Brühwurst / Bratwurst/Food / Fleisch & Wurstwaren / Wurst & Schinken / Rohwurst');
INSERT INTO public.metro_categories VALUES (114, '/Food / Trockensortiment / Konserven');
INSERT INTO public.metro_categories VALUES (115, '/Food / Fleisch & Wurstwaren / Wurst & Schinken / Brühwurst / Brühwurst/Aufschnitt');
INSERT INTO public.metro_categories VALUES (242, '/Food / Trockensortiment / Konserven');
INSERT INTO public.metro_categories VALUES (249, '/Food / Trockensortiment / Backen, Zucker, Mehl, Salz, Desserts');
INSERT INTO public.metro_categories VALUES (210, '/Food / Convenience / Pasta & Fertiggerichte');
INSERT INTO public.metro_categories VALUES (252, '/Food / Molkereiprodukte / Milch');
INSERT INTO public.metro_categories VALUES (250, '/Food / Käse');
INSERT INTO public.metro_categories VALUES (251, '/Food / Frühstück, Kaffee & Tee / Frühstück');
INSERT INTO public.metro_categories VALUES (255, '/Food / Trockensortiment / Essig & Öle');
INSERT INTO public.metro_categories VALUES (258, '/Food / Trockensortiment / Backen, Zucker, Mehl, Salz, Desserts');
INSERT INTO public.metro_categories VALUES (254, '/Food / Gemüse / Kartoffeln');
INSERT INTO public.metro_categories VALUES (256, '');
INSERT INTO public.metro_categories VALUES (266, '/Food / Fleisch & Wurstwaren / Fleisch / Schwein');
INSERT INTO public.metro_categories VALUES (268, '/Food / Trockensortiment / Asia Food');
INSERT INTO public.metro_categories VALUES (270, '/Food / Trockensortiment / Asia Food');
INSERT INTO public.metro_categories VALUES (269, '/Food / Süßes & Salziges / Snack, Chips & Dips');
INSERT INTO public.metro_categories VALUES (267, '/Food / Trockensortiment / Konserven');


--
-- Data for Name: recipe_ingredients; Type: TABLE DATA; Schema: public; Owner: dennis
--

INSERT INTO public.recipe_ingredients VALUES (9, 134, 6, 0);
INSERT INTO public.recipe_ingredients VALUES (9, 8, 3, 0);
INSERT INTO public.recipe_ingredients VALUES (9, 21, 2, 0);
INSERT INTO public.recipe_ingredients VALUES (9, 34, 1, 0);
INSERT INTO public.recipe_ingredients VALUES (9, 1, 0.6, 0);
INSERT INTO public.recipe_ingredients VALUES (9, 12, 800, 1);
INSERT INTO public.recipe_ingredients VALUES (9, 6, 50, 8);
INSERT INTO public.recipe_ingredients VALUES (9, 5, 150, 1);
INSERT INTO public.recipe_ingredients VALUES (10, 28, 5, 0);
INSERT INTO public.recipe_ingredients VALUES (10, 8, 6, 5);
INSERT INTO public.recipe_ingredients VALUES (10, 52, 500, 1);
INSERT INTO public.recipe_ingredients VALUES (10, 5, 100, 1);
INSERT INTO public.recipe_ingredients VALUES (10, 26, 100, 1);
INSERT INTO public.recipe_ingredients VALUES (10, 53, 200, 1);
INSERT INTO public.recipe_ingredients VALUES (10, 25, 75, 1);
INSERT INTO public.recipe_ingredients VALUES (10, 2, 20, 5);
INSERT INTO public.recipe_ingredients VALUES (12, 34, 6, 5);
INSERT INTO public.recipe_ingredients VALUES (12, 28, 3, 0);
INSERT INTO public.recipe_ingredients VALUES (12, 35, 4, 5);
INSERT INTO public.recipe_ingredients VALUES (12, 8, 4, 5);
INSERT INTO public.recipe_ingredients VALUES (12, 7, 8, 4);
INSERT INTO public.recipe_ingredients VALUES (12, 36, 50, 1);
INSERT INTO public.recipe_ingredients VALUES (12, 24, 50, 1);
INSERT INTO public.recipe_ingredients VALUES (12, 37, 50, 1);
INSERT INTO public.recipe_ingredients VALUES (12, 38, 50, 1);
INSERT INTO public.recipe_ingredients VALUES (12, 5, 50, 1);
INSERT INTO public.recipe_ingredients VALUES (12, 26, 50, 1);
INSERT INTO public.recipe_ingredients VALUES (13, 39, 3, 10);
INSERT INTO public.recipe_ingredients VALUES (13, 41, 12, 4);
INSERT INTO public.recipe_ingredients VALUES (13, 42, 450, 1);
INSERT INTO public.recipe_ingredients VALUES (13, 8, 2, 5);
INSERT INTO public.recipe_ingredients VALUES (13, 2, 6, 5);
INSERT INTO public.recipe_ingredients VALUES (13, 7, 6, 4);
INSERT INTO public.recipe_ingredients VALUES (12, 20, 6, 7);
INSERT INTO public.recipe_ingredients VALUES (102, 1, 920, 1);
INSERT INTO public.recipe_ingredients VALUES (102, 4, 600, 1);
INSERT INTO public.recipe_ingredients VALUES (102, 5, 20, 1);
INSERT INTO public.recipe_ingredients VALUES (102, 7, 30, 1);
INSERT INTO public.recipe_ingredients VALUES (102, 64, 8, 1);
INSERT INTO public.recipe_ingredients VALUES (102, 199, 1, 4);
INSERT INTO public.recipe_ingredients VALUES (102, 219, 80, 1);
INSERT INTO public.recipe_ingredients VALUES (229, 12, 100, 1);
INSERT INTO public.recipe_ingredients VALUES (16, 73, 500, 1);
INSERT INTO public.recipe_ingredients VALUES (16, 24, 25, 1);
INSERT INTO public.recipe_ingredients VALUES (21, 54, 120, 1);
INSERT INTO public.recipe_ingredients VALUES (21, 22, 120, 1);
INSERT INTO public.recipe_ingredients VALUES (229, 73, 800, 1);
INSERT INTO public.recipe_ingredients VALUES (229, 194, 2, 5);
INSERT INTO public.recipe_ingredients VALUES (229, 20, 3, 7);
INSERT INTO public.recipe_ingredients VALUES (23, 52, 50, 1);
INSERT INTO public.recipe_ingredients VALUES (25, 9, 120, 1);
INSERT INTO public.recipe_ingredients VALUES (24, 33, 120, 1);
INSERT INTO public.recipe_ingredients VALUES (27, 96, 300, 1);
INSERT INTO public.recipe_ingredients VALUES (26, 9, 120, 1);
INSERT INTO public.recipe_ingredients VALUES (30, 5, 92, 1);
INSERT INTO public.recipe_ingredients VALUES (30, 6, 100, 8);
INSERT INTO public.recipe_ingredients VALUES (13, 63, 900, 1);
INSERT INTO public.recipe_ingredients VALUES (30, 8, 2, 0);
INSERT INTO public.recipe_ingredients VALUES (32, 39, 275, 1);
INSERT INTO public.recipe_ingredients VALUES (30, 14, 15, 1);
INSERT INTO public.recipe_ingredients VALUES (214, 229, 1, 0);
INSERT INTO public.recipe_ingredients VALUES (38, 28, 2, 0);
INSERT INTO public.recipe_ingredients VALUES (38, 77, 0.75, 0);
INSERT INTO public.recipe_ingredients VALUES (38, 66, 0.75, 0);
INSERT INTO public.recipe_ingredients VALUES (38, 165, 0.5, 0);
INSERT INTO public.recipe_ingredients VALUES (38, 8, 0.3, 0);
INSERT INTO public.recipe_ingredients VALUES (38, 82, 10, 5);
INSERT INTO public.recipe_ingredients VALUES (38, 24, 10, 1);
INSERT INTO public.recipe_ingredients VALUES (38, 4, 8, 2);
INSERT INTO public.recipe_ingredients VALUES (38, 53, 10, 1);
INSERT INTO public.recipe_ingredients VALUES (38, 12, 10, 1);
INSERT INTO public.recipe_ingredients VALUES (38, 38, 5, 1);
INSERT INTO public.recipe_ingredients VALUES (38, 26, 5, 1);
INSERT INTO public.recipe_ingredients VALUES (9, 28, 6, 0);
INSERT INTO public.recipe_ingredients VALUES (30, 20, 12, 7);
INSERT INTO public.recipe_ingredients VALUES (30, 26, 8, 1);
INSERT INTO public.recipe_ingredients VALUES (40, 168, 500, 1);
INSERT INTO public.recipe_ingredients VALUES (38, 145, 10, 1);
INSERT INTO public.recipe_ingredients VALUES (33, 63, 450, 1);
INSERT INTO public.recipe_ingredients VALUES (38, 148, 2.25, 0);
INSERT INTO public.recipe_ingredients VALUES (40, 9, 200, 1);
INSERT INTO public.recipe_ingredients VALUES (40, 8, 2, 5);
INSERT INTO public.recipe_ingredients VALUES (40, 90, 1.5, 1);
INSERT INTO public.recipe_ingredients VALUES (40, 5, 7.5, 1);
INSERT INTO public.recipe_ingredients VALUES (40, 26, 10, 1);
INSERT INTO public.recipe_ingredients VALUES (30, 34, 1.3500, 0);
INSERT INTO public.recipe_ingredients VALUES (21, 5, 4.75, 1);
INSERT INTO public.recipe_ingredients VALUES (16, 5, 5, 1);
INSERT INTO public.recipe_ingredients VALUES (27, 114, 3.1418, 1);
INSERT INTO public.recipe_ingredients VALUES (30, 37, 10, 1);
INSERT INTO public.recipe_ingredients VALUES (30, 51, 7, 1);
INSERT INTO public.recipe_ingredients VALUES (8, 27, 400, 8);
INSERT INTO public.recipe_ingredients VALUES (8, 8, 1, 5);
INSERT INTO public.recipe_ingredients VALUES (8, 9, 200, 1);
INSERT INTO public.recipe_ingredients VALUES (8, 158, 1, 0);
INSERT INTO public.recipe_ingredients VALUES (22, 194, 1.5, 5);
INSERT INTO public.recipe_ingredients VALUES (23, 194, 1, 5);
INSERT INTO public.recipe_ingredients VALUES (42, 169, 100, 1);
INSERT INTO public.recipe_ingredients VALUES (214, 152, 500, 1);
INSERT INTO public.recipe_ingredients VALUES (214, 114, 5, 1);
INSERT INTO public.recipe_ingredients VALUES (214, 55, 20, 1);
INSERT INTO public.recipe_ingredients VALUES (42, 170, 500, 1);
INSERT INTO public.recipe_ingredients VALUES (43, 52, 150, 1);
INSERT INTO public.recipe_ingredients VALUES (43, 33, 300, 1);
INSERT INTO public.recipe_ingredients VALUES (43, 18, 250, 1);
INSERT INTO public.recipe_ingredients VALUES (43, 172, 1500, 1);
INSERT INTO public.recipe_ingredients VALUES (44, 33, 200, 1);
INSERT INTO public.recipe_ingredients VALUES (49, 59, 200, 1);
INSERT INTO public.recipe_ingredients VALUES (48, 34, 100, 1);
INSERT INTO public.recipe_ingredients VALUES (48, 173, 100, 1);
INSERT INTO public.recipe_ingredients VALUES (47, 35, 150, 1);
INSERT INTO public.recipe_ingredients VALUES (47, 174, 50, 1);
INSERT INTO public.recipe_ingredients VALUES (89, 70, 15, 1);
INSERT INTO public.recipe_ingredients VALUES (46, 16, 150, 1);
INSERT INTO public.recipe_ingredients VALUES (33, 42, 200, 1);
INSERT INTO public.recipe_ingredients VALUES (89, 206, 50, 1);
INSERT INTO public.recipe_ingredients VALUES (89, 207, 20, 1);
INSERT INTO public.recipe_ingredients VALUES (229, 210, 150, 1);
INSERT INTO public.recipe_ingredients VALUES (229, 38, 2, 1);
INSERT INTO public.recipe_ingredients VALUES (51, 5, 10, 1);
INSERT INTO public.recipe_ingredients VALUES (51, 26, 15, 1);
INSERT INTO public.recipe_ingredients VALUES (51, 20, 1, 7);
INSERT INTO public.recipe_ingredients VALUES (51, 8, 1, 5);
INSERT INTO public.recipe_ingredients VALUES (229, 37, 2, 1);
INSERT INTO public.recipe_ingredients VALUES (51, 46, 150, 8);
INSERT INTO public.recipe_ingredients VALUES (51, 45, 300, 1);
INSERT INTO public.recipe_ingredients VALUES (51, 7, 2, 4);
INSERT INTO public.recipe_ingredients VALUES (51, 33, 250, 1);
INSERT INTO public.recipe_ingredients VALUES (51, 66, 1, 5);
INSERT INTO public.recipe_ingredients VALUES (229, 145, 2, 1);
INSERT INTO public.recipe_ingredients VALUES (229, 36, 2, 1);
INSERT INTO public.recipe_ingredients VALUES (229, 5, 2, 1);
INSERT INTO public.recipe_ingredients VALUES (54, 140, 1, 5);
INSERT INTO public.recipe_ingredients VALUES (54, 176, 2, 5);
INSERT INTO public.recipe_ingredients VALUES (54, 20, 1, 7);
INSERT INTO public.recipe_ingredients VALUES (54, 34, 2, 5);
INSERT INTO public.recipe_ingredients VALUES (54, 21, 500, 1);
INSERT INTO public.recipe_ingredients VALUES (54, 86, 500, 1);
INSERT INTO public.recipe_ingredients VALUES (54, 42, 200, 1);
INSERT INTO public.recipe_ingredients VALUES (54, 174, 100, 1);
INSERT INTO public.recipe_ingredients VALUES (54, 7, 4, 4);
INSERT INTO public.recipe_ingredients VALUES (54, 5, 5, 1);
INSERT INTO public.recipe_ingredients VALUES (54, 26, 5, 1);
INSERT INTO public.recipe_ingredients VALUES (50, 8, 6, 5);
INSERT INTO public.recipe_ingredients VALUES (50, 52, 250, 1);
INSERT INTO public.recipe_ingredients VALUES (50, 66, 2, 5);
INSERT INTO public.recipe_ingredients VALUES (50, 132, 0.5000, 3);
INSERT INTO public.recipe_ingredients VALUES (28, 4, 520, 8);
INSERT INTO public.recipe_ingredients VALUES (31, 90, 0.18, 1);
INSERT INTO public.recipe_ingredients VALUES (31, 26, 0.2, 1);
INSERT INTO public.recipe_ingredients VALUES (33, 8, 120, 1);
INSERT INTO public.recipe_ingredients VALUES (33, 183, 240, 1);
INSERT INTO public.recipe_ingredients VALUES (28, 64, 0.2500, 1);
INSERT INTO public.recipe_ingredients VALUES (28, 160, 1, 0);
INSERT INTO public.recipe_ingredients VALUES (229, 26, 2, 1);
INSERT INTO public.recipe_ingredients VALUES (213, 51, 2, 1);
INSERT INTO public.recipe_ingredients VALUES (57, 134, 9, 0);
INSERT INTO public.recipe_ingredients VALUES (45, 70, 200, 1);
INSERT INTO public.recipe_ingredients VALUES (30, 53, 16, 1);
INSERT INTO public.recipe_ingredients VALUES (30, 68, 25, 1);
INSERT INTO public.recipe_ingredients VALUES (74, 6, 0.010, 2);
INSERT INTO public.recipe_ingredients VALUES (213, 159, 200, 1);
INSERT INTO public.recipe_ingredients VALUES (21, 26, 6, 1);
INSERT INTO public.recipe_ingredients VALUES (70, 1, 1, 0);
INSERT INTO public.recipe_ingredients VALUES (70, 55, 125, 1);
INSERT INTO public.recipe_ingredients VALUES (70, 18, 150, 1);
INSERT INTO public.recipe_ingredients VALUES (70, 64, 1, 5);
INSERT INTO public.recipe_ingredients VALUES (70, 2, 2, 5);
INSERT INTO public.recipe_ingredients VALUES (70, 15, 500, 8);
INSERT INTO public.recipe_ingredients VALUES (70, 50, 50, 1);
INSERT INTO public.recipe_ingredients VALUES (70, 180, 50, 1);
INSERT INTO public.recipe_ingredients VALUES (70, 181, 50, 1);
INSERT INTO public.recipe_ingredients VALUES (43, 171, 3, 0);
INSERT INTO public.recipe_ingredients VALUES (17, 95, 5, 1);
INSERT INTO public.recipe_ingredients VALUES (71, 96, 1, 0);
INSERT INTO public.recipe_ingredients VALUES (71, 103, 1, 0);
INSERT INTO public.recipe_ingredients VALUES (72, 184, 0.5, 0);
INSERT INTO public.recipe_ingredients VALUES (72, 185, 0.2, 0);
INSERT INTO public.recipe_ingredients VALUES (72, 186, 0.2, 0);
INSERT INTO public.recipe_ingredients VALUES (72, 188, 0.2, 0);
INSERT INTO public.recipe_ingredients VALUES (72, 187, 0.2, 0);
INSERT INTO public.recipe_ingredients VALUES (72, 189, 0.2, 0);
INSERT INTO public.recipe_ingredients VALUES (72, 190, 0.2, 0);
INSERT INTO public.recipe_ingredients VALUES (72, 191, 0.3, 0);
INSERT INTO public.recipe_ingredients VALUES (72, 192, 0.15, 0);
INSERT INTO public.recipe_ingredients VALUES (47, 194, 50, 1);
INSERT INTO public.recipe_ingredients VALUES (73, 195, 0.15, 0);
INSERT INTO public.recipe_ingredients VALUES (73, 196, 4, 4);
INSERT INTO public.recipe_ingredients VALUES (73, 5, 1, 3);
INSERT INTO public.recipe_ingredients VALUES (73, 87, 0.5, 3);
INSERT INTO public.recipe_ingredients VALUES (74, 8, 0.5, 0);
INSERT INTO public.recipe_ingredients VALUES (74, 179, 0.5, 0);
INSERT INTO public.recipe_ingredients VALUES (74, 35, 0.6, 0);
INSERT INTO public.recipe_ingredients VALUES (74, 77, 0.5, 0);
INSERT INTO public.recipe_ingredients VALUES (74, 150, 0.005, 0);
INSERT INTO public.recipe_ingredients VALUES (74, 73, 2, 0);
INSERT INTO public.recipe_ingredients VALUES (74, 20, 4, 7);
INSERT INTO public.recipe_ingredients VALUES (81, 55, 1, 3);
INSERT INTO public.recipe_ingredients VALUES (74, 137, 1, 2);
INSERT INTO public.recipe_ingredients VALUES (74, 139, 0.02, 0);
INSERT INTO public.recipe_ingredients VALUES (213, 228, 5, 8);
INSERT INTO public.recipe_ingredients VALUES (213, 4, 600, 1);
INSERT INTO public.recipe_ingredients VALUES (223, 218, 0.25, 0);
INSERT INTO public.recipe_ingredients VALUES (81, 73, 150, 1);
INSERT INTO public.recipe_ingredients VALUES (81, 23, 4, 4);
INSERT INTO public.recipe_ingredients VALUES (81, 7, 4, 4);
INSERT INTO public.recipe_ingredients VALUES (81, 5, 0.4, 3);
INSERT INTO public.recipe_ingredients VALUES (81, 26, 0.4, 3);
INSERT INTO public.recipe_ingredients VALUES (81, 136, 100, 1);
INSERT INTO public.recipe_ingredients VALUES (81, 95, 40, 1);
INSERT INTO public.recipe_ingredients VALUES (81, 183, 250, 1);
INSERT INTO public.recipe_ingredients VALUES (81, 201, 125, 1);
INSERT INTO public.recipe_ingredients VALUES (82, 28, 1, 0);
INSERT INTO public.recipe_ingredients VALUES (82, 194, 120, 1);
INSERT INTO public.recipe_ingredients VALUES (82, 6, 4, 4);
INSERT INTO public.recipe_ingredients VALUES (82, 71, 1, 4);
INSERT INTO public.recipe_ingredients VALUES (82, 202, 5, 4);
INSERT INTO public.recipe_ingredients VALUES (82, 26, 5, 1);
INSERT INTO public.recipe_ingredients VALUES (82, 5, 5, 1);
INSERT INTO public.recipe_ingredients VALUES (82, 203, 1, 9);
INSERT INTO public.recipe_ingredients VALUES (82, 55, 5, 1);
INSERT INTO public.recipe_ingredients VALUES (79, 160, 400, 1);
INSERT INTO public.recipe_ingredients VALUES (79, 198, 7, 1);
INSERT INTO public.recipe_ingredients VALUES (79, 5, 2, 3);
INSERT INTO public.recipe_ingredients VALUES (79, 55, 0.5, 3);
INSERT INTO public.recipe_ingredients VALUES (79, 7, 3, 4);
INSERT INTO public.recipe_ingredients VALUES (79, 4, 230, 8);
INSERT INTO public.recipe_ingredients VALUES (83, 194, 4, 5);
INSERT INTO public.recipe_ingredients VALUES (83, 21, 8, 5);
INSERT INTO public.recipe_ingredients VALUES (83, 35, 2, 5);
INSERT INTO public.recipe_ingredients VALUES (83, 20, 4, 7);
INSERT INTO public.recipe_ingredients VALUES (83, 7, 8, 4);
INSERT INTO public.recipe_ingredients VALUES (83, 38, 20, 1);
INSERT INTO public.recipe_ingredients VALUES (83, 36, 20, 1);
INSERT INTO public.recipe_ingredients VALUES (83, 145, 20, 1);
INSERT INTO public.recipe_ingredients VALUES (83, 5, 10, 1);
INSERT INTO public.recipe_ingredients VALUES (83, 34, 4, 5);
INSERT INTO public.recipe_ingredients VALUES (97, 4, 5, 2);
INSERT INTO public.recipe_ingredients VALUES (84, 103, 1, 5);
INSERT INTO public.recipe_ingredients VALUES (84, 104, 18, 1);
INSERT INTO public.recipe_ingredients VALUES (84, 105, 18, 1);
INSERT INTO public.recipe_ingredients VALUES (85, 21, 4, 5);
INSERT INTO public.recipe_ingredients VALUES (85, 201, 125, 1);
INSERT INTO public.recipe_ingredients VALUES (85, 23, 4, 4);
INSERT INTO public.recipe_ingredients VALUES (85, 7, 4, 4);
INSERT INTO public.recipe_ingredients VALUES (85, 5, 5, 1);
INSERT INTO public.recipe_ingredients VALUES (85, 24, 5, 1);
INSERT INTO public.recipe_ingredients VALUES (97, 5, 5, 4);
INSERT INTO public.recipe_ingredients VALUES (225, 51, 1, 3);
INSERT INTO public.recipe_ingredients VALUES (225, 131, 1, 3);
INSERT INTO public.recipe_ingredients VALUES (225, 138, 2, 8);
INSERT INTO public.recipe_ingredients VALUES (225, 147, 10, 1);
INSERT INTO public.recipe_ingredients VALUES (97, 158, 2, 0);
INSERT INTO public.recipe_ingredients VALUES (30, 73, 8, 0);
INSERT INTO public.recipe_ingredients VALUES (30, 142, 4, 0);
INSERT INTO public.recipe_ingredients VALUES (30, 143, 2.4000, 0);
INSERT INTO public.recipe_ingredients VALUES (30, 161, 21, 1);
INSERT INTO public.recipe_ingredients VALUES (30, 210, 500, 1);
INSERT INTO public.recipe_ingredients VALUES (30, 213, 20, 1);
INSERT INTO public.recipe_ingredients VALUES (99, 212, 1, 0);
INSERT INTO public.recipe_ingredients VALUES (88, 206, 50, 1);
INSERT INTO public.recipe_ingredients VALUES (30, 214, 8, 1);
INSERT INTO public.recipe_ingredients VALUES (86, 28, 250, 1);
INSERT INTO public.recipe_ingredients VALUES (86, 77, 125, 1);
INSERT INTO public.recipe_ingredients VALUES (86, 205, 125, 1);
INSERT INTO public.recipe_ingredients VALUES (86, 194, 20, 1);
INSERT INTO public.recipe_ingredients VALUES (99, 4, 2, 2);
INSERT INTO public.recipe_ingredients VALUES (87, 30, 150, 1);
INSERT INTO public.recipe_ingredients VALUES (90, 160, 640, 1);
INSERT INTO public.recipe_ingredients VALUES (90, 4, 380, 1);
INSERT INTO public.recipe_ingredients VALUES (90, 5, 12, 1);
INSERT INTO public.recipe_ingredients VALUES (92, 160, 50, 1);
INSERT INTO public.recipe_ingredients VALUES (92, 4, 50, 1);
INSERT INTO public.recipe_ingredients VALUES (92, 208, 2, 1);
INSERT INTO public.recipe_ingredients VALUES (4, 4, 1, 2);
INSERT INTO public.recipe_ingredients VALUES (4, 14, 12, 1);
INSERT INTO public.recipe_ingredients VALUES (93, 6, 15, 4);
INSERT INTO public.recipe_ingredients VALUES (93, 28, 5, 0);
INSERT INTO public.recipe_ingredients VALUES (93, 65, 15, 5);
INSERT INTO public.recipe_ingredients VALUES (93, 66, 750, 1);
INSERT INTO public.recipe_ingredients VALUES (93, 67, 0.5, 9);
INSERT INTO public.recipe_ingredients VALUES (93, 77, 1, 0);
INSERT INTO public.recipe_ingredients VALUES (93, 145, 15, 4);
INSERT INTO public.recipe_ingredients VALUES (93, 165, 750, 1);
INSERT INTO public.recipe_ingredients VALUES (93, 194, 5, 5);
INSERT INTO public.recipe_ingredients VALUES (94, 209, 240, 1);
INSERT INTO public.recipe_ingredients VALUES (94, 15, 150, 8);
INSERT INTO public.recipe_ingredients VALUES (94, 194, 1, 5);
INSERT INTO public.recipe_ingredients VALUES (94, 67, 30, 1);
INSERT INTO public.recipe_ingredients VALUES (94, 18, 2, 4);
INSERT INTO public.recipe_ingredients VALUES (94, 5, 1, 3);
INSERT INTO public.recipe_ingredients VALUES (94, 26, 0.5, 3);
INSERT INTO public.recipe_ingredients VALUES (94, 147, 60, 1);
INSERT INTO public.recipe_ingredients VALUES (98, 1, 10, 1);
INSERT INTO public.recipe_ingredients VALUES (98, 26, 0.3000, 1);
INSERT INTO public.recipe_ingredients VALUES (98, 90, 0.2000, 1);
INSERT INTO public.recipe_ingredients VALUES (98, 147, 50, 1);
INSERT INTO public.recipe_ingredients VALUES (98, 194, 1.5000, 5);
INSERT INTO public.recipe_ingredients VALUES (98, 211, 500, 1);
INSERT INTO public.recipe_ingredients VALUES (98, 68, 5, 8);
INSERT INTO public.recipe_ingredients VALUES (101, 168, 1, 0);
INSERT INTO public.recipe_ingredients VALUES (59, 3, 500, 1);
INSERT INTO public.recipe_ingredients VALUES (225, 6, 96, 1);
INSERT INTO public.recipe_ingredients VALUES (103, 26, 2, 1);
INSERT INTO public.recipe_ingredients VALUES (88, 207, 20, 1);
INSERT INTO public.recipe_ingredients VALUES (15, 1, 1000, 1);
INSERT INTO public.recipe_ingredients VALUES (103, 220, 230, 1);
INSERT INTO public.recipe_ingredients VALUES (103, 68, 35, 8);
INSERT INTO public.recipe_ingredients VALUES (103, 7, 60, 8);
INSERT INTO public.recipe_ingredients VALUES (103, 131, 4, 1);
INSERT INTO public.recipe_ingredients VALUES (103, 5, 3, 1);
INSERT INTO public.recipe_ingredients VALUES (103, 20, 4, 1);
INSERT INTO public.recipe_ingredients VALUES (15, 4, 600, 8);
INSERT INTO public.recipe_ingredients VALUES (15, 5, 25, 1);
INSERT INTO public.recipe_ingredients VALUES (15, 7, 15, 1);
INSERT INTO public.recipe_ingredients VALUES (15, 64, 4, 1);
INSERT INTO public.recipe_ingredients VALUES (15, 135, 50, 1);
INSERT INTO public.recipe_ingredients VALUES (108, 18, 25, 1);
INSERT INTO public.recipe_ingredients VALUES (108, 28, 1, 0);
INSERT INTO public.recipe_ingredients VALUES (108, 194, 0.2000, 0);
INSERT INTO public.recipe_ingredients VALUES (108, 221, 0.5000, 0);
INSERT INTO public.recipe_ingredients VALUES (108, 5, 3, 1);
INSERT INTO public.recipe_ingredients VALUES (108, 90, 1, 1);
INSERT INTO public.recipe_ingredients VALUES (108, 132, 4, 1);
INSERT INTO public.recipe_ingredients VALUES (200, 5, 23, 1);
INSERT INTO public.recipe_ingredients VALUES (200, 49, 8, 4);
INSERT INTO public.recipe_ingredients VALUES (200, 51, 16, 1);
INSERT INTO public.recipe_ingredients VALUES (200, 55, 8, 4);
INSERT INTO public.recipe_ingredients VALUES (200, 76, 4, 9);
INSERT INTO public.recipe_ingredients VALUES (200, 77, 4, 0);
INSERT INTO public.recipe_ingredients VALUES (200, 130, 3.7700, 1);
INSERT INTO public.recipe_ingredients VALUES (200, 150, 120, 1);
INSERT INTO public.recipe_ingredients VALUES (200, 161, 0.4800, 1);
INSERT INTO public.recipe_ingredients VALUES (200, 226, 2, 2);
INSERT INTO public.recipe_ingredients VALUES (200, 227, 0.8000, 0);
INSERT INTO public.recipe_ingredients VALUES (200, 235, 183, 1);
INSERT INTO public.recipe_ingredients VALUES (191, 225, 40, 1);
INSERT INTO public.recipe_ingredients VALUES (68, 158, 4, 0);
INSERT INTO public.recipe_ingredients VALUES (76, 4, 100, 1);
INSERT INTO public.recipe_ingredients VALUES (76, 5, 0.5000, 3);
INSERT INTO public.recipe_ingredients VALUES (76, 49, 1, 4);
INSERT INTO public.recipe_ingredients VALUES (76, 197, 0.3000, 0);
INSERT INTO public.recipe_ingredients VALUES (76, 198, 0.5000, 1);
INSERT INTO public.recipe_ingredients VALUES (76, 199, 1, 3);
INSERT INTO public.recipe_ingredients VALUES (76, 200, 50, 1);
INSERT INTO public.recipe_ingredients VALUES (186, 5, 0, 0);
INSERT INTO public.recipe_ingredients VALUES (186, 7, 50, 1);
INSERT INTO public.recipe_ingredients VALUES (186, 8, 150, 1);
INSERT INTO public.recipe_ingredients VALUES (186, 12, 100, 1);
INSERT INTO public.recipe_ingredients VALUES (186, 20, 4, 7);
INSERT INTO public.recipe_ingredients VALUES (186, 26, 0, 0);
INSERT INTO public.recipe_ingredients VALUES (186, 34, 700, 1);
INSERT INTO public.recipe_ingredients VALUES (186, 35, 500, 1);
INSERT INTO public.recipe_ingredients VALUES (186, 37, 0, 0);
INSERT INTO public.recipe_ingredients VALUES (186, 38, 0, 0);
INSERT INTO public.recipe_ingredients VALUES (186, 55, 0, 0);
INSERT INTO public.recipe_ingredients VALUES (186, 173, 400, 1);
INSERT INTO public.recipe_ingredients VALUES (186, 223, 500, 1);
INSERT INTO public.recipe_ingredients VALUES (186, 224, 0, 0);
INSERT INTO public.recipe_ingredients VALUES (55, 5, 5, 1);
INSERT INTO public.recipe_ingredients VALUES (55, 7, 5, 4);
INSERT INTO public.recipe_ingredients VALUES (55, 23, 3, 4);
INSERT INTO public.recipe_ingredients VALUES (55, 26, 5, 1);
INSERT INTO public.recipe_ingredients VALUES (55, 33, 150, 1);
INSERT INTO public.recipe_ingredients VALUES (55, 34, 3, 5);
INSERT INTO public.recipe_ingredients VALUES (55, 76, 4, 5);
INSERT INTO public.recipe_ingredients VALUES (55, 96, 1, 5);
INSERT INTO public.recipe_ingredients VALUES (55, 125, 6, 5);
INSERT INTO public.recipe_ingredients VALUES (55, 176, 1, 5);
INSERT INTO public.recipe_ingredients VALUES (55, 177, 300, 1);
INSERT INTO public.recipe_ingredients VALUES (228, 173, 0.2, 0);
INSERT INTO public.recipe_ingredients VALUES (228, 35, 0.2, 0);
INSERT INTO public.recipe_ingredients VALUES (228, 28, 0.4, 0);
INSERT INTO public.recipe_ingredients VALUES (228, 194, 0.2, 0);
INSERT INTO public.recipe_ingredients VALUES (228, 34, 0.2, 0);
INSERT INTO public.recipe_ingredients VALUES (228, 20, 4, 7);
INSERT INTO public.recipe_ingredients VALUES (228, 210, 0.2, 0);
INSERT INTO public.recipe_ingredients VALUES (203, 222, 0.2, 0);
INSERT INTO public.recipe_ingredients VALUES (228, 38, 2, 1);
INSERT INTO public.recipe_ingredients VALUES (228, 7, 50, 8);
INSERT INTO public.recipe_ingredients VALUES (1, 1, 75, 1);
INSERT INTO public.recipe_ingredients VALUES (1, 5, 50, 1);
INSERT INTO public.recipe_ingredients VALUES (1, 26, 40.5000, 1);
INSERT INTO public.recipe_ingredients VALUES (1, 27, 2, 0);
INSERT INTO public.recipe_ingredients VALUES (1, 31, 5, 0);
INSERT INTO public.recipe_ingredients VALUES (1, 90, 2, 1);
INSERT INTO public.recipe_ingredients VALUES (1, 194, 1, 0);
INSERT INTO public.recipe_ingredients VALUES (228, 5, 1, 1);
INSERT INTO public.recipe_ingredients VALUES (228, 26, 1, 1);
INSERT INTO public.recipe_ingredients VALUES (225, 170, 7, 1);
INSERT INTO public.recipe_ingredients VALUES (225, 237, 100, 1);
INSERT INTO public.recipe_ingredients VALUES (225, 239, 7, 1);
INSERT INTO public.recipe_ingredients VALUES (225, 241, 0.2100, 0);
INSERT INTO public.recipe_ingredients VALUES (225, 243, 14, 1);
INSERT INTO public.recipe_ingredients VALUES (225, 26, 3, 1);
INSERT INTO public.recipe_ingredients VALUES (225, 233, 2.7, 1);
INSERT INTO public.recipe_ingredients VALUES (225, 196, 1.48, 1);
INSERT INTO public.recipe_ingredients VALUES (225, 132, 1, 1);
INSERT INTO public.recipe_ingredients VALUES (225, 231, 1, 1);
INSERT INTO public.recipe_ingredients VALUES (225, 137, 175, 1);
INSERT INTO public.recipe_ingredients VALUES (52, 245, 548, 1);
INSERT INTO public.recipe_ingredients VALUES (52, 4, 6.6000, 0);
INSERT INTO public.recipe_ingredients VALUES (52, 175, 2, 0);
INSERT INTO public.recipe_ingredients VALUES (52, 90, 26, 1);
INSERT INTO public.recipe_ingredients VALUES (52, 130, 8, 1);
INSERT INTO public.recipe_ingredients VALUES (52, 235, 260, 1);
INSERT INTO public.recipe_ingredients VALUES (52, 26, 8, 1);
INSERT INTO public.recipe_ingredients VALUES (224, 5, 1.5000, 1);
INSERT INTO public.recipe_ingredients VALUES (224, 6, 3, 4);
INSERT INTO public.recipe_ingredients VALUES (224, 20, 4, 7);
INSERT INTO public.recipe_ingredients VALUES (224, 26, 3, 1);
INSERT INTO public.recipe_ingredients VALUES (224, 53, 1, 3);
INSERT INTO public.recipe_ingredients VALUES (224, 73, 200, 1);
INSERT INTO public.recipe_ingredients VALUES (224, 130, 1, 3);
INSERT INTO public.recipe_ingredients VALUES (224, 131, 1, 3);
INSERT INTO public.recipe_ingredients VALUES (224, 137, 200, 8);
INSERT INTO public.recipe_ingredients VALUES (224, 150, 25, 1);
INSERT INTO public.recipe_ingredients VALUES (224, 194, 2, 5);
INSERT INTO public.recipe_ingredients VALUES (224, 230, 250, 1);
INSERT INTO public.recipe_ingredients VALUES (224, 231, 1, 3);
INSERT INTO public.recipe_ingredients VALUES (224, 232, 1, 9);
INSERT INTO public.recipe_ingredients VALUES (224, 233, 1, 3);
INSERT INTO public.recipe_ingredients VALUES (224, 234, 1, 3);
INSERT INTO public.recipe_ingredients VALUES (53, 13, 2, 2);
INSERT INTO public.recipe_ingredients VALUES (53, 20, 100, 1);
INSERT INTO public.recipe_ingredients VALUES (78, 204, 140, 1);
INSERT INTO public.recipe_ingredients VALUES (78, 218, 80, 1);
INSERT INTO public.recipe_ingredients VALUES (53, 26, 10, 1);
INSERT INTO public.recipe_ingredients VALUES (53, 28, 8, 0);
INSERT INTO public.recipe_ingredients VALUES (53, 34, 7.5000, 0);
INSERT INTO public.recipe_ingredients VALUES (53, 36, 50, 1);
INSERT INTO public.recipe_ingredients VALUES (53, 37, 17, 1);
INSERT INTO public.recipe_ingredients VALUES (53, 53, 32, 1);
INSERT INTO public.recipe_ingredients VALUES (53, 73, 8, 2);
INSERT INTO public.recipe_ingredients VALUES (53, 145, 50, 1);
INSERT INTO public.recipe_ingredients VALUES (53, 194, 2.5000, 0);
INSERT INTO public.recipe_ingredients VALUES (53, 210, 2.5000, 0);
INSERT INTO public.recipe_ingredients VALUES (53, 214, 10, 1);
INSERT INTO public.recipe_ingredients VALUES (53, 5, 22, 1);
INSERT INTO public.recipe_ingredients VALUES (53, 82, 4, 1);
INSERT INTO public.recipe_ingredients VALUES (53, 38, 2, 1);
INSERT INTO public.recipe_ingredients VALUES (91, 16, 240, 1);
INSERT INTO public.recipe_ingredients VALUES (34, 49, 9, 1);
INSERT INTO public.recipe_ingredients VALUES (34, 178, 50, 1);
INSERT INTO public.recipe_ingredients VALUES (36, 16, 70, 1);
INSERT INTO public.recipe_ingredients VALUES (36, 21, 1, 5);
INSERT INTO public.recipe_ingredients VALUES (35, 9, 1, 12);
INSERT INTO public.recipe_ingredients VALUES (136, 131, 5, 1);
INSERT INTO public.recipe_ingredients VALUES (136, 103, 3.65, 0);
INSERT INTO public.recipe_ingredients VALUES (136, 8, 0.6, 0);
INSERT INTO public.recipe_ingredients VALUES (136, 137, 760, 1);
INSERT INTO public.recipe_ingredients VALUES (35, 70, 1, 12);
INSERT INTO public.recipe_ingredients VALUES (37, 21, 0.1000, 5);
INSERT INTO public.recipe_ingredients VALUES (37, 162, 20, 1);
INSERT INTO public.recipe_ingredients VALUES (231, 18, 75, 1);
INSERT INTO public.recipe_ingredients VALUES (231, 15, 250, 1);
INSERT INTO public.recipe_ingredients VALUES (231, 64, 25, 1);
INSERT INTO public.recipe_ingredients VALUES (231, 55, 75, 1);
INSERT INTO public.recipe_ingredients VALUES (231, 5, 0.5, 3);
INSERT INTO public.recipe_ingredients VALUES (231, 156, 0.5, 3);
INSERT INTO public.recipe_ingredients VALUES (231, 1, 500, 1);
INSERT INTO public.recipe_ingredients VALUES (250, 18, 37.5, 1);
INSERT INTO public.recipe_ingredients VALUES (250, 55, 50, 1);
INSERT INTO public.recipe_ingredients VALUES (250, 114, 0.5, 4);
INSERT INTO public.recipe_ingredients VALUES (251, 2, 0.5000, 5);
INSERT INTO public.recipe_ingredients VALUES (251, 180, 30, 1);
INSERT INTO public.recipe_ingredients VALUES (265, 47, 0.2000, 0);
INSERT INTO public.recipe_ingredients VALUES (265, 246, 0.3000, 0);
INSERT INTO public.recipe_ingredients VALUES (265, 112, 0.01, 0);
INSERT INTO public.recipe_ingredients VALUES (266, 218, 1, 0);
INSERT INTO public.recipe_ingredients VALUES (39, 5, 10, 1);
INSERT INTO public.recipe_ingredients VALUES (39, 6, 50, 1);
INSERT INTO public.recipe_ingredients VALUES (39, 8, 4, 5);
INSERT INTO public.recipe_ingredients VALUES (39, 20, 3, 7);
INSERT INTO public.recipe_ingredients VALUES (39, 26, 10, 1);
INSERT INTO public.recipe_ingredients VALUES (39, 28, 0.8000, 0);
INSERT INTO public.recipe_ingredients VALUES (39, 34, 700, 1);
INSERT INTO public.recipe_ingredients VALUES (39, 53, 2, 3);
INSERT INTO public.recipe_ingredients VALUES (39, 78, 200, 1);
INSERT INTO public.recipe_ingredients VALUES (39, 132, 1.5000, 3);
INSERT INTO public.recipe_ingredients VALUES (39, 161, 0.5000, 3);
INSERT INTO public.recipe_ingredients VALUES (267, 1, 10, 1);
INSERT INTO public.recipe_ingredients VALUES (267, 5, 2, 1);
INSERT INTO public.recipe_ingredients VALUES (267, 194, 0.4, 0);
INSERT INTO public.recipe_ingredients VALUES (267, 170, 1, 0);
INSERT INTO public.recipe_ingredients VALUES (267, 26, 2, 1);
INSERT INTO public.recipe_ingredients VALUES (267, 227, 0.5, 0);
INSERT INTO public.recipe_ingredients VALUES (267, 33, 1.2, 0);
INSERT INTO public.recipe_ingredients VALUES (270, 7, 0.1000, 2);
INSERT INTO public.recipe_ingredients VALUES (270, 19, 0.3000, 0);
INSERT INTO public.recipe_ingredients VALUES (270, 183, 0.5, 0);
INSERT INTO public.recipe_ingredients VALUES (270, 247, 0.5000, 0);
INSERT INTO public.recipe_ingredients VALUES (2, 15, 1, 0);
INSERT INTO public.recipe_ingredients VALUES (271, 4, 7, 2);
INSERT INTO public.recipe_ingredients VALUES (271, 5, 145, 1);
INSERT INTO public.recipe_ingredients VALUES (271, 26, 15, 1);
INSERT INTO public.recipe_ingredients VALUES (271, 49, 510, 1);
INSERT INTO public.recipe_ingredients VALUES (271, 67, 230, 1);
INSERT INTO public.recipe_ingredients VALUES (271, 90, 27, 1);
INSERT INTO public.recipe_ingredients VALUES (271, 165, 1, 0);
INSERT INTO public.recipe_ingredients VALUES (271, 210, 1.5000, 0);
INSERT INTO public.recipe_ingredients VALUES (271, 249, 775, 1);
INSERT INTO public.recipe_ingredients VALUES (273, 28, 7.50, 0);
INSERT INTO public.recipe_ingredients VALUES (273, 250, 5, 0);
INSERT INTO public.recipe_ingredients VALUES (280, 9, 0.1500, 0);
INSERT INTO public.recipe_ingredients VALUES (280, 91, 0.1000, 0);
INSERT INTO public.recipe_ingredients VALUES (280, 207, 0.1500, 0);
INSERT INTO public.recipe_ingredients VALUES (280, 253, 0.1, 0);
INSERT INTO public.recipe_ingredients VALUES (80, 18, 25, 1);
INSERT INTO public.recipe_ingredients VALUES (80, 49, 5, 1);
INSERT INTO public.recipe_ingredients VALUES (80, 108, 10, 1);
INSERT INTO public.recipe_ingredients VALUES (80, 151, 30, 1);
INSERT INTO public.recipe_ingredients VALUES (80, 163, 8, 1);
INSERT INTO public.recipe_ingredients VALUES (80, 171, 80, 1);
INSERT INTO public.recipe_ingredients VALUES (80, 172, 30, 1);
INSERT INTO public.recipe_ingredients VALUES (80, 254, 30, 1);
INSERT INTO public.recipe_ingredients VALUES (282, 4, 1, 2);
INSERT INTO public.recipe_ingredients VALUES (274, 5, 71, 1);
INSERT INTO public.recipe_ingredients VALUES (274, 6, 150, 8);
INSERT INTO public.recipe_ingredients VALUES (274, 67, 430, 1);
INSERT INTO public.recipe_ingredients VALUES (274, 90, 0.3600, 1);
INSERT INTO public.recipe_ingredients VALUES (274, 194, 750, 1);
INSERT INTO public.recipe_ingredients VALUES (274, 209, 6, 0);
INSERT INTO public.recipe_ingredients VALUES (274, 251, 7, 2);
INSERT INTO public.recipe_ingredients VALUES (274, 256, 0.5, 0);
INSERT INTO public.recipe_ingredients VALUES (41, 5, 19, 1);
INSERT INTO public.recipe_ingredients VALUES (41, 15, 300, 8);
INSERT INTO public.recipe_ingredients VALUES (41, 26, 9.8, 1);
INSERT INTO public.recipe_ingredients VALUES (41, 90, 10.2, 1);
INSERT INTO public.recipe_ingredients VALUES (41, 194, 1, 0);
INSERT INTO public.recipe_ingredients VALUES (58, 94, 500, 1);
INSERT INTO public.recipe_ingredients VALUES (278, 4, 2, 2);
INSERT INTO public.recipe_ingredients VALUES (278, 5, 46, 1);
INSERT INTO public.recipe_ingredients VALUES (278, 55, 154, 1);
INSERT INTO public.recipe_ingredients VALUES (278, 96, 500, 1);
INSERT INTO public.recipe_ingredients VALUES (278, 252, 3, 0);
INSERT INTO public.recipe_ingredients VALUES (278, 255, 6, 1);
INSERT INTO public.recipe_ingredients VALUES (278, 257, 150, 1);
INSERT INTO public.recipe_ingredients VALUES (278, 259, 121, 1);
INSERT INTO public.recipe_ingredients VALUES (290, 5, 2.3000, 1);
INSERT INTO public.recipe_ingredients VALUES (290, 14, 10.5000, 1);
INSERT INTO public.recipe_ingredients VALUES (290, 26, 0.4000, 1);
INSERT INTO public.recipe_ingredients VALUES (290, 90, 1, 1);
INSERT INTO public.recipe_ingredients VALUES (290, 159, 149, 1);
INSERT INTO public.recipe_ingredients VALUES (290, 258, 3, 1);
INSERT INTO public.recipe_ingredients VALUES (7, 17, 350, 1);
INSERT INTO public.recipe_ingredients VALUES (11, 7, 50, 1);
INSERT INTO public.recipe_ingredients VALUES (59, 4, 1000, 1);
INSERT INTO public.recipe_ingredients VALUES (11, 73, 1, 0);
INSERT INTO public.recipe_ingredients VALUES (11, 36, 1.2, 1);
INSERT INTO public.recipe_ingredients VALUES (11, 194, 141, 1);
INSERT INTO public.recipe_ingredients VALUES (11, 23, 5, 1);
INSERT INTO public.recipe_ingredients VALUES (59, 5, 8, 1);
INSERT INTO public.recipe_ingredients VALUES (3, 18, 58, 1);
INSERT INTO public.recipe_ingredients VALUES (3, 5, 6, 1);
INSERT INTO public.recipe_ingredients VALUES (7, 16, 355, 1);
INSERT INTO public.recipe_ingredients VALUES (3, 15, 550, 1);
INSERT INTO public.recipe_ingredients VALUES (3, 26, 0.2, 1);
INSERT INTO public.recipe_ingredients VALUES (3, 90, 0.4, 1);
INSERT INTO public.recipe_ingredients VALUES (3, 1, 55.3, 1);
INSERT INTO public.recipe_ingredients VALUES (11, 20, 14, 1);
INSERT INTO public.recipe_ingredients VALUES (14, 6, 5, 1);
INSERT INTO public.recipe_ingredients VALUES (14, 138, 7, 8);
INSERT INTO public.recipe_ingredients VALUES (14, 159, 100, 1);
INSERT INTO public.recipe_ingredients VALUES (295, 261, 830, 1);
INSERT INTO public.recipe_ingredients VALUES (295, 194, 190, 1);
INSERT INTO public.recipe_ingredients VALUES (295, 149, 400, 1);
INSERT INTO public.recipe_ingredients VALUES (295, 40, 250, 1);
INSERT INTO public.recipe_ingredients VALUES (295, 150, 5, 1);
INSERT INTO public.recipe_ingredients VALUES (295, 131, 5, 1);
INSERT INTO public.recipe_ingredients VALUES (295, 114, 1, 1);
INSERT INTO public.recipe_ingredients VALUES (295, 37, 1, 1);
INSERT INTO public.recipe_ingredients VALUES (295, 130, 1, 1);
INSERT INTO public.recipe_ingredients VALUES (295, 5, 2, 1);
INSERT INTO public.recipe_ingredients VALUES (295, 199, 6, 1);
INSERT INTO public.recipe_ingredients VALUES (295, 90, 1, 1);
INSERT INTO public.recipe_ingredients VALUES (2, 1, 34, 1);
INSERT INTO public.recipe_ingredients VALUES (2, 27, 302, 1);
INSERT INTO public.recipe_ingredients VALUES (2, 5, 4, 1);
INSERT INTO public.recipe_ingredients VALUES (2, 26, 3, 1);
INSERT INTO public.recipe_ingredients VALUES (2, 33, 570, 1);
INSERT INTO public.recipe_ingredients VALUES (295, 138, 8, 1);
INSERT INTO public.recipe_ingredients VALUES (295, 259, 12, 1);
INSERT INTO public.recipe_ingredients VALUES (295, 235, 4, 1);
INSERT INTO public.recipe_ingredients VALUES (296, 262, 531, 1);
INSERT INTO public.recipe_ingredients VALUES (296, 263, 500, 1);
INSERT INTO public.recipe_ingredients VALUES (296, 55, 21, 1);
INSERT INTO public.recipe_ingredients VALUES (296, 114, 5, 1);
INSERT INTO public.recipe_ingredients VALUES (296, 68, 27, 1);
INSERT INTO public.recipe_ingredients VALUES (296, 118, 15, 1);
INSERT INTO public.recipe_ingredients VALUES (300, 222, 1100, 1);
INSERT INTO public.recipe_ingredients VALUES (300, 6, 30, 1);
INSERT INTO public.recipe_ingredients VALUES (300, 5, 12, 1);
INSERT INTO public.recipe_ingredients VALUES (136, 199, 30, 1);
INSERT INTO public.recipe_ingredients VALUES (136, 6, 188, 1);
INSERT INTO public.recipe_ingredients VALUES (136, 90, 1.6, 1);
INSERT INTO public.recipe_ingredients VALUES (136, 26, 2, 1);
INSERT INTO public.recipe_ingredients VALUES (136, 51, 26.01, 1);
INSERT INTO public.recipe_ingredients VALUES (136, 231, 15.2, 1);
INSERT INTO public.recipe_ingredients VALUES (136, 130, 1.7, 1);
INSERT INTO public.recipe_ingredients VALUES (136, 25, 0.5, 1);
INSERT INTO public.recipe_ingredients VALUES (136, 233, 1.2, 1);
INSERT INTO public.recipe_ingredients VALUES (136, 138, 47, 1);
INSERT INTO public.recipe_ingredients VALUES (136, 68, 14, 1);
INSERT INTO public.recipe_ingredients VALUES (136, 213, 1, 1);
INSERT INTO public.recipe_ingredients VALUES (106, 177, 0.3000, 0);
INSERT INTO public.recipe_ingredients VALUES (106, 183, 0.3000, 0);
INSERT INTO public.recipe_ingredients VALUES (106, 194, 0.0500, 0);
INSERT INTO public.recipe_ingredients VALUES (106, 174, 0.1, 0);
INSERT INTO public.recipe_ingredients VALUES (308, 266, 1, 0);
INSERT INTO public.recipe_ingredients VALUES (308, 265, 1, 0);
INSERT INTO public.recipe_ingredients VALUES (308, 15, 1, 0);
INSERT INTO public.recipe_ingredients VALUES (309, 18, 100, 1);
INSERT INTO public.recipe_ingredients VALUES (309, 108, 100, 1);
INSERT INTO public.recipe_ingredients VALUES (309, 151, 200, 1);
INSERT INTO public.recipe_ingredients VALUES (309, 254, 200, 1);
INSERT INTO public.recipe_ingredients VALUES (292, 30, 0.030, 0);
INSERT INTO public.recipe_ingredients VALUES (292, 225, 0.0620, 0);
INSERT INTO public.recipe_ingredients VALUES (293, 5, 2.6000, 1);
INSERT INTO public.recipe_ingredients VALUES (293, 20, 13.9000, 1);
INSERT INTO public.recipe_ingredients VALUES (293, 73, 100, 1);
INSERT INTO public.recipe_ingredients VALUES (293, 114, 0.2000, 1);
INSERT INTO public.recipe_ingredients VALUES (293, 130, 1.2000, 1);
INSERT INTO public.recipe_ingredients VALUES (293, 131, 5.5000, 1);
INSERT INTO public.recipe_ingredients VALUES (293, 137, 800, 1);
INSERT INTO public.recipe_ingredients VALUES (293, 138, 5, 1);
INSERT INTO public.recipe_ingredients VALUES (293, 149, 520, 1);
INSERT INTO public.recipe_ingredients VALUES (293, 150, 18.8000, 1);
INSERT INTO public.recipe_ingredients VALUES (293, 161, 0.2000, 1);
INSERT INTO public.recipe_ingredients VALUES (293, 194, 210, 1);
INSERT INTO public.recipe_ingredients VALUES (293, 199, 20, 1);
INSERT INTO public.recipe_ingredients VALUES (293, 231, 1.6000, 1);
INSERT INTO public.recipe_ingredients VALUES (293, 233, 1.7000, 1);
INSERT INTO public.recipe_ingredients VALUES (293, 235, 3, 1);
INSERT INTO public.recipe_ingredients VALUES (293, 6, 14, 1);
INSERT INTO public.recipe_ingredients VALUES (293, 260, 20, 1);
INSERT INTO public.recipe_ingredients VALUES (225, 228, 200, 1);
INSERT INTO public.recipe_ingredients VALUES (2, 12, 28, 1);
INSERT INTO public.recipe_ingredients VALUES (2, 6, 25, 1);
INSERT INTO public.recipe_ingredients VALUES (2, 4, 376, 1);
INSERT INTO public.recipe_ingredients VALUES (2, 214, 1.4, 1);
INSERT INTO public.recipe_ingredients VALUES (2, 14, 3, 1);
INSERT INTO public.recipe_ingredients VALUES (2, 213, 1, 1);
INSERT INTO public.recipe_ingredients VALUES (2, 258, 0.1, 1);
INSERT INTO public.recipe_ingredients VALUES (2, 37, 0.7, 1);
INSERT INTO public.recipe_ingredients VALUES (2, 138, 3, 1);
INSERT INTO public.recipe_ingredients VALUES (2, 259, 6, 1);
INSERT INTO public.recipe_ingredients VALUES (2, 194, 280, 1);
INSERT INTO public.recipe_ingredients VALUES (2, 18, 30, 1);
INSERT INTO public.recipe_ingredients VALUES (309, 49, 25, 1);
INSERT INTO public.recipe_ingredients VALUES (309, 96, 0.7500, 1);
INSERT INTO public.recipe_ingredients VALUES (309, 103, 750, 1);
INSERT INTO public.recipe_ingredients VALUES (309, 264, 750, 1);
INSERT INTO public.recipe_ingredients VALUES (31, 268, 180, 1);
INSERT INTO public.recipe_ingredients VALUES (31, 16, 60, 1);
INSERT INTO public.recipe_ingredients VALUES (31, 5, 5, 1);
INSERT INTO public.recipe_ingredients VALUES (94, 268, 120, 1);
INSERT INTO public.recipe_ingredients VALUES (41, 268, 2, 0);
INSERT INTO public.recipe_ingredients VALUES (310, 15, 50, 8);
INSERT INTO public.recipe_ingredients VALUES (310, 268, 13, 1);
INSERT INTO public.recipe_ingredients VALUES (310, 1, 25, 1);
INSERT INTO public.recipe_ingredients VALUES (310, 55, 3, 1);
INSERT INTO public.recipe_ingredients VALUES (311, 7, 50, 8);
INSERT INTO public.recipe_ingredients VALUES (311, 68, 55, 8);
INSERT INTO public.recipe_ingredients VALUES (311, 36, 1, 1);
INSERT INTO public.recipe_ingredients VALUES (311, 5, 1, 1);
INSERT INTO public.recipe_ingredients VALUES (311, 26, 1, 1);
INSERT INTO public.recipe_ingredients VALUES (311, 269, 1, 0);
INSERT INTO public.recipe_ingredients VALUES (103, 149, 1600, 1);
INSERT INTO public.recipe_ingredients VALUES (103, 256, 190, 1);
INSERT INTO public.recipe_ingredients VALUES (103, 56, 1, 1);
INSERT INTO public.recipe_ingredients VALUES (313, 47, 1, 0);
INSERT INTO public.recipe_ingredients VALUES (316, 137, 400, 8);
INSERT INTO public.recipe_ingredients VALUES (316, 272, 80, 1);
INSERT INTO public.recipe_ingredients VALUES (316, 235, 3, 4);
INSERT INTO public.recipe_ingredients VALUES (316, 138, 2, 4);
INSERT INTO public.recipe_ingredients VALUES (316, 273, 2, 4);
INSERT INTO public.recipe_ingredients VALUES (316, 150, 2, 3);
INSERT INTO public.recipe_ingredients VALUES (316, 20, 3, 7);
INSERT INTO public.recipe_ingredients VALUES (316, 55, 2, 3);
INSERT INTO public.recipe_ingredients VALUES (316, 7, 2, 4);
INSERT INTO public.recipe_ingredients VALUES (316, 5, 0.1, 1);
INSERT INTO public.recipe_ingredients VALUES (316, 161, 0.1, 1);
INSERT INTO public.recipe_ingredients VALUES (314, 4, 760, 1);
INSERT INTO public.recipe_ingredients VALUES (314, 14, 4, 1);
INSERT INTO public.recipe_ingredients VALUES (314, 47, 608, 1);
INSERT INTO public.recipe_ingredients VALUES (314, 274, 200, 1);
INSERT INTO public.recipe_ingredients VALUES (314, 5, 8, 1);
INSERT INTO public.recipe_ingredients VALUES (314, 68, 33, 1);
INSERT INTO public.recipe_ingredients VALUES (314, 26, 1.5, 1);
INSERT INTO public.recipe_ingredients VALUES (314, 161, 1.7, 1);
INSERT INTO public.recipe_ingredients VALUES (314, 235, 14, 1);
INSERT INTO public.recipe_ingredients VALUES (314, 94, 1.2, 0);
INSERT INTO public.recipe_ingredients VALUES (314, 183, 0.5, 0);
INSERT INTO public.recipe_ingredients VALUES (315, 271, 0.8, 0);
INSERT INTO public.recipe_ingredients VALUES (315, 40, 0.1, 0);
INSERT INTO public.recipe_ingredients VALUES (315, 76, 0.15, 0);
INSERT INTO public.recipe_ingredients VALUES (315, 274, 0.05, 0);
INSERT INTO public.recipe_ingredients VALUES (315, 137, 400, 8);
INSERT INTO public.recipe_ingredients VALUES (315, 272, 0.08, 0);
INSERT INTO public.recipe_ingredients VALUES (315, 235, 3, 4);
INSERT INTO public.recipe_ingredients VALUES (315, 138, 2, 4);
INSERT INTO public.recipe_ingredients VALUES (315, 273, 2, 3);
INSERT INTO public.recipe_ingredients VALUES (315, 150, 2, 3);
INSERT INTO public.recipe_ingredients VALUES (315, 20, 3, 7);
INSERT INTO public.recipe_ingredients VALUES (315, 55, 2, 3);
INSERT INTO public.recipe_ingredients VALUES (315, 7, 2, 4);
INSERT INTO public.recipe_ingredients VALUES (315, 5, 0.002, 0);
INSERT INTO public.recipe_ingredients VALUES (315, 161, 0.5, 1);
INSERT INTO public.recipe_ingredients VALUES (318, 15, 0.25, 0);
INSERT INTO public.recipe_ingredients VALUES (318, 18, 0.1, 0);
INSERT INTO public.recipe_ingredients VALUES (318, 1, 0.5, 0);
INSERT INTO public.recipe_ingredients VALUES (318, 55, 0.06, 0);
INSERT INTO public.recipe_ingredients VALUES (318, 5, 1, 11);
INSERT INTO public.recipe_ingredients VALUES (318, 64, 21, 1);
INSERT INTO public.recipe_ingredients VALUES (318, 2, 2, 5);
INSERT INTO public.recipe_ingredients VALUES (317, 15, 300, 1);
INSERT INTO public.recipe_ingredients VALUES (317, 18, 0.05, 0);
INSERT INTO public.recipe_ingredients VALUES (317, 5, 0.5, 3);
INSERT INTO public.recipe_ingredients VALUES (317, 55, 1, 4);
INSERT INTO public.recipe_ingredients VALUES (317, 275, 720, 1);
INSERT INTO public.recipe_ingredients VALUES (319, 225, 0.432, 0);
INSERT INTO public.recipe_ingredients VALUES (319, 204, 0.1, 0);
INSERT INTO public.recipe_ingredients VALUES (319, 21, 0.1500, 0);
INSERT INTO public.recipe_ingredients VALUES (319, 34, 0.1500, 0);
INSERT INTO public.recipe_ingredients VALUES (319, 177, 0.1000, 0);
INSERT INTO public.recipe_ingredients VALUES (315, 94, 0.5000, 0);
INSERT INTO public.recipe_ingredients VALUES (314, 201, 250, 1);
INSERT INTO public.recipe_ingredients VALUES (320, 261, 1.2, 0);
INSERT INTO public.recipe_ingredients VALUES (320, 176, 150, 1);
INSERT INTO public.recipe_ingredients VALUES (320, 40, 150, 1);
INSERT INTO public.recipe_ingredients VALUES (320, 225, 800, 1);
INSERT INTO public.recipe_ingredients VALUES (320, 15, 180, 8);
INSERT INTO public.recipe_ingredients VALUES (320, 42, 400, 1);
INSERT INTO public.recipe_ingredients VALUES (320, 49, 65, 1);
INSERT INTO public.recipe_ingredients VALUES (320, 90, 0.4, 1);
INSERT INTO public.recipe_ingredients VALUES (320, 130, 0.4, 1);
INSERT INTO public.recipe_ingredients VALUES (320, 25, 0.2, 1);
INSERT INTO public.recipe_ingredients VALUES (320, 131, 2, 1);
INSERT INTO public.recipe_ingredients VALUES (320, 231, 1, 1);
INSERT INTO public.recipe_ingredients VALUES (320, 114, 0.2, 1);
INSERT INTO public.recipe_ingredients VALUES (320, 68, 10, 8);
INSERT INTO public.recipe_ingredients VALUES (320, 5, 1, 1);
INSERT INTO public.recipe_ingredients VALUES (323, 280, 1, 0);
INSERT INTO public.recipe_ingredients VALUES (321, 280, 1, 0);
INSERT INTO public.recipe_ingredients VALUES (326, 24, 50, 1);
INSERT INTO public.recipe_ingredients VALUES (326, 281, 100, 1);
INSERT INTO public.recipe_ingredients VALUES (326, 19, 25, 1);
INSERT INTO public.recipe_ingredients VALUES (326, 67, 20, 1);
INSERT INTO public.recipe_ingredients VALUES (326, 68, 5, 8);
INSERT INTO public.recipe_ingredients VALUES (327, 282, 150, 1);
INSERT INTO public.recipe_ingredients VALUES (327, 283, 150, 1);
INSERT INTO public.recipe_ingredients VALUES (327, 176, 1, 5);
INSERT INTO public.recipe_ingredients VALUES (327, 6, 2, 4);
INSERT INTO public.recipe_ingredients VALUES (324, 249, 550, 1);
INSERT INTO public.recipe_ingredients VALUES (324, 18, 30, 1);
INSERT INTO public.recipe_ingredients VALUES (324, 81, 200, 1);
INSERT INTO public.recipe_ingredients VALUES (324, 27, 314, 1);
INSERT INTO public.recipe_ingredients VALUES (324, 15, 274, 1);
INSERT INTO public.recipe_ingredients VALUES (324, 5, 7.61, 1);
INSERT INTO public.recipe_ingredients VALUES (324, 4, 781, 1);
INSERT INTO public.recipe_ingredients VALUES (324, 130, 0.1, 1);
INSERT INTO public.recipe_ingredients VALUES (324, 258, 0.4, 1);
INSERT INTO public.recipe_ingredients VALUES (324, 68, 13, 1);
INSERT INTO public.recipe_ingredients VALUES (324, 90, 0.2, 1);
INSERT INTO public.recipe_ingredients VALUES (324, 26, 0.94, 1);
INSERT INTO public.recipe_ingredients VALUES (324, 246, 200, 1);


--
-- Data for Name: shopping_tours; Type: TABLE DATA; Schema: public; Owner: dennis
--

INSERT INTO public.shopping_tours VALUES (1, 38, 0, '2024-06-07 08:00:00+00');
INSERT INTO public.shopping_tours VALUES (2, 38, 0, '2024-06-11 08:00:00+00');
INSERT INTO public.shopping_tours VALUES (3, 38, 3, '2024-06-11 08:00:00+00');
INSERT INTO public.shopping_tours VALUES (4, 38, 3, '2024-06-12 08:00:00+00');
INSERT INTO public.shopping_tours VALUES (5, 38, 3, '2024-06-13 08:00:00+00');
INSERT INTO public.shopping_tours VALUES (6, 38, 3, '2024-06-14 08:00:00+00');
INSERT INTO public.shopping_tours VALUES (7, 38, 3, '2024-06-15 08:00:00+00');
INSERT INTO public.shopping_tours VALUES (9, 38, 7, '2024-06-07 08:00:00+00');
INSERT INTO public.shopping_tours VALUES (8, 38, 5, '2024-06-14 07:00:00+00');
INSERT INTO public.shopping_tours VALUES (10, 38, 4, '2024-06-14 08:00:00+00');
INSERT INTO public.shopping_tours VALUES (12, 51, 0, '2024-07-12 08:00:00+00');
INSERT INTO public.shopping_tours VALUES (13, 52, 0, '2024-07-12 10:00:00+00');
INSERT INTO public.shopping_tours VALUES (14, 53, 0, '2024-07-05 10:00:00+00');
INSERT INTO public.shopping_tours VALUES (15, 54, 0, '2024-10-14 00:00:00+00');
INSERT INTO public.shopping_tours VALUES (16, 54, 3, '2024-10-14 00:00:00+00');


--
-- Data for Name: steps; Type: TABLE DATA; Schema: public; Owner: dennis
--

INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (237, 1, 'Schnippeln und Anbraten', 'Frühlingszwiebeln in  Ringe schneiden, weiße Teile mit Karotten in Margarine anbraten. Mit Currypulver bestäuben.', '00:01:00', '00:03:00', 200);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (238, 2, 'Ablöschen', 'Mit Limettensaft ablöschen, dann Brühe dazu', '00:00:00', '00:03:00', 200);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (239, 3, 'Abschmecken', 'Sahneersatz, Orangensaft, Salz zugeben, abschmecken', '00:00:00', '00:05:00', 200);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (92, 1, 'Schritt 1', 'Linsen einweichen (viele Linsen brauchen sonst sehr lange zum Durchwerden)', '00:00:00', '00:00:00', 38);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (93, 2, 'Schritt 2', 'Zwiebeln anbraten', '00:00:00', '00:00:00', 38);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (94, 3, 'Schritt 3', 'Karotten, Lauch und Sellerie dazu, auch anbraten', '00:00:00', '00:00:00', 38);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (95, 4, 'Schritt 4', 'Tomatenmark kurz rösten und mit Wasser ablöschen', '00:00:00', '00:00:00', 38);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (96, 5, 'Schritt 5', 'Linsen und Restwasser dazugeben und kochen. DIESER SCHRITT BRAUCHT ZEIT', '00:00:00', '00:00:00', 38);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (97, 6, 'Schritt 6', 'Würzen und abschmecken', '00:00:00', '00:00:00', 38);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (127, 1, 'Gemüse schneiden', 'Paprika in ca. 1 cm² große Stücke schneiden. Karotten waschen und würfeln, Knoblauch hacken', '00:02:00', '00:02:00', 30);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (128, 2, 'Zwiebeln anbraten', 'Sonnenblumenöl in den Topf geben und zwiebeln nach und nach anbraten', '00:02:00', '00:01:00', 30);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (129, 3, 'Gemüse anbraten', 'Paprika zu den Zwiebeln geben und anbraten. Dann die restlichen Zutaten hinzufügen', '00:02:00', '00:01:00', 30);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (130, 4, 'Würzen', 'Nun noch das Chili würzen. Die Gewürzmengen sind tatsächlich kalibriert und sollten ungefähr passen.', '00:01:00', '00:00:30', 30);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (240, 4, 'Pürieren', 'Suppe durchpürieren, bis sie sämig ist. Mit Frühlingszwiebelgrün bestreut servieren.', '00:00:00', '00:05:00', 200);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (75, 1, 'Schritt 1', 'Wasser kochen', '00:00:00', '00:00:00', 4);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (76, 2, 'Schritt 2', 'Brühepulver zugeben', '00:00:00', '00:00:00', 4);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (249, 1, 'Vorheizen', 'Den Ofen auf 250 - 275 °C Ober-/Unterhitze aufheizen.', '00:20:00', '00:00:00', 251);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (250, 2, 'Glasieren', 'Die Kanelbullar (Zimtschnecken) mit verschlagenem Ei bepinseln. Anschließend mit Zucker bestreuen.', '00:02:00', '00:02:00', 251);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (85, 1, 'Schritt 1', 'Kocht Reis. Ihr wisst, wie man Reis kocht.', '00:00:00', '00:00:00', 59);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (86, 1, 'Schritt 1', 'Soßen und Nudeln und Reis kochen', '00:00:00', '00:00:00', 64);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (251, 3, 'Backen', 'Auf der mittleren Stufe im vorgeheizten Backofen etwa 5 - 8 Minuten backen. Unter einem Handtuch abkühlen lassen.', '00:02:00', '00:06:00', 251);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (101, 1, 'Schritt 1', 'Zutaten in einem passenden Topf vermischen', '00:00:00', '00:00:00', 42);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (102, 2, 'Schritt 2', 'Unter ständigem Rühren auf niedriger Stufe kochen, bis der Brei breiig ist', '00:00:00', '00:00:00', 42);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (103, 1, 'Schritt 1', 'Hefezopf bei 180°C backen', '00:00:00', '00:00:00', 43);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (104, 2, 'Schritt 2', 'Porridge und Rührei ansetzen', '00:00:00', '00:00:00', 43);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (105, 3, 'Schritt 3', 'Speck und Pilze in Butter anbraten (getrennt), Baked Beans aufwärmen', '00:00:00', '00:00:00', 43);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (106, 4, 'Schritt 4', 'Brote und Aufschnitt auslegen', '00:00:00', '00:00:00', 43);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (107, 1, 'Schritt 1', 'Zwiebeln rösten', '00:00:00', '00:00:00', 8);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (108, 2, 'Schritt 2', 'Spätzle, Sahne und Käae in Gasbräter geben', '00:00:00', '00:00:00', 8);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (109, 3, 'Schritt 3', 'Unter ständigem Rühren erwärmen. Kurz vor Ende Zwiebeln hinzufügen', '00:00:00', '00:00:00', 8);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (110, 4, 'Schritt 4', 'Würzen', '00:00:00', '00:00:00', 8);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (111, 1, 'Schritt 1', 'Kartoffeln waschen, schälen und schneiden (1-2cm Würfel oder Scheiben)', '00:00:00', '00:00:00', 10);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (112, 3, 'Schritt 3', 'Kartoffeln in Öl mit geschlossenem Deckel anbraten', '00:00:00', '00:00:00', 10);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (113, 2, 'Schritt 2', 'Kartoffeln kurz in kaltes Wasser lesen, damit Stärke austreten kann', '00:00:00', '00:00:00', 10);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (114, 4, 'Schritt 4', '10 Minuten vor Ende gewürfelte Zwiebeln hinzugeben', '00:00:00', '00:00:00', 10);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (115, 5, 'Schritt 5', '5 Minuten vor Ende offen braten und würzen', '00:00:00', '00:00:00', 10);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (120, 1, 'Chilli Kochen', 'Chilli halb zubereiten, aber noch nicht würzen ', '00:00:00', '00:00:00', 29);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (121, 2, 'Sojahack hinzugben', 'Sojahack in den Topf geben', '00:03:00', '00:00:12', 29);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (122, 3, 'Abschmecken', 'Die restlichen Gewürze aus dem chilli base rezept hinzugen', '00:05:00', '00:00:06', 29);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (141, 1, 'Kichererbsen kochen', 'Kichererbsen mit etwas Backpulver kochen, bis sie weich sind', '00:00:00', '00:30:00', 103);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (142, 2, 'Hummus machen', 'Tahini mit Zitronensaft und ein bisschen Wasser vom Kichererbsenkochen im Mixer schaumig pürieren. Die restlichen Zutaten dazugeben und pürieren, bis eine cremige Masse entsteht. Der Knoblauch sollte vorher etwas kleingeschnitten werden.', '00:10:00', '00:10:00', 103);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (241, 1, 'Schritt 1', 'Sojabolognese kochen', '00:00:00', '00:00:00', 7);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (242, 2, 'Schritt 2', 'Abwechselnd Bolognese, Lasagneplatten und Gouda schichten. Währenddessen Bechamelsoße ansetzen.', '00:00:00', '00:00:00', 7);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (243, 3, 'Schritt 3', 'Bechamelsoße als oberste Schicht ausgießen, bei 180°C Umluft backen', '00:00:00', '00:00:00', 7);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (247, 1, 'Schritt 1', 'Butter schmelzen, Mehl darin anbräunen, Gewürze hinzufügen', '00:00:00', '00:00:00', 3);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (248, 2, 'Schritt 2', 'Milch unter Rühren zugeben. Dabei immer warten, bis die Milch völlig eingezogen ist, bevor mehr dazugegeben wird', '00:00:00', '00:00:00', 3);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (155, 5, 'Pizza Boden vorbacken', 'E Grill auf 230 Grad stellen und den Boden für ca. 30s - 1m braten', '00:00:00', '00:05:00', 15);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (261, 1, 'Gemüse schneiden', 'Zwiebeln in dünne Scheiben schneiden, Paprika in 2cm-Quadrate, Kartoffeln in 1.5cm-Würfel, Knoblauch fein würfeln', '00:00:00', '00:10:00', 39);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (158, 1, 'Zwiebeln anschwitzen', 'Zwiebeln in Butter anschwitzen. Dann alles außer Brühe und Sauerkraut dazu, kurz anbraten', '00:05:00', '00:10:00', 108);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (159, 2, 'Fertig kochen', 'Mit Sauerkraut ablöschen, Brühe dazu, köcheln lassen bis durch', '00:05:00', '00:20:00', 108);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (160, 3, 'Abschmecken', 'Wild würzen oder Dennis rufen', '00:10:00', '00:01:00', 108);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (262, 2, 'Gemüse anbraten', 'Erst Zwiebeln, später Knoblauch, Paprika und dann Kartoffeln andünsten. Kümmel mit den Zwiebeln anrösten', '00:00:00', '00:02:00', 39);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (263, 3, 'Ablöschen', 'Mit Brühe ablöschen', '00:03:00', '00:00:00', 39);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (264, 4, 'Abschmecken', 'Beim Würzen nicht fest an die Gewichte halten', '00:00:00', '00:01:00', 39);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (265, 5, 'Bratwurst braten', 'Bratwurst in 1-1.5cm-Scheiben anbraten und zum Gulasch reichen', '00:00:00', '00:03:00', 39);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (193, 1, 'Gemüse vorbereiten', 'Waschen, putzen, mundgerecht kleinschneiden', '00:00:00', '00:00:00', 186);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (194, 2, 'Gemüse scharf anbraten', 'Zwiebeln + Zuccini, dan Paprika, dann Aubergine. Dann mit Tomatenmark und Tomaten ablöschen, würzen.', '00:00:00', '00:00:00', 186);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (195, 3, 'köcheln', 'köcheln', '00:10:00', '00:05:00', 186);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (209, 2, 'Kochen', 'Kokosmilch dazu, köcheln', '00:15:00', '00:05:00', 136);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (211, 4, 'Anrichten', 'Tofu in Zentimeterwürfel zerschneiden und unterrühren', '00:04:00', '00:00:15', 136);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (215, 1, 'Schritt 1', 'Zwiebeln in Topf anbraten, Gemüse dazu, Sahne dazu, Mehl dazu', '00:00:00', '00:00:00', 1);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (216, 2, 'Schritt 2', 'Köcheln lassen', '00:00:00', '00:00:00', 1);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (217, 3, 'Schritt 3', 'Mit Salz, Pfeffer und Muskat abschmecken', '00:00:00', '00:00:00', 1);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (229, 1, 'Hefe aufschwämmen', 'Hefe, Honig mit Schluck Wasser ~30°C und etwas Mehl aufschwämmen, viertel- bis halbe Stunde stehen lassen, bis sich Blasen bilden', '00:25:00', '00:00:30', 102);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (230, 2, 'Autolyseteig ansetzen', 'Mehl, Wasser ~18°C, Öl verkneten, bis ein glatter Teig entsteht. In Teigwanne ruhen lassen, bis der Hefesponge fertig ist', '00:00:00', '00:05:00', 102);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (231, 3, 'Teig fertig machen und portionieren', 'Nacheinander Hefesponge und Salz in den Autolyseteig einkneten. In Portionen von ~120g einteilen, rundschleifen und bedeckt ruhen lassen. Bei größeren Mengen kann auch erst nur ein Teil portioniert werden, um Platz zu sparen', '00:05:00', '00:15:00', 102);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (232, 4, 'Fladen backen', 'Vorportionierten Teig auf 0.5-1cm Dicke auswellen und sofort in Ofen mit 250°C einschießen. Backen, bis sich eine Tasche bildet und beide Seiten gebräunt sind $\rightarrow$ ggf. wenden. Beim Auswellen großzügig mehlen. Fixzeit ist für Ofen vorheizen angedacht.', '00:40:00', '00:20:00', 102);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (278, 1, 'Zusammenrühren', 'Gekochte Nudeln in vorbereitete Form mit Pesto tun, verrühren, Tomaten untermischen', '00:01:00', '00:00:03', 270);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (301, 1, 'Schnippeln', 'Suppengemüse fein würfeln', '00:01:00', '00:05:00', 271);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (302, 2, 'Andünsten', 'Gemüse + Gewürze in Öl (man könnte auch Butter nehmen) anschwitzen', '00:05:00', '00:01:24', 271);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (303, 3, 'Fertig machen', 'Mit Wasser ablöschen und so lang köcheln lassen wie gewünscht (Wird ziemlich lange immer intensiver)', '00:00:00', '00:03:00', 271);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (304, 1, 'Schneiden', 'Kartoffeln in 1-2cm-Würfel schneiden (evtl. vorher schälen)', '00:01:00', '00:02:00', 273);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (305, 2, 'Kochen', 'Kartoffeln in heiße Brühe werfen und kochen, Maultaschen ein paar Minuten vor Servieren dazu (Sollte zu dem Zeitpunkt sprudelnd kochen)', '00:00:30', '00:01:00', 273);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (312, 1, 'Kochen', 'Topf + Wasser + Warm', '00:04:12', '00:01:24', 282);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (314, 1, 'Schritt 1', 'Zwiebeln klein hacken', '00:00:00', '00:00:00', 41);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (315, 2, 'Schritt 2', 'Alles verquirlen', '00:00:00', '00:00:00', 41);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (316, 3, 'Schritt 3', 'Im Gasbräter braten. Ihr habt schonmal Rührei gemacht.', '00:00:00', '00:00:00', 41);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (317, 1, 'Schritt 1', 'Kocht Nudeln. Ihr wisst, wie man Nudeln kocht.', '00:08:00', '00:00:00', 58);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (318, 1, 'Rotkohl schneiden', 'Rotkohl in dünne streifen schneiden', '00:02:00', '00:06:00', 278);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (319, 2, 'Äpfel schneiden', 'Äpfel entkernen (Kerngehäuseausstecher) dann Äpfel achteln und in dünne scheiben schneiden', '01:00:00', '00:10:00', 278);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (320, 3, 'Rotkohl kochen', 'Ein Topf mit Wasser aufsetzen und dann Rotkohl und restliche zutaten hinzufügen', '00:00:00', '00:00:00', 278);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (325, 1, 'Schritt 1', 'Zwiebeln andünsten, später Knoblauch dazugeben, parallel dazu Sojahack ansetzen', '00:00:00', '00:00:00', 11);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (326, 2, 'Schritt 2', 'Sojahack anbraten, Zwiebeln dazugeben, mit Tomaten ablöschen', '00:00:00', '00:00:00', 11);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (327, 3, 'Schritt 3', 'Würzen, köchlen lassen, abschmecken', '00:00:00', '00:00:00', 11);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (328, 1, 'Schritt 1', 'Brühe ansetzen', '00:00:00', '00:00:00', 14);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (329, 2, 'Schritt 2', 'Sojagranulat in heißer Brühe  einweichen', '00:00:00', '00:00:00', 14);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (330, 3, 'Würzen', 'Öl und sojasoße hinzugeben', '00:00:00', '00:00:00', 14);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (331, 4, 'Abgießen', 'Überschüssiges Wasser Abgießen', '00:00:00', '00:00:00', 14);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (332, 1, 'Tofu schneiden', 'Tofu in Würfel mit 1-1.5 cm Kantenlänge schneiden', '00:01:00', '00:01:00', 300);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (334, 1, 'Anbraten', 'Beschichtete Pfanne vorheizen und Öl und Gewürze hinzugeben. Dann die Bananen durch schwenken der Pfanne anbraten bis eine schöne Kruste entsteht. Ggf. in mehreren Durchgängen anbraten damit die Pfanne nicht zu voll wird. 120°C 3.5kw Induplatte', '00:01:00', '00:08:00', 136);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (335, 1.5, 'Zwiebeln anbraten', 'Zwiebeln mit öl anbraten und in Topf geben.', '00:01:00', '00:02:00', 136);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (338, 1, 'Variante 1', 'Pita aufschneiden, Hummus darin verteilen, evtl. Rohkost und andere Zutaten in die Tasche füllen', '00:00:00', '00:03:00', 106);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (339, 2, 'Variante 2', 'Hummus in Schüssel füllen, mit Pita servieren', '00:00:00', '00:01:00', 106);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (340, 3, 'Schritt 3', 'Flüssigkeiten dazu, nicht zu lange köcheln (sonst wird''s arg dick)', '00:00:00', '00:00:00', 2);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (341, 1, 'Blumenkohl in Röschen', '', '00:02:00', '00:03:00', 315);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (342, 0.5, 'Gemüse waschen', '', '00:00:00', '00:02:00', 315);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (343, 2, 'Ingwer + Knoblauch reiben', '', '00:01:00', '00:10:00', 315);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (344, 3, 'Frühlingszwiebeln schneiden', '', '00:00:00', '00:05:00', 315);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (345, 1, 'Frühlingszwiebel, Ingwer, Knoblauch anbraten', '', '00:05:00', '00:05:00', 316);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (346, 2, 'Mit restilchen Zutaten ablöschen', '', '00:03:00', '00:10:00', 316);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (347, 1, 'Teig vorbereiten', '', '00:05:00', '00:05:00', 317);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (348, 2, 'Teig ruhen lassen', '', '00:45:00', '00:00:00', 317);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (349, 3, 'Formen', '', '00:05:00', '00:10:00', 317);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (350, 4, 'Teig ruhen', '', '00:30:00', '00:00:00', 317);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (351, 5, 'Dämpfen', 'Zutaten in Pfanne/breiten Topf zum Kochen bringen. Dann Hefeklöße vorsichtig rein und mit geschlossenem Deckel Dämpfen', '00:30:00', '00:02:00', 317);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (352, 6, 'Abkülen', 'mit geschlossenem deckel', '00:05:00', '00:00:00', 317);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (363, 0, 'Zwiebeln', 'Rote Zwiebeln fein würfeln und in Margarine anschwitzen', '00:00:00', '00:00:00', 320);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (364, 1, 'Süßkartoffeln', 'Süßkartoffeln in ~2cm Würfel schneiden, kurz anrösten und Gewürze dazu', '00:00:00', '00:00:00', 320);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (365, 2, 'Kochen', 'Milch dazugeben und köcheln lassen bis die Süßkartoffeln weich sind', '00:00:00', '00:00:00', 320);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (366, 3, 'Kartoffeln Stampfen', 'Bis Brei enststeht, dann Abschmecken (ist noch nicht ausgewogen)', '00:00:00', '00:00:00', 320);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (367, 4, 'Monad', 'Alles in Wraps füllen und Wickeln', '00:00:00', '00:00:00', 320);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (368, 1, 'Zeug mixen', 'Alle Zutaten außer Ricotta fein mixen', '00:01:00', '00:01:00', 326);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (369, 2, 'Zusammenrühren', 'Paste mit Ricotta vermischen', '00:00:30', '00:03:00', 326);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (370, 1, 'Anrichten', 'Risotto auf Teller schöpfen, Klecks Ricotta daraufgeben. Bei viel Zeit Basilikumblatt auf die Ricottakleckse drapieren', '00:00:00', '00:02:00', 322);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (375, 1, 'Rote Beete würfeln', 'Rote Beeten schälen und in etwa 5-7mm starke Würfel schneiden', '00:01:00', '00:02:00', 327);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (376, 2, 'Rote Beete kochen und Zwiebeln schneiden', 'Rote Beete in Wasser geben und kochen. Sie sollte etwas Biss behalten. Wenn die richtige Garung erreicht ist, den Saft abgiessen (damit die Würfel nicht weitergaren) und aufheben. Währen die Beete kocht die Zwiebeln schneiden.', '00:02:00', '00:01:00', 327);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (377, 3, 'Risotto machen', 'Zwiebeln in hoher Pfanne oder großem Topf anschwitzen. Graupen dazugeben und mit Port ablöschen. Gewürze und Mehl einrühren und nach und nach Rote-Beete-Saft dazugeben, bis die Graupen gar sind. Zum Schluss, kurz vor dem Servieren, die Rote-Beete-Würfel untermischen.', '00:05:00', '00:04:00', 327);
INSERT INTO public.steps OVERRIDING SYSTEM VALUE VALUES (378, 1, 'test', 'abc', '00:01:00', '00:02:00', 329);


--
-- Data for Name: users; Type: TABLE DATA; Schema: public; Owner: dennis
--

INSERT INTO public.users VALUES (1, 'dennis', 'dennis@kobert.dev', '$2b$12$79hdHFhnpFZlVezuyI4TH.7itapMKHbH7Vv5DSmhxf9EA19QW4ZRe', false, '2024-02-06 23:15:38.837939+00');
INSERT INTO public.users VALUES (0, 'test', '', 'password', true, '2024-05-29 18:39:11.36942+00');


--
-- Data for Name: user_groups; Type: TABLE DATA; Schema: public; Owner: dennis
--



--
-- Data for Name: weights; Type: TABLE DATA; Schema: public; Owner: dennis
--

INSERT INTO public.weights VALUES (5, 2, 0.058);
INSERT INTO public.weights VALUES (5, 35, 0.21);
INSERT INTO public.weights VALUES (5, 34, 0.155);
INSERT INTO public.weights VALUES (5, 8, 0.05);
INSERT INTO public.weights VALUES (7, 20, 0.003);
INSERT INTO public.weights VALUES (10, 39, 0.450);
INSERT INTO public.weights VALUES (12, 9, 0.0175);
INSERT INTO public.weights VALUES (5, 21, 0.099);
INSERT INTO public.weights VALUES (12, 69, 0.025);
INSERT INTO public.weights VALUES (12, 70, 0.033);
INSERT INTO public.weights VALUES (5, 76, 0.035);
INSERT INTO public.weights VALUES (5, 176, 0.05);
INSERT INTO public.weights VALUES (5, 167, 0.075);
INSERT INTO public.weights VALUES (5, 66, 0.14);
INSERT INTO public.weights VALUES (5, 96, 0.135);
INSERT INTO public.weights VALUES (5, 140, 0.08);
INSERT INTO public.weights VALUES (5, 125, 0.004);
INSERT INTO public.weights VALUES (2, 6, 0.9);
INSERT INTO public.weights VALUES (2, 7, 0.9);
INSERT INTO public.weights VALUES (5, 165, 0.8);
INSERT INTO public.weights VALUES (5, 64, 0.042);
INSERT INTO public.weights VALUES (5, 183, 0.042);
INSERT INTO public.weights VALUES (5, 194, 0.05);
INSERT INTO public.weights VALUES (5, 82, 0.0005);
INSERT INTO public.weights VALUES (9, 203, 0.03);
INSERT INTO public.weights VALUES (5, 103, 0.115);
INSERT INTO public.weights VALUES (9, 67, 0.06);
INSERT INTO public.weights VALUES (5, 65, 0.01);
INSERT INTO public.weights VALUES (5, 89, 0.06);
INSERT INTO public.weights VALUES (9, 76, 0.176);
INSERT INTO public.weights VALUES (9, 232, 0.025);




--
-- Name: event_meals_meal_id_seq; Type: SEQUENCE SET; Schema: public; Owner: dennis
--

SELECT pg_catalog.setval('public.event_meals_meal_id_seq', 193, true);


--
-- Name: events_event_id_seq; Type: SEQUENCE SET; Schema: public; Owner: dennis
--

SELECT pg_catalog.setval('public.events_event_id_seq', 58, true);


--
-- Name: food_prep_prep_id_seq; Type: SEQUENCE SET; Schema: public; Owner: dennis
--

SELECT pg_catalog.setval('public.food_prep_prep_id_seq', 1, false);


--
-- Name: food_properties_property_id_seq; Type: SEQUENCE SET; Schema: public; Owner: dennis
--

SELECT pg_catalog.setval('public.food_properties_property_id_seq', 48, true);


--
-- Name: groups_id_seq; Type: SEQUENCE SET; Schema: public; Owner: dennis
--

SELECT pg_catalog.setval('public.groups_id_seq', 1, false);


--
-- Name: ingredient_sources_ingredient_source_id_seq; Type: SEQUENCE SET; Schema: public; Owner: dennis
--

SELECT pg_catalog.setval('public.ingredient_sources_ingredient_source_id_seq', 271, true);


--
-- Name: ingredients_ingredient_id_seq; Type: SEQUENCE SET; Schema: public; Owner: dennis
--

SELECT pg_catalog.setval('public.ingredients_ingredient_id_seq', 283, true);


--
-- Name: inventories_inventory_id_seq; Type: SEQUENCE SET; Schema: public; Owner: dennis
--

SELECT pg_catalog.setval('public.inventories_inventory_id_seq', 2, true);


--
-- Name: places_place_id_seq; Type: SEQUENCE SET; Schema: public; Owner: dennis
--

SELECT pg_catalog.setval('public.places_place_id_seq', 11, true);


--
-- Name: recipes_recipe_id_seq; Type: SEQUENCE SET; Schema: public; Owner: dennis
--

SELECT pg_catalog.setval('public.recipes_recipe_id_seq', 337, true);


--
-- Name: shopping_tours_tour_id_seq; Type: SEQUENCE SET; Schema: public; Owner: dennis
--

SELECT pg_catalog.setval('public.shopping_tours_tour_id_seq', 14, true);


--
-- Name: steps_step_id_seq; Type: SEQUENCE SET; Schema: public; Owner: dennis
--

SELECT pg_catalog.setval('public.steps_step_id_seq', 379, true);


--
-- Name: stores_store_id_seq; Type: SEQUENCE SET; Schema: public; Owner: dennis
--

SELECT pg_catalog.setval('public.stores_store_id_seq', 1, false);


--
-- Name: units_unit_id_seq; Type: SEQUENCE SET; Schema: public; Owner: dennis
--

SELECT pg_catalog.setval('public.units_unit_id_seq', 12, true);


--
-- Name: users_id_seq; Type: SEQUENCE SET; Schema: public; Owner: dennis
--

SELECT pg_catalog.setval('public.users_id_seq', 1, true);


--
-- PostgreSQL database dump complete
--



-- Add owner_id to recipes table
ALTER TABLE recipes ADD COLUMN owner_id BIGINT;

-- Default all existing recipes to the first admin user (update this ID if needed)
UPDATE recipes SET owner_id = (SELECT id FROM users WHERE is_admin = true LIMIT 1);

-- After all data is migrated, make the column NOT NULL
ALTER TABLE recipes ALTER COLUMN owner_id SET NOT NULL;

-- Add foreign key constraint
ALTER TABLE recipes ADD CONSTRAINT recipe_owner_fk FOREIGN KEY (owner_id) REFERENCES users(id);

-- Add owner_id to events table
ALTER TABLE events ADD COLUMN owner_id BIGINT;

-- Default all existing events to the first admin user (update this ID if needed)
UPDATE events SET owner_id = (SELECT id FROM users WHERE is_admin = true LIMIT 1);

-- After all data is migrated, make the column NOT NULL
ALTER TABLE events ALTER COLUMN owner_id SET NOT NULL;

-- Add foreign key constraint
ALTER TABLE events ADD CONSTRAINT event_owner_fk FOREIGN KEY (owner_id) REFERENCES users(id);

-- Add owner_id to ingredients table
ALTER TABLE ingredients ADD COLUMN owner_id BIGINT;

-- Default all existing ingredients to the first admin user (update this ID if needed)
UPDATE ingredients SET owner_id = (SELECT id FROM users WHERE is_admin = true LIMIT 1);

-- After all data is migrated, make the column NOT NULL
ALTER TABLE ingredients ALTER COLUMN owner_id SET NOT NULL;

-- Add foreign key constraint
ALTER TABLE ingredients ADD CONSTRAINT event_owner_fk FOREIGN KEY (owner_id) REFERENCES users(id);


-- Add owner_id to inventories table
ALTER TABLE inventories ADD COLUMN owner_id BIGINT;

-- Default all existing inventories to the first admin user (update this ID if needed)
UPDATE inventories SET owner_id = (SELECT id FROM users WHERE is_admin = true LIMIT 1);

-- After all data is migrated, make the column NOT NULL
ALTER TABLE inventories ALTER COLUMN owner_id SET NOT NULL;

-- Add foreign key constraint
ALTER TABLE inventories ADD CONSTRAINT event_owner_fk FOREIGN KEY (owner_id) REFERENCES users(id);


UPDATE recipes SET owner_id = (SELECT id FROM users WHERE is_admin = true LIMIT 1);
UPDATE events SET owner_id = (SELECT id FROM users WHERE is_admin = true LIMIT 1);

UPDATE ingredients SET owner_id = (SELECT id FROM users WHERE is_admin = true LIMIT 1);

UPDATE inventories SET owner_id = (SELECT id FROM users WHERE is_admin = true LIMIT 1);

SET session_replication_role = DEFAULT;
