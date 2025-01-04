use log::info;
use radix_common::math::Decimal;
use radix_common::types::{ComponentAddress, ResourceAddress};
use radix_common::ScryptoSbor;
use radix_event_stream::event_handler::HandlerRegistry;
use radix_event_stream::macros::event_handler;
use radix_event_stream::processor::TransactionStreamProcessor;
use radix_event_stream::sources::gateway::GatewayTransactionStream;
use std::env;

#[derive(Debug, Clone)]
struct State {
    number: u64,
}

#[derive(ScryptoSbor, Debug)]
pub struct InstantiateEvent {
    x_address: ResourceAddress,
    y_address: ResourceAddress,
    input_fee_rate: Decimal,
    liquidity_pool_address: ComponentAddress,
    pool_address: ComponentAddress,
}

#[event_handler]
pub async fn handle_instantiate_event(
    context: EventHandlerContext<State>,
    event: InstantiateEvent,
) -> Result<(), EventHandlerError> {
    info!(
        "Handling the {}th instantiate event: {:#?}",
        context.state.number, event
    );
    context.state.number += 1;
    Ok(())
}

#[tokio::main]
async fn main() {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    // Create a new handler registry
    let mut handler_registry: HandlerRegistry = HandlerRegistry::new();

    // Add the instantiate event handler to the registry
    handler_registry.add_handler(
        "package_rdx1p5l6dp3slnh9ycd7gk700czwlck9tujn0zpdnd0efw09n2zdnn0lzx",
        "InstantiateEvent",
        handle_instantiate_event,
    );

    // Create a new transaction stream, which the processor will use
    // as a source of transactions.
    let stream = GatewayTransactionStream::new()
        .gateway_url("https://mainnet.radixdlt.com".to_string())
        .from_state_version(186079126)
        .buffer_capacity(1000)
        .limit_per_page(100);

    // Start with parameters.
    TransactionStreamProcessor::new(
        stream,
        handler_registry,
        State { number: 1 },
    )
    .run()
    .await
    .unwrap();
}
