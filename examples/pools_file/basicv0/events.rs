use event_name_derive::EventName;
use radix_client::gateway::models::EventEmitterIdentifier;
use radix_engine_common::ScryptoSbor;
use radix_event_stream::encodings::encode_bech32;
use radix_event_stream::{handler::EventHandler, EventName};

use scrypto::data::scrypto::scrypto_decode;
use scrypto::math::decimal::Decimal;
use scrypto::network::NetworkDefinition;
use scrypto::prelude::IndexMap;
use scrypto::types::ComponentAddress;
use scrypto::types::ResourceAddress;
use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

use super::pool::BasicV0Pool;
use super::poolstore::Pool;
use super::poolstore::PoolStore;

#[derive(ScryptoSbor, Debug, EventName)]
pub struct InstantiateEvent {
    x_address: ResourceAddress,
    y_address: ResourceAddress,
    input_fee_rate: Decimal,
    liquidity_pool_address: ComponentAddress,
    pool_address: ComponentAddress,
}

#[derive(Debug, Clone)]
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

        let decoded =
            match scrypto_decode::<InstantiateEvent>(&event.binary_sbor_data())
            {
                Ok(decoded) => decoded,
                Err(error) => {
                    log::error!(
                        "Failed to decode InstantiateEvent: {:#?}",
                        error
                    );
                    return None;
                }
            };

        Some(Box::new(decoded))
    }
    fn process(
        &self,
        event: &Box<dyn radix_event_stream::streaming::Event>,
        _: &Box<dyn radix_event_stream::streaming::Transaction>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let decoded =
            match scrypto_decode::<InstantiateEvent>(&event.binary_sbor_data())
            {
                Ok(decoded) => decoded,
                Err(error) => {
                    log::error!(
                        "Failed to decode InstantiateEvent: {:#?}",
                        error
                    );
                    return Err("Failed to decode InstantiateEvent".into());
                }
            };
        let component_address = encode_bech32(
            decoded.pool_address.as_node_id().as_bytes(),
            &NetworkDefinition::mainnet(),
        )
        .unwrap();
        let native_address = encode_bech32(
            decoded.liquidity_pool_address.as_node_id().as_bytes(),
            &NetworkDefinition::mainnet(),
        )
        .unwrap();
        let pool: Box<dyn Pool> = Box::new(BasicV0Pool {
            component_address: component_address.clone(),
            native_pool_address: Some(native_address.clone()),
        });
        self.pool_store.borrow_mut().add_pool(&pool);
        Ok(())
    }
}

#[derive(ScryptoSbor, Debug, EventName)]
pub struct ContributionEvent {
    pub contributed_resources: IndexMap<ResourceAddress, Decimal>,
    pub pool_units_minted: Decimal,
}

#[derive(Debug, Clone)]
pub struct ContributionEventHandler {
    pub pool_store: Rc<RefCell<PoolStore>>,
}

impl EventHandler for ContributionEventHandler {
    fn identify(
        &self,
        event: &Box<dyn radix_event_stream::streaming::Event>,
    ) -> Option<Box<dyn Debug>> {
        if event.name() != ContributionEvent::event_name() {
            return None;
        }
        let native_address = match event.emitter() {
            EventEmitterIdentifier::Method { entity, .. } => {
                &entity.entity_address
            }
            _ => return None,
        };
        if self
            .pool_store
            .borrow()
            .find_by_native_address(native_address)
            .is_some()
        {
            let decoded = match scrypto_decode::<ContributionEvent>(
                &event.binary_sbor_data(),
            ) {
                Ok(decoded) => decoded,
                Err(error) => {
                    log::error!(
                        "Failed to decode ContributionEvent: {:#?}",
                        error
                    );
                    return None;
                }
            };
            Some(Box::new(decoded))
        } else {
            None
        }
    }
    fn process(
        &self,
        _: &Box<dyn radix_event_stream::streaming::Event>,
        _: &Box<dyn radix_event_stream::streaming::Transaction>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}
