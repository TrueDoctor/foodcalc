use crate::api::Router;

mod home;
mod ingredients_tab;

pub fn frontend_router() -> Router {
    Router::new()
        .nest("/", home::home_router())
        .nest("/ingredients", ingredients_tab::ingredients_router())
}
