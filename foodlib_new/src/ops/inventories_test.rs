// inventories_test.rs

use crate::{entities::inventory::*, error::Error, ops::inventories::InventoryOps};
use bigdecimal::BigDecimal;
use pretty_assertions::assert_eq;

#[sqlx::test(fixtures("../fixtures/minimal.sql"))]
async fn test_create_inventory(pool: sqlx::PgPool) {
    let ops = InventoryOps::new(pool.into());

    let inventory = Inventory {
        id: -1,
        name: "Test Inventory".to_string(),
        owner_id: 1,
    };

    let created = ops.create(inventory.clone()).await.unwrap();
    assert_eq!(created.name, inventory.name);
    assert!(created.id > 0);

    // Verify it exists in DB
    let fetched = ops.get(created.id).await.unwrap();
    assert_eq!(fetched, created);
}

#[sqlx::test(fixtures("../fixtures/minimal.sql"))]
async fn test_update_inventory(pool: sqlx::PgPool) {
    let ops = InventoryOps::new(pool.into());

    // Create a test inventory to update
    let inventory = ops
        .create(Inventory {
            id: -1,
            name: "Test Inventory".to_string(),
            owner_id: 1,
        })
        .await
        .unwrap();

    let updated = ops
        .update(Inventory {
            id: inventory.id,
            name: "Updated Inventory".to_string(),
            owner_id: 1,
        })
        .await
        .unwrap();

    assert_eq!(updated.name, "Updated Inventory");

    // Verify changes persisted
    let fetched = ops.get(inventory.id).await.unwrap();
    assert_eq!(fetched, updated);
}

#[sqlx::test(fixtures("../fixtures/minimal.sql"))]
async fn test_delete_inventory(pool: sqlx::PgPool) {
    let ops = InventoryOps::new(pool.into());

    // Create temporary inventory to delete
    let inventory = ops
        .create(Inventory {
            id: -1,
            name: "To Delete".to_string(),
            owner_id: 1,
        })
        .await
        .unwrap();

    ops.delete(inventory.id).await.unwrap();

    let err = ops.get(inventory.id).await.unwrap_err();
    assert!(matches!(err, Error::Database(_)));
}

#[sqlx::test(fixtures("../fixtures/minimal.sql"))]
async fn test_list_inventories(pool: sqlx::PgPool) {
    let ops = InventoryOps::new(pool.into());

    let inventories = ops.list().await.unwrap();
    assert!(!inventories.is_empty());

    // Verify inventories from fixture are present
    let main_pantry = inventories
        .iter()
        .find(|i| i.name == "Main Pantry")
        .unwrap();
    assert_eq!(main_pantry.id, 1);

    let backup_storage = inventories
        .iter()
        .find(|i| i.name == "Backup Storage")
        .unwrap();
    assert_eq!(backup_storage.id, 2);
}

#[sqlx::test(fixtures("../fixtures/minimal.sql"))]
async fn test_inventory_items(pool: sqlx::PgPool) {
    let ops = InventoryOps::new(pool.into());

    // Test adding item
    let item = InventoryItem {
        inventory_id: 1,
        ingredient_id: 1,
        amount: BigDecimal::from(100),
    };

    let created = ops.add_item(item.clone()).await.unwrap();
    assert_eq!(created, item);

    // Test updating item
    let mut updated = created.clone();
    updated.amount = BigDecimal::from(200);
    let result = ops.update_item(updated.clone()).await.unwrap();
    assert_eq!(result, updated);

    // Test fetching items
    let items = ops.get_items(1).await.unwrap();
    let mut updated: InventoryItemWithName = updated.into();
    updated.name = "Pasta".into();
    assert!(items.contains(&updated));

    // Test deleting item
    ops.delete_item(1, 1).await.unwrap();
    let items = ops.get_items(1).await.unwrap();
    assert!(!items.iter().any(|x| x.ingredient_id == item.ingredient_id));
}
