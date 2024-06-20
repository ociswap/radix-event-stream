pub mod basicv0;
use crate::basicv0::definitions::{State, TransactionContext};
use crate::basicv0::events;
use log::error;
use radix_common::network::NetworkDefinition;
use radix_event_stream::macros::transaction_handler;
use radix_event_stream::sources::database::DatabaseTransactionStream;
use radix_event_stream::transaction_handler::TransactionHandler;
use radix_event_stream::{
    event_handler::HandlerRegistry,
    processor::TransactionStreamProcessor,
    sources::{file::FileTransactionStream, gateway::GatewayTransactionStream},
};
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::env;
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    let arg = match env::args().nth(1) {
        Some(arg) => arg,
        None => {
            error!("Please provide the argument 'file' to run from a file, or 'gateway' to run from a gateway.");
            std::process::exit(1);
        }
    };

    // Create a new database and initialize a simple schema with
    // one table: `events` with "id" and "data" columns as integer and bytes respectively
    let database_url = "sqlite:examples/pools/basicv0/my_database.db?mode=rwc";
    let pool: Pool<Sqlite> = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
        .unwrap();
    sqlx::query(
        r#"
            CREATE TABLE IF NOT EXISTS events (
                id INTEGER PRIMARY KEY,
                data BYTES NOT NULL
            )
            "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    // Create a new handler registry for event handlers
    let mut handler_registry = HandlerRegistry::new();

    // Add the 'InstantiateEvent' handler to the registry
    handler_registry.add_handler(
        "package_rdx1p5l6dp3slnh9ycd7gk700czwlck9tujn0zpdnd0efw09n2zdnn0lzx",
        "InstantiateEvent",
        events::handle_instantiate_event,
    );

    #[transaction_handler]
    async fn transaction_handler(
        context: TransactionHandlerContext<State>,
    ) -> Result<(), TransactionHandlerError> {
        let tx = context.state.pool.begin().await.unwrap();
        let mut transaction_context = TransactionContext { transaction: tx };

        // Handle the events in the transaction
        context
            .event_processor
            .process_events(
                context.state,
                context.handler_registry,
                &mut transaction_context,
            )
            .await?;

        // Commit the database transaction
        transaction_context.transaction.commit().await.unwrap();
        Ok(())
    }

    match arg.as_str() {
        "file" => {
            run_from_file(handler_registry, pool, transaction_handler).await
        }
        "gateway" => {
            run_from_gateway(handler_registry, pool, transaction_handler).await
        }
        "database" => {
            run_from_database(handler_registry, pool, transaction_handler).await
        }
        _ => {
            unreachable!();
        }
    }
}

async fn run_from_file(
    handler_registry: HandlerRegistry,
    pool: Pool<Sqlite>,
    transaction_handler: impl TransactionHandler<State> + 'static,
) {
    // Create a new transaction stream from a file, which the processor will use
    // as a source of transactions.
    let stream = FileTransactionStream::new(
        "examples/pools/transactions.json".to_string(),
    );

    let state = State {
        number: 0,
        pool: Arc::new(pool),
        network: NetworkDefinition::mainnet(),
    };

    // Start with parameters.
    TransactionStreamProcessor::new(stream, handler_registry, state)
        .transaction_handler(transaction_handler)
        .run()
        .await
        .unwrap();
}

async fn run_from_gateway(
    handler_registry: HandlerRegistry,
    pool: Pool<Sqlite>,
    transaction_handler: impl TransactionHandler<State> + 'static,
) {
    // Create a new transaction stream, which the processor will use
    // as a source of transactions.
    let stream = GatewayTransactionStream::new()
        .gateway_url("https://mainnet.radixdlt.com".to_string())
        .from_state_version(1919391)
        .buffer_capacity(1000)
        .limit_per_page(100);

    let state = State {
        number: 0,
        pool: Arc::new(pool),
        network: NetworkDefinition::mainnet(),
    };

    // Start with parameters.
    TransactionStreamProcessor::new(stream, handler_registry, state)
        .transaction_handler(transaction_handler)
        .run()
        .await
        .unwrap();
}

async fn run_from_database(
    handler_registry: HandlerRegistry,
    pool: Pool<Sqlite>,
    transaction_handler: impl TransactionHandler<State> + 'static,
) {
    // Create a new transaction stream, which the processor will use
    // as a source of transactions.

    let stream = DatabaseTransactionStream::new(
        // This database is public, but I would recommend not using it for anything outside
        // of testing.
        "postgresql://radix:radix@db.radix.live/radix_ledger".to_string(),
    )
    .from_state_version(1919391)
    .buffer_capacity(1_000_000)
    .limit_per_page(10_000);

    let state = State {
        number: 0,
        pool: Arc::new(pool),
        network: NetworkDefinition::mainnet(),
    };

    // Start with parameters.
    TransactionStreamProcessor::new(stream, handler_registry, state)
        .transaction_handler(transaction_handler)
        .default_logger_with_report_interval(Duration::from_millis(500))
        .run()
        .await
        .unwrap();
}
