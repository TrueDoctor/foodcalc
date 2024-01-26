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
        Commands::UpdatePrices => todo!(),
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
                        Ok(meals) => match meals.len() {
                            0 => {
                                println!("No Meals found");
                            }
                            1 => {
                                todo!();
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
                        },
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
                        "latex" | "tex" => food_base.format_subrecipes_latex(subrecipes).await,
                        "markdown" => food_base.format_subrecipes_markdown(subrecipes).await,
                        _ => "Unknown Format".to_string(),
                    };
                    println!("{}", output);
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

                if let Some(_event) = event {
                    // There currently isn't a method for deleting an event
                    todo!();
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
                        let query =
                            food_base.update_ingredient(ingredient.change_name(name.name.clone()));
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
                        let query = food_base
                            .update_ingredient(ingredient.change_energy(energy.energy.clone()));
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
                        let query = food_base
                            .update_ingredient(ingredient.change_comment(comment.comment.clone()));
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
                                let amount = if let Some(amount_tuple) = parse_package_size(amount) {
                                    amount_tuple
                                } else {
                                    println!("Error: Invalid amount");
                                    return;
                                };

                                let ingredient_ref = cli_ingredient.ingredient.as_str();
                                let ingredient = food_base
                                    .get_ingredient_from_string_reference(
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
                                    amount,
                                );

                                match query.await {
                                    Ok(_) => {
                                        if cli.debug {
                                            println!("Added {} of {} to Recipe", cli_ingredient.amount.as_str(), ingredient.name);
                                        }
                                    }
                                    Err(error) => {
                                        println!("Error: {}", error)
                                    }
                                };
                            },
                            EditRecipeIngredientsType::Remove(ingredient) => {
                                let ingredient_ref = ingredient.ingredient.as_str();
                                let ingredient = food_base
                                    .get_ingredient_from_string_reference(
                                        ingredient_ref.to_string(),
                                    );
                                let ingredient = if let Some(ingredient) = ingredient.await {
                                    ingredient
                                } else {
                                    println!("Ingredient not found");
                                    return;
                                };

                                let query = food_base.remove_recipe_ingredient(
                                    recipe.recipe_id,
                                    ingredient.ingredient_id,
                                );

                                match query.await {
                                    Ok(_) => {
                                        if cli.debug {
                                            println!("Removed Ingredient \"{}\" from Recipe", ingredient.name);
                                        }
                                    }
                                    Err(error) => {
                                        println!("Error: {}", error)
                                    }
                                };
                            },
                            EditRecipeIngredientsType::Amount(cli_ingredient) => {
                                let amount = cli_ingredient.amount.as_str();
                                let amount = if let Some(amount_tuple) = parse_package_size(amount) {
                                    amount_tuple
                                } else {
                                    println!("Error: Invalid amount");
                                    return;
                                };

                                let ingredient_ref = cli_ingredient.ingredient.as_str();
                                let ingredient = food_base
                                    .get_ingredient_from_string_reference(
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
                                    amount,
                                );

                                match query.await {
                                    Ok(_) => {
                                        if cli.debug {
                                            println!("Updated Amount for {} to {}", ingredient.name, cli_ingredient.amount.as_str());
                                        }
                                    }
                                    Err(error) => {
                                        println!("Error: {}", error)
                                    }
                                };
                            },
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
                                    .map(|(i, step)| {
                                        RecipeStep {
                                            step_order: i as f64,
                                            ..step
                                        }
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
                let event = food_base
                    .get_event_from_string_reference(_event_ref.to_string())
                    .await;

                let event = if let Some(event) = event {
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
                        }
                    },
                }
            }
        },
    }
}
