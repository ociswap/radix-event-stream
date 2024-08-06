use chrono::{DateTime, Utc};
use log::info;
use radix_common::math::Decimal;
use radix_common::prelude::{
    scrypto_encode, AddressBech32Decoder, NetworkDefinition,
};
use radix_common::types::{ComponentAddress, ResourceAddress};
use radix_common::ScryptoSbor;
use radix_event_stream::event_handler::HandlerRegistry;
use radix_event_stream::macros::event_handler;
use radix_event_stream::models::{Event, EventEmitter, Transaction};
use radix_event_stream::processor::TransactionProcessor;
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

    let mut transactions = Vec::new();

    let event = InstantiateEvent {
        x_address: ResourceAddress::try_from_bech32(
            &AddressBech32Decoder::new(&NetworkDefinition::mainnet()),
            "resource_rdx1t52pvtk5wfhltchwh3rkzls2x0r98fw9cjhpyrf3vsykhkuwrf7jg8",
        ).unwrap(),
        y_address: ResourceAddress::try_from_bech32(
            &AddressBech32Decoder::new(&NetworkDefinition::mainnet()),
            "resource_rdx1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxradxrd",
        ).unwrap(),
        inpu_fee_rate: Decimal::one(),
        liquidity_pool_address: ComponentAddress::try_from_bech32(
                &AddressBech32Decoder::new(&NetworkDefinition::mainnet()),
                "pool_rdx1ckyg8aujf09uh8qlz6asst75g5w6pl6vu8nl6qrhskawcndyk6585y",
            )
            .unwrap(),
        pool_address: ComponentAddress::try_from_bech32(
            &AddressBech32Decoder::new(&NetworkDefinition::mainnet()),
            "component_rdx1cz89w3ecvh9jvdd892vycs44rr042lteg75zgdydq9csn5d87snvdw",
        )
        .unwrap(),
    };

    let encoded_event = scrypto_encode(&event).unwrap();

    transactions.push(Transaction {
        events: vec![Event {
            name: "InstantiateEvent".to_string(),
            binary_sbor_data: encoded_event,
            emitter: EventEmitter::Function {
                package_address: "package_rdx1p5l6dp3slnh9ycd7gk700czwlck9tujn0zpdnd0efw09n2zdnn0lzx".to_string(),
                blueprint_name: "BasicPool".to_string(),
            },
        }],
        confirmed_at: Some(DateTime::<Utc>::default()),
        ..Transaction::default()
    });

    let mut processor =
        TransactionProcessor::new(handler_registry, State { number: 1 });

    // Process the transactions by passing in the transactions to the TransactionProcessor
    // see also the process_transaction method to process only one transaction
    processor.process_transactions(&transactions).await.unwrap();
    // Do some checking here to see if the event handlers are correct.
}
