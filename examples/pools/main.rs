pub mod basicv0;
use crate::basicv0::definitions::{AppState, TxHandle};
use crate::basicv0::events;
use log::error;
use radix_engine_common::network::NetworkDefinition;
use radix_event_stream::error::TransactionHandlerError;
use radix_event_stream::transaction_handler::TransactionHandlerContext;
use radix_event_stream::{
    event_handler::HandlerRegistry, processor::TransactionStreamProcessor,
    sources::gateway::GatewayTransactionStream,
};
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::{env, rc::Rc};

fn main() {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    let arg = match env::args().nth(1) {
        Some(arg) => arg,
        None => {
            error!("Please provide the argument 'file' to run from a file, or 'gateway' to run from a gateway.");
            std::process::exit(1);
        }
    };

    // Create a tokio runtime to run async code inside handlers.
    let runtime = tokio::runtime::Runtime::new().unwrap();

    // Create a new database and initialize a simple schema with
    // one table: `events` with "id" and "data" columns as integer and text
    let database_url = "sqlite:examples/pools/basicv0/my_database.db?mode=rwc";
    let pool = runtime.block_on(async {
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
        pool
    });

    // Create a new handler registry
    let mut handler_registry: HandlerRegistry<AppState, TxHandle> =
        HandlerRegistry::new();

    // Add the instantiate event handler to the registry
    handler_registry.add_handler(
        "package_rdx1p5l6dp3slnh9ycd7gk700czwlck9tujn0zpdnd0efw09n2zdnn0lzx",
        "InstantiateEvent",
        events::handle_instantiate_event,
    );

    // Define a generic handler for transactions,
    // which the processor will call for each transaction.
    fn transaction_handler(
        context: TransactionHandlerContext<AppState, TxHandle>,
    ) -> Result<(), TransactionHandlerError> {
        // Do something like start a database transaction
        let mut transaction_context =
            context.app_state.async_runtime.block_on(async {
                // start a database transaction
                let tx = context.app_state.pool.begin().await.unwrap();
                TxHandle { transaction: tx }
            });

        // Handle the events in the transaction
        context.transaction.handle_events(
            context.app_state,
            context.handler_registry,
            &mut transaction_context,
        )?;

        // Commit the database transaction
        context.app_state.async_runtime.block_on(async {
            transaction_context.transaction.commit().await.unwrap();
        });
        Ok(())
    }

    if arg == "file" {
        // Create a new transaction stream from a file, which the processor will use
        // as a source of transactions.
        let stream =
            radix_event_stream::sources::file::FileTransactionStream::new(
                "examples/pools/transactions.json".to_string(),
            );

        // Start with parameters.
        TransactionStreamProcessor::run_with(
            stream,
            handler_registry,
            transaction_handler,
            AppState {
                number: 0,
                async_runtime: Rc::new(runtime),
                pool: Rc::new(pool),
                network: NetworkDefinition::mainnet(),
            },
        );
    } else if arg == "gateway" {
        // Create a new transaction stream, which the processor will use
        // as a source of transactions.
        let stream = GatewayTransactionStream::new(
            1919391,
            100,
            "https://mainnet.radixdlt.com".to_string(),
        );

        // Start with parameters.
        TransactionStreamProcessor::run_with(
            stream,
            handler_registry,
            transaction_handler,
            AppState {
                number: 0,
                async_runtime: Rc::new(runtime),
                pool: Rc::new(pool),
                network: NetworkDefinition::mainnet(),
            },
        );
    }
}
