use axum::extract::State;
use maud::{html, Markup};

use crate::MyAppState;

pub(crate) fn inventories_router() -> axum::Router<MyAppState> {
    axum::Router::new()
        .route("/search", axum::routing::post(search))
        .route("/edit", axum::routing::put(add_inventory))
        .route("/add", axum::routing::get(edit_inventory_form))
        .route("/", axum::routing::get(inventories_view))
}

pub async fn search(State(state): State<MyAppState>, query: String) -> Markup {
    let query = query.replace("search=", "").to_lowercase();
    let inventories = state
        .db_connection
        .get_inventories()
        .await
        .unwrap_or_default();

    let filtered_inventories = inventories
        .iter()
        .filter(|x| x.name.to_lowercase().contains(&query));

    html! {
        @for inventory in filtered_inventories {
            (format_inventory(inventory))
        }
    }
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

    inventories_view(State(state)).await
}

pub async fn inventories_view(State(state): State<MyAppState>) -> Markup {
    let inventories = state
        .db_connection
        .get_inventories()
        .await
        .unwrap_or_default();

    html! {
        div id="inventories" class="flex flex-col items-center justify-center gap-10" {
                div class="flex flex-row items-center justify-center mb-2 gap-5 h-10 w-full" hx-target="this"  hx-swap="outerHTML" {
                    button hx-get="/inventories/add" class="btn btn-primary"  { "+" };
                    select class="fc-select" {
                        option value="-1" { "Select inventory" };
                        @for inventory in inventories.iter() {
                            (format_inventory(inventory))
                        }
                    }
                    input type="search" placeholder="Search for ingredient" id="search" name="search" autocomplete="off"
                        autofocus="autofocus" hx-post="/inventories/search" hx-trigger="keyup changed delay:20ms, search"
                        hx-target="#search-results" hx-indicator=".htmx-indicator" class="text";
                    @for inventory in inventories.iter() {
                        (format_inventory(inventory))
                    }
            }
            span class="htmx-indicator" { "Searching..." }
            table class="text-inherit table-auto object-center" display="block" max-height="60vh" overflow-y="scroll" {
                thead { tr { th { "Name" } } }
                tbody id="search-results" {
                }
            }
            div hx-target="this"  hx-swap="outerHTML" {
                button hx-get="/inventories/add_ingredient" class="btn btn-primary"  { "+" }
            }
        }
    }
}

pub async fn edit_inventory_form(State(state): State<MyAppState>) -> Markup {
    html! {
        form hx-put="/inventories/edit" hx-target="#inventories" hx-swap="outerHTML" {
            div class="flex flex-col items-center justify-center gap-5" {
                div class="flex flex-row items-center justify-center mb-2 gap-5 h-10 w-full"{
                    h1 { "Edit inventory" }
                    input type="text" name="name" placeholder="Name" value="" required="required" class="text";
                    input type="hidden" name="inventory_id" value="-1";
                    button class="btn btn-primary" type="submit" { "Submit" }
                }
            }
        }
    }
}

fn format_inventory(inventory: &foodlib::Inventory) -> Markup {
    html! {
        option value=(format!("{}", inventory.inventory_id)) { (inventory.name) };
    }
}
