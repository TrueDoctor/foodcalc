use clap::{Args, Parser, Subcommand};
use std::str::FromStr;

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,

    #[arg(short, long)]
    /// Enable Debug Output
    pub debug: bool,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// List all Places, Events, Ingredients, Recipes or Meals
    List(ListCommand),

    /// Get info about an Ingredient, Event, Recipe or Meal
    Info(InfoCommand),

    /// Calculate meals
    Calc(CalcCommand),

    /// Add a new Ingredient, Recipe, User or Event
    Add(AddCommand),

    #[clap(alias = "rm")]
    #[clap(alias = "del")]
    /// Delete an Ingredient, Recipe, User or Event
    Delete(DeleteCommand),
}

// ---- List Commands ----
#[derive(Debug, Args)]
pub struct ListCommand {
    pub list_type: ListTypes,

    #[clap(short = 'p', long = "place")]
    /// Place to reference (use ID or name)
    pub place: Option<String>,
    #[clap(short = 'e', long = "event")]
    /// Event to reference (use ID or name)
    pub event: Option<String>,

    #[clap(short = 'i', long = "ingredient")]
    /// Ingredient to reference (use ID or name)
    pub ingredient: Option<String>,

    #[clap(short = 'r', long = "recipe")]
    /// Recipe to reference (use ID or name)
    pub recipe: Option<String>,

    #[clap(short = 'm', long = "meal")]
    /// Meal to reference (use ID or name)
    pub meal: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ListTypes {
    /// List all places
    Places,

    /// List all events
    Events,

    /// List all ingredients
    Ingredients,

    /// List all reciepes
    Recipes,

    /// List all meals
    Meals,

    /// List all users
    Users,
}

impl FromStr for ListTypes {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "places" => Ok(ListTypes::Places),
            "events" => Ok(ListTypes::Events),
            "ingredients" => Ok(ListTypes::Ingredients),
            "recipes" => Ok(ListTypes::Recipes),
            "meals" => Ok(ListTypes::Meals),
            "users" => Ok(ListTypes::Users),
            _ => Err(format!("Unknown List Type: {}", s)),
        }
    }
}

// ---- Info Commands ----
#[derive(Debug, Args)]
pub struct InfoCommand {
    #[clap(subcommand)]
    pub show_type: InfoCommands,
}

#[derive(Debug, Subcommand)]
pub enum InfoCommands {
    /// Get info about an Ingredient
    Ingredient(InfoIngredient),

    /// Get info about an Event
    Event(InfoEvent),

    /// Get info about a Recipe
    Recipe(InfoRecipe),

    /// Get info about a Meal
    Meal(InfoMeal),
}

#[derive(Debug, Args)]
pub struct InfoIngredient {
    /// Recipe to show (use ID or name)
    pub ingredient_ref: String,
}

#[derive(Debug, Args)]
pub struct InfoEvent {
    /// Event to show (use ID or name)
    pub event_ref: String,
}

#[derive(Debug, Args)]
pub struct InfoRecipe {
    /// Recipe to show (use ID or name)
    pub recipe_ref: String,
}

#[derive(Debug, Args)]
pub struct InfoMeal {
    /// Event to show (use ID or name)
    pub event_ref: String,

    /// Recipe to show (use ID or name)
    pub recipe_ref: String,

    /// Start time - only needed if a recipe is cooked multiple times in one event
    pub start_time: Option<String>,
}

// ---- Calc Commands ----
#[derive(Debug, Args)]
pub struct CalcCommand {
    #[clap(subcommand)]
    pub print_type: CalcCommands,

    #[arg(short, long)]
    // The Output File
    pub output_file: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum CalcCommands {
    #[clap(alias = "event")]
    /// Print the mealplan for a given event
    Mealplan(CalcMealplan),

    #[clap(alias = "meals")]
    /// Print the recipe for a given meal or for all meals in an event
    Meal(CalcMeal),

    /// Print a given Recipe
    Recipe(CalcRecipe),
}

#[derive(Debug, Args)]
pub struct CalcMealplan {
    /// Event to print (use ID or name)
    pub event_ref: String,
}

#[derive(Debug, Args)]
pub struct CalcMeal {
    /// Event to show (use ID or name)
    pub event_ref: String,

    /// Recipe to show (use ID or name)
    pub recipe_ref: String,

    /// Start time - only needed if a recipe is cooked multiple times in one event
    pub start_time: Option<String>,
}

#[derive(Debug, Args)]
pub struct CalcRecipe {
    /// Recipe to print (use ID or name)
    pub recipe: String,

    /// Number of people to cook for
    #[clap(default_value = "1")]
    pub people: u32,

    /// Number of calories per serving
    #[clap(default_value = "2400")]
    pub calories: u32,

    #[arg(short, long, default_value = "markdown")]
    // The Output Format
    pub format: String,
}

// ----- Add Commands -----
#[derive(Debug, Args)]
pub struct AddCommand {
    #[clap(subcommand)]
    pub add_type: AddCommands,
}

pub enum AddCommands {
    /// Add a new Ingredient
    Ingredient(AddIngredient),

    /// Add a new Recipe
    Recipe(AddRecipe),

    /// Add a new User
    User(AddUser),

    /// Add a new Event
    Event(AddEvent),
}

#[derive(Debug, Args)]
pub struct AddIngredient {
    /// Name of the new ingredient
    pub name: String,

    /// Calories per 100g
    pub energy: u32,

    #[clap(default_value = "")]
    /// Add comments
    pub comment: String,
}

#[derive(Debug, Args)]
pub struct AddRecipe {
    /// Name of the new recipe
    pub name: String,

    #[clap(default_value = "")]
    /// Add comments
    pub comment: String,
}

#[derive(Debug, Args)]
pub struct AddUser {
    /// Username of the new user
    pub user: String,

    /// Password of the new user
    pub password: String,

    /// Email of the new user
    pub email: String,

    #[arg(short, long)]
    /// Give the user admin permissions
    pub admin: bool,
}

#[derive(Debug, Args)]
pub struct AddEvent {
    /// Name of the new event
    pub name: String,

    /// Buget for the event
    pub budget: u32,

    #[clap(default_value = "")]
    /// Add comments
    pub comment: String,
}

//// --- Delete Commands ----
#[derive(Debug, Args)]
pub struct DeleteCommand {
    #[clap(subcommand)]
    pub delete_type: DeleteCommands,
}

#[derive(Debug, Subcommand)]
pub enum DeleteCommands {
    /// Delete an Ingredient
    Ingredient(DeleteIngredient),

    /// Delete a Recipe
    Recipe(DeleteRecipe),

    /// Delete a User
    User(DeleteUser),

    /// Delete an Event
    Event(DeleteEvent),
}

#[derive(Debug, Args)]
pub struct DeleteIngredient {
    /// Ingredient to delete (use ID or name)
    pub ingredient: String,
}

#[derive(Debug, Args)]
pub struct DeleteRecipe {
    /// Recipe to delete (use ID or name)
    pub recipe: String,
}

#[derive(Debug, Args)]
pub struct DeleteUser {
    /// User to delete (use ID or name)
    pub user: String,
}

#[derive(Debug, Args)]
pub struct DeleteEvent {
    /// Event to delete (use ID or name)
    pub event: String,
}

/// ---- Edit Commands ----
#[derive(Debug, Args)]
pub struct EditCommand {
    #[clap(subcommand)]
    pub edit_type: EditCommands,
}

#[derive(Debug, Subcommand)]
pub enum EditCommands {
    /// Edit an Ingredient
    Ingredient(EditIngredient),

    /// Edit a Recipe
    Recipe(EditRecipe),

    /// Edit a User
    User(EditUser),

    /// Edit an Event
    Event(EditEvent),
}

//### Ingredient
#[derive(Debug, Args)]
pub struct EditIngredient {
    /// Ingredient to edit (use ID or name)
    pub ingredient: String,

    #[clap(subcommand)]
    pub edit_type: EditIngredientType,
}

#[derive(Debug, Subcommand)]
pub enum EditIngredientType {
    /// Edit the name of an Ingredient
    Name(EditIngredientName),

    /// Edit the energy of an Ingredient
    Energy(EditIngredientEnergy),

    /// Edit the comment of an Ingredient
    Comment(EditIngredientComment),
}

#[derive(Debug, Args)]
pub struct EditIngredientName {
    /// New name of the ingredient
    pub name: String,
}

#[derive(Debug, Args)]
pub struct EditIngredientEnergy {
    /// New energy of the ingredient
    pub energy: u32,
}

#[derive(Debug, Args)]
pub struct EditIngredientComment {
    /// New comment of the ingredient
    #[clap(default_value = "")]
    pub comment: String,
}

//### Recipe
#[derive(Debug, Args)]
pub struct EditRecipe {
    /// Recipe to edit (use ID or name)
    pub recipe: String,

    #[clap(subcommand)]
    pub edit_type: EditRecipeType,
}

#[derive(Debug, Subcommand)]
pub enum EditRecipeType {
    /// Edit the name of a Recipe
    Name(EditRecipeName),

    /// Edit the comment of a Recipe
    Comment(EditRecipeComment),

    /// Edit the ingredients of a Recipe
    Ingredients(EditRecipeIngredients),

    /// Edit the steps of a Recipe
    Steps(EditRecipeSteps),
}

#[derive(Debug, Args)]
pub struct EditRecipeName {
    /// New name of the recipe
    pub name: String,
}

#[derive(Debug, Args)]
pub struct EditRecipeComment {
    /// New comment of the recipe
    #[clap(default_value = "")]
    pub comment: String,
}

#[derive(Debug, Args)]
pub struct EditRecipeIngredients {
    /// New ingredients of the recipe
    #[clap(subcommand)]
    pub ingredient_edit_type: EditRecipeIngredientsType,
}

#[derive(Debug, Subcommand)]
pub enum EditRecipeIngredientsType {
    /// Add an ingredient to the recipe
    Add(EditRecipeIngredientsAdd),

    /// Remove an ingredient from the recipe
    Remove(EditRecipeIngredientsRemove),

    /// Change the amount of an ingredient in the recipe
    Amount(EditRecipeIngredientsAmount),
}

#[derive(Debug, Args)]
pub struct EditRecipeIngredientsAdd {
    /// Ingredient to add (use ID or name)
    pub ingredient: String,

    /// Amount of the ingredient
    pub amount: u32,
}

#[derive(Debug, Args)]
pub struct EditRecipeIngredientsRemove {
    /// Ingredient to remove (use ID or name)
    pub ingredient: String,
}

#[derive(Debug, Args)]
pub struct EditRecipeIngredientsAmount {
    /// Ingredient to edit (use ID or name)
    pub ingredient: String,

    /// New amount of the ingredient
    pub amount: u32,
}

#[derive(Debug, Args)]
pub struct EditRecipeSteps {
    /// New steps of the recipe
    #[clap(subcommand)]
    pub step_edit_type: EditRecipeStepsType,
}

#[derive(Debug, Subcommand)]
pub enum EditRecipeStepsType {
    /// Add a step to the recipe
    Add(EditRecipeStepsAdd),

    /// Remove a step from the recipe
    Remove(EditRecipeStepsRemove),

    /// Reorder the steps of the recipe
    Reorder(EditRecipeStepsReorder),

    /// Edit a Step of the recipe
    Edit(EditRecipeStepsEdit),
}

#[derive(Debug, Args)]
pub struct EditRecipeStepsAdd {
    /// Name of the step
    pub name: String,

    #[clap(default_value = "")]
    /// Description of the step
    pub description: String,

    #[clap(default_value = "0 min")]
    /// Fixed amount of minutes the step takes
    pub fixed_time: String,

    #[clap(default_value = "0 min")]
    /// Amount of minutes per 1kg the step takes
    pub scaled_time: String,

    /// Position of the step in the recipe, by default the step is added to the end
    pub position: Option<u32>,
}

#[derive(Debug, Args)]
pub struct EditRecipeStepsRemove {
    /// Step to remove (use index or name)
    pub step: String,
}

#[derive(Debug, Args)]
pub struct EditRecipeStepsReorder {
    /// New order of the steps
    pub order: Vec<String>,
}

#[derive(Debug, Args)]
pub struct EditRecipeStepsEdit {
    /// Step to edit (use index or name)
    pub step: String,

    #[clap(subcommand)]
    pub edit_type: EditRecipeStepsEditType,
}

#[derive(Debug, Subcommand)]
pub enum EditRecipeStepsEditType {
    /// Edit the name of a step
    Name(EditRecipeStepsEditName),

    /// Edit the description of a step
    Description(EditRecipeStepsEditDescription),

    /// Edit the duration of a step
    Duration(EditRecipeStepsEditDuration),
}

#[derive(Debug, Args)]
pub struct EditRecipeStepsEditName {
    /// New name of the step
    pub name: String,
}

#[derive(Debug, Args)]
pub struct EditRecipeStepsEditDescription {
    /// New description of the step
    #[clap(default_value = "")]
    pub description: String,
}

#[derive(Debug, Args)]
pub struct EditRecipeStepsEditDuration {
    #[clap(subcommand)]
    pub duration_type: EditRecipeStepsEditDurationType,
}

#[derive(Debug, Subcommand)]
pub enum EditRecipeStepsEditDurationType {
    /// Edit the fixed duration of a step
    Fixed(EditRecipeStepsEditDurationFixed),

    /// Edit the scaled duration of a step
    Scaled(EditRecipeStepsEditDurationScaled),
}

#[derive(Debug, Args)]
pub struct EditRecipeStepsEditDurationFixed {
    /// New fixed duration of the step
    pub fixed_time: String,
}

#[derive(Debug, Args)]
pub struct EditRecipeStepsEditDurationScaled {
    /// New scaled duration of the step
    pub scaled_time: String,
}

//### User
#[derive(Debug, Args)]
pub struct EditUser {
    /// User to edit (use ID or name)
    pub user: String,

    #[clap(subcommand)]
    pub edit_type: EditUserType,
}

#[derive(Debug, Subcommand)]
pub enum EditUserType {
    /// Edit the username of a User
    Username(EditUserUsername),

    /// Edit the password of a User
    Password(EditUserPassword),

    /// Edit the email of a User
    Email(EditUserEmail),

    /// Edit the admin status of a User
    Admin(EditUserAdmin),
}

#[derive(Debug, Args)]
pub struct EditUserUsername {
    /// New username of the user
    pub username: String,
}

#[derive(Debug, Args)]
pub struct EditUserPassword {
    /// New password of the user
    pub password: String,
}

#[derive(Debug, Args)]
pub struct EditUserEmail {
    /// New email of the user
    pub email: String,
}

#[derive(Debug, Args)]
pub struct EditUserAdmin {
    #[clap(default_value = "true")]
    /// New admin status of the user
    pub admin: bool,
}

//### Event
#[derive(Debug, Args)]
pub struct EditEvent {
    /// Event to edit (use ID or name)
    pub event: String,

    #[clap(subcommand)]
    pub edit_type: EditEventType,
}

#[derive(Debug, Subcommand)]
pub enum EditEventType {
    /// Edit the name of an Event
    Name(EditEventName),

    /// Edit the comment of an Event
    Comment(EditEventComment),

    /// Edit the budget of an Event
    Budget(EditEventBudget),

    /// Meals of an Event
    Meals(EditEventMeals),
}

#[derive(Debug, Args)]
pub struct EditEventName {
    /// New name of the event
    pub name: String,
}

#[derive(Debug, Args)]
pub struct EditEventComment {
    /// New comment of the event
    #[clap(default_value = "")]
    pub comment: String,
}

#[derive(Debug, Args)]
pub struct EditEventBudget {
    /// New budget of the event
    pub budget: u32,
}

#[derive(Debug, Args)]
pub struct EditEventMeals {
    /// New meals of the event
    #[clap(subcommand)]
    pub meal_edit_type: EditEventMealsType,
}

#[derive(Debug, Subcommand)]
pub enum EditEventMealsType {
    /// Add a meal to the event
    Add(EditEventMealsAdd),

    /// Remove a meal from the event
    Remove(EditEventMealsRemove),

    /// Reorder the meals of the event
    Edit(EditEventMealsEdit),
}

#[derive(Debug, Args)]
pub struct EditEventMealsAdd {
    /// Recipe to add (use ID or name)
    pub recipe: String,

    /// Servings to prepare
    pub servings: u32,

    /// Calories per serving
    pub calories: u32,

    /// Start time of the meal
    pub start_time: String,

    /// End time of the meal
    pub end_time: String,

    #[clap(default_value = "")]
    /// Comment
    pub comment: String,
}

#[derive(Debug, Args)]
pub struct EditEventMealsRemove {
    /// Meal to remove (use index or name)
    pub meal: String,
}

#[derive(Debug, Args)]
pub struct EditEventMealsEdit {
    /// Meal to edit (use index or name)
    pub meal: String,

    #[clap(subcommand)]
    pub edit_type: EditEventMealsEditType,
}

#[derive(Debug, Subcommand)]
pub enum EditEventMealsEditType {
    /// Edit the recipe of a meal
    Recipe(EditEventMealsEditRecipe),

    /// Location for giving out the meal
    Location(EditEventMealsEditLocation),

    /// Edit the servings of a meal
    Servings(EditEventMealsEditServings),

    /// Edit the calories of a meal
    Calories(EditEventMealsEditCalories),

    /// Edit the start time of a meal
    StartTime(EditEventMealsEditStartTime),

    /// Edit the end time of a meal
    EndTime(EditEventMealsEditEndTime),

    /// Edit the comment of a meal
    Comment(EditEventMealsEditComment),
}

#[derive(Debug, Args)]
pub struct EditEventMealsEditRecipe {
    /// New recipe of the meal
    pub recipe: String,
}

#[derive(Debug, Args)]
pub struct EditEventMealsEditLocation {
    /// New location of the meal
    pub location: String,
}

#[derive(Debug, Args)]
pub struct EditEventMealsEditServings {
    /// New servings of the meal
    pub servings: u32,
}

#[derive(Debug, Args)]
pub struct EditEventMealsEditCalories {
    /// New calories of the meal
    pub calories: u32,
}

#[derive(Debug, Args)]
pub struct EditEventMealsEditStartTime {
    /// New start time of the meal
    pub start_time: String,
}

#[derive(Debug, Args)]
pub struct EditEventMealsEditEndTime {
    /// New end time of the meal
    pub end_time: String,
}

#[derive(Debug, Args)]
pub struct EditEventMealsEditComment {
    /// New comment of the meal
    #[clap(default_value = "")]
    pub comment: String,
}
