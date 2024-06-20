#[derive(PartialEq, Eq, Debug, Hash)]
pub enum FungibleVaultEventType {
    DepositEvent,
    LockFeeEvent,
    PayFeeEvent,
    RecallEvent,
    WithdrawEvent,
}

pub use radix_engine::blueprints::resource::fungible_vault::DepositEvent;
pub use radix_engine::blueprints::resource::fungible_vault::LockFeeEvent;
pub use radix_engine::blueprints::resource::fungible_vault::PayFeeEvent;
pub use radix_engine::blueprints::resource::fungible_vault::RecallEvent;
pub use radix_engine::blueprints::resource::fungible_vault::WithdrawEvent;
