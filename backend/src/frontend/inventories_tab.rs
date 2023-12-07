use axum::extract::State;
use bigdecimal::BigDecimal;
use foodlib::{IngredientWithWeight, Inventory, Ingredient, InventoryIngredient};
use maud::{html, Markup};
use regex::Regex;

use crate::MyAppState;

pub(crate) fn inventories_router() -> axum::Router<MyAppState> {
    axum::Router::new()
        .route("/commit", axum::routing::put(commit_inventory))
        .route("/add-inventory", axum::routing::get(handle_add_inventory))
        .route("/edit-inventory", axum::routing::put(handle_edit_inventory))
        .route("/", axum::routing::get(select_inventory_form))
        .route("/select", axum::routing::put(handle_select))
        .route("/manage", axum::routing::put(handle_manage))
        .route( "/delete-inventory", axum::routing::put(handle_delete_inventory))
        .route( "/add-ingredient", axum::routing::put(add_ingredient_form))
        .route( "/commit-ingredient", axum::routing::put(handle_ingredient_commit))
}

// Request parameters
static INVENTORY_ID: &'static str = "inventory-id";
static FILTER_TEXT: &'static str = "filter-text";
static INGREDIENT_NAME: &'static str = "ingredient-name";
static INGREDIENT_AMOUNT: &'static str = "ingredient-amount";

// htmx ids
static SEARCH_RESULTS_DIV: &'static str = "search-results";
static INVENTORIES_DIV: &'static str = "inventories";
static INVENTORY_CONTENTS_DIV: &'static str = "contents";
static INGREDIENTS_DATALIST: &'static str = "ingredients";

// TODO: Refactor request paths to constants

pub struct InventoryHeaderData {
    pub inventory_id: i32,
    pub filter_text: String,
}

pub struct InventoryItemData {
    pub inventory_id: i32,
    pub ingredient_name: String,
    pub ingredient_amount: BigDecimal,
}

impl Default for InventoryHeaderData {
    fn default() -> Self {
        InventoryHeaderData { inventory_id: -1, filter_text: String::new() }
    }
}

impl Default for InventoryItemData {
    fn default() -> Self {
        InventoryItemData { inventory_id: -1, ingredient_name: String::new(), ingredient_amount: BigDecimal::from(0) }
    }
}

pub fn parse_inventory_header_data(query: String) -> InventoryHeaderData {
    // TODO: Let Dennis look this over and see if it's best practice
    let parameter_regex = Regex::new(format!(r"(?<param_name>{}|{})=(?<param_value>[^&]*)&?", INVENTORY_ID, FILTER_TEXT).as_str()).unwrap();
    
    let mut data: InventoryHeaderData = InventoryHeaderData::default();
    for (_, [key, value]) in parameter_regex.captures_iter(query.as_str()).map(|c| c.extract()) {
        if key == INVENTORY_ID { data.inventory_id = value.parse().unwrap(); }
        else if key == FILTER_TEXT { data.filter_text = String::from(value.replace("%20", " ")); }
    }
    data
}

pub fn parse_inventory_item_data(query: String) -> InventoryItemData {
    let parameter_regex = Regex::new(format!(r"(?<param_name>{}|{}|{})=(?<param_value>[^&]*)&?", INVENTORY_ID, INGREDIENT_NAME, INGREDIENT_AMOUNT).as_str()).unwrap();
    
    let mut data: InventoryItemData = InventoryItemData::default();
    for (_, [key, value]) in parameter_regex.captures_iter(query.as_str()).map(|c| c.extract()) {
        if key == INVENTORY_ID { data.inventory_id = value.parse().unwrap(); }
        else if key == INGREDIENT_NAME { data.ingredient_name = String::from(value.replace("%20", " ")); }
        else if key == INGREDIENT_AMOUNT { data.ingredient_amount = value.parse().unwrap(); }
    }
    data
}

fn return_to_inv_selection_error() -> Markup {
    html! {
        div id="error" class="flex flex-col items-center justify-center text-red-500" {
            div {
                h1 { "Error" }
                p { "Failed to add ingredient" }
            }
            button class="btn btn-primary" hx-get="/inventories" hx-target=(["#", INVENTORIES_DIV].concat())  { "Back" }
        }
    }
}

fn return_to_inv_overview_error(inventory_id: i32) -> Markup {
    html! {
        div id="error" class="flex flex-col items-center justify-center text-red-500" {
            form {
                h1 { "Error" }
                p { "Failed to add ingredient" }
                input type="hidden" value=(inventory_id);
            }
            button class="btn btn-primary" hx-get="/inventories/select" hx-target=(INVENTORIES_DIV)  { "Back" }
        }
    }
}

pub async fn handle_ingredient_commit(State(state): State<MyAppState>, query: String) -> Markup {
    let item_data = parse_inventory_item_data(query.clone());
    let header_data = parse_inventory_header_data(query);
    let ingredient_id = state
        .db_connection
        .get_ingredient_from_string_reference(item_data.ingredient_name.clone())
        .await
        .unwrap_or(Ingredient {ingredient_id: -1, name: String::new(), energy: BigDecimal::from(-1), comment: None})
        .ingredient_id;

    dbg!(format!("requested name {} yielded ingredient id {}", item_data.ingredient_name, ingredient_id));
    if ingredient_id < 0 { 
        add_ingredient_form(
            State(state), 
            format!("{}={}&{}={}", INVENTORY_ID, item_data.inventory_id, FILTER_TEXT, header_data.filter_text))
            .await
    }
    else {
        let Ok(res) = state.db_connection.update_inventory_item(InventoryIngredient {
                inventory_id: item_data.inventory_id,
                ingredient_id: ingredient_id, 
                amount: item_data.ingredient_amount
        }).await else {
            return return_to_inv_overview_error(item_data.inventory_id)
        };
        dbg!(res);
        (render_filtered_inventory_contents(State(state), item_data.inventory_id, header_data.filter_text)).await
    }
}

pub async fn add_ingredient_form(State(_state): State<MyAppState>, query: String) -> Markup
{
    let header_data = parse_inventory_header_data(query);
    html! {
        form hx-put="inventories/commit-ingredient" hx-target=(["#", INVENTORY_CONTENTS_DIV].concat()) hx-swap="outerHTML" {
            div class="flex flex-row items-center justify-center mb-2 gap-5 h-10 w-full"{
                h1 { "Add ingredient" }
                input type="hidden" name=(INVENTORY_ID) value=(header_data.inventory_id);
                input type="hidden" name=(FILTER_TEXT) value=(header_data.filter_text);
                input type="text" list=(INGREDIENTS_DATALIST) name=(INGREDIENT_NAME) placeholder="Ingredient" required="required" class="text";
                input type="number" name=(INGREDIENT_AMOUNT) placeholder="Amount" value="" step="0.01" min="0.05" required="required";
                button class="btn btn-primary" type="submit" { "Submit" }
            }
        }
    }
}

pub async fn commit_inventory(
    State(state): State<MyAppState>,
    form: axum::extract::Form<foodlib::Inventory>,
) -> Markup {
    let inventory = form.0;
    if inventory.inventory_id < 0 {
        let Ok(inventory_id) = state.db_connection.add_inventory(inventory.name).await else {
            return return_to_inv_selection_error();
        };
        manage_inventory_form(State(state), inventory_id).await
    } else {
        let id = inventory.inventory_id;
        let Ok(inventory_id) = state.db_connection.update_inventory(inventory).await else {
            return return_to_inv_overview_error(id)
        };
        manage_inventory_form(State(state), inventory_id).await
    }
}

pub async fn handle_select(State(state): State<MyAppState>, query: String) -> Markup {
    let inventory_id = parse_inventory_header_data(query).inventory_id;
    
    manage_inventory_form(State(state.clone()), inventory_id).await
}

pub async fn handle_add_inventory(State(state): State<MyAppState>) -> Markup {
    add_or_edit_inventory_form(State(state), -1, String::new())
}

pub async fn handle_delete_inventory(State(state): State<MyAppState>, query: String) -> Markup {
    let data = parse_inventory_header_data(query);

    let Ok(_) = state.db_connection.delete_inventory(data.inventory_id).await else {
        return return_to_inv_overview_error(data.inventory_id);
    };
    select_inventory_form(State(state)).await
}

pub async fn handle_manage(State(state): State<MyAppState>, query: String) -> Markup {
    let data = parse_inventory_header_data(query);
    render_filtered_inventory_contents(State(state), data.inventory_id, data.filter_text).await
}

pub async fn handle_edit_inventory(State(state): State<MyAppState>, query: String) -> Markup {
    let data = parse_inventory_header_data(query);
    
    let inventory_name = state
        .db_connection
        .get_inventory_from_id(data.inventory_id)
        .await
        .unwrap()
        .name;

    add_or_edit_inventory_form(State(state), data.inventory_id, inventory_name)
}

pub async fn select_inventory_form(State(state): State<MyAppState>) -> Markup {
    let inventories = state
        .db_connection
        .get_inventories()
        .await
        .unwrap_or_default();
    html! {
        div class="flex flex-col items-center justify-center gap-5" id=(INVENTORIES_DIV) {
            form hx-put="/inventories/select" hx-target="this" hx-swap="outerHTML" hx-trigger="change" {
                div class="flex flex-row items-center justify-center mb-2 gap-5 h-10 w-full" {
                    button hx-get="/inventories/add-inventory" class="btn btn-primary" hx-target=(["#", INVENTORIES_DIV].concat()) { "+" };
                    select class="fc-select" name=(INVENTORY_ID) hx-indicator=".htmx-indicator" {
                        option value="-1" selected { "Select inventory" }; 
                        @for inventory in inventories.iter() { (inventory.format_for_select(-1)) }
                    }
                }
            }
        }
    }
}

pub async fn manage_inventory_form(
    State(state): State<MyAppState>, 
    selected_inventory_id: i32) -> Markup {

    let inventories = state
        .db_connection
        .get_inventories()
        .await
        .unwrap_or_default();
        
    html! {
        div class="flex flex-col items-center justify-center gap-5" id=(INVENTORIES_DIV) {
            form hx-put="/inventories/manage" hx-target=(["#", INVENTORY_CONTENTS_DIV].concat()) hx-trigger="change" {
                div class="flex flex-row items-center justify-center mb-2 gap-5 h-10 w-full" {
                    button hx-get="/inventories/add-inventory" class="btn btn-primary" hx-target=(["#", INVENTORIES_DIV].concat()) { "+" };
                    button hx-put="/inventories/edit-inventory" class="btn btn-primary" hx-target=(["#", INVENTORIES_DIV].concat()) { "Edit" }
                    select class="fc-select" name=(INVENTORY_ID) hx-indicator=".htmx-indicator" hx-target=(["#", INVENTORY_CONTENTS_DIV].concat()) {
                        @for inventory in inventories.iter() { (inventory.format_for_select(selected_inventory_id)) }
                    }
                    button hx-put="/inventories/delete-inventory" class="btn btn-primary" hx-target=(["#", INVENTORIES_DIV].concat()) { "Delete" }
                    input type="text" class="text w-full" name=(FILTER_TEXT);       // TODO: Would be nice if this updated the contents without having to press enter
                }
            }
            (render_filtered_inventory_contents(State(state), selected_inventory_id, String::new()).await)
        }
    }
}

pub async fn render_filtered_inventory_contents(
    State(state): State<MyAppState>,
    inventory_id: i32,
    filter: String,
) -> Markup {
    let contents = state
        .db_connection
        .get_filtered_inventory_contents(inventory_id, filter.clone())
        .await
        .unwrap_or_default();

    let ingredient_list = state
        .db_connection
        .get_ingredients()
        .await
        .unwrap_or_default();

    html! {
        div id=(INVENTORY_CONTENTS_DIV) {
            div id=(SEARCH_RESULTS_DIV) {
                datalist id=(INGREDIENTS_DATALIST) { @for ingredient in ingredient_list { (ingredient.format_for_datalist()) } }
                span class="htmx-indicator" { "Searching..." }
                table class="text-inherit table-auto object-center" padding="0 0.5em" display="block" 
                max-height="60vh" overflow-y="scroll" {
                    thead { tr { th { "Name" } th { "Amount" } th {} } }
                    tbody { @for item in contents { (item.format_for_ingredient_table()) } }
                }
            }
            form hx-target="this" hx-put="/inventories/add-ingredient" hx-swap="outerHTML" {
                input type="hidden" name=(INVENTORY_ID) value=(inventory_id);
                input type="hidden" name=(FILTER_TEXT) value=(filter);
                button type="submit" class="btn btn-primary"  { "+" }
            }
        }
    }
}

pub fn add_or_edit_inventory_form(State(_state): State<MyAppState>, inventory_id: i32, inventory_name: String) -> Markup {
    html! {
        form hx-put="/inventories/commit" hx-target="this" hx-swap="outerHTML" {
            div class="flex flex-col items-center justify-center gap-5" {
                div class="flex flex-row items-center justify-center mb-2 gap-5 h-10 w-full"{
                    h1 { @if inventory_id > 0 { "Edit inventory" } @else { "Add inventory" } }
                    input type="text" name="name" placeholder="Name" value=(inventory_name) required="required" class="text";
                    input type="hidden" name="inventory_id" value=(inventory_id);
                    button class="btn btn-primary" type="submit" { "Submit" }
                }
            }
        }
    }
}

// TODO: Refactor when we build functions for GUI components
pub trait SelectFormattable {
    fn format_for_select(&self, selected_index: i32) -> Markup;
}

impl SelectFormattable for Inventory {
    fn format_for_select(&self, selected_index: i32) -> Markup {
        html! {
            @if selected_index == self.inventory_id {
                option selected value=(format!("{}", self.inventory_id)) { (self.name) };
            }
            @else {
                option value=(format!("{}", self.inventory_id)) { (self.name) };
            }
        }
    }
}

impl SelectFormattable for IngredientWithWeight {
    fn format_for_select(&self, selected_index: i32) -> Markup {
        html! {
            @if selected_index == self.ingredient_id {
                option selected value=(format!("{}", self.ingredient_id)) { (self.name) };
            }
            @else {
                option value=(format!("{}", self.ingredient_id)) { (self.name) };
            }
        }
    }
}

pub trait IngredientTableFormattable {
    fn format_for_ingredient_table(&self) -> Markup;
}

impl IngredientTableFormattable for IngredientWithWeight {
    fn format_for_ingredient_table(&self) -> Markup {
        html! {
            tr id=(format!("ingredient-{}", self.ingredient_id)) { // TODO: Put into form
                td { 
                    input type="text" list=(INGREDIENTS_DATALIST) class="text w-full" name="selected-ingredient" 
                    hx-post="/inventories/select-ingredient" hx-indicator=".htmx-indicator" value=(self.name); 
                }
                td { input type="number" name="amount" value=(self.amount) required="required"; }
                td { button class="btn btn-primary" type="submit" { "X" } }
            }
        }
    }
}

pub trait DataListFormattable {
    fn format_for_datalist(&self) -> Markup;
}

impl DataListFormattable for Ingredient {
    fn format_for_datalist(&self) -> Markup {
        html! {
            option value=(self.name) { }
        }
    }
}
