use radix_engine_toolkit::functions::scrypto_sbor::{
    encode_string_representation, StringRepresentation,
};
use radix_event_stream::decoder::HandlerRegistry;
use radix_event_stream::eventstream::EventStreamProcessor;
use radix_event_stream::models::PoolType;
use radix_event_stream::poolstore::PoolStore;

use std::cell::RefCell;
use std::rc::Rc;
use std::thread::sleep;

use log::{info, warn};
use radix_client::gateway::models::*;
use radix_client::GatewayClientBlocking;
use radix_event_stream::basicv0;
use radix_event_stream::models::*;

fn main() {
    let settings = radix_event_stream::settings::Settings::init();
    // Settings::init() brings .env into the environment variables of the process.
    // That's why LOG_LEVEL works here. Maybe this is not very transparent.
    env_logger::init_from_env(env_logger::Env::new().filter("LOG_LEVEL"));

    info!("Starting fetcher");

    let pool_store = PoolStore::new(settings.network);
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
    // decoder_registry.add_handler(Box::new(basicv0::events::SwapEventDecoder {
    //     pool_store: Rc::clone(&pool_store_rc),
    // }));
    // decoder_registry.add_handler(Box::new(
    //     basicv0::events::ContributionEventDecoder {
    //         pool_store: Rc::clone(&pool_store_rc),
    //     },
    // ));
    // decoder_registry.add_handler(Box::new(
    //     basicv0::events::RedemptionEventDecoder {
    //         pool_store: Rc::clone(&pool_store_rc),
    //     },
    // ));

    // Create a new transaction stream
    let stream = radix_event_stream::gateway::GatewayEventStream::new(
        settings.start_from_state_version,
        settings.limit_per_page,
        settings.radix_gateway_url,
    );

    let mut processor = EventStreamProcessor::new(stream, decoder_registry);
    processor.run();
}

// #[cfg(test)]
// mod tests {
//     use fetcher::basicv0;

//     use super::*;
//     use fetcher::EventName;

//     #[test]
//     fn test_event_name_derive() {
//         println!("{}", basicv0::events::InstantiateEvent::event_name()); // Prints "MyEvent"
//     }
// }
