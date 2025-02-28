use crate::FoodLib;
use axum::{
    extract::Form,
    routing::{get, put},
};
use axum_login::login_required;
use bigdecimal::BigDecimal;
use foodlib_new::{
    auth::AuthBackend,
    ingredient::Ingredient,
    inventory::{Inventory, InventoryItem, InventoryItemWithName},
    user::User,
};
use maud::{html, Markup};
use serde::Deserialize;

use super::MResponse;
use crate::frontend::LOGIN_URL;
use crate::MyAppState;

pub(crate) fn inventories_router() -> axum::Router<MyAppState> {
    axum::Router::new()
        .route("/commit", put(commit_inventory))
        .route("/add-inventory", get(handle_add_inventory))
        .route("/edit-inventory", put(handle_edit_inventory))
        .route("/", get(select_inventory_form))
        .route("/select", put(handle_select))
        .route("/manage", put(handle_manage))
        .route("/delete-inventory", put(handle_delete_inventory))
        .route("/add-ingredient", put(add_ingredient_form))
        .route("/commit-ingredient", put(handle_ingredient_commit))
        .route("/delete-ingredient", put(handle_ingredient_delete))
        .route("/change-ingredient-amount", put(handle_ingredient_change))
        .route("/abort", put(handle_abort))
        .route_layer(login_required!(AuthBackend, login_url = LOGIN_URL))
}

// Request parameters
static INVENTORY_ID: &str = "inventory_id";
static FILTER_TEXT: &str = "filter_text";
static INGREDIENT_ID: &str = "ingredient_id";
static INGREDIENT_NAME: &str = "ingredient_name";
static INGREDIENT_AMOUNT: &str = "ingredient_amount";

// htmx ids
static SEARCH_RESULTS_DIV: &str = "search-results";
static INVENTORIES_DIV: &str = "inventories";
static INVENTORY_CONTENTS_DIV: &str = "contents";
static INGREDIENTS_DATALIST: &str = "ingredients";

#[derive(Debug, Deserialize)]
pub struct InventoryHeaderData {
    pub inventory_id: i32,
    pub filter_text: Option<String>,
}

impl Default for InventoryHeaderData {
    fn default() -> Self {
        InventoryHeaderData {
            inventory_id: -1,
            filter_text: None,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct InventoryItemData {
    pub inventory_id: i32,
    pub filter_text: Option<String>,
    pub ingredient_id: i32,
    pub ingredient_name: String,
    pub ingredient_amount: BigDecimal,
}

impl Default for InventoryItemData {
    fn default() -> Self {
        InventoryItemData {
            inventory_id: -1,
            filter_text: None,
            ingredient_id: -1,
            ingredient_name: String::new(),
            ingredient_amount: BigDecimal::from(0),
        }
    }
}

impl From<InventoryItemData> for InventoryItem {
    fn from(item: InventoryItemData) -> Self {
        InventoryItem {
            inventory_id: item.inventory_id,
            ingredient_id: item.ingredient_id,
            amount: item.ingredient_amount,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateInventoryItemData {
    pub inventory_id: i32,
    pub filter_text: Option<String>,
    pub ingredient_id: i32,
    pub ingredient_amount: BigDecimal,
}

impl From<UpdateInventoryItemData> for InventoryItem {
    fn from(item: UpdateInventoryItemData) -> Self {
        InventoryItem {
            inventory_id: item.inventory_id,
            ingredient_id: item.ingredient_id,
            amount: item.ingredient_amount,
        }
    }
}

async fn handle_ingredient_change(
    foodlib: FoodLib,
    Form(data): Form<UpdateInventoryItemData>,
) -> MResponse {
    let item: InventoryItem = data.clone().into();
    let updated = foodlib.inventories().update_item(item).await?;

    // Get ingredient name for display
    let ingredient = foodlib.ingredients().get(updated.ingredient_id).await?;

    let ingredient_with_weight = InventoryItemWithName {
        name: ingredient.name,
        ..updated.into()
    };

    Ok(
        ingredient_with_weight.format_for_ingredient_table(InventoryHeaderData {
            inventory_id: data.inventory_id,
            filter_text: data.filter_text.clone(),
        }),
    )
}

async fn handle_ingredient_delete(
    foodlib: FoodLib,
    Form(data): Form<UpdateInventoryItemData>,
) -> MResponse {
    foodlib
        .inventories()
        .delete_item(data.inventory_id, data.ingredient_id)
        .await?;

    render_filtered_inventory_contents(foodlib, data.inventory_id, data.filter_text.clone()).await
}

async fn handle_ingredient_commit(
    foodlib: FoodLib,
    Form(data): Form<InventoryItemData>,
) -> MResponse {
    // Try to find the ingredient by name
    let Ok(ingredient) = foodlib
        .ingredients()
        .get_by_name(&data.ingredient_name)
        .await
    else {
        return Ok(add_ingredient_form_with_header_data(InventoryHeaderData {
            inventory_id: data.inventory_id,
            filter_text: data.filter_text.clone(),
        }));
    };
    let items = foodlib.inventories().get_items(data.inventory_id).await?;

    let item_in_inventory = items.iter().any(|x| x.ingredient_id == ingredient.id);
    // Create inventory item
    let item = InventoryItem {
        inventory_id: data.inventory_id,
        ingredient_id: ingredient.id,
        amount: data.ingredient_amount,
    };
    if item_in_inventory {
        foodlib.inventories().update_item(item).await?;
    } else {
        foodlib.inventories().add_item(item).await?;
    }
    render_filtered_inventory_contents(foodlib, data.inventory_id, data.filter_text.clone()).await
}

async fn handle_abort(Form(header_data): Form<InventoryHeaderData>) -> Markup {
    add_ingredient_button(header_data.inventory_id, header_data.filter_text.clone())
}

async fn add_ingredient_form(Form(header_data): Form<InventoryHeaderData>) -> Markup {
    add_ingredient_form_with_header_data(header_data)
}

fn add_ingredient_form_with_header_data(header_data: InventoryHeaderData) -> Markup {
    html! {
        div id="add-ingredient-div" class="gap-5 mb-2 flex flex-row items-center justify-center"{
            form hx-put="/inventories/commit-ingredient" id="add-ingredient-form" hx-target=(["#", INVENTORY_CONTENTS_DIV].concat()) hx-swap="outerHTML" {
                div class="flex flex-row items-center justify-center gap-5 h-10 w-full"{
                    h1 { "Add ingredient" }
                    input type="hidden" name=(INVENTORY_ID) value=(header_data.inventory_id);
                    input type="hidden" name=(FILTER_TEXT) value=(header_data.filter_text.clone().unwrap_or_default());
                    input type="hidden" name=(INGREDIENT_ID) value=(-1);
                    input type="text" list=(INGREDIENTS_DATALIST) name=(INGREDIENT_NAME) placeholder="Ingredient" required="required" class="text";
                    input class="text" type="text" name=(INGREDIENT_AMOUNT) placeholder="Amount (kg)" value="" step="0.01" min="0.05" required="required";
                    button class="btn btn-primary" type="submit" { "Submit" }
                }
            }
            form hx-put="/inventories/abort" hx-target=(["#", INVENTORY_CONTENTS_DIV].concat()) hx-swap="outerHTML" {
                input type="hidden" name=(INVENTORY_ID) value=(header_data.inventory_id);
                input type="hidden" name=(FILTER_TEXT) value=(header_data.filter_text.clone().unwrap_or_default());
                button class="btn btn-cancel" type="submit" { "Abort" }
            }
        }
    }
}

pub async fn commit_inventory(
    foodlib: FoodLib,
    user: User,
    Form(mut inventory): Form<Inventory>,
) -> MResponse {
    if inventory.id < 0 {
        // Creating new inventory
        inventory.owner_id = user.id;
        let inventory_id = foodlib.inventories().create(inventory).await?;
        manage_inventory_form(foodlib, inventory_id.id).await
    } else {
        // Updating existing inventory (keeping existing owner)
        let id = inventory.id;
        foodlib.inventories().update(inventory).await?;
        manage_inventory_form(foodlib, id).await
    }
}

pub async fn handle_select(foodlib: FoodLib, header_data: Form<InventoryHeaderData>) -> MResponse {
    manage_inventory_form(foodlib, header_data.inventory_id).await
}

pub async fn handle_add_inventory(user: User) -> Markup {
    add_or_edit_inventory_form(-1, String::new(), user.id)
}

pub async fn handle_delete_inventory(
    foodlib: FoodLib,
    data: Form<InventoryHeaderData>,
) -> MResponse {
    foodlib.inventories().delete(data.inventory_id).await?;
    select_inventory_form(foodlib).await
}

pub async fn handle_manage(foodlib: FoodLib, data: Form<InventoryHeaderData>) -> MResponse {
    render_filtered_inventory_contents(foodlib, data.inventory_id, data.filter_text.clone()).await
}

pub async fn handle_edit_inventory(
    foodlib: FoodLib,
    user: User,
    data: Form<InventoryHeaderData>,
) -> Markup {
    let inventory = foodlib.inventories().get(data.inventory_id).await.unwrap();
    add_or_edit_inventory_form(data.inventory_id, inventory.name, user.id)
}

pub async fn select_inventory_form(foodlib: FoodLib) -> MResponse {
    let inventories = foodlib.inventories().list().await?;

    Ok(html! {
        div class="flex flex-col items-center justify-center gap-5" id=(INVENTORIES_DIV) {
            form hx-put="/inventories/select" hx-target="this" hx-swap="outerHTML" hx-trigger="change" {
                div class="flex flex-row items-center justify-center mb-2 gap-5 h-10 w-full" {
                    button hx-get="/inventories/add-inventory" class="btn btn-primary" hx-target=(["#", INVENTORIES_DIV].concat()) { "Add Inventory (+)" };
                    select class="fc-select" name=(INVENTORY_ID) hx-indicator=".htmx-indicator" {
                        option value="-1" selected { "Select inventory" };
                        @for inventory in inventories.iter() { (inventory.format_for_select(-1)) }
                    }
                }
            }
        }
    })
}

pub async fn manage_inventory_form(foodlib: FoodLib, selected_inventory_id: i32) -> MResponse {
    let inventories = foodlib.inventories().list().await?;

    Ok(html! {
        div class="flex flex-col items-center justify-center gap-5" id=(INVENTORIES_DIV) {
            div class="flex flex-col items-center justify-center gap-5" id=(INVENTORIES_DIV) {
                div class="flex flex-col items-center justify-center" id=(INVENTORIES_DIV) {
                    form class="w-full" hx-put="/inventories/manage" hx-target=(["#", INVENTORY_CONTENTS_DIV].concat()) hx-trigger="keyup" {
                        div class="flex flex-row items-center justify-between mb-2 h-10 w-full gap-5" {
                            button hx-get="/inventories/add-inventory" class="btn btn-primary" hx-target=(["#", INVENTORIES_DIV].concat()) { "Add Inventory (+)" };
                            div class="flex flex-row items-center gap-5" {
                                "Select Inventory:"
                                select class="fc-select" name=(INVENTORY_ID) hx-indicator=".htmx-indicator" hx-target=("#content") hx-trigger="change" hx-put="/inventories/select" {
                                    @for inventory in inventories.iter() { (inventory.format_for_select(selected_inventory_id)) }
                                }
                            }
                            button hx-put="/inventories/edit-inventory" class="btn btn-primary" hx-target=(["#", INVENTORIES_DIV].concat()) { "Edit Inventory" }
                            button hx-put="/inventories/delete-inventory" class="btn btn-cancel" hx-target=(["#", INVENTORIES_DIV].concat()) { "Delete Inventory" }
                        }
                        div class="h-10" {}
                        div class="flex flex-row items-center justify-stretch gap-5 h-10 w-full" {
                            input class="grow text h-full" type="search" placeholder="Search for Ingredient" id="search" name=(FILTER_TEXT);

                        }
                    }
                }(render_filtered_inventory_contents(foodlib, selected_inventory_id, None).await?)
            }
        }
    })
}

pub async fn render_filtered_inventory_contents(
    foodlib: FoodLib,
    inventory_id: i32,
    filter: Option<String>,
) -> MResponse {
    let mut contents = if inventory_id > 0 {
        foodlib.inventories().get_items(inventory_id).await?
    } else {
        Vec::new()
    };
    if let Some(ref filter) = filter {
        contents.retain(|x| x.name.contains(filter));
    }

    let ingredient_list = foodlib.ingredients().list().await?;

    Ok(html! {
        div id=(INVENTORY_CONTENTS_DIV) class="flex flex-col items-center justify-center mb-16 w-full"{
            (add_ingredient_button(inventory_id, filter.clone()))
            div id=(SEARCH_RESULTS_DIV) class="w-full" {
                datalist id=(INGREDIENTS_DATALIST) { @for ingredient in ingredient_list { (ingredient.format_for_datalist()) } }
                span class="htmx-indicator" { "Searching..." }
                table class="text-inherit table-auto object-center table-fixed" padding="0 0.5em" display="block"
                max-height="60vh" overflow-y="scroll" {
                    thead { tr { th { "Name" } th { "Amount (kg)" } th { "Delete" } } }
                    tbody { @for item in contents { (item.format_for_ingredient_table(InventoryHeaderData { inventory_id, filter_text: filter.clone()})) } }
                }
            }
        }
    })
}

pub fn add_ingredient_button(inventory_id: i32, filter: Option<String>) -> Markup {
    html! {
        form hx-target="this" hx-put="/inventories/add-ingredient" hx-swap="outerHTML" style="margin-bottom: 0px;"{
            input type="hidden" name=(INVENTORY_ID) value=(inventory_id);
            input type="hidden" name=(FILTER_TEXT) value=(filter.unwrap_or_default().clone());
            button type="submit" class="btn btn-primary"  { "Add Ingredient (+)" }
        }
    }
}

pub fn add_or_edit_inventory_form(
    inventory_id: i32,
    inventory_name: String,
    owner_id: i64,
) -> Markup {
    html! {
        form hx-put="/inventories/commit" hx-target="this" hx-swap="outerHTML" {
            div class="flex flex-col items-center justify-center gap-5" {
                div class="flex flex-row items-center justify-center mb-2 gap-5 h-10 w-full"{
                    h1 { @if inventory_id > 0 { "Edit inventory" } @else { "Add inventory" } }
                    input type="text" name="name" placeholder="Name" value=(inventory_name) required="required" class="text";
                    input type="hidden" name="id" value=(inventory_id);
                    input type="hidden" name="owner_id" value=(owner_id);
                    button class="btn btn-primary" type="submit" { "Submit" }
                }
            }
        }
    }
}

// Trait implementations for formatter functions

trait SelectFormattable {
    fn format_for_select(&self, selected_index: i32) -> Markup;
}

impl SelectFormattable for Inventory {
    fn format_for_select(&self, selected_index: i32) -> Markup {
        html! {
            @if selected_index == self.id {
                option selected value=(format!("{}", self.id)) { (self.name) };
            }
            @else {
                option value=(format!("{}", self.id)) { (self.name) };
            }
        }
    }
}

trait IngredientTableFormattable {
    fn format_for_ingredient_table(&self, header_data: InventoryHeaderData) -> Markup;
}

impl IngredientTableFormattable for InventoryItemWithName {
    fn format_for_ingredient_table(&self, header_data: InventoryHeaderData) -> Markup {
        let form_id = format!("ingredient-{}-form", self.ingredient_id);
        html! {
            tr id=(format!("ingredient-{}", self.ingredient_id)) style="text-align:center"{
                td style="text-align:left" {
                    input class=(form_id) type="hidden" name=(INVENTORY_ID) value=(header_data.inventory_id);
                    input class=(form_id) type="hidden" name=(FILTER_TEXT) value=(header_data.filter_text.unwrap_or_default());
                    input class=(form_id) type="hidden" name=(INGREDIENT_ID) value=(self.ingredient_id);
                    div class=(format!("w-full {}",form_id)) name=(INGREDIENT_NAME) { (self.name) }
                }
                td {
                    input class=(format!("text {}",form_id)) name="ingredient_amount" value=(self.amount) required="required" hx-put="/inventories/change-ingredient-amount" hx-target=(format!("#ingredient-{}", self.ingredient_id)) hx-include=(format!(".{}", form_id)) hx-trigger="keyup[keyCode==13]" hx-swap="outerHTML";
                }
                td { button hx-include=(format!(".{}", form_id)) class="btn btn-cancel" hx-put="/inventories/delete-ingredient" type="submit" hx-target=(format!("#{}", INVENTORY_CONTENTS_DIV)) hx-swap="innerHTML" { "X" } }
            }
        }
    }
}

trait DataListFormattable {
    fn format_for_datalist(&self) -> Markup;
}

impl DataListFormattable for Ingredient {
    fn format_for_datalist(&self) -> Markup {
        html! {
            option value=(self.name) { }
        }
    }
}
