pub mod basicv0;
use radix_event_stream::handler::HandlerRegistry;

use radix_event_stream::streaming::TransactionStreamProcessor;
use scrypto::network::NetworkDefinition;
use std::cell::RefCell;
use std::env;
use std::rc::Rc;

use log::info;

use crate::basicv0::models::PoolType;
use crate::basicv0::poolstore::PoolStore;

fn main() {
    env::set_var("RUST_LOG", "info");
    env_logger::init();
    info!("Starting fetcher");

    let pool_store = PoolStore::new(NetworkDefinition::mainnet());
    let pool_store_rc = Rc::new(RefCell::new(pool_store));

    pool_store_rc.borrow_mut().add_package_address(
        "package_rdx1p5l6dp3slnh9ycd7gk700czwlck9tujn0zpdnd0efw09n2zdnn0lzx",
        PoolType::BasicV0,
    );

    let mut decoder_registry = HandlerRegistry::new();

    // Register decoder for each event type
    decoder_registry.add_handler(Box::new(
        basicv0::events::InstantiateEventHandler {
            pool_store: Rc::clone(&pool_store_rc),
        },
    ));

    // Create a new transaction stream
    let stream =
        radix_event_stream::sources::gateway::GatewayTransactionStream::new(
            8000000,
            100,
            "https://mainnet.radixdlt.com".to_string(),
        );

    TransactionStreamProcessor::run_with(stream, decoder_registry);
}
