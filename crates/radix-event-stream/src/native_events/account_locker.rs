#[derive(PartialEq, Eq, Debug, Hash)]
pub enum AccountLockerEventType {
    ClaimEvent,
    RecoverEvent,
    StoreEvent,
}

pub use radix_engine::blueprints::locker::ClaimEvent;
pub use radix_engine::blueprints::locker::RecoverEvent;
pub use radix_engine::blueprints::locker::StoreEvent;
