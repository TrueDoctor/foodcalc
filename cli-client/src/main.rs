mod args;

use args::*;
use clap::Parser;

fn main() {
    let cli = CLI::parse();

    //println!("{:?}", cli);

    match &cli.command {
        Commands::List(list) => {
            let _place_filter = list.place.as_ref();
            let _event_filter = list.event.as_ref();
            let _ingredient_filter = list.ingredient.as_ref();
            let _recipe_filter = list.recipe.as_ref();
            let _meal_filter = list.meal.as_ref();

            match &list.list_type {
                ListTypes::Places => {
                    //TODO List Places
                    println!("Listing Places");
                },
                ListTypes::Events => {
                    //TODO List Events
                    println!("Listing Events");
                },
                ListTypes::Ingredients => {
                    //TODO List Ingredients
                    println!("Listing Ingredients");
                },
                ListTypes::Recipes => {
                    //TODO List Reciepes
                    println!("Listing Reciepes");
                },
                ListTypes::Meals => {
                    //TODO List Meals
                    println!("Listing Meals");
                },
                
            }
        },
        Commands::Show(show_statement) => {
            match &show_statement.show_type {
                ShowCommands::Event(event) => {
                    let event_ref = event.event.as_str();
                    //TODO Show Event
                    println!("Showing Event {:?}", event_ref);
                },
                ShowCommands::Recipe(recipe) => {
                    let recipe_ref = recipe.recipe.as_str();
                    //TODO Show Recipe
                    println!("Showing Recipe {:?}", recipe_ref);
                },
                ShowCommands::Meal(meal) => {
                    let meal_ref = meal.meal.as_str();
                    //TODO Show Meal
                    println!("Showing Meal {:?}", meal_ref);
                },
            }
        },
        Commands::Print(print_data) => {
            match &print_data.print_type {
                PrintCommands::Mealplan(event) => {
                    let event_ref = event.event.as_ref();
                    //TODO Print Event
                    println!("Printing Event {:?}", event_ref);
                },
                PrintCommands::Meal(meal) => {
                    let meal_ref = meal.meal.as_str();
                    //TODO Print Meal
                    println!("Printing Meal {:?}", meal_ref);
                },
            }
        }
        #[allow(unreachable_patterns)]
        _default => {
            println!("Unknown Command");
        },
    }
}
