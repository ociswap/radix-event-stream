use std::{rc::Rc, sync::Arc};

use event_handler::event_handler;
use radix_engine_common::ScryptoSbor;
use radix_event_stream::{
    encodings::encode_bech32,
    models::{EventHandlerContext, Transaction},
};
use sbor::rust::collections::IndexMap;
use scrypto::{
    math::Decimal,
    network::NetworkDefinition,
    types::{ComponentAddress, ResourceAddress},
};
use sqlx::{Executor, Sqlite};
use std::sync::Mutex;

// Define a global state
#[derive(Debug, Clone)]
pub struct AppState {
    pub number: u64,
    pub async_runtime: Rc<tokio::runtime::Runtime>,
    pub pool: sqlx::Pool<sqlx::Sqlite>,
    pub transaction:
        Arc<Mutex<Option<sqlx::Transaction<'static, sqlx::Sqlite>>>>,
}

// Copy and paste events over from a blueprint
#[derive(ScryptoSbor, Debug)]
pub struct InstantiateEvent {
    x_address: ResourceAddress,
    y_address: ResourceAddress,
    input_fee_rate: Decimal,
    liquidity_pool_address: ComponentAddress,
    pool_address: ComponentAddress,
}

// Implement the event handler
#[event_handler]
pub fn handle_instantiate_event(
    input: EventHandlerContext<AppState>,
    event: InstantiateEvent,
) {
    // Encode the component address as a bech32 string
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

    input.app_state.async_runtime.block_on(async {
        let tx = &input.app_state.transaction;
        let mut tx_guard = tx.lock().unwrap();
        // insert the new event into the database as text.
        let ding = tx_guard.as_mut().unwrap();

        sqlx::query("INSERT INTO events (data) VALUES (?)")
            .bind(format!("{:#?}", event))
            .execute(&mut **ding)
            .await
            .unwrap();
    });

    // Register new event handlers for the new component
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
pub fn handle_swap_event(
    input: EventHandlerContext<AppState>,
    event: SwapEvent,
) {
}

#[derive(ScryptoSbor, Debug)]
pub struct ContributionEvent {
    pub contributed_resources: IndexMap<ResourceAddress, Decimal>,
    pub pool_units_minted: Decimal,
}

#[event_handler]
pub fn handle_contribution_event(
    input: EventHandlerContext<AppState>,
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
    input: EventHandlerContext<AppState>,
    event: RedemptionEvent,
) {
}
