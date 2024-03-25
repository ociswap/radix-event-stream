pub mod basicv0;

use std::env;

use radix_event_stream::{
    handler::HandlerRegistry, models::Transaction,
    processor::TransactionStreamProcessor,
    sources::gateway::GatewayTransactionStream,
};

use crate::basicv0::events::{self, AppState};

fn main() {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    // Create a new handler registry
    let mut handler_registry: HandlerRegistry<AppState> =
        HandlerRegistry::new();

    handler_registry.add_handler(
        "package_rdx1p5l6dp3slnh9ycd7gk700czwlck9tujn0zpdnd0efw09n2zdnn0lzx",
        "InstantiateEvent",
        events::handle_instantiate_event,
    );

    // Create a new transaction stream
    let stream = GatewayTransactionStream::new(
        8000000,
        100,
        "https://mainnet.radixdlt.com".to_string(),
    );

    fn transaction_handler(
        app_state: &mut AppState,
        transaction: &Transaction,
        handler_registry: &mut HandlerRegistry<AppState>,
    ) {
        transaction.handle_events(app_state, handler_registry);
    }

    // Start with parameters.
    TransactionStreamProcessor::run_with(
        stream,
        handler_registry,
        transaction_handler,
        AppState { number: 0 },
    );
}
