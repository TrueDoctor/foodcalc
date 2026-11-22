use crate::entities::types::ids::{EventId, GroupId, UserId};
use crate::policy::{GlobalPermission, Permission};
use std::collections::{HashMap, HashSet};

/// Dynamic role slug (e.g., "viewer", "editor", "manager", or a custom DB role).
pub type RoleSlug = String;

#[derive(Debug, Default, Clone)]
pub struct UserPolicy {
    pub user_id: UserId,
    pub global_permissions: HashSet<GlobalPermission>,
    /// Map of `group_id -> resolved set of permissions` (union of all roles in that group)
    pub group_permissions: HashMap<GroupId, HashSet<Permission>>,
    /// Map of `event_id -> direct event-level permissions` (union of direct event roles)
    pub event_overrides: HashMap<EventId, HashSet<Permission>>,
}

impl UserPolicy {
    pub fn has_global_permission(&self, perm: GlobalPermission) -> bool {
        if self.global_permissions.contains(&perm) {
            return true;
        }
        match perm {
            GlobalPermission::ViewUsers => {
                self.global_permissions.contains(&GlobalPermission::ManageUsers)
            }
            _ => false,
        }
    }

    pub fn has_group_permission(&self, group_id: GroupId, perm: Permission) -> bool {
        self.group_permissions
            .get(&group_id)
            .is_some_and(|perms| perms.contains(&perm))
    }

    /// Evaluates `Group Permissions ∪ Direct Event Override Permissions`.
    pub fn has_event_permission(&self, event_id: EventId, group_id: GroupId, perm: Permission) -> bool {
        self.has_group_permission(group_id, perm)
            || self
            .event_overrides
            .get(&event_id)
            .is_some_and(|perms| perms.contains(&perm))
    }

    /// Returns the complete resolved permission set for serialization into `EventDetail`.
    pub fn resolved_event_permissions(&self, event_id: EventId, group_id: GroupId) -> Vec<Permission> {
        let mut combined = self
            .group_permissions
            .get(&group_id)
            .cloned()
            .unwrap_or_default();

        if let Some(overrides) = self.event_overrides.get(&event_id) {
            combined.extend(overrides.iter().copied());
        }

        // Deterministic order: HashSet-backed sources otherwise make the
        // serialized `permissions` array unstable between requests.
        let mut perms: Vec<Permission> = combined.into_iter().collect();
        perms.sort();
        perms.dedup();
        perms
    }
}