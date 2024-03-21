use super::pool::*;
use crate::decoder::decode_programmatic_json;
use crate::decoder::encode_bech32;
use crate::decoder::EventHandler;
// use crate::decoder::EventDecoder;
use crate::{poolstore::PoolStore, EventName};
use event_name_derive::EventName;
use log::error;
use radix_client::gateway::models::{Event, EventEmitterIdentifier};
use radix_engine_common::ScryptoSbor;
use scrypto::math::decimal::Decimal;
use scrypto::prelude::IndexMap;
use scrypto::types::ComponentAddress;
use scrypto::types::ResourceAddress;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(ScryptoSbor, Debug, EventName)]
pub struct InstantiateEvent {
    x_address: ResourceAddress,
    y_address: ResourceAddress,
    input_fee_rate: Decimal,
    liquidity_pool_address: ComponentAddress,
    pool_address: ComponentAddress,
}

// impl ProcessableEvent for InstantiateEvent {
//     fn update_pool_store(&self, pool_store: &mut PoolStore) {
//         let pool_address = encode_bech32(
//             &self.liquidity_pool_address.to_vec(),
//             &pool_store.network,
//         )
//         .expect("Failed to encode pool address");
//         let component_address =
//             encode_bech32(&self.pool_address.to_vec(), &pool_store.network)
//                 .expect("Failed to encode component address");

//         pool_store.pools.insert(
//             component_address.clone(),
//             Box::new(BasicV0Pool {
//                 component_address: component_address.clone(),
//                 native_pool_address: Some(pool_address.clone()),
//             }),
//         );
//         pool_store
//             .native_address_map
//             .insert(pool_address.clone(), component_address.clone());
//     }
// }

pub struct InstantiateEventHandler {
    pub pool_store: Rc<RefCell<PoolStore>>,
}

impl EventHandler for InstantiateEventHandler {}

// impl EventDecoder for InstantiateEventDecoder {
//     fn decode(&self, event: &Event) -> Option<Box<dyn ProcessableEvent>> {
//         if event.name != InstantiateEvent::event_name() {
//             return None;
//         }
//         let package_address = match &event.emitter {
//             EventEmitterIdentifier::Function {
//                 package_address, ..
//             } => package_address,
//             _ => return None,
//         };
//         if self
//             .pool_store
//             .borrow()
//             .find_by_package_address(&package_address)
//             .is_some()
//         {
//             let decoded =
//                 match decode_programmatic_json::<InstantiateEvent>(&event.data)
//                 {
//                     Ok(decoded) => decoded,
//                     Err(error) => {
//                         error!(
//                             "Failed to decode InstantiateEvent: {:#?}",
//                             error
//                         );
//                         return None;
//                     }
//                 };
//             Some(Box::new(decoded))
//         } else {
//             None
//         }
//     }
// }

#[derive(ScryptoSbor, Debug, EventName)]
pub struct SwapEvent {
    input_address: ResourceAddress,
    input_amount: Decimal,
    output_address: ResourceAddress,
    output_amount: Decimal,
    input_fee_lp: Decimal,
}

// impl ProcessableEvent for SwapEvent {}

pub struct SwapEventDecoder {
    pub pool_store: Rc<RefCell<PoolStore>>,
}

// impl EventDecoder for SwapEventDecoder {
//     fn decode(&self, event: &Event) -> Option<Box<dyn ProcessableEvent>> {
//         if event.name != SwapEvent::event_name() {
//             return None;
//         }
//         let component_address = match &event.emitter {
//             EventEmitterIdentifier::Method { entity, .. } => {
//                 &entity.entity_address
//             }
//             _ => return None,
//         };
//         if self
//             .pool_store
//             .borrow()
//             .find_by_component_address(&component_address)
//             .is_some()
//         {
//             let decoded =
//                 match decode_programmatic_json::<SwapEvent>(&event.data) {
//                     Ok(decoded) => decoded,
//                     Err(error) => {
//                         error!("Failed to decode SwapEvent: {:#?}", error);
//                         return None;
//                     }
//                 };
//             Some(Box::new(decoded))
//         } else {
//             None
//         }
//     }
// }

#[derive(ScryptoSbor, Debug, EventName)]
pub struct ContributionEvent {
    pub contributed_resources: IndexMap<ResourceAddress, Decimal>,
    pub pool_units_minted: Decimal,
}

// impl ProcessableEvent for ContributionEvent {}

pub struct ContributionEventDecoder {
    pub pool_store: Rc<RefCell<PoolStore>>,
}

// impl EventDecoder for ContributionEventDecoder {
//     fn decode(&self, event: &Event) -> Option<Box<dyn ProcessableEvent>> {
//         if event.name != ContributionEvent::event_name() {
//             return None;
//         }
//         let native_address = match &event.emitter {
//             EventEmitterIdentifier::Method { entity, .. } => {
//                 &entity.entity_address
//             }
//             _ => return None,
//         };
//         if self
//             .pool_store
//             .borrow()
//             .find_by_native_address(&native_address)
//             .is_some()
//         {
//             let decoded = match decode_programmatic_json::<ContributionEvent>(
//                 &event.data,
//             ) {
//                 Ok(decoded) => decoded,
//                 Err(error) => {
//                     error!("Failed to decode ContributionEvent: {:#?}", error);
//                     return None;
//                 }
//             };
//             Some(Box::new(decoded))
//         } else {
//             None
//         }
//     }
// }

#[derive(ScryptoSbor, Debug, EventName)]
pub struct RedemptionEvent {
    pub pool_unit_tokens_redeemed: Decimal,
    pub redeemed_resources: IndexMap<ResourceAddress, Decimal>,
}

// impl ProcessableEvent for RedemptionEvent {}

pub struct RedemptionEventDecoder {
    pub pool_store: Rc<RefCell<PoolStore>>,
}

// impl EventDecoder for RedemptionEventDecoder {
//     fn decode(&self, event: &Event) -> Option<Box<dyn ProcessableEvent>> {
//         if event.name != RedemptionEvent::event_name() {
//             return None;
//         }
//         let native_address = match &event.emitter {
//             EventEmitterIdentifier::Method { entity, .. } => {
//                 &entity.entity_address
//             }
//             _ => return None,
//         };
//         if self
//             .pool_store
//             .borrow()
//             .find_by_native_address(&native_address)
//             .is_some()
//         {
//             let decoded = match decode_programmatic_json::<RedemptionEvent>(
//                 &event.data,
//             ) {
//                 Ok(decoded) => decoded,
//                 Err(error) => {
//                     error!("Failed to decode RedemptionEvent: {:#?}", error);
//                     return None;
//                 }
//             };
//             Some(Box::new(decoded))
//         } else {
//             None
//         }
//     }
// }
