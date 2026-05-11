use foodlib_new::{
    auth_context::AuthCtx,
    error::{Error, Result},
    user::Group,
};
use maud::{html, Markup};

/// Groups the user could plausibly move an entity into:
/// - Admins: every group except the current one
/// - Members: every group they belong to except the current one
async fn candidate_groups(
    foodlib: &foodlib_new::FoodLib,
    ctx: &AuthCtx,
    current_group_id: i32,
) -> Result<Vec<Group>> {
    let groups = if ctx.user.is_admin {
        foodlib.users().get_all_groups().await?
    } else {
        foodlib.users().get_user_groups(ctx.user.id).await?
    };
    Ok(groups
        .into_iter()
        .filter(|g| g.id != current_group_id)
        .collect())
}

/// Renders a labelled "Owner:" dropdown meant to be embedded inside a larger
/// form (e.g. the event metadata form). Submits no button of its own — the
/// surrounding form's save action picks up `group_id`. Falls back to a plain
/// read-only label when the user has no other groups to move into.
pub async fn owner_select(
    foodlib: &foodlib_new::FoodLib,
    ctx: &AuthCtx,
    current_group_id: i32,
) -> Result<Markup> {
    let candidates = candidate_groups(foodlib, ctx, current_group_id).await?;
    let current = foodlib.users().get_group(current_group_id).await.ok();
    let current_label = current
        .as_ref()
        .map(|g| {
            if g.is_personal {
                format!("{} (personal)", g.name)
            } else {
                g.name.clone()
            }
        })
        .unwrap_or_else(|| "—".into());

    Ok(html! {
        label for="group_id" { "Owner:" }
        @if candidates.is_empty() {
            span { (current_label) }
            input type="hidden" name="group_id" value=(current_group_id);
        } @else {
            select class="fc-select" name="group_id" required="required" {
                option value=(current_group_id) selected { (current_label) }
                @for g in &candidates {
                    option value=(g.id) {
                        (g.name)
                        @if g.is_personal { " (personal)" }
                    }
                }
            }
        }
    })
}

/// Authorizes a move request: the user must already have access to the source
/// group (caller should have asserted this) and must be allowed to move into
/// the target group. Admins may move anywhere; non-admins may only move into
/// groups they belong to.
pub fn assert_can_move_to(ctx: &AuthCtx, target_group_id: i32) -> Result<()> {
    if ctx.user.is_admin || ctx.group_ids.contains(&target_group_id) {
        Ok(())
    } else {
        Err(Error::Forbidden(
            "You can only move resources into groups you belong to".into(),
        ))
    }
}
