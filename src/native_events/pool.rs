/// https://github.com/radixdlt/radixdlt-scrypto/blob/main/radix-engine/src/blueprints/pool/v1/events.rs

pub mod one_resource_pool {

    #[derive(Debug, PartialEq, Eq, Hash)]
    pub enum OneResourcePoolEventType {
        ContributionEvent,
        RedemptionEvent,
        WithdrawEvent,
        DepositEvent,
    }

    pub use radix_engine::blueprints::pool::v1::events::one_resource_pool::ContributionEvent;
    pub use radix_engine::blueprints::pool::v1::events::one_resource_pool::DepositEvent;
    pub use radix_engine::blueprints::pool::v1::events::one_resource_pool::RedemptionEvent;
    pub use radix_engine::blueprints::pool::v1::events::one_resource_pool::WithdrawEvent;
}

pub mod two_resource_pool {

    #[derive(Debug, PartialEq, Eq, Hash)]
    pub enum TwoResourcePoolEventType {
        ContributionEvent,
        RedemptionEvent,
        WithdrawEvent,
        DepositEvent,
    }

    pub use radix_engine::blueprints::pool::v1::events::two_resource_pool::ContributionEvent;
    pub use radix_engine::blueprints::pool::v1::events::two_resource_pool::DepositEvent;
    pub use radix_engine::blueprints::pool::v1::events::two_resource_pool::RedemptionEvent;
    pub use radix_engine::blueprints::pool::v1::events::two_resource_pool::WithdrawEvent;
}

pub mod multi_resource_pool {
    #[derive(Debug, PartialEq, Eq, Hash)]
    pub enum MultiResourcePoolEventType {
        ContributionEvent,
        RedemptionEvent,
        WithdrawEvent,
        DepositEvent,
    }

    pub use radix_engine::blueprints::pool::v1::events::multi_resource_pool::ContributionEvent;
    pub use radix_engine::blueprints::pool::v1::events::multi_resource_pool::DepositEvent;
    pub use radix_engine::blueprints::pool::v1::events::multi_resource_pool::RedemptionEvent;
    pub use radix_engine::blueprints::pool::v1::events::multi_resource_pool::WithdrawEvent;
}
