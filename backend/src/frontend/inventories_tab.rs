use axum::extract::State;
use bigdecimal::BigDecimal;
use foodlib::{IngredientWithWeight, Inventory};
use maud::{html, Markup};
use regex::Regex;

use crate::MyAppState;

pub(crate) fn inventories_router() -> axum::Router<MyAppState> {
    axum::Router::new()
        .route("/edit", axum::routing::put(add_inventory))
        .route("/add", axum::routing::get(add_inventory_form))
        .route("/", axum::routing::post(select_inventory_form))
        .route("/select", axum::routing::put(handle_select))
        .route("/manage", axum::routing::put(handle_manage))
}

pub async fn search(State(state): State<MyAppState>, query: String) -> Markup {
    let query = query.replace("search=", "").to_lowercase();
    let inventories = state
        .db_connection
        .get_inventories()
        .await
        .unwrap_or_default();
    html! {}
}

pub async fn add_inventory(
    State(state): State<MyAppState>,
    form: axum::extract::Form<foodlib::Inventory>,
) -> Markup {
    let inventory = form.0;
    let Ok(inventory_id) = state.db_connection.add_inventory(inventory.name).await else {
        return html! {
            div id="error" class="flex flex-col items-center justify-center text-red-500" {
                div {
                    h1 { "Error" }
                    p { "Failed to add inventory" }
                    button class="btn btn-primary" hx-get="/inventories" hx-target="#content"  { "Back" }
                }
            }
        };
    };

    edit_inventory_form(State(state), inventory_id).await
}

pub async fn inventories_view(State(state): State<MyAppState>) -> Markup {
    select_inventory_form(State(state)).await
}

pub async fn handle_select(State(state): State<MyAppState>, query: String) -> Markup {
    let inventory_id = query.replace("selected-inv=", "").parse().unwrap();
    html! {
        (edit_inventory_form(State(state.clone()), inventory_id).await)
    }
}


pub async fn handle_manage(State(state): State<MyAppState>, query: String) -> Markup {
    //let param_type_regex = r"(?<param-name>selected-inv|filter-text)";
    //let keyval_regex = param_type_regex + r"=(?<param-value>.*)";
    let parameter_regex = Regex::new(r"(?<param_name>selected-inv|filter-text)=(?<param_value>[^&]*)&?").unwrap();
    
    let mut inventory_id: i32 = -1;
    let mut filter_text: String = String::new();
    for (_, [key, value]) in parameter_regex.captures_iter(query.as_str()).map(|c| c.extract()) {
        dbg!(format!("parameter: {}, value: {}", key, value));
        if key == "selected-inv" { inventory_id = value.parse().unwrap(); }
        else if key == "filter-text" { filter_text = String::from(value); }
    }
    render_filtered_inventory_contents(State(state), inventory_id, filter_text).await
}

pub async fn select_inventory_form(State(state): State<MyAppState>) -> Markup {
    let inventories = state
        .db_connection
        .get_inventories()
        .await
        .unwrap_or_default();
    html! {
        form hx-put="/inventories/select" hx-target="this" hx-swap="outerHTML" hx-trigger="change" {
            div class="flex flex-col items-center justify-center gap-5" {
                div class="flex flex-row items-center justify-center mb-2 gap-5 h-10 w-full" {
                    button hx-get="/inventories/add" class="btn btn-primary" hx-target="#inventories" { "+" };
                    select class="fc-select" name="selected-inv" hx-post="/inventories/select" hx-indicator=".htmx-indicator"
                        hx-target="#search-results" {
                        option value="-1" selected { "Select inventory" }; 
                        @for inventory in inventories.iter() { (inventory.format_for_select(-1)) }
                    }
                }
            }
        }
    }
}

pub async fn edit_inventory_form(
    State(state): State<MyAppState>, 
    selected_inventory_id: i32) -> Markup {

    let inventories = state
        .db_connection
        .get_inventories()
        .await
        .unwrap_or_default();
        
    html! {
        div class="flex flex-col items-center justify-center gap-5" {
            form hx-put="/inventories/manage" hx-target="#search-results" hx-swap="outerHTML" hx-trigger="change" {
                div class="flex flex-row items-center justify-center mb-2 gap-5 h-10 w-full" {
                    button hx-get="/inventories/add" class="btn btn-primary" hx-target="#inventories" { "+" };
                    button class="btn btn-primary" { "Edit" }
                    select class="fc-select" name="selected-inv" hx-post="/inventories/select" hx-indicator=".htmx-indicator"
                        hx-target="#search-results" {
                        @for inventory in inventories.iter() { (inventory.format_for_select(selected_inventory_id)) }
                    }
                    button class="btn btn-primary" { "Delete" }
                    input type="text" class="text w-full" name="filter-text";       // TODO: Would be nice if this updated the contents without having to press enter
                }
            }
            span class="htmx-indicator" { "Searching..." }
            table class="text-inherit table-auto object-center" padding="0 0.5em" display="block" 
            max-height="60vh" overflow-y="scroll" {
                thead { tr { th { "Name" } th { "Amount" } th {} } }
                (render_filtered_inventory_contents(State(state), selected_inventory_id, String::new()).await)
            }
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

    html! {
        tbody id="search-results" { @for item in contents { (item.format_for_ingredient_table()) } }
    }
}

pub async fn add_inventory_form(State(_state): State<MyAppState>) -> Markup {
    html! {
        form hx-put="/inventories/edit" hx-target="this" hx-swap="outerHTML" {
            div class="flex flex-col items-center justify-center gap-5" {
                div class="flex flex-row items-center justify-center mb-2 gap-5 h-10 w-full"{
                    h1 { "Edit inventory" }
                    input type="text" name="name" placeholder="Name" value="" required="required" class="text";
                    input type="hidden" name="inventory_id" value="-1";
                    button class="btn btn-primary" type="submit" { "Submit" }
                }
            }
            span class="htmx-indicator" { "Searching..." }
            table class="text-inherit table-auto object-center" padding="0 0.5em" display="block" 
            max-height="60vh" overflow-y="scroll" {
                thead { tr { th { "Name" } th { "Amount" } th {} } }
                tbody id="search-results" {}
            }
            div hx-target="this"  hx-swap="outerHTML" {
                button hx-get="/inventories/add_ingredient" class="btn btn-primary"  { "+" }
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

pub trait IngredientTableFormattable {
    fn format_for_ingredient_table(&self) -> Markup;
}

impl IngredientTableFormattable for IngredientWithWeight {
    fn format_for_ingredient_table(&self) -> Markup {
        html! {
            tr id=(format!("ingredient-{}", self.ingredient_id)) {
                td { select class="fc-select w-full" name="selected-ingredient" hx-post="/inventories/select-ingredient" hx-indicator=".htmx-indicator" { option value=(self.ingredient_id) { (self.name) } } }
                td { input type="number" name="amount" value=(self.amount) required="required"; }
                td { button class="btn btn-primary" type="submit" { "X" } }
            }
        }
    }
}
