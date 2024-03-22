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

    // Create a custom application state
    let pool_store = PoolStore::new(NetworkDefinition::mainnet());
    let pool_store_rc = Rc::new(RefCell::new(pool_store));

    // add the package address of an ociswap pool
    pool_store_rc.borrow_mut().add_package_address(
        "package_rdx1p5l6dp3slnh9ycd7gk700czwlck9tujn0zpdnd0efw09n2zdnn0lzx",
        PoolType::BasicV0,
    );

    // Create a new handler registry
    let mut decoder_registry = HandlerRegistry::new();

    // Register handlers for each event type, passing in the application state
    // Applications with different state requirements can create their own handlers
    // and pass in different state.
    decoder_registry.add_handler(Box::new(
        basicv0::events::InstantiateEventHandler {
            pool_store: Rc::clone(&pool_store_rc),
        },
    ));
    decoder_registry.add_handler(Box::new(
        basicv0::events::ContributionEventHandler {
            pool_store: Rc::clone(&pool_store_rc),
        },
    ));

    // Create a new transaction stream with a json file source
    let stream = radix_event_stream::sources::file::FileTransactionStream::new(
        "examples/pools_file/transactions.json".to_string(),
    );
    // Start with parameters.
    info!("Starting stream from json file.");
    TransactionStreamProcessor::run_with(stream, decoder_registry.clone());

    // Create a new transaction stream with a yaml file source
    let stream = radix_event_stream::sources::file::FileTransactionStream::new(
        "examples/pools_file/transactions.yaml".to_string(),
    );
    // Start with parameters.
    info!("Starting stream from yaml file.");
    TransactionStreamProcessor::run_with(stream, decoder_registry);
}
