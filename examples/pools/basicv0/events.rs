use super::definitions::*;
use auto_decode::auto_decode;
use radix_engine_common::ScryptoSbor;
use radix_event_stream::{
    encodings::encode_bech32, error::EventHandlerError,
    event_handler::EventHandlerContext,
};
use sbor::rust::collections::IndexMap;
use scrypto::{
    math::Decimal,
    types::{ComponentAddress, ResourceAddress},
};

async fn add_to_database(
    transaction_context: &mut TxHandle,
    data: Vec<u8>,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO events (data) VALUES (?)")
        .bind(data)
        .execute(&mut *transaction_context.transaction)
        .await
        .map(|_| ())
}

// Copy and paste events over from a blueprint
#[derive(ScryptoSbor, Debug)]
pub struct InstantiateEvent {
    x_address: ResourceAddress,
    y_address: ResourceAddress,
    context_fee_rate: Decimal,
    liquidity_pool_address: ComponentAddress,
    pool_address: ComponentAddress,
}

// Implement the event handler
#[auto_decode]
pub fn handle_instantiate_event(
    context: EventHandlerContext<AppState, TxHandle>,
    event: InstantiateEvent,
) -> Result<(), EventHandlerError> {
    // Encode the component address as a bech32 string
    let component_address = encode_bech32(
        event.pool_address.as_node_id().as_bytes(),
        &context.app_state.network,
    )
    .map_err(|err| EventHandlerError::EventRetryError(err.into()))?;
    let native_address = encode_bech32(
        event.liquidity_pool_address.as_node_id().as_bytes(),
        &context.app_state.network,
    )
    .map_err(|err| EventHandlerError::UnrecoverableError(err.into()))?;

    context.app_state.async_runtime.block_on(async {
        add_to_database(
            context.transaction_handle,
            context.event.binary_sbor_data.clone(),
        )
        .await
        .map_err(|err| EventHandlerError::UnrecoverableError(err.into()))
    })?;

    // Register new event handlers for the new component
    context.handler_registry.add_handler(
        &component_address,
        "SwapEvent",
        handle_swap_event,
    );
    context.handler_registry.add_handler(
        &native_address,
        "ContributionEvent",
        handle_contribution_event,
    );
    Ok(())
}

#[derive(ScryptoSbor, Debug)]
pub struct SwapEvent {
    context_address: ResourceAddress,
    context_amount: Decimal,
    output_address: ResourceAddress,
    output_amount: Decimal,
    context_fee_lp: Decimal,
}

#[auto_decode]
pub fn handle_swap_event(
    context: EventHandlerContext<AppState, TxHandle>,
    event: SwapEvent,
) -> Result<(), EventHandlerError> {
    // info!("Handling swap event: {:#?}", event);
    context.app_state.async_runtime.block_on(async {
        add_to_database(
            context.transaction_handle,
            context.event.binary_sbor_data.clone(),
        )
        .await
        .map_err(|err| EventHandlerError::UnrecoverableError(err.into()))
    })?;
    Ok(())
}

#[derive(ScryptoSbor, Debug)]
pub struct ContributionEvent {
    pub contributed_resources: IndexMap<ResourceAddress, Decimal>,
    pub pool_units_minted: Decimal,
}

#[auto_decode]
pub fn handle_contribution_event(
    context: EventHandlerContext<AppState, TxHandle>,
    event: ContributionEvent,
) -> Result<(), EventHandlerError> {
    context.app_state.async_runtime.block_on(async {
        add_to_database(
            context.transaction_handle,
            context.event.binary_sbor_data.clone(),
        )
        .await
        .map_err(|err| EventHandlerError::UnrecoverableError(err.into()))
    })?;
    Ok(())
}

#[derive(ScryptoSbor, Debug)]
pub struct RedemptionEvent {
    pub pool_unit_tokens_redeemed: Decimal,
    pub redeemed_resources: IndexMap<ResourceAddress, Decimal>,
}

#[auto_decode]
pub fn handle_redemption_event(
    context: EventHandlerContext<AppState, TxHandle>,
    event: RedemptionEvent,
) -> Result<(), EventHandlerError> {
    context.app_state.async_runtime.block_on(async {
        add_to_database(
            context.transaction_handle,
            context.event.binary_sbor_data.clone(),
        )
        .await
        .map_err(|err| EventHandlerError::UnrecoverableError(err.into()))
    })?;
    Ok(())
}
