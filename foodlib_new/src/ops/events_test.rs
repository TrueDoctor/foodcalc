use crate::entities::event::*;
use crate::error::Error;
use crate::ops::events::EventOps;
use bigdecimal::BigDecimal;
use pretty_assertions::assert_eq;
use sqlx::PgPool;
use std::str::FromStr;
use std::sync::Arc;
use time::macros::datetime;

#[sqlx::test(fixtures("minimal", "complex_events"))]
async fn test_shopping_list_with_source_overrides(pool: PgPool) {
    let pool = Arc::new(pool);
    let ops = EventOps::new(pool.clone());

    // Debug the recipe hierarchy
    println!("\nRecipe hierarchy (meta_recipes):");
    let meta_recipes =
        sqlx::query!("SELECT * FROM meta_recipes WHERE child_id = 100 OR parent_id = 100")
            .fetch_all(&*pool)
            .await
            .unwrap();
    println!("{:#?}", meta_recipes);

    // Debug event_meals
    println!("\nEvent meals:");
    let meals = sqlx::query!("SELECT * FROM event_meals WHERE event_id = 100")
        .fetch_all(&*pool)
        .await
        .unwrap();
    println!("{:#?}", meals);

    // Debug shopping_tour_ingredients view
    println!("\nShopping tour ingredients view:");
    let tour_ingredients =
        sqlx::query!("SELECT * FROM shopping_tour_ingredients WHERE tour_id = 100")
            .fetch_all(&*pool)
            .await
            .unwrap();
    println!("{:#?}", tour_ingredients);

    // Debug the event_ingredients view
    println!("\nEvent ingredients view:");
    let event_ingredients = sqlx::query!("SELECT * FROM event_ingredients WHERE event_id = 100")
        .fetch_all(&*pool)
        .await
        .unwrap();
    println!("{:#?}", event_ingredients);

    // Debug resolved recipes
    println!("\nResolved recipes:");
    let resolved = sqlx::query!(
        "SELECT * FROM resolved_recipes WHERE recipe_id IN (SELECT recipe_id FROM event_meals WHERE event_id = 100)"
    )
    .fetch_all(&*pool)
    .await.unwrap();
    println!("{:#?}", resolved);

    // Get shopping list for festival's first shopping tour
    let items = ops.get_shopping_list(100).await.unwrap();

    println!("\nShopping list items via API:");
    for item in &items {
        println!(
            "- {} (ID: {}): {}kg at ${}",
            item.ingredient_name,
            item.ingredient_id,
            item.weight,
            item.price.clone().unwrap()
        );
    }

    let tomatoes = items
        .iter()
        .find(|i| i.ingredient_id == 2)
        .expect("Tomatoes should be in shopping list");

    assert_eq!(
        *tomatoes.price.as_ref().unwrap(),
        BigDecimal::from_str("1.99").unwrap() * &tomatoes.weight
    );
}

#[sqlx::test(fixtures("minimal", "complex_events"))]
async fn test_prep_tasks_affect_shopping_lists(pool: PgPool) {
    let ops = EventOps::new(Arc::new(pool));

    // Both events prepare the same base sauce recipe
    // Shopping lists should reflect the prep dates rather than meal dates

    // Festival prep shopping (tour 100 on June 29)
    let items = ops.get_shopping_list(100).await.unwrap();
    let sauce_ingredients: Vec<_> = items.iter().filter(|i| i.tour_id == Some(100)).collect();
    assert!(!sauce_ingredients.is_empty());

    // Main festival shopping (tour 101 on July 1) should not include sauce ingredients
    let items = ops.get_shopping_list(101).await.unwrap();
    let sauce_ingredients: Vec<_> = items
        .iter()
        .filter(|i| i.ingredient_id == 2) // tomatoes
        .collect();
    assert!(sauce_ingredients.is_empty());
}

#[sqlx::test(fixtures("minimal", "complex_events"))]
async fn test_upcoming_and_past_events(pool: PgPool) {
    let ops = EventOps::new(Arc::new(pool));

    // Check around July 1, 2024
    let date = datetime!(2024-07-01 00:00:00 UTC);

    let upcoming = ops.get_upcoming_events(date).await.unwrap();
    assert!(upcoming.iter().any(|e| e.id == 100)); // Festival ongoing
    assert!(upcoming.iter().any(|e| e.id == 101)); // Week 28 prep upcoming

    // Check after all events
    let date = datetime!(2024-07-09 00:00:00 UTC);
    let past = ops.get_past_events(date).await.unwrap();
    assert!(past.iter().any(|e| e.id == 100)); // Festival finished
    assert!(past.iter().any(|e| e.id == 101)); // Week 28 prep finished
}

#[sqlx::test(fixtures("minimal", "complex_events"))]
async fn test_event_crud_operations(pool: PgPool) {
    let ops = EventOps::new(Arc::new(pool));

    // Create
    let event = ops
        .create(Event {
            id: -1,
            name: "Test Event".to_string(),
            comment: Some("Test Comment".to_string()),
            budget: Some(BigDecimal::from(500)),
            owner_id: 1,
        })
        .await
        .unwrap();
    assert!(event.id > 0);

    // Read
    let fetched = ops.get(event.id).await.unwrap();
    assert_eq!(fetched.name, "Test Event");

    // Update
    let updated = ops
        .update(Event {
            name: "Updated Event".to_string(),
            ..event.clone()
        })
        .await
        .unwrap();
    assert_eq!(updated.name, "Updated Event");

    // Delete
    ops.delete(event.id).await.unwrap();
    assert!(matches!(
        ops.get(event.id).await,
        Err(Error::NotFound { .. })
    ));
}

#[sqlx::test(fixtures("minimal", "complex_events"))]
async fn test_shopping_tour_management(pool: PgPool) {
    let ops = EventOps::new(Arc::new(pool));

    let event_id = 100; // Summer Festival

    // Add new tour
    let tour = ops
        .add_shopping_tour(ShoppingTour {
            id: -1,
            event_id,
            tour_date: datetime!(2024-07-02 08:00:00 UTC),
            store_id: 1,
            store_name: None,
        })
        .await
        .unwrap();

    // Update tour
    let updated = ops
        .update_shopping_tour(ShoppingTour {
            store_id: 2, // Change to Farmers Market
            ..tour.clone()
        })
        .await
        .unwrap();
    assert_eq!(updated.store_id, 2);

    // Get all tours
    let tours = ops.get_shopping_tours(event_id).await.unwrap();
    assert!(tours.len() >= 3); // Two original + one new

    // Delete tour
    ops.delete_shopping_tour(tour.id).await.unwrap();
    let tours = ops.get_shopping_tours(event_id).await.unwrap();
    assert!(!tours.iter().any(|t| t.id == tour.id));
}

#[sqlx::test(fixtures("minimal", "complex_events"))]
async fn test_source_override_management(pool: PgPool) {
    let ops = EventOps::new(Arc::new(pool));

    // Add new override
    let override_ = ops.set_source_override(101, 100).await.ok();
    assert_eq!(
        override_,
        Some(SourceOverride {
            event_id: 101,
            source_id: 100
        })
    );

    // Get all overrides
    let overrides = ops.get_source_overrides(101).await.unwrap();
    assert!(overrides.iter().any(|o| o.source_id == 100));

    // Delete override
    ops.delete_source_override(101, 100).await.unwrap();
    let overrides = ops.get_source_overrides(101).await.unwrap();
    assert!(!overrides.iter().any(|o| o.source_id == 100));
}
