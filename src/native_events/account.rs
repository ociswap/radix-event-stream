#[derive(PartialEq, Eq, Debug, Hash)]
pub enum AccountEventType {
    AddAuthorizedDepositorEvent,
    DepositEvent,
    RejectedDepositEvent,
    RemoveAuthorizedDepositorEvent,
    RemoveResourcePreferenceEvent,
    SetDefaultDepositRuleEvent,
    SetResourcePreferenceEvent,
    WithdrawEvent,
}

pub use radix_engine::blueprints::account::AddAuthorizedDepositorEvent;
pub use radix_engine::blueprints::account::DepositEvent;
pub use radix_engine::blueprints::account::RejectedDepositEvent;
pub use radix_engine::blueprints::account::RemoveAuthorizedDepositorEvent;
pub use radix_engine::blueprints::account::RemoveResourcePreferenceEvent;
pub use radix_engine::blueprints::account::SetDefaultDepositRuleEvent;
pub use radix_engine::blueprints::account::SetResourcePreferenceEvent;
pub use radix_engine::blueprints::account::WithdrawEvent;
