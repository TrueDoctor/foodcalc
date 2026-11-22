use crate::policy::UnknownPermissionError;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GlobalPermission {
    /// Baseline right to create top-level resources (groups, events, recipes,
    /// ingredients, inventories). Granted on registration; revocable to lock down an account.
    CreateResources,
    /// Read-only access to the global user directory.
    ViewUsers,
    /// Full user administration (edit profiles, reset passwords, delete accounts,
    /// assign global permissions). Implies `ViewUsers`.
    ManageUsers,
    /// Edit shared reference data (stores, units, allergen/property definitions).
    ManageMasterData,
    /// Edit instance-wide configuration (global default member roles).
    ManageSettings,
    /// Read and export the security audit log (ISO 27001 A.8.15 / A.5.28).
    ViewAuditLog,
}

impl GlobalPermission {
    /// Granted automatically when a user registers.
    pub const DEFAULT_REGISTERED: &'static [Self] = &[
        Self::CreateResources,
        Self::ViewUsers,
    ];

    /// Every global permission, in display order. Used by permission-editing
    /// UIs so checkboxes never need a hand-maintained list.
    pub const ALL: &'static [Self] = &[
        Self::CreateResources,
        Self::ViewUsers,
        Self::ManageUsers,
        Self::ManageMasterData,
        Self::ManageSettings,
        Self::ViewAuditLog,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CreateResources => "user:create_resources",
            Self::ViewUsers => "admin:view_users",
            Self::ManageUsers => "admin:manage_users",
            Self::ManageMasterData => "admin:manage_master_data",
            Self::ManageSettings => "admin:manage_settings",
            Self::ViewAuditLog => "admin:view_audit_log",
        }
    }
}

impl FromStr for GlobalPermission {
    type Err = UnknownPermissionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "user:create_resources" | "admin:allow_creating" => Ok(Self::CreateResources),
            "admin:view_users" | "admin:view_user_profiles" => Ok(Self::ViewUsers),
            "admin:manage_users" => Ok(Self::ManageUsers),
            "admin:manage_master_data" => Ok(Self::ManageMasterData),
            "admin:manage_settings" => Ok(Self::ManageSettings),
            "admin:view_audit_log" => Ok(Self::ViewAuditLog),
            _ => Err(UnknownPermissionError(s.to_owned())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    /// `ALL` is consumed by the admin checkbox editor: a variant missing from
    /// it would silently disappear from the UI, and a typo in a slug would
    /// round-trip into an unparseable DB value.
    #[test]
    fn all_covers_every_variant_and_round_trips() {
        for perm in GlobalPermission::ALL {
            assert_eq!(GlobalPermission::from_str(perm.as_str()).unwrap(), *perm);
        }
        assert_eq!(GlobalPermission::ALL.len(), 6);
    }
}
