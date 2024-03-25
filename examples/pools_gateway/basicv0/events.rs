use crate::handler::{AppState, BasicV0};

use super::super::handler::Handler;

use radix_engine_common::ScryptoSbor;
use radix_event_stream::{
    encodings::encode_bech32, streaming::EventHandlerInput,
};
use sbor::rust::collections::IndexMap;
use scrypto::{
    math::Decimal,
    network::NetworkDefinition,
    types::{ComponentAddress, ResourceAddress},
};

#[derive(ScryptoSbor, Debug)]
pub struct InstantiateEvent {
    x_address: ResourceAddress,
    y_address: ResourceAddress,
    input_fee_rate: Decimal,
    liquidity_pool_address: ComponentAddress,
    pool_address: ComponentAddress,
}

pub fn handle_instantiate_event(
    input: EventHandlerInput<AppState, Handler, InstantiateEvent>,
) {
    let component_address = encode_bech32(
        input.event.pool_address.as_node_id().as_bytes(),
        &NetworkDefinition::mainnet(),
    )
    .unwrap();
    let native_address = encode_bech32(
        input.event.liquidity_pool_address.as_node_id().as_bytes(),
        &NetworkDefinition::mainnet(),
    )
    .unwrap();
    input
        .handler_registry
        .add_handler(component_address, Handler::BasicV0(BasicV0::SwapEvent));
    input.handler_registry.add_handler(
        native_address.clone(),
        Handler::BasicV0(BasicV0::ContributionEvent),
    );
    input.handler_registry.add_handler(
        native_address,
        Handler::BasicV0(BasicV0::RedemptionEvent),
    );
}

#[derive(ScryptoSbor, Debug)]
pub struct SwapEvent {
    input_address: ResourceAddress,
    input_amount: Decimal,
    output_address: ResourceAddress,
    output_amount: Decimal,
    input_fee_lp: Decimal,
}

pub fn handle_swap_event(
    input: EventHandlerInput<AppState, Handler, SwapEvent>,
) {
}

#[derive(ScryptoSbor, Debug)]
pub struct ContributionEvent {
    pub contributed_resources: IndexMap<ResourceAddress, Decimal>,
    pub pool_units_minted: Decimal,
}

pub fn handle_contribution_event(
    input: EventHandlerInput<AppState, Handler, ContributionEvent>,
) {
}

#[derive(ScryptoSbor, Debug)]
pub struct RedemptionEvent {
    pub pool_unit_tokens_redeemed: Decimal,
    pub redeemed_resources: IndexMap<ResourceAddress, Decimal>,
}

pub fn handle_redemption_event(
    input: EventHandlerInput<AppState, Handler, RedemptionEvent>,
) {
}
