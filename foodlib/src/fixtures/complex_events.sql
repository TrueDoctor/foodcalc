-- Complex event scenarios
-- Event 1: Multi-day festival with multiple shopping tours and shared prep tasks
INSERT INTO events (event_id, event_name, comment, budget, owner_id) VALUES
    (100, 'Summer Festival', 'Three day festival', 1000.00, 1);

-- Event 2: Weekly meal prep
INSERT INTO events (event_id, event_name, comment, budget, owner_id) VALUES
    (101, 'Week 28 Meal Prep', 'Weekly meal preparation', 200.00, 1);

-- Add some additional recipes
INSERT INTO recipes (recipe_id, name, comment, owner_id) VALUES
    (100, 'Basic Tomato Sauce Base', 'Base sauce for multiple dishes', 1),
    (101, 'Festival Pasta', 'Large batch pasta dish', 1),
    (102, 'Weekly Lunch Box', 'Meal prep standard', 1);

INSERT INTO recipe_ingredients (recipe_id, ingredient_id, amount, unit_id) VALUES
    (101, 1, 10.0, 0),   -- 10kg pasta
    (101, 4, 0.5, 0);    -- 0.5kg olive oil

-- Add ingredients for Basic Tomato Sauce Base (recipe_id = 100)
INSERT INTO recipe_ingredients (recipe_id, ingredient_id, amount, unit_id) VALUES
    (100, 2, 5.0, 0),    -- 5kg tomatoes
    (100, 3, 0.1, 0),    -- 0.1kg garlic
    (100, 4, 0.2, 0);    -- 0.2kg olive oil


-- Recipe relationships (meta_recipes)
INSERT INTO meta_recipes (parent_id, child_id, weight) VALUES
    (101, 100, 2.0),  -- Festival Pasta uses 2kg of Basic Tomato Sauce
    (102, 100, 0.5);  -- Lunch Box uses 0.5kg of Basic Tomato Sauce

-- Event meals using these recipes
INSERT INTO event_meals (event_id, recipe_id, place_id, energy_per_serving, servings, start_time, end_time) VALUES
    (100, 101, 1, 2500, 100, '2024-07-01 18:00:00+00', '2024-07-01 20:00:00+00'),
    (100, 101, 1, 2500, 100, '2024-07-02 18:00:00+00', '2024-07-02 20:00:00+00'),
    (101, 102, 1, 2000, 10, '2024-07-08 12:00:00+00', '2024-07-08 13:00:00+00');

-- Food prep schedules
INSERT INTO food_prep (prep_id, event_id, recipe_id, prep_date, use_from, use_until) VALUES
    (100, 100, 100, '2024-06-30 14:00:00+00', '2024-06-30 14:00:00+00', '2024-07-02 20:00:00+00'),  -- Sauce prep for festival
    (101, 101, 100, '2024-07-07 15:00:00+00', '2024-07-07 15:00:00+00', '2024-07-08 13:00:00+00');  -- Sauce prep for meal prep

-- Additional ingredient sources with different prices
INSERT INTO ingredient_sources (ingredient_source_id, ingredient_id, store_id, package_size, unit_id, price) VALUES
    (100, 2, 1, 1.0, 0, 2.99),  -- Regular tomatoes at Local Grocery
    (101, 2, 2, 1.0, 0, 1.99);  -- Cheaper tomatoes at Farmers Market

-- Source overrides for events
INSERT INTO event_source_overrides (event_id, ingredient_source_id) VALUES
    (100, 101);  -- Festival uses cheaper tomatoes

-- Shopping tours
INSERT INTO shopping_tours (tour_id, event_id, tour_date, store_id) VALUES
    (100, 100, '2024-06-29 10:00:00+00', 2),  -- Festival main shopping at Farmers Market
    (101, 100, '2024-07-01 08:00:00+00', 1),  -- Festival resupply at Local Grocery
    (102, 101, '2024-07-07 09:00:00+00', 1);  -- Weekly prep shopping at Local Grocery

-- Additional test inventory
INSERT INTO inventories (inventory_id, name, owner_id) VALUES
    (100, 'Festival Storage', 1);

-- Event inventory association
INSERT INTO event_inventories (event_id, inventory_id) VALUES
    (100, 100),
    (100, 1);  -- Also use Main Pantry

-- Inventory contents
INSERT INTO inventory_ingredients (inventory_id, ingredient_id, amount) VALUES
    (100, 2, 5.0);  -- 5kg tomatoes in Festival Storage
