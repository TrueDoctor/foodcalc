-- Basic Units
INSERT INTO units (unit_id, name) VALUES
  (0, 'kg'),
  (1, 'g'),
  (2, 'L'),
  (3, 'mL'),
  (4, 'piece'),
  (5, 'slice');

-- Ingredients with energy in kJ/g
INSERT INTO ingredients (ingredient_id, name, energy, comment) VALUES 
  (1, 'Pasta', 15.7, 'Dried pasta'),
  (2, 'Tomato', 0.18, 'Fresh tomatoes'),
  (3, 'Garlic', 1.49, 'Fresh garlic'),
  (4, 'Olive Oil', 37.0, 'Extra virgin'),
  (5, 'Flour', 14.7, 'All-purpose flour'),
  (6, 'Sugar', 16.8, 'Granulated sugar'),
  (7, 'Eggs', 6.3, 'Large eggs'),
  (8, 'Milk', 2.6, 'Whole milk, 3.5% fat'),
  (9, 'Butter', 30.2, 'Unsalted butter');

-- Base conversions between units
INSERT INTO base_conversions (from_unit, to_unit, from_amount, to_amount) VALUES
  (1, 0, 1000, 1),    -- 1000g = 1kg
  (3, 2, 1000, 1),    -- 1000mL = 1L
  (0, 0, 1, 1),       -- base unit self-conversion
  (2, 2, 1, 1);       -- base unit self-conversion

-- Weight conversions (all based on kg as base unit)
INSERT INTO weights (unit_id, ingredient_id, weight) VALUES
  -- Pasta
  (0, 1, 1.0),    -- 1kg = 1kg
  (1, 1, 0.001),  -- 1g = 0.001kg
  
  -- Tomatoes
  (0, 2, 1.0),    -- 1kg = 1kg
  (1, 2, 0.001),  -- 1g = 0.001kg
  (4, 2, 0.150),  -- 1 piece ≈ 150g = 0.150kg
  
  -- Garlic
  (0, 3, 1.0),    -- 1kg = 1kg
  (1, 3, 0.001),  -- 1g = 0.001kg
  (4, 3, 0.005),  -- 1 piece (clove) ≈ 5g = 0.005kg
  
  -- Olive Oil (density ≈ 0.92 kg/L)
  (0, 4, 1.0),    -- 1kg = 1kg
  (2, 4, 0.92),   -- 1L = 0.92kg
  (3, 4, 0.00092),-- 1mL = 0.00092kg
  
  -- Flour
  (0, 5, 1.0),    -- 1kg = 1kg
  (1, 5, 0.001),  -- 1g = 0.001kg
  
  -- Sugar
  (0, 6, 1.0),    -- 1kg = 1kg
  (1, 6, 0.001),  -- 1g = 0.001kg
  
  -- Eggs
  (0, 7, 1.0),    -- 1kg = 1kg
  (4, 7, 0.060),  -- 1 piece ≈ 60g = 0.060kg
  
  -- Milk (density ≈ 1.03 kg/L)
  (0, 8, 1.0),    -- 1kg = 1kg
  (2, 8, 1.03),   -- 1L = 1.03kg
  (3, 8, 0.00103),-- 1mL = 0.00103kg
  
  -- Butter
  (0, 9, 1.0),    -- 1kg = 1kg
  (1, 9, 0.001),  -- 1g = 0.001kg
  (5, 9, 0.010);  -- 1 slice ≈ 10g = 0.010kg

-- Stores
INSERT INTO stores (store_id, name) VALUES
  (1, 'Local Grocery'),
  (2, 'Farmers Market');

-- Ingredient sources with prices (in currency per package)
INSERT INTO ingredient_sources (ingredient_source_id, ingredient_id, store_id, package_size, unit_id, price, url, comment) VALUES
  (1, 1, 1, 0.5, 0, 2.99, NULL, '500g package dried pasta'),
  (2, 2, 2, 1.0, 0, 3.99, NULL, '1kg fresh local tomatoes'),
  (3, 3, 1, 0.250, 0, 1.99, NULL, '250g garlic net'),
  (4, 4, 1, 1.0, 2, 5.99, NULL, '1L extra virgin olive oil'),
  (5, 5, 1, 1.0, 0, 2.49, NULL, '1kg all-purpose flour'),
  (6, 6, 1, 1.0, 0, 1.99, NULL, '1kg granulated sugar'),
  (7, 7, 2, 10.0, 4, 4.99, NULL, '10-piece egg carton'),
  (8, 8, 1, 1.0, 2, 1.99, NULL, '1L whole milk'),
  (9, 9, 1, 0.250, 0, 3.99, NULL, '250g butter block');

-- Basic recipes
INSERT INTO recipes (recipe_id, name, comment) VALUES
  (1, 'Simple Pasta', 'Quick weeknight dinner'),
  (2, 'Basic Cake', 'Classic vanilla cake'),
  (3, 'Tomato Sauce', 'Basic sauce for pasta');

-- Recipe ingredients
INSERT INTO recipe_ingredients (recipe_id, ingredient_id, amount, unit_id) VALUES
  -- Simple Pasta with Tomato Sauce
  (1, 1, 500, 1),    -- 500g pasta
  -- Basic Cake
  (2, 5, 300, 1),    -- 300g flour
  (2, 6, 200, 1),    -- 200g sugar
  (2, 7, 4, 4),      -- 4 eggs
  (2, 8, 250, 3),    -- 250mL milk
  (2, 9, 200, 1),    -- 200g butter
  -- Tomato Sauce
  (3, 2, 500, 1),    -- 500g tomatoes
  (3, 3, 3, 4),      -- 3 cloves garlic
  (3, 4, 30, 3);     -- 30mL olive oil

-- Meta recipe (combining pasta with sauce)
INSERT INTO meta_recipes (parent_id, child_id, weight) VALUES
  (1, 3, 0.5);        -- Add 500g of tomato sauce to pasta

-- Places
INSERT INTO places (place_id, name, comment) VALUES
  (1, 'Main Kitchen', 'Primary cooking area'),
  (2, 'Backup Kitchen', 'Secondary cooking area');

-- Events
INSERT INTO events (event_id, event_name, comment, budget) VALUES
  (1, 'Family Dinner', 'Weekly family meal', 50.00),
  (2, 'Birthday Party', 'Birthday celebration', 100.00);

-- Event meals (energy in kJ per serving)
INSERT INTO event_meals (event_id, recipe_id, place_id, comment, energy_per_serving, servings, start_time, end_time) VALUES
  (1, 1, 1, 'Family pasta night', 2500, 4, '2024-12-04 18:00:00+00', '2024-12-04 19:00:00+00'),
  (2, 2, 1, 'Birthday cake', 1800, 12, '2024-12-04 14:00:00+00', '2024-12-04 15:00:00+00');

-- Inventories
INSERT INTO inventories (inventory_id, name) VALUES
  (1, 'Main Pantry'),
  (2, 'Backup Storage');

-- Inventory contents (all in kg)
INSERT INTO inventory_ingredients (inventory_id, ingredient_id, amount) VALUES
  (1, 1, 2.0),    -- 2kg pasta
  (1, 5, 1.5),    -- 1.5kg flour
  (1, 6, 1.0),    -- 1kg sugar
  (2, 4, 0.460);  -- 0.46kg olive oil (500mL)

-- Event inventories
INSERT INTO event_inventories (event_id, inventory_id) VALUES
  (1, 1),  -- Family dinner uses main pantry
  (2, 1),  -- Birthday party uses main pantry
  (2, 2);  -- Birthday party also uses backup storage

-- Shopping tours
INSERT INTO shopping_tours (tour_id, event_id, tour_date, store_id) VALUES
  (1, 1, '2024-12-03 10:00:00+00', 1),  -- Family dinner shopping
  (2, 2, '2024-12-03 14:00:00+00', 2);  -- Birthday party shopping

-- Food prep schedules
INSERT INTO food_prep (prep_id, event_id, recipe_id, prep_date, use_from, use_until) VALUES
  (1, 2, 2, '2024-12-04 10:00:00+00', '2024-12-04 10:00:00+00', '2024-12-04 14:00:00+00');  -- Cake prep

-- Recipe steps (durations in minutes)
INSERT INTO steps (step_id, step_order, step_name, step_description, fixed_duration, duration_per_kg, recipe_id) OVERRIDING SYSTEM VALUE VALUES
  -- Simple Pasta
  (1, 1, 'Boil Water', 'Fill large pot with water and bring to boil', '00:10:00', '00:00:00', 1),
  (2, 2, 'Cook Pasta', 'Add pasta and cook until al dente', '00:02:00', '00:08:00', 1),
  -- Basic Cake
  (3, 1, 'Prep', 'Preheat oven to 180°C and grease cake tin', '00:10:00', '00:00:00', 2),
  (4, 2, 'Mix Dry', 'Combine flour and sugar', '00:05:00', '00:01:00', 2),
  (5, 3, 'Mix Wet', 'Beat eggs, milk, and butter', '00:05:00', '00:01:00', 2),
  (6, 4, 'Combine', 'Fold wet ingredients into dry mix', '00:05:00', '00:02:00', 2),
  (7, 5, 'Bake', 'Bake at 180°C', '00:30:00', '00:10:00', 2),
  -- Tomato Sauce
  (8, 1, 'Prep Ingredients', 'Dice tomatoes and mince garlic', '00:10:00', '00:05:00', 3),
  (9, 2, 'Cook Sauce', 'Simmer all ingredients', '00:20:00', '00:15:00', 3);

-- Users for testing
INSERT INTO users (id, username, email, password_hash, is_admin, created_at) VALUES
  (1, 'admin', 'admin@example.com', '$2a$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LQtXDOQSZCDILT2rG', true, '2024-12-04 00:00:00+00'),
  (2, 'user', 'user@example.com', '$2a$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LQtXDOQSZCDILT2rG', false, '2024-12-04 00:00:00+00');

-- Groups
INSERT INTO groups (id, name) VALUES
  (1, 'Administrators'),
  (2, 'Users');

-- User group assignments
INSERT INTO user_groups (user_id, group_id) VALUES
  (1, 1),  -- admin in Administrators group
  (2, 2);  -- user in Users group
