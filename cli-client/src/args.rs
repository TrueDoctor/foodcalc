use clap::{Args, Parser, Subcommand};
use std::str::FromStr;

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct CLI {
    #[clap(subcommand)]
    pub command: Commands,

    #[arg(short, long)]
    /// Enable Debug Output
    pub debug: bool,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    List(ListCommand),
    Show(ShowCommand),
    Print(PrintCommand),
    User(UserCommand),
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
            _ => Err(format!("Unknown List Type: {}", s)),
        }
    }
}

// ---- Show Commands ----
#[derive(Debug, Args)]
pub struct ShowCommand {
    #[clap(subcommand)]
    pub show_type: ShowCommands,
}

#[derive(Debug, Subcommand)]
pub enum ShowCommands {
    /// Show an event
    Event(ShowEvent),
    /// Show a recipe
    Recipe(ShowRecipe),
    /// Show a meal
    Meal(ShowMeal),
}

#[derive(Debug, Args)]
pub struct ShowEvent {
    /// Event to show (use ID or name)
    pub event: String,
}

#[derive(Debug, Args)]
pub struct ShowRecipe {
    /// Recipe to show (use ID or name)
    pub recipe: String,
}

#[derive(Debug, Args)]
pub struct ShowMeal {
    /// Event to show (use ID or name)
    pub event: String,
    /// Recipe to show (use ID or name)
    pub recipe: String,
}

// ---- Print Commands ----
#[derive(Debug, Args)]
pub struct PrintCommand {
    #[clap(subcommand)]
    pub print_type: PrintCommands,

    #[arg(short, long, default_value = "h")]
    // The Output Format
    pub format: String,

    #[arg(short, long)]
    // The Output File
    pub output_file: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum PrintCommands {
    #[clap(alias = "event")]
    /// Print the mealplan for a given event
    Mealplan(PrintEvent),
    /// Print the recipe for a given meal
    Meal(PrintMeal),
}

#[derive(Debug, Args)]
pub struct PrintEvent {
    /// Event to print (use ID or name)
    pub event: String,

    #[clap(short = 'd', long = "day")]
    /// Show only meals cooked on the given day (date of number of the day)
    pub day: Option<String>,
}

#[derive(Debug, Args)]
pub struct PrintMeal {
    /// Meal to print (use ID or name)
    pub meal: String,
}

// ---- Admin Commands ----
#[derive(Debug, Args)]
pub struct UserCommand {
    #[clap(subcommand)]
    pub user_type: UserCommands,
}

#[derive(Debug, Subcommand)]
pub enum UserCommands {

    #[clap(alias = "add")]
    /// Print the mealplan for a given event
    Create(CreateArgs),

    #[clap(alias = "rm")]
    /// Print the recipe for a given meal
    Remove(UserArg),

    /// Gives a List of all known Users and their Permissions
    List,
}

#[derive(Debug, Args)]
pub struct CreateArgs {
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
pub struct UserArg {
    /// Event to print (use ID or name)
    pub user: String,
}
