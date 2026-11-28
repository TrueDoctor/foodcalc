use crate::types::ids::GroupId;
use crate::user::User;

pub async fn members_of_group(foodlib: &crate::FoodLib, group_id: GroupId) -> crate::Result<Vec<User>> {
    let pool = foodlib.pool_arc();
    let rows = sqlx::query_as!(
        User,
        r#"
        SELECT u.id, u.username, u.email, u.password_hash, u.is_admin, u.created_at
        FROM user_groups ug
        JOIN users u ON u.id = ug.user_id
        WHERE ug.group_id = $1
        ORDER BY u.username
        "#,
        group_id.get()
    )
    .fetch_all(&*pool)
    .await?;
    Ok(rows)
}