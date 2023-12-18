use axum::extract::{State, Form};
use bigdecimal::BigDecimal;
use foodlib::{IngredientWithWeight, Inventory, Ingredient, InventoryIngredient};
use maud::{html, Markup};
use serde::Deserialize;

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
        .route( "/delete-ingredient", axum::routing::put(handle_ingredient_delete))
        .route( "/change-ingredient-amount", axum::routing::put(handle_ingredient_change))
}

// Request parameters
static INVENTORY_ID: &'static str = "inventory_id";
static FILTER_TEXT: &'static str = "filter_text";
static INGREDIENT_ID: &'static str = "ingredient_id";
static INGREDIENT_NAME: &'static str = "ingredient_name";
static INGREDIENT_AMOUNT: &'static str = "ingredient_amount";

// htmx ids
static SEARCH_RESULTS_DIV: &'static str = "search-results";
static INVENTORIES_DIV: &'static str = "inventories";
static INVENTORY_CONTENTS_DIV: &'static str = "contents";
static INGREDIENTS_DATALIST: &'static str = "ingredients";

// TODO: Refactor request paths to constants

#[derive(Debug, Deserialize)]
pub struct InventoryHeaderData {
    pub inventory_id: i32,
    pub filter_text: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct InventoryItemData {
    pub inventory_id: i32,
    pub filter_text: Option<String>,
    pub ingredient_id: i32,
    pub ingredient_name: String,
    pub ingredient_amount: BigDecimal,
}

#[derive(Debug, Deserialize)]
pub struct UpdateInventoryItemData {
    pub inventory_id: i32,
    pub filter_text: Option<String>,
    pub ingredient_id: i32,
    pub ingredient_amount: BigDecimal,
}

impl Default for InventoryHeaderData {
    fn default() -> Self {
        InventoryHeaderData { inventory_id: -1, filter_text: None }
    }
}

impl Default for InventoryItemData {
    fn default() -> Self {
        InventoryItemData { inventory_id: -1, filter_text: None, ingredient_id: -1, ingredient_name: String::new(), ingredient_amount: BigDecimal::from(0) }
    }
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

pub async fn handle_ingredient_change(State(state): State<MyAppState>, data: axum::extract::Form<UpdateInventoryItemData>) -> Markup {
    state   .db_connection
            .update_inventory_item(InventoryIngredient { inventory_id: data.inventory_id, ingredient_id: data.ingredient_id, amount: data.ingredient_amount.clone() })
            .await
            .unwrap_or_else(|_|log::warn!("Failed to update ingredient {} in inventory {}", data.ingredient_id, data.inventory_id));

        let ingredient_name = state.db_connection
                                .get_ingredient_from_string_reference(data.ingredient_id.to_string())
                                .await
                                .unwrap_or_default()
                                .name;
        let ingredient = IngredientWithWeight{ ingredient_id: data.ingredient_id, name: ingredient_name, amount: data.ingredient_amount.clone() };

        ingredient.format_for_ingredient_table(InventoryHeaderData { inventory_id: data.inventory_id, filter_text: data.filter_text.clone() })
}

pub async fn handle_ingredient_delete(State(state): State<MyAppState>, data: axum::extract::Form<UpdateInventoryItemData>) -> Markup {
    // TODO: Delete ingredient
    state.db_connection
        .delete_inventory_item(InventoryIngredient{ inventory_id: data.inventory_id, ingredient_id: data.ingredient_id, amount: data.ingredient_amount.clone() })
        .await
        .unwrap_or_else(|_|log::warn!("Failed to delete ingredient {} from inventory {}", data.ingredient_id, data.inventory_id));
    (render_filtered_inventory_contents(State(state), data.inventory_id, data.filter_text.clone())).await
}

pub async fn handle_ingredient_commit(State(state): State<MyAppState>, data: Form<InventoryItemData>) -> Markup {
    let ingredient_id = state
        .db_connection
        .get_ingredient_from_string_reference(data.ingredient_name.clone())
        .await
        .unwrap_or(Ingredient {ingredient_id: -1, name: String::new(), energy: BigDecimal::from(-1), comment: None})
        .ingredient_id;

    dbg!(format!("requested name {} yielded ingredient id {}", data.ingredient_name, ingredient_id));
    if ingredient_id < 0 { 
        add_ingredient_form_with_header_data(InventoryHeaderData { inventory_id: data.inventory_id, filter_text: data.filter_text.clone() })
    }
    else {
        let Ok(res) = state.db_connection.update_inventory_item(InventoryIngredient {
                inventory_id: data.inventory_id,
                ingredient_id: ingredient_id, 
                amount: data.ingredient_amount.clone()
        }).await else {
            return return_to_inv_overview_error(data.inventory_id)
        };
        dbg!(res);
        (render_filtered_inventory_contents(State(state), data.inventory_id, data.filter_text.clone())).await
    }
}



pub async fn add_ingredient_form(Form(header_data): Form<InventoryHeaderData>) -> Markup
{
    add_ingredient_form_with_header_data(header_data)
}

pub fn add_ingredient_form_with_header_data(header_data: InventoryHeaderData) -> Markup
{
    html! {
        form hx-put="inventories/commit-ingredient" hx-target=(["#", INVENTORY_CONTENTS_DIV].concat()) hx-swap="outerHTML" {
            div class="flex flex-row items-center justify-center mb-2 gap-5 h-10 w-full"{
                h1 { "Add ingredient" }
                input type="hidden" name=(INVENTORY_ID) value=(header_data.inventory_id);
                input type="hidden" name=(FILTER_TEXT) value=(header_data.filter_text.unwrap_or_default());
                input type="hidden" name=(INGREDIENT_ID) value=(-1);
                input type="text" list=(INGREDIENTS_DATALIST) name=(INGREDIENT_NAME) placeholder="Ingredient" required="required" class="text";
                input type="number" name=(INGREDIENT_AMOUNT) placeholder="Amount" value="" step="0.01" min="0.05" required="required";
                button class="btn btn-primary" type="submit" { "Submit" }
            }
        }
    }
}

pub async fn commit_inventory(
    State(state): State<MyAppState>,
    form: Form<foodlib::Inventory>,
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

pub async fn handle_select(State(state): State<MyAppState>, header_data: axum::extract::Form<InventoryHeaderData>) -> Markup {
    manage_inventory_form(State(state.clone()), header_data.inventory_id).await
}

pub async fn handle_add_inventory(State(state): State<MyAppState>) -> Markup {
    add_or_edit_inventory_form(State(state), -1, String::new())
}

pub async fn handle_delete_inventory(State(state): State<MyAppState>, data: axum::extract::Form<InventoryHeaderData>) -> Markup {
    let Ok(_) = state.db_connection.delete_inventory(data.inventory_id).await else {
        return return_to_inv_overview_error(data.inventory_id);
    };
    select_inventory_form(State(state)).await
}

pub async fn handle_manage(State(state): State<MyAppState>, data: axum::extract::Form<InventoryHeaderData>) -> Markup {
    render_filtered_inventory_contents(State(state), data.inventory_id, data.filter_text.clone()).await
}

#[axum::debug_handler]
pub async fn handle_edit_inventory(State(state): State<MyAppState>, data: axum::extract::Form<InventoryHeaderData>) -> Markup {
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
            (render_filtered_inventory_contents(State(state), selected_inventory_id, None).await)
        }
    }
}

pub async fn render_filtered_inventory_contents(
    State(state): State<MyAppState>,
    inventory_id: i32,
    filter: Option<String>,
) -> Markup {
    let contents = state
        .db_connection
        .get_filtered_inventory_contents(inventory_id, filter.clone())
        .await
        .unwrap_or_default();
    dbg!(contents.clone());

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
                    tbody { @for item in contents { (item.format_for_ingredient_table( InventoryHeaderData { inventory_id: inventory_id, filter_text: filter.clone() })) } }
                }
            }
            form hx-target="this" hx-put="/inventories/add-ingredient" hx-swap="outerHTML" {
                input type="hidden" name=(INVENTORY_ID) value=(inventory_id);
                input type="hidden" name=(FILTER_TEXT) value=(filter.unwrap_or_default());
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
    fn format_for_ingredient_table(&self, header_data: InventoryHeaderData) -> Markup;
}

impl IngredientTableFormattable for IngredientWithWeight {
    fn format_for_ingredient_table(&self, header_data: InventoryHeaderData) -> Markup {
        let form_id = format!("ingredient-{}-form", self.ingredient_id);
        html! {
            tr id=(format!("ingredient-{}", self.ingredient_id)) { // TODO: Put into form
                td { 
                    input class=(form_id) type="hidden" name=(INVENTORY_ID) value=(header_data.inventory_id);
                    input class=(form_id) type="hidden" name=(FILTER_TEXT) value=(header_data.filter_text.unwrap_or_default());
                    input class=(form_id) type="hidden" name=(INGREDIENT_ID) value=(self.ingredient_id);
                    div class=(format!("text w-full {}",form_id)) type="text" name=(INGREDIENT_NAME) { (self.name) } 
                }
                td { input class=(form_id) type="number" name="ingredient_amount" value=(self.amount) required="required" hx-put="inventories/change-ingredient-amount" hx-target="" hx-include=(format!(".{}", form_id)) hx-trigger="keyup[keyCode==13]" hx-swap="innerHTML"; }
                td { button hx-include=(format!(".{}", form_id)) class="btn btn-primary" hx-put="inventories/delete-ingredient" type="submit" hx-target=(format!("#{}", INVENTORY_CONTENTS_DIV)) hx-swap="innerHTML" { "X" } }
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