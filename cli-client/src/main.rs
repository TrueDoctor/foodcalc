mod args;

use foodlib::*;
use std::env;

use args::*;
use clap::Parser;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let database_url = &env::var("DATABASE_URL").expect("DATABASE_URL env var was not set");

    let food_base = FoodBase::new(database_url)
        .await
        .expect("Failed to connect to database");

    let cli = CLI::parse();

    if cli.debug {
        println!("{:?}", cli);
    }

    match &cli.command {
        Commands::List(list) => {
            let _place_flag = list.place.as_ref();
            let _event_flag = list.event.as_ref();
            let _ingredient_flag = list.ingredient.as_ref();
            let _recipe_flag = list.recipe.as_ref();
            let _meal_flag = list.meal.as_ref();

            let _place_filter = {
                if let Some(place) = &list.place {
                    let results = food_base
                        .get_place_from_string_reference(place.to_string())
                        .await;
                    if results.is_ok() {
                        Some(results.unwrap())
                    } else {
                        None
                    }
                } else {
                    None
                }
            };

            let _event_filter = {
                if let Some(event) = &list.event {
                    food_base
                        .get_event_from_string_reference(event.to_string())
                        .await
                } else {
                    None
                }
            };

            let _ingredient_filter = {
                if let Some(ingredient) = &list.ingredient {
                    food_base
                        .get_ingredient_from_string_reference(ingredient.to_string())
                        .await
                } else {
                    None
                }
            };

            let _recipe_filter = {
                if let Some(recipe) = &list.recipe {
                    food_base
                        .get_recipe_from_string_reference(recipe.to_string())
                        .await
                } else {
                    None
                }
            };

            let _meal_filter: Option<Meal> = None;

            if cli.debug {
                println!("Place Filter: {:?}", _place_filter);
                println!("Event Filter: {:?}", _event_filter);
                println!("Ingredient Filter: {:?}", _ingredient_filter);
                println!("Recipe Filter: {:?}", _recipe_filter);
                println!("Meal Filter: {:?}", _meal_filter);
            }

            match &list.list_type {
                ListTypes::Places => {
                    let places = food_base.get_places().await;
                    places.unwrap().iter().for_each(|place| {
                        print!("{}\t{}", place.place_id, place.name);
                        if let Some(comment) = &place.comment {
                            print!("\t{}", comment);
                        }
                        println!();
                    });
                }
                ListTypes::Events => {
                    let events = food_base.get_events().await;
                    events.unwrap().iter().for_each(|e| {
                        print!("{}\t{}", e.event_id, e.event_name);
                        if let Some(comment) = &e.comment {
                            print!("\t{}", comment);
                        }

                        if let Some(budget) = &e.budget {
                            print!("\t{}", budget.to_bigdecimal(2));
                        }
                        println!();
                    });
                }
                ListTypes::Ingredients => {
                    println!("Listing Ingredients");
                    let ingredients = food_base.get_ingredients().await;

                    println!("{:?}", ingredients);
                    ingredients.unwrap().iter().for_each(|i| {
                        print!("{}\t{}\t{}", i.ingredient_id, i.name, i.energy);
                        if let Some(comment) = &i.comment {
                            print!("\t{}", comment);
                        }
                        println!();
                    });
                }
                ListTypes::Recipes => {
                    let recipes = food_base.get_recipes().await;
                    recipes.unwrap().iter().for_each(|r| {
                        print!("{}\t{}", r.recipe_id, r.name);
                        if let Some(comment) = &r.comment {
                            print!("\t{}", comment);
                        }
                        println!();
                    });
                }
                ListTypes::Meals => {
                    //TODO List Meals
                    println!("Listing Meals");
                }
            }
        }
        Commands::Show(show_statement) => {
            match &show_statement.show_type {
                ShowCommands::Event(event) => {
                    let event_ref = event.event.as_str();

                    let event = food_base
                        .get_event_from_string_reference(event_ref.to_string())
                        .await;

                    if let Some(event) = event {
                        print!("{} ({})", event.event_name, event.event_id);
                        if let Some(comment) = &event.comment {
                            print!(", {}", comment);
                        }
                        println!();
                        if let Some(budget) = &event.budget {
                            println!("Budget: {}€", budget.to_bigdecimal(2));
                        }

                        let _ = food_base.get_event_cost(event.event_id).await.map(|cost| {
                            println!("Cost: {}€", cost.to_bigdecimal(2));
                        });

                        let meals = food_base
                            .get_event_meals(event.event_id)
                            .await
                            .unwrap_or_else(|_| Vec::new());

                        if meals.len() > 0 {
                            println!("Meals:");
                            meals.iter().for_each(|meal| {
                                println!(
                                    "\t{}  -  {}\t{} ({} Servings)",
                                    meal.start_time, meal.end_time, meal.name, meal.servings
                                );
                            });
                        } else {
                            println!("No Meals");
                        }
                    } else {
                        println!("Event not found");
                    }
                }
                ShowCommands::Recipe(recipe) => {
                    let recipe_ref = recipe.recipe.as_str();
                    //TODO Show Recipe
                    println!("Showing Recipe {:?}", recipe_ref);
                }
                ShowCommands::Meal(meal) => {
                    let meal_ref = meal.meal.as_str();
                    //TODO Show Meal
                    println!("Showing Meal {:?}", meal_ref);
                }
            }
        }
        Commands::Print(print_data) => {
            match &print_data.print_type {
                PrintCommands::Mealplan(event) => {
                    let event_ref = event.event.as_ref();
                    //TODO Print Event
                    println!("Printing Event {:?}", event_ref);
                }
                PrintCommands::Meal(meal) => {
                    let meal_ref = meal.meal.as_str();
                    //TODO Print Meal
                    println!("Printing Meal {:?}", meal_ref);
                }
            }
        }
        Commands::User(user_data) => {
            match &user_data.user_type {
                UserCommands::Create(params) => {
                    let user_name = params.user.as_str();
                    let user_password = params.password.as_str();
                    let user_email = params.email.as_str();
                    let is_admin = params.admin;

                    let credentials = Credenitals {
                        username: user_name.to_string(),
                        password: user_password.to_string(),
                    };

                    match food_base.create_user(user_email.to_string(), credentials, is_admin).await {
                        Ok(_) => {
                            println!("Created user '{}' (Admin: {})", user_name, is_admin);
                        }
                        Err(error) => {
                            println!("Error: {}", error)
                        }
                    }
                 }
                UserCommands::Remove(params) => {
                    let user_ref = params.user.as_str();
                    let user = food_base.get_user_by_string_reference(user_ref.to_string()).await;

                    if let Some(user) = user {
                        match food_base.delete_user(user.id).await {
                            Ok(_) => {
                                println!("User {} removed", user.username);
                            }
                            Err(error) => {
                                println!("Error: {}", error)
                            }
                        }
                    } else {
                        println!("User not found");
                    }
                }
                UserCommands::List => {
                    food_base.get_users().await.unwrap().iter().for_each(|user| {
                        print!("{}\t{}", user.id, user.username);
                        if user.is_admin {
                            print!("\tAdmin");
                        }
                        println!();
                    });
                }
            }
        }
        #[allow(unreachable_patterns)]
        _default => {
            println!("Unknown Command");
        }
    }
}
