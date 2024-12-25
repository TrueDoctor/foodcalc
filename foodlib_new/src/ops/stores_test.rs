// foodlib_new/src/ops/stores_test.rs

use pretty_assertions::assert_eq;

use crate::{entities::store::*, error::Error, ops::stores::StoreOps};

#[sqlx::test(fixtures("../fixtures/minimal.sql"))]
async fn test_create_store(pool: sqlx::PgPool) {
    let ops = StoreOps::new(pool.into());

    let store = Store {
        id: -1,
        name: "Test Store".to_string(),
    };

    let created = ops.create(store.clone()).await.unwrap();
    assert_eq!(created.name, store.name);
    assert!(created.id > 0);

    // Verify it exists in DB
    let fetched = ops.get(created.id).await.unwrap();
    assert_eq!(fetched, created);
}

#[sqlx::test(fixtures("../fixtures/minimal.sql"))]
async fn test_get_store(pool: sqlx::PgPool) {
    let ops = StoreOps::new(pool.into());

    // Get existing store from fixture
    let store = ops.get(1).await.unwrap();
    assert_eq!(store.name, "Local Grocery");

    // Test getting non-existent store
    let err = ops.get(9999).await.unwrap_err();
    assert!(matches!(err, Error::Database(_)));
}

#[sqlx::test(fixtures("../fixtures/minimal.sql"))]
async fn test_get_by_name(pool: sqlx::PgPool) {
    let ops = StoreOps::new(pool.into());

    let store = ops.get_by_name("Local Grocery").await.unwrap();
    assert_eq!(store.id, 1);

    let err = ops.get_by_name("NonExistentStore").await.unwrap_err();
    assert!(matches!(err, Error::Database(_)));
}

#[sqlx::test(fixtures("../fixtures/minimal.sql"))]
async fn test_update_store(pool: sqlx::PgPool) {
    let ops = StoreOps::new(pool.into());

    // Create a test store to update
    let store = ops
        .create(Store {
            id: -1,
            name: "Test Store".to_string(),
        })
        .await
        .unwrap();

    let updated = ops
        .update(Store {
            id: store.id,
            name: "Updated Store".to_string(),
        })
        .await
        .unwrap();

    assert_eq!(updated.name, "Updated Store");

    // Verify changes persisted
    let fetched = ops.get(store.id).await.unwrap();
    assert_eq!(fetched, updated);
}

#[sqlx::test(fixtures("../fixtures/minimal.sql"))]
async fn test_delete_store(pool: sqlx::PgPool) {
    let ops = StoreOps::new(pool.into());

    // Create temporary store to delete
    let store = ops
        .create(Store {
            id: -1,
            name: "To Delete".to_string(),
        })
        .await
        .unwrap();

    // Delete and verify cascading deletes worked
    ops.delete(store.id).await.unwrap();

    let err = ops.get(store.id).await.unwrap_err();
    assert!(matches!(err, Error::Database(_)));
}

#[sqlx::test(fixtures("../fixtures/minimal.sql"))]
async fn test_list_stores(pool: sqlx::PgPool) {
    let ops = StoreOps::new(pool.into());

    let stores = ops.list().await.unwrap();
    assert!(!stores.is_empty());

    // Verify stores from fixture are present
    let local = stores.iter().find(|s| s.name == "Local Grocery").unwrap();
    assert_eq!(local.id, 1);

    let farmers = stores.iter().find(|s| s.name == "Farmers Market").unwrap();
    assert_eq!(farmers.id, 2);
}
