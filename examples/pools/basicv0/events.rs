use event_name_derive::EventName;
use radix_client::gateway::models::EventEmitterIdentifier;
use radix_engine_common::ScryptoEvent;
use radix_engine_common::ScryptoSbor;
use radix_event_stream::encodings::decode_programmatic_json;
use radix_event_stream::{handler::EventHandler, EventName};
use scrypto::math::decimal::Decimal;
use scrypto::prelude::IndexMap;
use scrypto::types::ComponentAddress;
use scrypto::types::ResourceAddress;
use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

use super::poolstore::PoolStore;

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

#[derive(Debug)]
pub struct InstantiateEventHandler {
    pub pool_store: Rc<RefCell<PoolStore>>,
}

impl EventHandler for InstantiateEventHandler {
    fn identify(
        &self,
        event: &Box<dyn radix_event_stream::streaming::Event>,
    ) -> Option<Box<dyn Debug>> {
        if event.name() != InstantiateEvent::event_name() {
            return None;
        }
        let package_address = match event.emitter() {
            EventEmitterIdentifier::Function {
                package_address, ..
            } => package_address,
            _ => return None,
        };
        if self
            .pool_store
            .borrow()
            .find_by_package_address(package_address)
            .is_none()
        {
            return None;
        }

        let decoded = match decode_programmatic_json::<InstantiateEvent>(
            &event.programmatic_json(),
        ) {
            Ok(decoded) => decoded,
            Err(error) => {
                log::error!("Failed to decode InstantiateEvent: {:#?}", error);
                return None;
            }
        };
        Some(Box::new(decoded))
    }
    fn process(
        &self,
        _: &Box<dyn radix_event_stream::streaming::Event>,
        _: &Box<dyn radix_event_stream::streaming::Transaction>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

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
