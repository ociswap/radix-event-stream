#[derive(PartialEq, Eq, Debug, Hash)]
pub enum ResourceManagerEventType {
    BurnFungibleResourceEvent,
    BurnNonFungibleResourceEvent,
    MintFungibleResourceEvent,
    MintNonFungibleResourceEvent,
    VaultCreationEvent,
}

pub use radix_engine::blueprints::resource::BurnFungibleResourceEvent;
pub use radix_engine::blueprints::resource::BurnNonFungibleResourceEvent;
pub use radix_engine::blueprints::resource::MintFungibleResourceEvent;
pub use radix_engine::blueprints::resource::MintNonFungibleResourceEvent;
pub use radix_engine::blueprints::resource::VaultCreationEvent;
