use foodlib_new::{
    auth_context::AuthCtx,
    error::{Error, Result},
    user::Group,
};
use maud::{html, Markup};

/// Renders a "Move to group" panel for an entity.
///
/// `endpoint` is the POST URL that handles the move (form data: `group_id`).
/// `target_id` is the htmx target to swap on success — usually the entity's
/// edit-view container.
pub async fn move_panel(
    foodlib: &foodlib_new::FoodLib,
    ctx: &AuthCtx,
    current_group_id: i32,
    endpoint: &str,
    target_id: &str,
) -> Result<Markup> {
    let candidates = candidate_groups(foodlib, ctx, current_group_id).await?;
    let current = foodlib.users().get_group(current_group_id).await.ok();
    let current_label = current
        .map(|g| {
            if g.is_personal {
                format!("{} (personal)", g.name)
            } else {
                g.name
            }
        })
        .unwrap_or_else(|| "—".into());

    Ok(html! {
        div class="flex flex-row items-center flex-wrap gap-3 my-2 text-sm opacity-80" {
            span { "Owner group:" }
            strong { (current_label) }
            @if candidates.is_empty() {
                span class="opacity-70" { "(no other groups available)" }
            } @else {
                form class="flex flex-row items-center gap-2"
                    hx-post=(endpoint)
                    hx-target=(target_id)
                    hx-swap="outerHTML" {
                    select class="fc-select" name="group_id" required="required" {
                        option value="" { "Move to..." }
                        @for g in &candidates {
                            option value=(g.id) {
                                (g.name)
                                @if g.is_personal { " (personal)" }
                            }
                        }
                    }
                    button class="btn btn-primary" type="submit" { "Move" }
                }
            }
        }
    })
}

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
