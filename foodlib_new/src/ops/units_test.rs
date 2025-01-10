// foodlib_new/src/ops/units_test.rs

use bigdecimal::BigDecimal;
use pretty_assertions::assert_eq;
use std::str::FromStr;

use crate::{entities::unit::*, error::Error, ops::units::UnitOps};

#[sqlx::test(fixtures("../fixtures/minimal.sql"))]
async fn test_create_unit(pool: sqlx::PgPool) {
    let ops = UnitOps::new(pool.into());

    let unit = Unit {
        id: -1,
        name: "Test Unit".to_string(),
    };

    let created = ops.create(unit.clone()).await.unwrap();
    assert_eq!(created.name, unit.name);
    assert!(created.id > 0);

    // Verify it exists in DB
    let fetched = ops.get(created.id).await.unwrap();
    assert_eq!(fetched, created);
}

#[sqlx::test(fixtures("../fixtures/minimal.sql"))]
async fn test_get_unit(pool: sqlx::PgPool) {
    let ops = UnitOps::new(pool.into());

    // Get existing unit from fixture (kg with id 0)
    let unit = ops.get(0).await.unwrap();
    assert_eq!(unit.name, "kg");

    // Test getting non-existent unit
    let err = ops.get(9999).await.unwrap_err();
    assert!(matches!(err, Error::Database(_)));
}

#[sqlx::test(fixtures("../fixtures/minimal.sql"))]
async fn test_update_unit(pool: sqlx::PgPool) {
    let ops = UnitOps::new(pool.into());

    // Create a test unit to update
    let unit = ops
        .create(Unit {
            id: -1,
            name: "Test Unit".to_string(),
        })
        .await
        .unwrap();

    let updated = ops
        .update(Unit {
            id: unit.id,
            name: "Updated Unit".to_string(),
        })
        .await
        .unwrap();

    assert_eq!(updated.name, "Updated Unit");

    // Verify changes persisted
    let fetched = ops.get(unit.id).await.unwrap();
    assert_eq!(fetched, updated);
}

#[sqlx::test(fixtures("../fixtures/minimal.sql"))]
async fn test_delete_unit(pool: sqlx::PgPool) {
    let ops = UnitOps::new(pool.into());

    // Create temporary unit to delete
    let unit = ops
        .create(Unit {
            id: -1,
            name: "To Delete".to_string(),
        })
        .await
        .unwrap();

    // Add a test conversion
    let conversion = UnitConversion {
        from_unit: unit.id,
        to_unit: 0, // kg
        from_amount: BigDecimal::from(1),
        to_amount: BigDecimal::from(1),
    };
    ops.add_conversion(conversion).await.unwrap();

    // Delete and verify cascading deletes worked
    ops.delete(unit.id).await.unwrap();

    let err = ops.get(unit.id).await.unwrap_err();
    assert!(matches!(err, Error::Database(_)));
}

#[sqlx::test(fixtures("../fixtures/minimal.sql"))]
async fn test_list_units(pool: sqlx::PgPool) {
    let ops = UnitOps::new(pool.into());

    let units = ops.list().await.unwrap();
    assert!(!units.is_empty());

    // Verify units from fixture are present
    let kg = units.iter().find(|u| u.name == "kg").unwrap();
    assert_eq!(kg.id, 0);

    let g = units.iter().find(|u| u.name == "g").unwrap();
    assert_eq!(g.id, 1);
}

#[sqlx::test(fixtures("../fixtures/minimal.sql"))]
async fn test_unit_conversion_crud(pool: sqlx::PgPool) {
    let ops = UnitOps::new(pool.into());

    // Test adding conversion
    let conversion = UnitConversion {
        from_unit: 5, // slice
        to_unit: 0,   // kg
        from_amount: BigDecimal::from(1000),
        to_amount: BigDecimal::from(1),
    };

    let created = ops.add_conversion(conversion.clone()).await.unwrap();
    assert_eq!(created.from_unit, conversion.from_unit);
    assert_eq!(created.to_amount, conversion.to_amount);

    // Test updating conversion
    let mut updated = conversion.clone();
    updated.from_amount = BigDecimal::from(500);
    updated.to_amount = BigDecimal::from_str("0.5").unwrap();

    let result = ops.update_conversion(updated.clone()).await.unwrap();
    assert_eq!(result, updated);

    ops.refresh_conversions()
        .await
        .expect("failed to refresh conversions");

    let conversions = ops.get_all_conversions().await.unwrap();

    println!("Available conversions:");
    for conv in &conversions {
        println!(
            "From unit {} to unit {}: {} -> {}",
            conv.from_unit, conv.to_unit, conv.from_amount, conv.to_amount
        );
    }
    // Test fetching conversion
    let fetched = ops.get_conversion(5, 0).await.unwrap();
    assert!(fetched.is_some());
    // Conversion factor should be 0.001 (1kg = 1000g)
    assert_eq!(fetched.unwrap(), BigDecimal::from_str("0.001").unwrap());

    // Test deleting conversion
    ops.delete_conversion(5, 0).await.unwrap();
    ops.refresh_conversions()
        .await
        .expect("failed to refresh conversions");

    let conversions = ops.get_all_conversions().await.unwrap();
    assert!(!conversions
        .iter()
        .any(|c| c.from_unit == 5 && c.to_unit == 0));
}

#[sqlx::test(fixtures("../fixtures/minimal.sql"))]
async fn test_conversion_calculations(pool: sqlx::PgPool) {
    let ops = UnitOps::new(pool.into());

    // Refresh materialized view after any conversion changes
    ops.refresh_conversions().await.unwrap();

    let conversions = ops.get_all_conversions().await.unwrap();
    println!("Available conversions:");
    for conv in &conversions {
        println!(
            "From unit {} to unit {}: {} -> {}",
            conv.from_unit, conv.to_unit, conv.from_amount, conv.to_amount
        );
    }

    // Test direct conversion (g to kg)
    let result = ops
        .convert_unit(
            BigDecimal::from(1000), // 1000g
            1,                      // from g
            0,                      // to kg
        )
        .await
        .unwrap();

    assert_eq!(result, BigDecimal::from(1)); // Should be 1kg

    // Test transitive conversion (g to L through kg)
    let result = ops
        .convert_unit(
            BigDecimal::from(1000), // 1000g
            1,                      // from g
            2,                      // to L
        )
        .await
        .unwrap();

    assert_eq!(result.round(3), BigDecimal::from_str("1.0").unwrap());

    // Test invalid conversion
    let err = ops
        .convert_unit(
            BigDecimal::from(1),
            9999, // invalid unit
            0,
        )
        .await
        .unwrap_err();

    assert!(matches!(err, Error::Validation { .. }));
}

#[sqlx::test(fixtures("../fixtures/minimal.sql"))]
async fn test_all_conversions(pool: sqlx::PgPool) {
    let ops = UnitOps::new(pool.into());

    // Refresh view first
    ops.refresh_conversions().await.unwrap();

    let conversions = ops.get_all_conversions().await.unwrap();

    // Check some basic conversions from fixture
    assert!(conversions
        .iter()
        .any(|c| c.from_unit == 1 && c.to_unit == 0)); // g to kg
    assert!(conversions
        .iter()
        .any(|c| c.from_unit == 3 && c.to_unit == 2)); // mL to L

    // Calculate and verify some conversion factors
    let g_to_kg = conversions
        .iter()
        .find(|c| c.from_unit == 1 && c.to_unit == 0)
        .unwrap();

    assert_eq!(
        (g_to_kg.to_amount.clone() / g_to_kg.from_amount.clone()).round(3),
        BigDecimal::from_str("0.001").unwrap()
    );
}
