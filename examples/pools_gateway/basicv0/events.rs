use radix_engine_common::ScryptoSbor;
use radix_event_stream::{encodings::encode_bech32, models::EventHandlerInput};
use sbor::rust::collections::IndexMap;
use scrypto::{
    math::Decimal,
    network::NetworkDefinition,
    types::{ComponentAddress, ResourceAddress},
};

#[derive(Debug, Clone)]
pub struct AppState {
    pub number: u64,
}

#[derive(ScryptoSbor, Debug)]
pub struct InstantiateEvent {
    x_address: ResourceAddress,
    y_address: ResourceAddress,
    input_fee_rate: Decimal,
    liquidity_pool_address: ComponentAddress,
    pool_address: ComponentAddress,
}
use event_handler::event_handler;

#[event_handler]
pub fn handle_instantiate_event(
    input: EventHandlerInput<AppState>,
    event: InstantiateEvent,
) {
    let component_address = encode_bech32(
        event.pool_address.as_node_id().as_bytes(),
        &NetworkDefinition::mainnet(),
    )
    .unwrap();
    let native_address = encode_bech32(
        event.liquidity_pool_address.as_node_id().as_bytes(),
        &NetworkDefinition::mainnet(),
    )
    .unwrap();
    input.handler_registry.add_handler(
        &component_address,
        "SwapEvent",
        handle_swap_event,
    );
    input.handler_registry.add_handler(
        &native_address,
        "ContributionEvent",
        handle_contribution_event,
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

#[event_handler]
pub fn handle_swap_event(input: EventHandlerInput<AppState>, event: SwapEvent) {
}

#[derive(ScryptoSbor, Debug)]
pub struct ContributionEvent {
    pub contributed_resources: IndexMap<ResourceAddress, Decimal>,
    pub pool_units_minted: Decimal,
}

#[event_handler]
pub fn handle_contribution_event(
    input: EventHandlerInput<AppState>,
    event: ContributionEvent,
) {
}

#[derive(ScryptoSbor, Debug)]
pub struct RedemptionEvent {
    pub pool_unit_tokens_redeemed: Decimal,
    pub redeemed_resources: IndexMap<ResourceAddress, Decimal>,
}

#[event_handler]
pub fn handle_redemption_event(
    input: EventHandlerInput<AppState>,
    event: RedemptionEvent,
) {
}
