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

    // Create a custom application state
    let pool_store = PoolStore::new(NetworkDefinition::mainnet());
    let pool_store_rc = Rc::new(RefCell::new(pool_store));

    // add the package address of an ociswap pool
    pool_store_rc.borrow_mut().add_package_address(
        "package_rdx1p5l6dp3slnh9ycd7gk700czwlck9tujn0zpdnd0efw09n2zdnn0lzx",
        PoolType::BasicV0,
    );

    // Create a new handler registry
    let mut handler_registry = HandlerRegistry::new();

    // Register handlers for each event type, passing in the application state
    // Applications with different state requirements can create their own handlers
    // and pass in different state.
    handler_registry.add_handler(basicv0::events::InstantiateEventHandler {
        pool_store: Rc::clone(&pool_store_rc),
    });
    handler_registry.add_handler(basicv0::events::ContributionEventHandler {
        pool_store: Rc::clone(&pool_store_rc),
    });

    // Create a new transaction stream
    let stream =
        radix_event_stream::sources::gateway::GatewayTransactionStream::new(
            8000000,
            100,
            "https://mainnet.radixdlt.com".to_string(),
        );

    // Start with parameters.
    TransactionStreamProcessor::run_with(
        stream,
        handler_registry.to_owned(),
        Some(std::time::Duration::from_secs(1)),
    );
}
