# Foodcalc CLI
This Project aims to creat a simple to use Commandline application that interfaces with a Database created by the Foodcalc Tool

## Structure
- list
    - places
    - events
    - ingredients
    - reciepes
    - meals
- show
    - event
    - meal
- print
    - mealplan (requires -e)
    - meal

## Functionality I want to Implement
    * List Places
    * List Events
    * List Ingredients
    * List Reciepes
    * List Meals
    * Print Information about Event
    * Print Information about Meal of Event
    * Print Information about Custom Meal
    * Print Mealplan for Event
    * Print Calculated Recipe for Meal of Event
    * Print Calculated Recipe for Custom Meal
    * Print Calculated Recipes for Event
    * Add Ingredient
    * Delete Ingredient
    * Edit Ingredient Name
    * Edit Ingredient Energy
    * Edit Ingredient Comment
    * Add Reciepe
    * Delete Reciepe
    * Edit Recipe Name
    * Edit Recipe Comment
    * Edit Recipe Ingredients: Add Ingredient
    * Edit Recipe Ingredients: Remove Ingredient
    * Edit Recipe Ingredients: Change Amount (Amount + Unit)
    * Reorder Recipe Steps
    * Edit Recipe Steps: Add Step
    * Edit Recipe Steps: Remove Step
    * Edit Recipe Steps: Change Name
    * Edit Recipe Steps: Change Description
    * Edit Recipe Steps: Change Duration (Fixed part)
    * Edit Recipe Steps: Change Duration (Scaled part)
    * Add User
    * Delete User
    * Change User Permissions
    * Add Event
    * Delete Event
    * Edit Event Buget
    * Edit Event Comment
    * Add Meal to Event
    * Remove Meal from Event
    * Edit Meal Recipe
    * Edit Meal Location
    * Edit Meal Start Time
    * Edit Meal End Time
    * Edit Meal Servings
    * Edit Meal Calories
    * Edit Meal Comment
    * Trigger Prices Update

## New Structure
* [ ] update prices
* [x] list
    * [x] places
    * [x] events
    * [x] ingredients
    * [x] reciepes
    * [x] meals
    * [x] users
* [ ] info
    * [ ] ingredient <ingredient_ref>
    * [x] event <event_ref>
    * [ ] reciepe <reciepe_ref> <people>
    * [x] meal <event_ref> <reciepe_ref> [<start_time>]
    * [x] users
* [x] calc
    * [x] mealplan <event_ref>
    * [x] meal <event_ref> <reciepe_ref> [<start_time>]
    * [x] reciepe <reciepe_ref> [<people>] [<calories>] [--format markdown]
* [x] add
    * [x] ingredient <name> <energy> [<comment>]
    * [x] reciepe <name> [<comment>]
    * [x] user <name> <password> <e-mail> [--is-admin]
    * [x] event <name> <budget> [<comment>]
* [ ] delete
    * [ ] ingredient <ingredient_ref>
    * [x] reciepe <reciepe_ref>
    * [x] user <user_ref>
    * [ ] event <event_ref>
* [ ] edit
    * [ ] ingredient <ingredient_ref>
        * [x] name <name>
        * [x] energy <energy>
        * [x] comment <comment>
        * [ ] add-source <url> [<amount> <price>]
    * [ ] recipe <recipe_ref>
        * [x] name <new_name>
        * [x] comment <new_comment>
        * [ ] ingredients
            * [ ] add <ingredient_ref> <amount>
            * [ ] remove <ingredient_ref>
            * [ ] amount <ingredient_ref> <amount>
        * [ ] steps
            * [ ] add <name> [<description>] [<duration_fixed> <duration_scaled>] [<index>]
            * [ ] remove <step_ref>
            * [ ] reorder <list_of_indices>
            * [ ] edit <step_ref>
                * [ ] name <name>
                * [ ] description
                * [ ] duration <fixed|scaled> <duration>
    * [ ] user <user_ref>
        * [x] name <name>
        * [ ] password <password>
        * [x] email <e-mail>
        * [x] admin <bool>
    * [ ] event <event_ref>
        * [x] name <name>
        * [x] budget <budget>
        * [x] comment <comment>
        * [ ] meals
            * [ ] add <recipe_ref> <servings> <calories> <start_time> <end_time> [<comment>] [<location_ref>]
            * [ ] remove <reciepe_ref> [<start_time>]
            * [ ] edit <reciepe_ref> [<start_time>]
                * [ ] recipe <recipe_ref>
                * [ ] location <location_ref>
                * [ ] servings <servings>
                * [ ] calories <calories>
                * [ ] start <start_time>
                * [ ] end <end_time>
                * [ ] comment <comment>
