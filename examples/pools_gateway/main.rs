pub mod basicv0;
pub mod handler;

use std::env;

use radix_event_stream::handler::HandlerRegistry;

use radix_event_stream::streaming::TransactionStreamProcessor;

use log::{info, LevelFilter};
use std::io::Write;

use crate::handler::{AppState, BasicV0, Handler};

fn main() {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    // Create a new handler registry
    let mut handler_registry: HandlerRegistry<Handler, AppState> =
        HandlerRegistry::new();

    handler_registry.add_handler(
        "package_rdx1p5l6dp3slnh9ycd7gk700czwlck9tujn0zpdnd0efw09n2zdnn0lzx"
            .to_string(),
        Handler::BasicV0(BasicV0::InstantiateEvent),
    );

    // Create a new transaction stream
    let stream =
        radix_event_stream::sources::gateway::GatewayTransactionStream::new(
            8000000,
            100,
            "https://mainnet.radixdlt.com".to_string(),
        );

    fn transaction_handler(
        app_state: &mut AppState,
        transaction: &radix_event_stream::streaming::Transaction,
        handler_registry: &mut HandlerRegistry<Handler, AppState>,
    ) {
        transaction.handle_events(app_state, handler_registry);
    }

    // Start with parameters.
    TransactionStreamProcessor::run_with(
        stream,
        handler_registry,
        transaction_handler,
        Some(std::time::Duration::from_secs(1)),
        AppState { number: 0 },
    );
}
