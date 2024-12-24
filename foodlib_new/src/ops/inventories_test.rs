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
    };

    let created = ops.create_inventory(inventory.clone()).await.unwrap();
    assert_eq!(created.name, inventory.name);
    assert!(created.id > 0);

    // Verify it exists in DB
    let fetched = ops.get_inventory(created.id).await.unwrap();
    assert_eq!(fetched, created);
}

#[sqlx::test(fixtures("../fixtures/minimal.sql"))]
async fn test_update_inventory(pool: sqlx::PgPool) {
    let ops = InventoryOps::new(pool.into());

    // Create a test inventory to update
    let inventory = ops
        .create_inventory(Inventory {
            id: -1,
            name: "Test Inventory".to_string(),
        })
        .await
        .unwrap();

    let updated = ops
        .update_inventory(Inventory {
            id: inventory.id,
            name: "Updated Inventory".to_string(),
        })
        .await
        .unwrap();

    assert_eq!(updated.name, "Updated Inventory");

    // Verify changes persisted
    let fetched = ops.get_inventory(inventory.id).await.unwrap();
    assert_eq!(fetched, updated);
}

#[sqlx::test(fixtures("../fixtures/minimal.sql"))]
async fn test_delete_inventory(pool: sqlx::PgPool) {
    let ops = InventoryOps::new(pool.into());

    // Create temporary inventory to delete
    let inventory = ops
        .create_inventory(Inventory {
            id: -1,
            name: "To Delete".to_string(),
        })
        .await
        .unwrap();

    ops.delete_inventory(inventory.id).await.unwrap();

    let err = ops.get_inventory(inventory.id).await.unwrap_err();
    assert!(matches!(err, Error::Database(_)));
}

#[sqlx::test(fixtures("../fixtures/minimal.sql"))]
async fn test_list_inventories(pool: sqlx::PgPool) {
    let ops = InventoryOps::new(pool.into());

    let inventories = ops.get_all_inventories().await.unwrap();
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

    let created = ops.add_inventory_item(item.clone()).await.unwrap();
    assert_eq!(created, item);

    // Test updating item
    let mut updated = created.clone();
    updated.amount = BigDecimal::from(200);
    let result = ops.update_inventory_item(updated.clone()).await.unwrap();
    assert_eq!(result, updated);

    // Test fetching items
    let items = ops.get_inventory_items(1).await.unwrap();
    assert!(items.contains(&updated));

    // Test deleting item
    ops.delete_inventory_item(1, 1).await.unwrap();
    let items = ops.get_inventory_items(1).await.unwrap();
    assert!(!items.contains(&item));
}
