use axum::Router;
use tokio::net::TcpListener;

mod events;
mod ingredients;
mod places;
mod reciepes;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .nest("/events", events::router())
        .nest("/ingredients", ingredients::router())
        .nest("/places", places::router())
        .nest("/reciepes", reciepes::router());

    println!("Server started successfully at 0.0.0.0:8080");

    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
