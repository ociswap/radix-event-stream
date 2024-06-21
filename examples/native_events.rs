use log::info;
use radix_engine::object_modules::metadata::SetMetadataEvent;
use radix_event_stream::event_handler::HandlerRegistry;
use radix_event_stream::macros::event_handler;
use radix_event_stream::native_events::metadata::MetadataEventType;
use radix_event_stream::native_events::NativeEventType;
use radix_event_stream::processor::TransactionStreamProcessor;
use radix_event_stream::sources::database::DatabaseTransactionStream;
use std::env;

#[derive(Debug, Clone)]
struct State {
    number: u64,
}

#[event_handler]
pub async fn handler(
    context: EventHandlerContext<State>,
    event: SetMetadataEvent,
) -> Result<(), EventHandlerError> {
    info!(
        "Handling the {}th event: {:#?}",
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

    // Add the event handler to the registry
    handler_registry.set_native_handler(
        // select the event type by using the enum
        NativeEventType::Metadata(MetadataEventType::SetMetadataEvent),
        handler,
    );

    // Create a new transaction stream, which the processor will use
    // as a source of transactions.
    let stream = DatabaseTransactionStream::new(
        // This database is public, but I would recommend not using it for anything outside
        // of testing.
        "postgresql://radix:radix@db.radix.live/radix_ledger".to_string(),
    )
    .from_state_version(1919391)
    .buffer_capacity(100_000)
    .limit_per_page(10_000);

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
