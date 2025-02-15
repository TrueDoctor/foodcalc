// users_test.rs

use crate::{entities::user::*, error::Error, ops::users::UserOps};
use pretty_assertions::assert_eq;
use time::OffsetDateTime;

#[sqlx::test(fixtures("../fixtures/minimal.sql"))]
async fn test_create_user(pool: sqlx::PgPool) {
    let ops = UserOps::new(pool.into());

    let user = User {
        id: -1,
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        password_hash: "$2a$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LQtXDOQSZCDILT2rG".to_string(),
        is_admin: false,
        created_at: OffsetDateTime::now_utc(),
    };

    let created = ops.create_user(user.clone()).await.unwrap();
    assert_eq!(created.username, user.username);
    assert_eq!(created.email, user.email);
    assert_eq!(created.is_admin, user.is_admin);
    assert!(created.id > 0);
}

#[sqlx::test(fixtures("../fixtures/minimal.sql"))]
async fn test_update_user(pool: sqlx::PgPool) {
    let ops = UserOps::new(pool.into());

    // Create a test user to update
    let user = ops
        .create_user(User {
            id: -1,
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password_hash: "$2a$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LQtXDOQSZCDILT2rG"
                .to_string(),
            is_admin: false,
            created_at: OffsetDateTime::now_utc(),
        })
        .await
        .unwrap();

    let updated = ops
        .update_user(User {
            id: user.id,
            username: "updateduser".to_string(),
            email: "updated@example.com".to_string(),
            password_hash: "$2a$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LQtXDOQSZCDILT2rG"
                .to_string(),
            is_admin: true,
            created_at: user.created_at,
        })
        .await
        .unwrap();

    assert_eq!(updated.username, "updateduser");
    assert_eq!(updated.email, "updated@example.com");
    assert_eq!(updated.is_admin, true);
}

#[sqlx::test(fixtures("../fixtures/minimal.sql"))]
async fn test_delete_user(pool: sqlx::PgPool) {
    let ops = UserOps::new(pool.into());

    // Create temporary user to delete
    let user = ops
        .create_user(User {
            id: -1,
            username: "todelete".to_string(),
            email: "todelete@example.com".to_string(),
            password_hash: "$2a$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LQtXDOQSZCDILT2rG"
                .to_string(),
            is_admin: false,
            created_at: OffsetDateTime::now_utc(),
        })
        .await
        .unwrap();

    ops.delete_user(user.id).await.unwrap();

    let err = ops.get_user(user.id).await.unwrap_err();
    assert!(matches!(err, Error::UserNotFound { .. }));
}

#[sqlx::test(fixtures("../fixtures/minimal.sql"))]
async fn test_list_users(pool: sqlx::PgPool) {
    let ops = UserOps::new(pool.into());

    let users = ops.get_all_users().await.unwrap();
    assert!(!users.is_empty());

    // Verify users from fixture are present
    let admin = users.iter().find(|u| u.username == "admin").unwrap();
    assert_eq!(admin.is_admin, true);

    let user = users.iter().find(|u| u.username == "user").unwrap();
    assert_eq!(user.is_admin, false);
}

#[sqlx::test(fixtures("../fixtures/minimal.sql"))]
async fn test_user_groups(pool: sqlx::PgPool) {
    let ops = UserOps::new(pool.into());

    // Create a new user
    let user = ops
        .create_user(User {
            id: -1,
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password_hash: "$2a$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LQtXDOQSZCDILT2rG"
                .to_string(),
            is_admin: false,
            created_at: OffsetDateTime::now_utc(),
        })
        .await
        .unwrap();

    // Create a new group
    let group = ops
        .create_group(Group {
            id: -1,
            name: "Test Group".to_string(),
        })
        .await
        .unwrap();

    // Add user to group
    let user_group = UserGroup {
        user_id: user.id,
        group_id: group.id,
    };
    ops.add_user_to_group(user_group.clone()).await.unwrap();

    // Verify user is in the group
    let user_groups = ops.get_user_groups(user.id).await.unwrap();
    assert!(user_groups.iter().any(|g| g.id == group.id));

    // Remove user from group
    ops.remove_user_from_group(user.id, group.id).await.unwrap();
    let user_groups = ops.get_user_groups(user.id).await.unwrap();
    assert!(!user_groups.iter().any(|g| g.id == group.id));
}
