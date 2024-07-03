mod args;

use foodlib::*;
use humantime::parse_duration;
use sqlx::postgres::types::{PgInterval, PgMoney};
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
        Commands::UpdatePrices => {
            let result = food_base.fetch_metro_prices(None).await;
            if let Err(prices) = result {
                println!("Error: {}", prices);
            }
        }
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
                ListType::Places => {
                    let places = food_base.get_places().await.unwrap();
                    let table = Table::new(places).with(table_config).to_string();
                    print!("{}", table);
                }
                ListType::Events => {
                    let events = food_base.get_all_events().await.unwrap();
                    let table = Table::new(events).with(table_config).to_string();
                    print!("{}", table);
                }
                ListType::Ingredients => {
                    let ingredients = food_base.get_ingredients().await.unwrap();
                    let table = Table::new(ingredients).with(table_config).to_string();
                    print!("{}", table);
                }
                ListType::Recipes => {
                    let recipes = food_base.get_recipes().await.unwrap();
                    let table = Table::new(recipes).with(table_config).to_string();
                    print!("{}", table);
                }
                ListType::Meals => {
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
                ListType::Users => {
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
                ListType::ShoppingTours(shopping_data) => {
                    let event = food_base
                        .get_event_from_string_reference(shopping_data.event_ref.to_string())
                        .await;

                    if event.is_none() {
                        println!("Error: {:?}", event);
                    }
                    let event = event.unwrap();

                    let shopping_tours = food_base.get_event_shopping_tours(event.event_id).await;

                    if shopping_tours.is_err() {
                        println!("Error: {:?}", shopping_tours);
                    }
                    let shopping_tours = shopping_tours.unwrap();

                    if shopping_tours.len() == 0 {
                        println!("No Shopping Tours planned");
                    } else {
                        let table = (Table::new(shopping_tours).with(table_config)).to_string();
                        println!("{}", table);
                    }
                }
                ListType::SourceOverrides(override_data) => {
                    let event = food_base
                        .get_event_from_string_reference(override_data.event_ref.to_string())
                        .await;

                    if event.is_none() {
                        println!("Error: {:?}", event);
                    }
                    let event = event.unwrap();

                    let shopping_tours = food_base.get_event_source_overrides(event.event_id).await;

                    if shopping_tours.is_err() {
                        println!("Error: {:?}", shopping_tours);
                    }
                    let shopping_tours = shopping_tours.unwrap();

                    //TODO: List actual Sources and not just their IDs
                    let table = Table::new(shopping_tours).with(table_config).to_string();
                    println!("{}", table);
                }
                ListType::FoodPrep(ford_prep_data) => {
                    let event = food_base
                        .get_event_from_string_reference(ford_prep_data.event_ref.to_string())
                        .await;

                    if event.is_none() {
                        println!("Error: {:?}", event);
                    }
                    let event = event.unwrap();

                    let shopping_tours = food_base.get_event_food_prep(event.event_id).await;

                    if shopping_tours.is_err() {
                        println!("Error: {:?}", shopping_tours);
                    }
                    let shopping_tours = shopping_tours.unwrap();

                    let table = Table::new(shopping_tours).with(table_config).to_string();
                    println!("{}", table);
                }
            }
        }
        Commands::Info(show_statement) => {
            match &show_statement.show_type {
                InfoType::Ingredient(ingredient) => {
                    let ingredient_ref = ingredient.ingredient_ref.as_str();

                    let ingredient =
                        food_base.get_ingredient_from_string_reference(ingredient_ref.to_string());

                    if let Some(_ingredient) = ingredient.await {
                        // Possible information to be displayed:
                        // - Name. (ID) [Comment]
                        // - Price / Energy
                        // - List of Recipes using this ingredient
                        // - List of Events using recipes with this ingredient
                        todo!();
                    } else {
                        println!("Ingredient not found");
                    }
                }
                InfoType::Event(event) => {
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
                InfoType::Recipe(recipe) => {
                    let recipe_ref = recipe.recipe_ref.as_str();

                    let recipe = food_base
                        .get_recipe_from_string_reference(recipe_ref.to_string())
                        .await;

                    if let Some(_recipe) = recipe {
                        // Possible information to be displayed:
                        // - Name. (ID) [Comment]
                        // - Price / Energy
                        // - Price / 100g
                        // - List of Events using recipes with this ingredient
                        todo!();
                    } else {
                        println!("Recipe not found");
                    }
                }
                InfoType::Meal(meal) => {
                    let meals = food_base.get_event_meal(meal.meal_id).await;

                    match meals {
                        Ok(_) => todo!(),
                        Err(error) => {
                            println!("Error: {}", error);
                        }
                    }
                }
                InfoType::User(user) => {
                    let user_ref = user.user_ref.as_str();

                    let user = food_base
                        .get_user_by_string_reference(user_ref.to_string())
                        .await;

                    if let Some(user) = user {
                        println!("{} ({})", user.username, user.id);
                        println!("E-Mail: {}", user.email);
                        println!("Admin: {}", user.is_admin);
                    } else {
                        println!("User not found");
                    }
                }
            }
        }
        Commands::Calc(print_data) => {
            match &print_data.print_type {
                CalcType::Mealplan(event) => {
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
                CalcType::Meal(meal) => {
                    let _event_ref = meal.event_ref.as_str();
                    let _recipe_ref = meal.recipe_ref.as_str();
                    //let start_time_ref = meal.start_time.as_str();

                    //TODO Print Meal
                    println!("Printing Meal ");
                }
                CalcType::Recipe(recipe) => {
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
                        // TODO Fix PDF Export
                        // "latex" | "tex" => {
                        //     let recipe_info = food_base
                        //         .fetch_user_input_meal(
                        //             recipe_data.recipe_id,
                        //             people as f64,
                        //             calories,
                        //             "" as String,
                        //         )
                        //         .await
                        //         .unwrap();
                        //     let _pdf = foodlib::typst::export_recipes(recipe_info);
                        //     return "";
                        // }
                        "markdown" => food_base.format_subrecipes_markdown(subrecipes).await,
                        _ => "Unknown Format".to_string(),
                    };
                    println!("{}", output);
                }
                CalcType::ShoppingList(tour_id) => {
                    let shopping_list = food_base.get_shopping_list(tour_id.tour_id).await;

                    if let Ok(shopping_list) = shopping_list {
                        let table = Table::new(shopping_list).with(table_config).to_string();
                        println!("{}", table);
                    }
                }
            }
        }
        Commands::Add(add_data) => match &add_data.add_type {
            AddType::Ingredient(ingredient) => {
                let name = ingredient.name.as_str();
                let energy = ingredient.energy;
                let comment = ingredient.comment.as_str();

                match food_base
                    .add_ingredient(name.to_string(), energy.into(), Some(comment.to_string()))
                    .await
                {
                    Ok(_) => {
                        if cli.debug {
                            println!("Created Ingredient '{}'", name);
                        }
                    }
                    Err(error) => {
                        println!("Error: {}", error)
                    }
                }
            }
            AddType::Recipe(recipe) => {
                let name = recipe.name.as_str();
                let comment = recipe.comment.as_str();

                match food_base
                    .insert_recipe(&Recipe {
                        recipe_id: -1,
                        name: name.to_string(),
                        comment: Some(comment.to_string()),
                    })
                    .await
                {
                    Ok(_) => {
                        if cli.debug {
                            println!("Created Recipe '{}'", name);
                        }
                    }
                    Err(error) => {
                        println!("Error: {}", error)
                    }
                }
            }
            AddType::User(user) => {
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
                        if cli.debug {
                            println!("Created user '{}' (Admin: {})", user_name, is_admin);
                        }
                    }
                    Err(error) => {
                        println!("Error: {}", error)
                    }
                }
            }
            AddType::Event(event) => {
                let name = event.name.as_str();
                let comment = &event.comment;
                let budget = if let Some(budget) = &event.budget {
                    Some(
                        PgMoney::from_bigdecimal(budget.clone(), 2)
                            .expect("Failed to convert budget to money"),
                    )
                } else {
                    None
                };

                match food_base
                    .add_event(name.to_string(), budget, comment.clone())
                    .await
                {
                    Ok(_) => {
                        if cli.debug {
                            println!("Created Event '{}'", name);
                        }
                    }
                    Err(error) => {
                        println!("Error: {}", error)
                    }
                }
            }
        },
        Commands::Delete(delete_data) => match &delete_data.delete_type {
            DeleteType::Ingredient(ingredient) => {
                let ingredient_ref = ingredient.ingredient.as_str();

                let ingredient = food_base
                    .get_ingredient_from_string_reference(ingredient_ref.to_string())
                    .await;

                if let Some(_ingredient) = ingredient {
                    // There currently isn't a method for deleting an ingredient
                    todo!();
                } else {
                    println!("Ingredient not found");
                }
            }
            DeleteType::Recipe(recipe) => {
                let _recipe_ref = recipe.recipe.as_str();
                let recipe = food_base
                    .get_recipe_from_string_reference(_recipe_ref.to_string())
                    .await;

                if let Some(_recipe) = recipe {
                    let query = food_base.delete_recipe(_recipe.recipe_id);

                    match query.await {
                        Ok(_) => {
                            println!("Recipe {} removed", _recipe.name);
                        }
                        Err(error) => {
                            println!("Error: {}", error)
                        }
                    }
                } else {
                    println!("Recipe not found");
                }
            }
            DeleteType::User(user) => {
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
            DeleteType::Event(event) => {
                let _event_ref = event.event.as_str();
                let event = food_base
                    .get_event_from_string_reference(_event_ref.to_string())
                    .await;

                if let Some(event) = event {
                    // There currently isn't a method for deleting an event
                    let query = food_base.delete_event(event.event_id).await;
                    match query {
                        Ok(_) => {}
                        Err(error) => println!("Error: {}", error),
                    }
                } else {
                    println!("Event not found");
                }
            }
        },
        Commands::Edit(edit_data) => match &edit_data.edit_type {
            EditType::Ingredient(cli_ingredient) => {
                let ingredient_ref = cli_ingredient.ingredient.as_str();
                let ingredient = food_base
                    .get_ingredient_from_string_reference(ingredient_ref.to_string())
                    .await;

                let ingredient = if let Some(ingredient) = ingredient {
                    ingredient
                } else {
                    println!("Ingredient not found");
                    return;
                };
                match &cli_ingredient.edit_type {
                    EditIngredientType::Name(name) => {
                        let ingredient = &ingredient.change_name(name.name.clone());
                        let query = food_base.update_ingredient(ingredient);
                        match query.await {
                            Ok(_) => {
                                if cli.debug {
                                    println!("Updated Ingredient");
                                }
                            }
                            Err(error) => {
                                println!("Error: {}", error)
                            }
                        }
                    }
                    EditIngredientType::Energy(energy) => {
                        let ingredient = &ingredient.change_energy(energy.energy.clone());
                        let query = food_base.update_ingredient(ingredient);
                        match query.await {
                            Ok(_) => {
                                if cli.debug {
                                    println!("Updated Ingredient");
                                }
                            }
                            Err(error) => {
                                println!("Error: {}", error)
                            }
                        }
                    }
                    EditIngredientType::Comment(comment) => {
                        let ingredient = &ingredient.change_comment(comment.comment.clone());
                        let query = food_base.update_ingredient(ingredient);
                        match query.await {
                            Ok(_) => {
                                if cli.debug {
                                    println!("Updated Ingredient");
                                }
                            }
                            Err(error) => {
                                println!("Error: {}", error)
                            }
                        }
                    }
                }
            }
            EditType::Recipe(cli_recipe) => {
                let recipe_ref = cli_recipe.recipe.as_str();

                let recipe = food_base
                    .get_recipe_from_string_reference(recipe_ref.to_string())
                    .await;

                let recipe = if let Some(recipe) = recipe {
                    recipe
                } else {
                    println!("Couldn't find Recipe by reference \"{}\"", recipe_ref);
                    return;
                };
                match &cli_recipe.edit_type {
                    EditRecipeType::Name(name) => {
                        let recipe = Recipe {
                            name: name.name.clone(),
                            ..recipe
                        };

                        let query = food_base.update_recipe(&recipe);
                        match query.await {
                            Ok(_) => {
                                if cli.debug {
                                    println!(
                                        "Updated Recipe name ({} => {})",
                                        recipe.name, name.name
                                    );
                                }
                            }
                            Err(error) => {
                                println!("Error: {}", error)
                            }
                        }
                    }
                    EditRecipeType::Comment(comment) => {
                        let recipe = Recipe {
                            comment: comment.comment.clone(),
                            ..recipe
                        };

                        let query = food_base.update_recipe(&recipe);
                        match query.await {
                            Ok(_) => {
                                if cli.debug {
                                    println!(
                                        "Updated Recipe comment ('{}' => '{}')",
                                        recipe.comment.unwrap_or_default(),
                                        comment.comment.clone().unwrap_or_default()
                                    );
                                }
                            }
                            Err(error) => {
                                println!("Error: {}", error)
                            }
                        }
                    }
                    EditRecipeType::Ingredients(ingredient) => {
                        // Ask Dennis on how to implement this
                        match &ingredient.ingredient_edit_type {
                            EditRecipeIngredientsType::Add(cli_ingredient) => {
                                let amount = cli_ingredient.amount.as_str();
                                let amount = if let Some(amount_tuple) = parse_package_size(amount)
                                {
                                    amount_tuple
                                } else {
                                    println!("Error: Invalid amount");
                                    return;
                                };

                                let ingredient_ref = cli_ingredient.ingredient.as_str();
                                let ingredient = food_base.get_ingredient_from_string_reference(
                                    ingredient_ref.to_string(),
                                );
                                let ingredient = if let Some(ingredient) = ingredient.await {
                                    ingredient
                                } else {
                                    println!("Ingredient not found");
                                    return;
                                };

                                let query = food_base.add_recipe_ingredient(
                                    recipe.recipe_id,
                                    ingredient.ingredient_id,
                                    amount.0,
                                    amount.1,
                                );

                                match query.await {
                                    Ok(_) => {
                                        if cli.debug {
                                            println!(
                                                "Added {} of {} to Recipe",
                                                cli_ingredient.amount.as_str(),
                                                ingredient.name
                                            );
                                        }
                                    }
                                    Err(error) => {
                                        println!("Error: {}", error)
                                    }
                                };
                            }
                            EditRecipeIngredientsType::Remove(ingredient) => {
                                let ingredient_ref = ingredient.ingredient.as_str();
                                let ingredient = food_base.get_ingredient_from_string_reference(
                                    ingredient_ref.to_string(),
                                );
                                let ingredient = if let Some(ingredient) = ingredient.await {
                                    ingredient
                                } else {
                                    println!("Ingredient not found");
                                    return;
                                };

                                let query = food_base.delete_recipe_ingredient(
                                    recipe.recipe_id,
                                    ingredient.ingredient_id,
                                );

                                match query.await {
                                    Ok(_) => {
                                        if cli.debug {
                                            println!(
                                                "Removed Ingredient \"{}\" from Recipe",
                                                ingredient.name
                                            );
                                        }
                                    }
                                    Err(error) => {
                                        println!("Error: {}", error)
                                    }
                                };
                            }
                            EditRecipeIngredientsType::Amount(cli_ingredient) => {
                                let amount = cli_ingredient.amount.as_str();
                                let amount = if let Some(amount_tuple) = parse_package_size(amount)
                                {
                                    amount_tuple
                                } else {
                                    println!("Error: Invalid amount");
                                    return;
                                };

                                let ingredient_ref = cli_ingredient.ingredient.as_str();
                                let ingredient = food_base.get_ingredient_from_string_reference(
                                    ingredient_ref.to_string(),
                                );
                                let ingredient = if let Some(ingredient) = ingredient.await {
                                    ingredient
                                } else {
                                    println!("Ingredient not found");
                                    return;
                                };

                                let query = food_base.update_recipe_ingredient(
                                    recipe.recipe_id,
                                    ingredient.ingredient_id,
                                    amount.0,
                                    amount.1,
                                );

                                match query.await {
                                    Ok(_) => {
                                        if cli.debug {
                                            println!(
                                                "Updated Amount for {} to {}",
                                                ingredient.name,
                                                cli_ingredient.amount.as_str()
                                            );
                                        }
                                    }
                                    Err(error) => {
                                        println!("Error: {}", error)
                                    }
                                };
                            }
                        }
                    }
                    EditRecipeType::Steps(steps) => {
                        let new_steps = food_base.get_recipe_steps(recipe.recipe_id).await;
                        let mut new_steps: Vec<RecipeStep> = if let Ok(steps) = new_steps {
                            steps
                        } else {
                            println!("Error: {}", new_steps.unwrap_err());
                            return;
                        };
                        match &steps.step_edit_type {
                            EditRecipeStepsType::Add(step) => {
                                let name = step.name.as_str();
                                let duration_fixed = PgInterval {
                                    microseconds: parse_duration(step.fixed_time.as_str())
                                        .unwrap()
                                        .as_micros()
                                        as i64,
                                    days: 0,
                                    months: 0,
                                };
                                let duration_scaled = PgInterval {
                                    microseconds: parse_duration(step.scaled_time.as_str())
                                        .unwrap()
                                        .as_micros()
                                        as i64,
                                    days: 0,
                                    months: 0,
                                };

                                let index = step.position;
                                let index = if let Some(index) = index {
                                    index as f64 - 1.
                                } else {
                                    new_steps.len() as f64
                                };

                                let added_step = RecipeStep {
                                    step_id: 0,
                                    step_order: index,
                                    step_name: name.to_string(),
                                    step_description: step.description.clone(),
                                    fixed_duration: duration_fixed,
                                    duration_per_kg: duration_scaled,
                                    recipe_id: 0,
                                };

                                new_steps = new_steps
                                    .into_iter()
                                    .map(|step| {
                                        if step.step_order >= index {
                                            RecipeStep {
                                                step_order: step.step_order + 1.,
                                                ..step
                                            }
                                        } else {
                                            step
                                        }
                                    })
                                    .collect::<Vec<_>>();

                                new_steps.push(added_step);
                            }
                            EditRecipeStepsType::Remove(_step) => {
                                let step_ref = _step.step.as_str();
                                let step_name = step_ref;
                                let step_order = step_ref.parse::<f64>().unwrap_or(-1.) - 1.;

                                let original_step = new_steps.iter().find(|step| {
                                    step.step_order == step_order || step.step_name == step_name
                                });

                                let original_step = if let Some(step) = original_step {
                                    step.clone()
                                } else {
                                    println!("Error: Step not found");
                                    return;
                                };

                                new_steps = new_steps
                                    .into_iter()
                                    .filter(|step| step.step_id != original_step.step_id)
                                    .enumerate()
                                    .map(|(i, step)| RecipeStep {
                                        step_order: i as f64,
                                        ..step
                                    })
                                    .collect::<Vec<_>>();
                            }
                            EditRecipeStepsType::Reorder(step) => {
                                let order = step.order.clone();
                                if new_steps.len() != order.len() {
                                    println!("Error: New Order must have the same length as the old one ({} != {})", order.len(), new_steps.len());
                                    return;
                                }

                                // Check if every index is contained in the new order
                                for i in 0..order.len() {
                                    if !order.contains(&(i as u32)) {
                                        println!("Error: New Order must contain every index ({} is missing)", i);
                                        return;
                                    }
                                }

                                // Build new order of Steps
                                new_steps = new_steps
                                    .into_iter()
                                    .enumerate()
                                    .map(|(i, step)| RecipeStep {
                                        step_order: order
                                            .iter()
                                            .position(|&x| x == i as u32)
                                            .unwrap()
                                            as f64,
                                        ..step
                                    })
                                    .collect::<Vec<_>>();
                            }
                            EditRecipeStepsType::Edit(step) => {
                                let step_ref = step.step.as_str();
                                let step_name = step_ref;
                                let step_order = step_ref.parse::<f64>().unwrap_or(-1.);

                                let original_step = new_steps.iter().find(|step| {
                                    step.step_order == step_order || step.step_name == step_name
                                });

                                let mut edited_step = if let Some(step) = original_step {
                                    step.clone()
                                } else {
                                    println!("Error: Step not found");
                                    return;
                                };
                                let original_step = edited_step.clone();

                                match &step.edit_type {
                                    EditRecipeStepsEditType::Name(name) => {
                                        edited_step.step_name = name.name.clone();
                                    }
                                    EditRecipeStepsEditType::Description(desc) => {
                                        edited_step.step_description = desc.description.clone();
                                    }
                                    EditRecipeStepsEditType::Duration(duration) => {
                                        let parsed_duration =
                                            parse_duration(duration.duration.as_str());
                                        let parsed_duration = if let Ok(duration) = parsed_duration
                                        {
                                            PgInterval {
                                                microseconds: duration.as_micros() as i64,
                                                days: 0,
                                                months: 0,
                                            }
                                        } else {
                                            println!("Error: {}", parsed_duration.unwrap_err());
                                            return;
                                        };

                                        match duration.duration_type {
                                            EditRecipeStepsEditDurationType::Fixed => {
                                                edited_step.fixed_duration = parsed_duration;
                                            }
                                            EditRecipeStepsEditDurationType::Scaled => {
                                                edited_step.duration_per_kg = parsed_duration;
                                            }
                                        }
                                    }
                                }

                                new_steps = new_steps
                                    .iter()
                                    .map(|step| {
                                        if step.step_id == original_step.step_id {
                                            edited_step.clone()
                                        } else {
                                            step.clone()
                                        }
                                    })
                                    .collect::<Vec<_>>();
                            }
                        }
                        let query =
                            food_base.update_recipe_steps(recipe.recipe_id, new_steps.into_iter());

                        match query.await {
                            Ok(_) => {
                                if cli.debug {
                                    println!("Updated Recipe Steps");
                                }
                            }
                            Err(error) => {
                                println!("Error: {}", error)
                            }
                        }
                    }
                }
            }
            EditType::User(cli_user) => {
                let _user_ref = cli_user.user.as_str();
                let user = food_base.get_user_by_string_reference(_user_ref.to_string());

                let user = if let Some(user) = user.await {
                    user
                } else {
                    println!("User not found");
                    return;
                };

                match &cli_user.edit_type {
                    EditUserType::Name(name) => {
                        let query = food_base.change_username(user.id, name.username.clone());
                        match query.await {
                            Ok(_) => {
                                if cli.debug {
                                    println!(
                                        "Updated User ({} => {})",
                                        user.username, name.username
                                    );
                                }
                            }
                            Err(error) => {
                                println!("Error: {}", error)
                            }
                        }
                    }
                    EditUserType::Password(password) => {
                        let query = food_base.update_password(user.id, password.password.clone());
                        match query.await {
                            Ok(_) => {
                                if cli.debug {
                                    println!("Updated User Password");
                                }
                            }
                            Err(error) => {
                                println!("Error: {}", error)
                            }
                        }
                    }
                    EditUserType::Email(email) => {
                        let query = food_base.change_email(user.id, email.email.clone());
                        match query.await {
                            Ok(_) => {
                                if cli.debug {
                                    println!(
                                        "Updated User E-Mail ({} => {})",
                                        user.email, email.email
                                    );
                                }
                            }
                            Err(error) => {
                                println!("Error: {}", error)
                            }
                        }
                    }
                    EditUserType::Promote => {
                        let query = food_base.change_is_admin(user.id, true);
                        match query.await {
                            Ok(_) => {
                                if user.is_admin {
                                    println!("Warning: User {} is already an Admin", user.username);
                                }
                                if cli.debug {
                                    println!("Promoted User {}", user.username);
                                }
                            }
                            Err(error) => {
                                println!("Error: {}", error)
                            }
                        }
                    }
                    EditUserType::Demote => {
                        let query = food_base.change_is_admin(user.id, false);
                        match query.await {
                            Ok(_) => {
                                if !user.is_admin {
                                    println!(
                                        "Warning: User {} wasn't an Admin in the first place",
                                        user.username
                                    );
                                }
                                if cli.debug {
                                    println!("Promoted User {}", user.username);
                                }
                            }
                            Err(error) => {
                                println!("Error: {}", error)
                            }
                        }
                    }
                }
            }
            EditType::Event(cli_event) => {
                let _event_ref = cli_event.event.as_str();
                let event_id = food_base
                    .get_event_from_string_reference(_event_ref.to_string())
                    .await;

                let event = if let Some(event) = event_id {
                    event
                } else {
                    println!("Event not found");
                    return;
                };
                match &cli_event.edit_type {
                    EditEventType::Name(name) => {
                        let event = Event {
                            event_name: name.name.clone(),
                            ..event
                        };
                        let query = food_base.update_event(&event).await;
                        match query {
                            Ok(_) => {
                                if cli.debug {
                                    println!(
                                        "Updated Event name ({} => {})",
                                        event.event_name, name.name
                                    );
                                }
                            }
                            Err(error) => {
                                println!("Error: {}", error)
                            }
                        }
                    }
                    EditEventType::Budget(budget) => {
                        let budget = if let Some(budget) = &budget.budget {
                            Some(
                                PgMoney::from_bigdecimal(budget.clone(), 2)
                                    .expect("Failed to convert budget to money"),
                            )
                        } else {
                            None
                        };
                        let event = Event {
                            budget: budget.clone(),
                            ..event
                        };

                        let query = food_base.update_event(&event).await;
                        match query {
                            Ok(_) => {
                                if cli.debug {
                                    println!(
                                        "Updated Event budget ({:?} => {:?})",
                                        event.budget, budget
                                    );
                                }
                            }
                            Err(error) => {
                                println!("Error: {}", error)
                            }
                        }
                    }
                    EditEventType::Comment(comment) => {
                        let event = Event {
                            comment: comment.comment.clone(),
                            ..event
                        };
                        let query = food_base.update_event(&event).await;
                        match query {
                            Ok(_) => {
                                if cli.debug {
                                    println!(
                                        "Updated Event comment ('{}' => '{}')",
                                        event.comment.unwrap_or_default(),
                                        comment.comment.clone().unwrap_or_default()
                                    );
                                }
                            }
                            Err(error) => {
                                println!("Error: {}", error)
                            }
                        }
                    }
                    EditEventType::Meals(meals) => match &meals.meal_edit_type {
                        EditEventMealsType::Add(meal) => {
                            let recipe_ref = meal.recipe.as_str();
                            let recipe = food_base
                                .get_recipe_from_string_reference(recipe_ref.to_string())
                                .await;
                            let recipe = if let Some(recipe) = recipe {
                                recipe
                            } else {
                                println!("Recipe not found");
                                return;
                            };

                            let servings = meal.servings;
                            let calories = meal.calories;
                            let start_time = meal.start_time;
                            let end_time = meal.end_time;
                            let place_id = meal.location;

                            let query = food_base.add_meal(
                                event.event_id,
                                recipe.recipe_id,
                                place_id,
                                start_time,
                                end_time,
                                calories.into(),
                                servings,
                                meal.comment.clone(),
                            );

                            match query.await {
                                Ok(_) => {
                                    if cli.debug {
                                        println!("Added Meal");
                                    }
                                }
                                Err(error) => {
                                    println!("Error: {}", error)
                                }
                            }
                        }
                        EditEventMealsType::Remove(meal) => {
                            let query = food_base.remove_meal(meal.meal_id);
                            match query.await {
                                Ok(_) => {
                                    if cli.debug {
                                        println!("Removed Meal");
                                    }
                                }
                                Err(error) => {
                                    println!("Error: {}", error)
                                }
                            }
                        }
                        EditEventMealsType::Edit(meal) => {
                            let meal_id = meal.meal_id;

                            // Check if Meal exists
                            if let Ok(meal) = food_base.get_event_meal(meal_id).await {
                                println!("Current State: \n");
                                if meal.event_id != event.event_id {
                                    println!("The meal you are trying to edit does not belong to the event you ame editing. Maby check the given meal/event ID");
                                    return;
                                }
                            }
                        }
                    },
                    EditEventType::Shopping(shopping_data) => match &shopping_data.edit_type {
                        EditEventShoppingType::Add(add_data) => match &add_data.edit_type {
                            EditEventShoppingAddType::Tour(tour_data) => {
                                let _ = food_base
                                    .add_event_shopping_tour(
                                        event.event_id,
                                        tour_data.store,
                                        tour_data.date,
                                    )
                                    .await;
                            }
                            EditEventShoppingAddType::SourceOverride(override_data) => {
                                let _ = food_base
                                    .add_event_source_override(
                                        event.event_id,
                                        override_data.source_id,
                                    )
                                    .await;
                            }
                            EditEventShoppingAddType::FoodPrep(prep_data) => {
                                let recipe = food_base
                                    .get_recipe_from_string_reference(prep_data.recipe_ref.clone())
                                    .await;
                                if recipe.is_none() {
                                    println!("Couldn't find Recipe");
                                    return;
                                }
                                let recipe = recipe.unwrap();

                                let _ = food_base
                                    .add_event_food_prep(
                                        event.event_id,
                                        recipe.recipe_id.clone(),
                                        prep_data.prep_date,
                                        prep_data.use_start_date,
                                        prep_data.use_end_date,
                                    )
                                    .await;
                            }
                        },
                        EditEventShoppingType::Delete(delete_data) => {
                            match &delete_data.edit_type {
                                EditEventShoppingDeleteType::Tour(tour_id) => {
                                    let _ =
                                        food_base.delete_event_shopping_tour(tour_id.tour_id).await;
                                }
                                EditEventShoppingDeleteType::SourceOverride(source_id) => {
                                    let ingredient = food_base
                                        .get_ingredient_from_string_reference(
                                            source_id.ingredient_id.clone(),
                                        )
                                        .await;
                                    if ingredient.is_none() {
                                        println!("Could not find Ingredient");
                                        return;
                                    }
                                    let ingredient = ingredient.unwrap();
                                    let _ = food_base
                                        .delete_event_source_override(ingredient.ingredient_id);
                                }
                                EditEventShoppingDeleteType::FoodPrep(prep_id) => {
                                    let _ = food_base.delete_event_food_prep(prep_id.prep_id).await;
                                }
                            }
                        }
                        EditEventShoppingType::Edit(edit_data) => match &edit_data.edit_type {
                            EditEventShoppingEditType::Tour(tour_data) => {
                                let tour_id = tour_data.tour_id;

                                if let Some(date) = tour_data.date {
                                    let _ = food_base
                                        .update_event_shopping_tour_date(tour_id, date)
                                        .await;
                                    println!("Updated tour date");
                                }

                                if let Some(store) = &tour_data.store {
                                    let store =
                                        food_base.get_store_by_ref((&store).to_string()).await;
                                    if store.is_ok() {
                                        let store = store.unwrap();

                                        let _ = food_base
                                            .update_event_shopping_tour_store(
                                                tour_id,
                                                store.store_id,
                                            )
                                            .await;

                                        println!("Updated tour destination")
                                    } else {
                                        println!("Could not find Store");
                                    }
                                }
                            }
                            EditEventShoppingEditType::SourceOverride(source_edit_data) => {
                                let _ = food_base
                                    .update_event_ingredient_source_override(
                                        event.event_id,
                                        source_edit_data.old_source_id,
                                        source_edit_data.new_source_id,
                                    )
                                    .await;
                            }
                            EditEventShoppingEditType::FoodPrep(prep_edit_data) => {
                                let prep_id = prep_edit_data.prep_id;

                                if let Some(recipe_ref) = prep_edit_data.recipe.clone() {
                                    let recipe_query = food_base
                                        .get_recipe_from_string_reference(recipe_ref)
                                        .await;
                                    if let Some(recipe) = recipe_query {
                                        let _ = food_base.update_event_food_prep_recipe_id(
                                            prep_id,
                                            recipe.recipe_id,
                                        );
                                    } else {
                                        println!("Could not find Recipe")
                                    }
                                }

                                if let Some(prep_date) = prep_edit_data.prep_date.clone() {
                                    let _ = food_base
                                        .update_event_food_prep_prep_date(prep_id, prep_date)
                                        .await;
                                }

                                if let Some(use_from) = prep_edit_data.start {
                                    let _ = food_base
                                        .update_event_food_prep_use_from(prep_id, use_from)
                                        .await;
                                }

                                if let Some(use_until) = prep_edit_data.end {
                                    let _ = food_base
                                        .update_event_food_prep_use_until(prep_id, use_until)
                                        .await;
                                }
                            }
                        },
                    },
                }
            }
        },
    }
}
