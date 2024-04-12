use log::info;
use radix_engine_common::math::Decimal;
use radix_engine_common::types::{ComponentAddress, ResourceAddress};
use radix_engine_common::ScryptoSbor;
use radix_event_stream::macros::event_handler;
use radix_event_stream::processor::SimpleTransactionStreamProcessor;
use radix_event_stream::sources::gateway::GatewayTransactionStream;
use radix_event_stream::stream::TransactionStream;
use radix_event_stream::{
    event_handler::HandlerRegistry, sources::channel::ChannelTransactionStream,
};
use std::env;

#[derive(Debug, Clone)]
struct State {
    number: u64,
}

#[derive(ScryptoSbor, Debug)]
pub struct InstantiateEvent {
    x_address: ResourceAddress,
    y_address: ResourceAddress,
    context_fee_rate: Decimal,
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
    let mut handler_registry: HandlerRegistry<State> = HandlerRegistry::new();

    // Add the instantiate event handler to the registry
    handler_registry.add_handler(
        "package_rdx1p5l6dp3slnh9ycd7gk700czwlck9tujn0zpdnd0efw09n2zdnn0lzx",
        "InstantiateEvent",
        handle_instantiate_event,
    );

    // Create a new transaction stream, which the processor will use
    // as a source of transactions.
    let mut transaction_stream = GatewayTransactionStream::new(
        1919391,
        100,
        "https://mainnet.radixdlt.com".to_string(),
    );

    // creating a new channel stream, which gives back a sender side.
    let (channel_stream, sender) = ChannelTransactionStream::new();

    // Create a new task which will push transactions to the channel stream in batches, to simulate
    // something like a testing environment.
    tokio::task::spawn(async move {
        let mut transactions = Vec::new();
        // first get 10 batches of 100 transactions
        for _ in 0..10 {
            let next = transaction_stream.next().await.unwrap();
            for transaction in next {
                transactions.push(transaction);
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        // then push a batch every 2 seconds
        for _ in 0..10 {
            println!("Pushing 100 more transactions in 2 seconds...");
            tokio::time::sleep(std::time::Duration::from_millis(2000)).await;
            for _ in 0..100 {
                sender.send(transactions.pop().unwrap()).await.unwrap();
            }
        }
        // When this task ends, the channel will be closed and the processor will finish.
    });

    // Start with parameters.
    SimpleTransactionStreamProcessor::run_with(
        channel_stream,
        handler_registry,
        State { number: 1 },
    )
    .await
    .unwrap();
}
