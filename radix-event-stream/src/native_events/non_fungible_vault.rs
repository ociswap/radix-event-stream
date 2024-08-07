#[derive(PartialEq, Eq, Debug, Hash)]
pub enum NonFungibleVaultEventType {
    DepositEvent,
    RecallEvent,
    WithdrawEvent,
}

pub use radix_engine::blueprints::resource::non_fungible_vault::DepositEvent;
pub use radix_engine::blueprints::resource::non_fungible_vault::RecallEvent;
pub use radix_engine::blueprints::resource::non_fungible_vault::WithdrawEvent;
