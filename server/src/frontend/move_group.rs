use foodlib_new::entities::user::Group;
use foodlib_new::error::Result;
use foodlib_new::types::ids::GroupId;
use foodlib_new::AppCtx;
use maud::{html, Markup};

/// Groups the user could plausibly move an entity into:
/// - Admins: every group except the current one
/// - Members: every group they belong to except the current one
async fn candidate_groups(
    state: &crate::AppState,
    actor: &AppCtx,
    current_group_id: GroupId,
) -> Result<Vec<Group>> {
    let groups = if actor.user.is_admin {
        foodlib_new::admin::groups::list_all_groups(state).await?
    } else {
        foodlib_new::admin::groups::get_user_groups(state, actor.user.id).await?
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
    state: &crate::AppState,
    actor: &AppCtx,
    current_group_id: GroupId,
) -> Result<Markup> {
    let candidates = candidate_groups(state, actor, current_group_id).await?;
    let current = foodlib_new::admin::groups::get_group(state, current_group_id)
        .await
        .ok();
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

