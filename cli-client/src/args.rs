use clap::{Args, Parser, Subcommand, ValueEnum};
use sqlx::types::BigDecimal;
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
    /// Fetch prices from Vendors
    UpdatePrices,

    /// List all Places, Events, Ingredients, Recipes or Meals
    List(ListCommand),

    #[clap(alias = "show")]
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

    /// Edit an Ingredient, Recipe, User or Event
    Edit(EditCommand),
}

// ---- List Commands ----
#[derive(Debug, Args)]
pub struct ListCommand {
    pub list_type: ListType,

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
pub enum ListType {
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

impl FromStr for ListType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "places" => Ok(ListType::Places),
            "events" => Ok(ListType::Events),
            "ingredients" => Ok(ListType::Ingredients),
            "recipes" => Ok(ListType::Recipes),
            "meals" => Ok(ListType::Meals),
            "users" => Ok(ListType::Users),
            _ => Err(format!("Unknown List Type: {}", s)),
        }
    }
}

// ---- Info Commands ----
#[derive(Debug, Args)]
pub struct InfoCommand {
    #[clap(subcommand)]
    pub show_type: InfoType,
}

#[derive(Debug, Subcommand)]
pub enum InfoType {
    /// Get info about an Ingredient
    Ingredient(InfoIngredientCommand),

    /// Get info about an Event
    Event(InfoEventCommand),

    /// Get info about a Recipe
    Recipe(InfoRecipeCommand),

    /// Get info about a Meal
    Meal(InfoMealCommand),

    /// Get info about a User
    User(InfoUserCommand),
}

#[derive(Debug, Args)]
pub struct InfoIngredientCommand {
    /// Recipe to show (use ID or name)
    pub ingredient_ref: String,
}

#[derive(Debug, Args)]
pub struct InfoEventCommand {
    /// Event to show (use ID or name)
    pub event_ref: String,
}

#[derive(Debug, Args)]
pub struct InfoRecipeCommand {
    /// Recipe to show (use ID or name)
    pub recipe_ref: String,
}

#[derive(Debug, Args)]
pub struct InfoMealCommand {
    /// Event to show (use ID or name)
    pub event_ref: String,

    /// Recipe to show (use ID or name)
    pub recipe_ref: String,

    /// Start time - only needed if a recipe is cooked multiple times in one event
    pub start_time: Option<String>,
}

#[derive(Debug, Args)]
pub struct InfoUserCommand {
    /// User to show (use ID or name)
    pub user_ref: String,
}

// ---- Calc Commands ----
#[derive(Debug, Args)]
pub struct CalcCommand {
    #[clap(subcommand)]
    pub print_type: CalcType,

    #[arg(short, long)]
    // The Output File
    pub output_file: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum CalcType {
    #[clap(alias = "event")]
    /// Print the mealplan for a given event
    Mealplan(CalcMealplanCommand),

    #[clap(alias = "meals")]
    /// Print the recipe for a given meal or for all meals in an event
    Meal(CalcMealCommand),

    /// Print a given Recipe
    Recipe(CalcRecipeCommand),
}

#[derive(Debug, Args)]
pub struct CalcMealplanCommand {
    /// Event to print (use ID or name)
    pub event_ref: String,
}

#[derive(Debug, Args)]
pub struct CalcMealCommand {
    /// Event to show (use ID or name)
    pub event_ref: String,

    /// Recipe to show (use ID or name)
    pub recipe_ref: String,

    /// Start time - only needed if a recipe is cooked multiple times in one event
    pub start_time: Option<String>,
}

#[derive(Debug, Args)]
pub struct CalcRecipeCommand {
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
    pub add_type: AddType,
}

#[derive(Debug, Subcommand)]
pub enum AddType {
    /// Add a new Ingredient
    Ingredient(AddIngredientCommand),

    /// Add a new Recipe
    Recipe(AddRecipeCommand),

    /// Add a new User
    User(AddUserCommand),

    /// Add a new Event
    Event(AddEventCommand),
}

#[derive(Debug, Args)]
pub struct AddIngredientCommand {
    /// Name of the new ingredient
    pub name: String,

    /// Calories per 100g
    pub energy: u32,

    #[clap(default_value = "")]
    /// Add comments
    pub comment: String,
}

#[derive(Debug, Args)]
pub struct AddRecipeCommand {
    /// Name of the new recipe
    pub name: String,

    #[clap(default_value = "")]
    /// Add comments
    pub comment: String,
}

#[derive(Debug, Args)]
pub struct AddUserCommand {
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
pub struct AddEventCommand {
    /// Name of the new event
    pub name: String,

    /// Buget for the event
    pub budget: Option<BigDecimal>,

    /// Add comments
    pub comment: Option<String>,
}

//// --- Delete Commands ----
#[derive(Debug, Args)]
pub struct DeleteCommand {
    #[clap(subcommand)]
    pub delete_type: DeleteType,
}

#[derive(Debug, Subcommand)]
pub enum DeleteType {
    /// Delete an Ingredient
    Ingredient(DeleteIngredientCommand),

    /// Delete a Recipe
    Recipe(DeleteRecipeCommand),

    /// Delete a User
    User(DeleteUserCommand),

    /// Delete an Event
    Event(DeleteEventCommand),
}

#[derive(Debug, Args)]
pub struct DeleteIngredientCommand {
    /// Ingredient to delete (use ID or name)
    pub ingredient: String,
}

#[derive(Debug, Args)]
pub struct DeleteRecipeCommand {
    /// Recipe to delete (use ID or name)
    pub recipe: String,
}

#[derive(Debug, Args)]
pub struct DeleteUserCommand {
    /// User to delete (use ID or name)
    pub user: String,
}

#[derive(Debug, Args)]
pub struct DeleteEventCommand {
    /// Event to delete (use ID or name)
    pub event: String,
}

/// ---- Edit Commands ----
#[derive(Debug, Args)]
pub struct EditCommand {
    #[clap(subcommand)]
    pub edit_type: EditType,
}

#[derive(Debug, Subcommand)]
pub enum EditType {
    /// Edit an Ingredient
    Ingredient(EditIngredientCommand),

    /// Edit a Recipe
    Recipe(EditRecipeCommand),

    /// Edit a User
    User(EditUserCommand),

    /// Edit an Event
    Event(EditEventCommand),
}

//### Ingredient
#[derive(Debug, Args)]
pub struct EditIngredientCommand {
    /// Ingredient to edit (use ID or name)
    pub ingredient: String,

    #[clap(subcommand)]
    pub edit_type: EditIngredientType,
}

#[derive(Debug, Subcommand)]
pub enum EditIngredientType {
    /// Edit the name of an Ingredient
    Name(EditIngredientNameCommand),

    /// Edit the energy of an Ingredient
    Energy(EditIngredientEnergyCommand),

    /// Edit the comment of an Ingredient
    Comment(EditIngredientCommentCommand),
}

#[derive(Debug, Args)]
pub struct EditIngredientNameCommand {
    /// New name of the ingredient
    pub name: String,
}

#[derive(Debug, Args)]
pub struct EditIngredientEnergyCommand {
    /// New energy of the ingredient
    pub energy: BigDecimal,
}

#[derive(Debug, Args)]
pub struct EditIngredientCommentCommand {
    /// New comment of the ingredient
    pub comment: Option<String>,
}

//### Recipe
#[derive(Debug, Args)]
pub struct EditRecipeCommand {
    /// Recipe to edit (use ID or name)
    pub recipe: String,

    #[clap(subcommand)]
    pub edit_type: EditRecipeType,
}

#[derive(Debug, Subcommand)]
pub enum EditRecipeType {
    /// Edit the name of a Recipe
    Name(EditRecipeNameCommand),

    /// Edit the comment of a Recipe
    Comment(EditRecipeCommentCommand),

    /// Edit the ingredients of a Recipe
    Ingredients(EditRecipeIngredientsCommand),

    /// Edit the steps of a Recipe
    Steps(EditRecipeStepsCommand),
}

#[derive(Debug, Args)]
pub struct EditRecipeNameCommand {
    /// New name of the recipe
    pub name: String,
}

#[derive(Debug, Args)]
pub struct EditRecipeCommentCommand {
    /// New comment of the recipe
    pub comment: Option<String>,
}

#[derive(Debug, Args)]
pub struct EditRecipeIngredientsCommand {
    /// New ingredients of the recipe
    #[clap(subcommand)]
    pub ingredient_edit_type: EditRecipeIngredientsType,
}

#[derive(Debug, Subcommand)]
pub enum EditRecipeIngredientsType {
    /// Add an ingredient to the recipe
    Add(EditRecipeIngredientsAddCommand),

    /// Remove an ingredient from the recipe
    Remove(EditRecipeIngredientsRemoveCommand),

    /// Change the amount of an ingredient in the recipe
    Amount(EditRecipeIngredientsAmountCommand),
}

#[derive(Debug, Args)]
pub struct EditRecipeIngredientsAddCommand {
    /// Ingredient to add (use ID or name)
    pub ingredient: String,

    /// Amount of the ingredient
    pub amount: u32,
}

#[derive(Debug, Args)]
pub struct EditRecipeIngredientsRemoveCommand {
    /// Ingredient to remove (use ID or name)
    pub ingredient: String,
}

#[derive(Debug, Args)]
pub struct EditRecipeIngredientsAmountCommand {
    /// Ingredient to edit (use ID or name)
    pub ingredient: String,

    /// New amount of the ingredient
    pub amount: u32,
}

#[derive(Debug, Args)]
pub struct EditRecipeStepsCommand {
    /// New steps of the recipe
    #[clap(subcommand)]
    pub step_edit_type: EditRecipeStepsType,
}

#[derive(Debug, Subcommand)]
pub enum EditRecipeStepsType {
    /// Add a step to the recipe
    Add(EditRecipeStepsAddCommand),

    /// Remove a step from the recipe
    Remove(EditRecipeStepsRemoveCommand),

    /// Reorder the steps of the recipe
    Reorder(EditRecipeStepsReorderCommand),

    /// Edit a Step of the recipe
    Edit(EditRecipeStepsEditCommand),
}

#[derive(Debug, Args)]
pub struct EditRecipeStepsAddCommand {
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
pub struct EditRecipeStepsRemoveCommand {
    /// Step to remove (use index or name)
    pub step: String,
}

#[derive(Debug, Args)]
pub struct EditRecipeStepsReorderCommand {
    /// New order of the steps
    pub order: Vec<String>,
}

#[derive(Debug, Args)]
pub struct EditRecipeStepsEditCommand {
    /// Step to edit (use index or name)
    pub step: String,

    #[clap(subcommand)]
    pub edit_type: EditRecipeStepsEditType,
}

#[derive(Debug, Subcommand)]
pub enum EditRecipeStepsEditType {
    /// Edit the name of a step
    Name(EditRecipeStepsEditNameCommand),

    /// Edit the description of a step
    Description(EditRecipeStepsEditDescriptionCommand),

    /// Edit the duration of a step
    Duration(EditRecipeStepsEditDurationCommand),
}

#[derive(Debug, Args)]
pub struct EditRecipeStepsEditNameCommand {
    /// New name of the step
    pub name: String,
}

#[derive(Debug, Args)]
pub struct EditRecipeStepsEditDescriptionCommand {
    /// New description of the step
    #[clap(default_value = "")]
    pub description: String,
}

#[derive(Debug, Args)]
pub struct EditRecipeStepsEditDurationCommand {
    /// Type
    pub duration_type: EditRecipeStepsEditDurationType,

    /// New Duration Value
    pub fixed_time: String,
}

#[derive(Debug, ValueEnum, Clone)]
pub enum EditRecipeStepsEditDurationType {
    /// Edit the fixed duration of a step
    Fixed,

    /// Edit the scaled duration of a step
    Scaled,
}

//### User
#[derive(Debug, Args)]
pub struct EditUserCommand {
    /// User to edit (use ID or name)
    pub user: String,

    #[clap(subcommand)]
    pub edit_type: EditUserType,
}

#[derive(Debug, Subcommand)]
pub enum EditUserType {
    /// Edit the username of a User
    Name(EditUserNameCommand),

    /// Edit the password of a User
    Password(EditUserPasswordCommand),

    /// Edit the email of a User
    Email(EditUserEmailCommand),

    /// Promote the user to admin
    Promote,

    /// Demote the user from admin
    Demote
}

#[derive(Debug, Args)]
pub struct EditUserNameCommand {
    /// New username of the user
    pub username: String,
}

#[derive(Debug, Args)]
pub struct EditUserPasswordCommand {
    /// New password of the user
    pub password: String,
}

#[derive(Debug, Args)]
pub struct EditUserEmailCommand {
    /// New email of the user
    pub email: String,
}

#[derive(Debug, Args)]
pub struct EditUserAdminCommand {
    /// New admin status of the user
    pub admin: bool,
}

//### Event
#[derive(Debug, Args)]
pub struct EditEventCommand {
    /// Event to edit (use ID or name)
    pub event: String,

    #[clap(subcommand)]
    pub edit_type: EditEventType,
}

#[derive(Debug, Subcommand)]
pub enum EditEventType {
    /// Edit the name of an Event
    Name(EditEventNameCommand),

    /// Edit the comment of an Event
    Comment(EditEventCommentCommand),

    /// Edit the budget of an Event
    Budget(EditEventBudgetCommand),

    /// Meals of an Event
    Meals(EditEventMealsCommand),
}

#[derive(Debug, Args)]
pub struct EditEventNameCommand {
    /// New name of the event
    pub name: String,
}

#[derive(Debug, Args)]
pub struct EditEventCommentCommand {
    /// New comment of the event
    pub comment: Option<String>,
}

#[derive(Debug, Args)]
pub struct EditEventBudgetCommand {
    /// New budget of the event
    pub budget: Option<BigDecimal>,
}

#[derive(Debug, Args)]
pub struct EditEventMealsCommand {
    /// New meals of the event
    #[clap(subcommand)]
    pub meal_edit_type: EditEventMealsType,
}

#[derive(Debug, Subcommand)]
pub enum EditEventMealsType {
    /// Add a meal to the event
    Add(EditEventMealsAddCommand),

    /// Remove a meal from the event
    Remove(EditEventMealsRemoveCommand),

    /// Reorder the meals of the event
    Edit(EditEventMealsEditCommand),
}

#[derive(Debug, Args)]
pub struct EditEventMealsAddCommand {
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
pub struct EditEventMealsRemoveCommand {
    /// Meal to remove (use index or name)
    pub meal: String,
}

#[derive(Debug, Args)]
pub struct EditEventMealsEditCommand {
    /// Recipe of meal to edit (use index or name)
    pub recipe: String,

    /// Start time of meal to edit
    pub start_time: String,

    #[clap(subcommand)]
    pub edit_type: EditEventMealsEditType,
}

#[derive(Debug, Subcommand)]
pub enum EditEventMealsEditType {
    /// Edit the recipe of a meal
    Recipe(EditEventMealsEditRecipeCommand),

    /// Location for giving out the meal
    Location(EditEventMealsEditLocationCommand),

    /// Edit the servings of a meal
    Servings(EditEventMealsEditServingsCommand),

    /// Edit the calories of a meal
    Calories(EditEventMealsEditCaloriesCommand),

    /// Edit the start time of a meal
    StartTime(EditEventMealsEditStartTimeCommand),

    /// Edit the end time of a meal
    EndTime(EditEventMealsEditEndTimeCommand),

    /// Edit the comment of a meal
    Comment(EditEventMealsEditCommentCommand),
}

#[derive(Debug, Args)]
pub struct EditEventMealsEditRecipeCommand {
    /// New recipe of the meal
    pub recipe: String,
}

#[derive(Debug, Args)]
pub struct EditEventMealsEditLocationCommand {
    /// New location of the meal
    pub location: String,
}

#[derive(Debug, Args)]
pub struct EditEventMealsEditServingsCommand {
    /// New servings of the meal
    pub servings: u32,
}

#[derive(Debug, Args)]
pub struct EditEventMealsEditCaloriesCommand {
    /// New calories of the meal
    pub calories: u32,
}

#[derive(Debug, Args)]
pub struct EditEventMealsEditStartTimeCommand {
    /// New start time of the meal
    pub start_time: String,
}

#[derive(Debug, Args)]
pub struct EditEventMealsEditEndTimeCommand {
    /// New end time of the meal
    pub end_time: String,
}

#[derive(Debug, Args)]
pub struct EditEventMealsEditCommentCommand {
    /// New comment of the meal
    #[clap(default_value = "")]
    pub comment: String,
}
