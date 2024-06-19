use std::str::FromStr;

use radix_engine_common::{
    math::Decimal, prelude::NonFungibleLocalId, types::NodeId, ScryptoSbor,
};
use radix_engine_interface::prelude::MetadataValue;
use sbor::rust::collections::indexmap::IndexSet;

/// resource manager
/// https://github.com/radixdlt/radixdlt-scrypto/blob/main/radix-engine/src/blueprints/resource/events/resource_manager.rs

#[derive(PartialEq, Eq, Debug, Hash)]
pub enum ResourceManagerEventType {
    VaultCreationEvent,
    MintFungibleResourceEvent,
    BurnFungibleResourceEvent,
    MintNonFungibleResourceEvent,
    BurnNonFungibleResourceEvent,
}

#[derive(PartialEq, Eq, Debug, Hash)]
pub enum NativeEventType {
    ResourceManager(ResourceManagerEventType),
    Metadata(MetadataEventType),
}

impl FromStr for NativeEventType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "VaultCreationEvent" => Ok(NativeEventType::ResourceManager(
                ResourceManagerEventType::VaultCreationEvent,
            )),
            "MintFungibleResourceEvent" => {
                Ok(NativeEventType::ResourceManager(
                    ResourceManagerEventType::MintFungibleResourceEvent,
                ))
            }
            "BurnFungibleResourceEvent" => {
                Ok(NativeEventType::ResourceManager(
                    ResourceManagerEventType::BurnFungibleResourceEvent,
                ))
            }
            "MintNonFungibleResourceEvent" => {
                Ok(NativeEventType::ResourceManager(
                    ResourceManagerEventType::MintNonFungibleResourceEvent,
                ))
            }
            "BurnNonFungibleResourceEvent" => {
                Ok(NativeEventType::ResourceManager(
                    ResourceManagerEventType::BurnNonFungibleResourceEvent,
                ))
            }
            "SetMetadataEvent" => Ok(NativeEventType::Metadata(
                MetadataEventType::SetMetadataEvent,
            )),
            "RemoveMetadataEvent" => Ok(NativeEventType::Metadata(
                MetadataEventType::RemoveMetadataEvent,
            )),
            _ => Err(()),
        }
    }
}

#[derive(ScryptoSbor, PartialEq, Eq, Debug)]
pub struct VaultCreationEvent {
    pub vault_id: NodeId,
}

#[derive(ScryptoSbor, PartialEq, Eq, Debug)]
pub struct MintFungibleResourceEvent {
    pub amount: Decimal,
}

#[derive(ScryptoSbor, PartialEq, Eq, Debug)]
pub struct BurnFungibleResourceEvent {
    pub amount: Decimal,
}

#[derive(ScryptoSbor, PartialEq, Eq, Debug)]
pub struct MintNonFungibleResourceEvent {
    pub ids: IndexSet<NonFungibleLocalId>,
}

#[derive(ScryptoSbor, PartialEq, Eq, Debug)]
pub struct BurnNonFungibleResourceEvent {
    pub ids: IndexSet<NonFungibleLocalId>,
}

/// metadata
/// https://github.com/radixdlt/radixdlt-scrypto/blob/main/radix-engine/src/object_modules/metadata/events.rs

#[derive(PartialEq, Eq, Debug, Hash)]
pub enum MetadataEventType {
    SetMetadataEvent,
    RemoveMetadataEvent,
}

#[derive(ScryptoSbor, Debug, PartialEq, Eq)]
pub struct SetMetadataEvent {
    pub key: String,
    pub value: MetadataValue,
}

#[derive(ScryptoSbor, Debug)]
pub struct RemoveMetadataEvent {
    pub key: String,
}
