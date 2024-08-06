#[derive(PartialEq, Eq, Debug, Hash)]
pub enum RoleAssignmentEventType {
    LockOwnerRoleEvent,
    SetOwnerRoleEvent,
    SetRoleEvent,
}

pub use radix_engine::object_modules::role_assignment::LockOwnerRoleEvent;
pub use radix_engine::object_modules::role_assignment::SetOwnerRoleEvent;
pub use radix_engine::object_modules::role_assignment::SetRoleEvent;
