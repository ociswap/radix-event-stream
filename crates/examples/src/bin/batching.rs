use log::info;
use radix_common::math::Decimal;
use radix_common::types::{ComponentAddress, ResourceAddress};
use radix_common::ScryptoSbor;
use radix_event_stream::event_handler::HandlerRegistry;
use radix_event_stream::macros::event_handler;
use radix_event_stream::processor::TransactionProcessor;
use radix_event_stream::sources::gateway::GatewayTransactionStream;
use radix_event_stream::stream::TransactionStream;
use std::env;

#[derive(Debug, Clone)]
struct State {
    number: u64,
}

#[derive(ScryptoSbor, Debug)]
pub struct InstantiateEvent {
    x_address: ResourceAddress,
    y_address: ResourceAddress,
    inpu_fee_rate: Decimal,
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
    let mut handler_registry = HandlerRegistry::new();

    // Add the instantiate event handler to the registry
    handler_registry.add_handler(
        "package_rdx1p5l6dp3slnh9ycd7gk700czwlck9tujn0zpdnd0efw09n2zdnn0lzx",
        "InstantiateEvent",
        handle_instantiate_event,
    );

    // Create a new transaction stream, which the processor will use
    // as a source of transactions.
    let mut gateway_stream = GatewayTransactionStream::new()
        .from_state_version(71656472)
        .gateway_url("https://mainnet.radixdlt.com".to_string())
        .buffer_capacity(1000)
        .limit_per_page(100);

    let mut transactions = Vec::new();
    let mut next = gateway_stream.start().await.unwrap();
    // first get 1000 transactions from the gateway stream
    for _ in 0..1000 {
        let transaction = next.recv().await.unwrap();
        transactions.push(transaction);
    }

    let mut processor =
        TransactionProcessor::new(handler_registry, State { number: 1 });

    // group the transactions into batches of 100
    let transaction_batches = transactions.chunks(100);

    for transaction_batch in transaction_batches {
        processor
            .process_transactions(transaction_batch)
            .await
            .unwrap();
    }
}
