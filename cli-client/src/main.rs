mod args;

use foodlib::*;
use std::env;

use args::*;
use clap::Parser;
use tabled::{
    builder::Builder,
    settings::{
        locator::ByColumnName, style::BorderSpanCorrection, Disable, Panel, Settings, Style,
    },
    Table,
};

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let database_url = &env::var("DATABASE_URL").expect("DATABASE_URL env var was not set");

    let food_base = FoodBase::new(database_url)
        .await
        .expect("Failed to connect to database");

    let table_config = Settings::default()
        //.with(ColumnNames::default())
        //.with(BorderSpanCorrection)
        .with(Style::rounded());

    let cli = Cli::parse();

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
                    let places = food_base.get_places().await.unwrap();
                    let table = Table::new(places).with(table_config).to_string();
                    print!("{}", table);
                }
                ListTypes::Events => {
                    let events = food_base.get_events().await.unwrap();
                    let table = Table::new(events).with(table_config).to_string();
                    print!("{}", table);
                }
                ListTypes::Ingredients => {
                    let ingredients = food_base.get_ingredients().await.unwrap();
                    let table = Table::new(ingredients).with(table_config).to_string();
                    print!("{}", table);
                }
                ListTypes::Recipes => {
                    let recipes = food_base.get_recipes().await.unwrap();
                    let table = Table::new(recipes).with(table_config).to_string();
                    print!("{}", table);
                }
                ListTypes::Meals => {
                    let meals = if _event_filter.is_some() {
                        food_base
                            .get_event_meals(_event_filter.unwrap().event_id)
                            .await
                    } else {
                        food_base.get_meals().await
                    };
                    let meals = meals.unwrap();

                    let table = Table::new(meals).with(table_config).to_string();
                    print!("{}", table);
                }
                ListTypes::Users => {
                    // TODO Improve Formatting
                    food_base
                        .get_users()
                        .await
                        .unwrap()
                        .iter()
                        .for_each(|user| {
                            print!("{}\t{}", user.id, user.username);
                            if user.is_admin {
                                print!("\tAdmin");
                            }
                            println!();
                        });
                }
            }
        }
        Commands::Info(show_statement) => {
            match &show_statement.show_type {
                InfoCommands::Ingredient(ingredient) => {
                    let ingredient_ref = ingredient.ingredient_ref.as_str();

                    let ingredient =
                        food_base.get_ingredient_from_string_reference(ingredient_ref.to_string());

                    if let Some(ingredient) = ingredient.await {
                        println!("Showing Ingredient {:?}", ingredient);
                        //TODO Add Ingredient Formatting
                    } else {
                        println!("Ingredient not found");
                    }
                }
                InfoCommands::Event(event) => {
                    let event_ref = event.event_ref.as_str();

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

                        if !meals.is_empty() {
                            println!("Meals:");
                            let table = Table::new(meals)
                                .with(Disable::column(ByColumnName::new("event_id")))
                                .with(table_config)
                                .to_string();
                            print!("{}", table);
                        } else {
                            println!("No Meals");
                        }
                    } else {
                        println!("Event not found");
                    }
                }
                InfoCommands::Recipe(recipe) => {
                    let recipe_ref = recipe.recipe_ref.as_str();
                    println!("Showing Recipe {:?}", recipe_ref);

                    let recipe = food_base
                        .get_recipe_from_string_reference(recipe_ref.to_string())
                        .await;

                    if let Some(recipe) = recipe {
                        println!("Showing Recipe {:?}", recipe);
                        //TODO Add Recipe Formatting
                    } else {
                        println!("Recipe not found");
                    }
                }
                InfoCommands::Meal(meal) => {
                    let _recipe_ref = meal.recipe_ref.as_str();
                    let recipe = food_base
                        .get_recipe_from_string_reference(_recipe_ref.to_string())
                        .await;

                    if recipe.is_none() {
                        println!("Recipe not found");
                        return;
                    }
                    let recipe = recipe.unwrap();

                    let _event_ref = meal.event_ref.as_str();
                    let event = food_base
                        .get_event_from_string_reference(_event_ref.to_string())
                        .await;

                    if event.is_none() {
                        println!("Event not found");
                        return;
                    }
                    let event = event.unwrap();

                    let meals = food_base
                        .get_event_meal(event.event_id, recipe.recipe_id)
                        .await;

                    //TODO Implement time filter

                    match meals {
                        Ok(meals) => {
                            match meals.len() {
                                0 => {
                                    println!("No Meals found");
                                }
                                1 => {
                                    let meal = meals.first().unwrap();
                                    // TODO Add better Meal Formatting
                                    println!("Showing Meal {:?}", meal);
                                    let ingredients = food_base
                                        .get_event_recipe_ingredients(
                                            meal.event_id,
                                            meal.recipe_id,
                                            meal.place_id,
                                            meal.start_time,
                                        )
                                        .await;
                                    if let Ok(ingredients) = ingredients {
                                        let headers = vec!["Ingredient", "Amount", "Price"];

                                        let mut builder = Builder::default();

                                        builder.set_header(headers);

                                        ingredients.iter().for_each(|ingredient| {
                                            builder.push_record(vec![
                                                ingredient.name.clone(),
                                                format!("{} kg", ingredient.weight),
                                                format!("{} €", ingredient.price.to_bigdecimal(2)),
                                            ]);
                                        });

                                        println!(
                                            "{}",
                                            builder
                                                .build()
                                                .with(Panel::footer(format!(
                                                    "{} €",
                                                    meal.price.to_bigdecimal(2)
                                                )))
                                                .with(table_config)
                                        );
                                    } else {
                                        println!("No Ingredients found");
                                    }
                                }
                                _ => {
                                    println!("Multiple Meals found: ");
                                    meals.iter().for_each(|meal| {
                                        println!(
                                            "{} - {}, {} Servings",
                                            meal.start_time, meal.end_time, meal.servings
                                        );
                                    });
                                }
                            }
                        }
                        Err(error) => {
                            println!("Error: {}", error);
                        }
                    }
                }
            }
        }
        Commands::Calc(print_data) => {
            match &print_data.print_type {
                CalcCommands::Mealplan(event) => {
                    let event_ref = &event.event_ref;

                    let event = food_base
                        .get_event_from_string_reference(event_ref.to_string())
                        .await;

                    match event {
                        Some(event) => {
                            let mut meals =
                                food_base.get_event_meals(event.event_id).await.unwrap();
                            meals.sort_by(|a, b| a.start_time.cmp(&b.start_time));

                            let mut days: Vec<_> =
                                meals.iter().map(|meal| meal.start_time.date()).collect();
                            days.dedup();

                            let mut tables: Vec<(String, Table)> = Vec::new();

                            for day in days.iter() {
                                let mut builder = Builder::default();
                                let date_str = day.format("%A, %d.%m.%Y").to_string();
                                //builder.push_record(vec![date_str]);

                                builder.push_record(vec![
                                    "Name".to_string(),
                                    "Place".to_string(),
                                    "Start".to_string(),
                                    "End".to_string(),
                                    "Servings".to_string(),
                                    "Comment".to_string(),
                                ]);
                                let meals = meals
                                    .iter()
                                    .filter(|meal| meal.start_time.date() == *day)
                                    .collect::<Vec<_>>();

                                for meal in meals.iter() {
                                    builder.push_record(vec![
                                        meal.name.clone(),
                                        meal.place.clone(),
                                        meal.start_time.format("%H:%M").to_string(),
                                        meal.end_time.format("%H:%M").to_string(),
                                        meal.servings.to_string(),
                                        meal.comment.clone().unwrap_or_default(),
                                    ]);
                                }

                                let table = builder
                                    .build()
                                    //.with(Panel::header(date_str.clone()))
                                    .clone();
                                tables.push((date_str, table));
                                //println!("{}", table);
                            }

                            if !tables.is_empty() {
                                if tables.len() > 1 {
                                    tables.into_iter().for_each(|(date, mut table)| {
                                        table
                                            .with(Panel::header(date.clone()))
                                            .with(table_config.clone())
                                            .with(BorderSpanCorrection);
                                        println!("{}", table);
                                    });
                                }
                            } else {
                                println!("No Meals");
                            }
                        }
                        None => {
                            println!("Could not find Event")
                        }
                    }
                }
                CalcCommands::Meal(meal) => {
                    let _event_ref = meal.event_ref.as_str();
                    let _recipe_ref = meal.recipe_ref.as_str();
                    //let start_time_ref = meal.start_time.as_str();

                    //TODO Print Meal
                    println!("Printing Meal ");
                }
                CalcCommands::Recipe(recipe) => {
                    let recipe_ref = &recipe.recipe;
                    let people = recipe.people;
                    let calories = recipe.calories;
                    let recipe_data = food_base
                        .get_recipe_from_string_reference(recipe_ref.to_string())
                        .await
                        .unwrap();

                    let subrecipes = food_base
                        .fetch_subrecipes_from_user_input(
                            recipe_data.recipe_id,
                            people as f64,
                            calories,
                        )
                        .await
                        .unwrap();

                    let output = match recipe.format.to_string().as_str() {
                        "latex" | "tex" => food_base.format_subrecipes_latex(subrecipes).await,
                        "markdown" => food_base.format_subrecipes_markdown(subrecipes).await,
                        _ => "Unknown Format".to_string(),
                    };
                    println!("{}", output);
                }
            }
        }
        Commands::Add(add_data) => {
            match &add_data.add_type {
                AddCommands::Ingredient(ingredient) => {
                    //TODO Implement
                    let _name = ingredient.name.as_str();
                    let _energy = ingredient.energy;
                    let _comment = ingredient.comment.as_str();
                }
                AddCommands::Recipe(recipe) => {
                    //TODO Implement
                    let _name = recipe.name.as_str();
                    let _comment = recipe.comment.as_str();
                }
                AddCommands::User(user) => {
                    let user_name = user.user.as_str();
                    let user_password = user.password.as_str();
                    let user_email = user.email.as_str();
                    let is_admin = user.admin;

                    let credentials = Credenitals {
                        username: user_name.to_string(),
                        password: user_password.to_string(),
                    };

                    match food_base
                        .create_user(user_email.to_string(), credentials, is_admin)
                        .await
                    {
                        Ok(_) => {
                            println!("Created user '{}' (Admin: {})", user_name, is_admin);
                        }
                        Err(error) => {
                            println!("Error: {}", error)
                        }
                    }
                }
                AddCommands::Event(event) => {
                    //TODO Implement
                    let _name = event.name.as_str();
                    let _budget = event.budget;
                    let _comment = event.comment.as_str();
                }
            }
        }
        Commands::Delete(delete_data) => match &delete_data.delete_type {
            DeleteCommands::Ingredient(ingredient) => {
                let ingredient_ref = ingredient.ingredient.as_str();
                println!("Deleting Ingredient {:?}", ingredient_ref);
            }
            DeleteCommands::Recipe(recipe) => {
                let recipe_ref = recipe.recipe.as_str();
                println!("Deleting Recipe {:?}", recipe_ref);
            }
            DeleteCommands::User(user) => {
                let user_ref = user.user.as_str();
                let user = food_base
                    .get_user_by_string_reference(user_ref.to_string())
                    .await;

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
            DeleteCommands::Event(event) => {
                let event_ref = event.event.as_str();
                println!("Deleting Event {:?}", event_ref);
            }
        },
        Commands::Edit(edit_data) => match &edit_data.edit_type {
            EditCommands::Ingredient(_ingredient) => todo!(),
            EditCommands::Recipe(recipe) => {
                let _recipe_ref = recipe.recipe.as_str();
                match &recipe.edit_type {
                    EditRecipeType::Name(_name) => todo!(),
                    EditRecipeType::Comment(_comment) => todo!(),
                    EditRecipeType::Ingredients(ingredient) => match &ingredient.ingredient_edit_type {
                        EditRecipeIngredientsType::Add(_ingredient) => todo!(),
                        EditRecipeIngredientsType::Remove(_ingredient) => todo!(),
                        EditRecipeIngredientsType::Amount(_ingredient) => todo!(),
                    },
                    EditRecipeType::Steps(steps) => match &steps.step_edit_type {
                        EditRecipeStepsType::Add(_step) => todo!(),
                        EditRecipeStepsType::Remove(_step) => todo!(),
                        EditRecipeStepsType::Reorder(_step) => todo!(),
                        EditRecipeStepsType::Edit(_step) => todo!(),
                    },
                }
            }
            EditCommands::User(user) => {
                let _user_ref = user.user.as_str();
                match &user.edit_type {
                    EditUserType::Name(_name) => todo!(),
                    EditUserType::Password(_password) => todo!(),
                    EditUserType::Email(_email) => todo!(),
                    EditUserType::Admin(_admin) => todo!(),
                }
            },
            EditCommands::Event(event) => {
                let _event_ref = event.event.as_str();
                match &event.edit_type {
                    EditEventType::Name(_name) => todo!(),
                    EditEventType::Budget(_budget) => todo!(),
                    EditEventType::Comment(_comment) => todo!(),
                    EditEventType::Meals(meals) => match &meals.meal_edit_type {
                        EditEventMealsType::Add(_meal) => todo!(),
                        EditEventMealsType::Remove(_meal) => todo!(),
                        EditEventMealsType::Edit(meal) => {
                            let _recipe_ref = meal.recipe.as_str();
                            let _start_time = meal.start_time.as_str();

                            match &meal.edit_type {
                                EditEventMealsEditType::Recipe(_recipe) => todo!(),
                                EditEventMealsEditType::Location(_place) => todo!(),
                                EditEventMealsEditType::Servings(_servings) => todo!(),
                                EditEventMealsEditType::Calories(_calories) => todo!(),
                                EditEventMealsEditType::StartTime(_start_time) => todo!(),
                                EditEventMealsEditType::EndTime(_end_time) => todo!(),
                                EditEventMealsEditType::Comment(_comment) => todo!(),
                            }
                        },
                    },
                }
            },
        },
    }
}
