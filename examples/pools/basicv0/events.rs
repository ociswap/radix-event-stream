use super::definitions::*;
use radix_engine_common::{
    math::Decimal,
    types::{ComponentAddress, ResourceAddress},
    ScryptoSbor,
};
use radix_event_stream::macros::event_handler;
use radix_event_stream::{encodings::encode_bech32m, error::EventHandlerError};
use sbor::rust::collections::IndexMap;

async fn add_to_database(
    transaction_context: &mut TransactionContext,
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

#[event_handler]
async fn handle_instantiate_event(
    context: EventHandlerContext<State, TransactionContext>,
    event: InstantiateEvent,
) -> Result<(), EventHandlerError> {
    let component_address = encode_bech32m(
        event.pool_address.as_node_id().as_bytes(),
        &context.state.network,
    )
    .map_err(|err| EventHandlerError::EventRetryError(err.into()))?;
    let native_address = encode_bech32m(
        event.liquidity_pool_address.as_node_id().as_bytes(),
        &context.state.network,
    )
    .map_err(|err| EventHandlerError::UnrecoverableError(err.into()))?;

    add_to_database(
        context.transaction_context,
        context.event.binary_sbor_data.clone(),
    )
    .await
    .map_err(|err| EventHandlerError::UnrecoverableError(err.into()))?;

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

#[event_handler]
pub async fn handle_swap_event(
    context: EventHandlerContext<State, TransactionContext>,
    event: SwapEvent,
) -> Result<(), EventHandlerError> {
    // info!("Handling swap event: {:#?}", event);
    add_to_database(
        context.transaction_context,
        context.event.binary_sbor_data.clone(),
    )
    .await
    .map_err(|err| EventHandlerError::UnrecoverableError(err.into()))?;
    Ok(())
}

#[derive(ScryptoSbor, Debug)]
pub struct ContributionEvent {
    pub contributed_resources: IndexMap<ResourceAddress, Decimal>,
    pub pool_units_minted: Decimal,
}

#[event_handler]
pub async fn handle_contribution_event(
    context: EventHandlerContext<State, TransactionContext>,
    event: ContributionEvent,
) -> Result<(), EventHandlerError> {
    add_to_database(
        context.transaction_context,
        context.event.binary_sbor_data.clone(),
    )
    .await
    .map_err(|err| EventHandlerError::UnrecoverableError(err.into()))?;
    Ok(())
}

#[derive(ScryptoSbor, Debug)]
pub struct RedemptionEvent {
    pub pool_unit_tokens_redeemed: Decimal,
    pub redeemed_resources: IndexMap<ResourceAddress, Decimal>,
}

#[event_handler]
pub async fn handle_redemption_event(
    context: EventHandlerContext<State, TransactionContext>,
    event: RedemptionEvent,
) -> Result<(), EventHandlerError> {
    add_to_database(
        context.transaction_context,
        context.event.binary_sbor_data.clone(),
    )
    .await
    .map_err(|err| EventHandlerError::UnrecoverableError(err.into()))?;
    Ok(())
}
