use axum::extract::State;
use foodlib::{IngredientWithWeight, Inventory, Ingredient};
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
}

pub struct InventoryHeaderData {
    pub inventory_id: i32,
    pub filter_text: String,
}

impl Default for InventoryHeaderData {
    fn default() -> Self {
        InventoryHeaderData { inventory_id: -1, filter_text: String::new() }
    }
}

pub fn parse_inventory_header_data(query: String) -> InventoryHeaderData {
    let parameter_regex = Regex::new(r"(?<param_name>selected-inv|filter-text)=(?<param_value>[^&]*)&?").unwrap();
    
    let mut data: InventoryHeaderData = InventoryHeaderData::default();
    for (_, [key, value]) in parameter_regex.captures_iter(query.as_str()).map(|c| c.extract()) {
        if key == "selected-inv" { data.inventory_id = value.parse().unwrap(); }
        else if key == "filter-text" { data.filter_text = String::from(value); }
    }
    data
}

pub async fn commit_inventory(
    State(state): State<MyAppState>,
    form: axum::extract::Form<foodlib::Inventory>,
) -> Markup {
    let inventory = form.0;
    if inventory.inventory_id < 0 {
        let Ok(inventory_id) = state.db_connection.add_inventory(inventory.name).await else {
            return html! {
                div id="error" class="flex flex-col items-center justify-center text-red-500" {
                    div {
                        h1 { "Error" }
                        p { "Failed to add inventory" }
                        button class="btn btn-primary" hx-get="/inventories" hx-target="#content"  { "Back" }
                    }
                }
            }
        };
        manage_inventory_form(State(state), inventory_id).await
    } else {
        let Ok(inventory_id) = state.db_connection.update_inventory(inventory).await else {
            return html! {
                div id="error" class="flex flex-col items-center justify-center text-red-500" {
                    div {
                        h1 { "Error" }
                        p { "Failed to rename inventory" }
                        button class="btn btn-primary" hx-get="/inventories" hx-target="#content"  { "Back" }
                    }
                }
            }
        };
        manage_inventory_form(State(state), inventory_id).await
    }
}

pub async fn handle_select(State(state): State<MyAppState>, query: String) -> Markup {
    let inventory_id = query.replace("selected-inv=", "").parse().unwrap();
    
    manage_inventory_form(State(state.clone()), inventory_id).await
}

pub async fn handle_add_inventory(State(state): State<MyAppState>) -> Markup {
    add_or_edit_inventory_form(State(state), -1, String::new())
}

pub async fn handle_delete_inventory(State(state): State<MyAppState>, query: String) -> Markup {
    let data = parse_inventory_header_data(query);

    let Ok(_) = state.db_connection.delete_inventory(data.inventory_id).await else {
        return html! {
            div id="error" class="flex flex-col items-center justify-center text-red-500" {
                div {
                    h1 { "Error" }
                    p { "Failed to delete inventory" }
                    button class="btn btn-primary" hx-get="/inventories" hx-target="#content"  { "Back" }
                }
            }
        };
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
        div class="flex flex-col items-center justify-center gap-5" id="inventories" {
            form hx-put="/inventories/select" hx-target="this" hx-swap="outerHTML" hx-trigger="change" {
                div class="flex flex-row items-center justify-center mb-2 gap-5 h-10 w-full" {
                    button hx-get="/inventories/add-inventory" class="btn btn-primary" hx-target="#inventories" { "+" };
                    select class="fc-select" name="selected-inv" hx-indicator=".htmx-indicator" {
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
        div class="flex flex-col items-center justify-center gap-5" id="inventories" {
            form hx-put="/inventories/manage" hx-target="#search-results" hx-trigger="change" {
                div class="flex flex-row items-center justify-center mb-2 gap-5 h-10 w-full" {
                    button hx-get="/inventories/add-inventory" class="btn btn-primary" hx-target="#inventories" { "+" };
                    button hx-put="/inventories/edit-inventory" class="btn btn-primary" hx-target="#inventories" { "Edit" }
                    select class="fc-select" name="selected-inv" hx-indicator=".htmx-indicator" hx-target="#search-results" {
                        @for inventory in inventories.iter() { (inventory.format_for_select(selected_inventory_id)) }
                    }
                    button hx-put="/inventories/delete-inventory" class="btn btn-primary" hx-target="#inventories" { "Delete" }
                    input type="text" class="text w-full" name="filter-text" ;//hx-trigger="keyup changed delay:20ms, search";       // TODO: Would be nice if this updated the contents without having to press enter
                }
            }
            (render_filtered_inventory_contents(State(state), selected_inventory_id, String::new()).await)
            div hx-target="this"  hx-swap="outerHTML" {
                button hx-get="/inventories/add_ingredient" class="btn btn-primary"  { "+" }
            }
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
        .get_filtered_inventory_contents(inventory_id, filter)
        .await
        .unwrap_or_default();

    let ingredient_list = state
        .db_connection
        .get_ingredients()
        .await
        .unwrap_or_default();

    html! {
        datalist id="ingredients" { @for ingredient in ingredient_list { (ingredient.format_for_datalist()) } }
        span class="htmx-indicator" { "Searching..." }
        table class="text-inherit table-auto object-center" padding="0 0.5em" display="block" 
        max-height="60vh" overflow-y="scroll" {
            thead { tr { th { "Name" } th { "Amount" } th {} } }
            tbody id="search-results" { @for item in contents { (item.format_for_ingredient_table()) } }
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
                    input type="text" list="ingredients" class="text w-full" name="selected-ingredient" 
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
