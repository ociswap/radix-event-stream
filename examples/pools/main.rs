pub mod basicv0;
use crate::basicv0::events::{self, AppState, TxCtx};
use core::panic;
use log::error;
use radix_event_stream::error::TransactionHandlerError;
use radix_event_stream::transaction_handler::TransactionHandlerContext;
use radix_event_stream::{
    event_handler::HandlerRegistry, processor::TransactionStreamProcessor,
    sources::gateway::GatewayTransactionStream,
};
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
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
    let mut handler_registry: HandlerRegistry<AppState, TxCtx> =
        HandlerRegistry::new();

    // Add the instantiate event handler to the registry
    handler_registry.add_handler(
        "package_rdx1p5l6dp3slnh9ycd7gk700czwlck9tujn0zpdnd0efw09n2zdnn0lzx",
        "InstantiateEvent",
        events::handle_instantiate_event,
    );
    fn transaction_handler(
        context: TransactionHandlerContext<'_, AppState, TxCtx>,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<(), TransactionHandlerError>>
                + Send
                + '_,
        >,
    > {
        Box::pin(async {
            // start a database transaction
            let tx = context.app_state.pool.begin().await.unwrap();
            let tx = TxCtx { tx };

            // Handle the events in the transaction
            context
                .transaction
                .handle_events(
                    context.app_state,
                    context.handler_registry,
                    Some(&mut tx),
                )
                .await?;

            // let transaction = {
            //     // Lock the mutex to get mutable access to the Tx context (if necessary)

            //     // Take the transaction out of the Option, replacing it with None
            //     // This step assumes `tx` within `TxCtx` is an Option of the transaction type
            //     if let Some(actual_tx) = tx_lock.take() {
            //         // Commit the transaction
            //         // Some(actual_tx.commit().await)
            //         actual_tx
            //     } else {
            //         // Handle case where tx was already taken or doesn't exist
            //         return Err(TransactionHandlerError::UnrecoverableError(
            //             anyhow::anyhow!(
            //                 "Transaction already taken or doesn't exist"
            //             )
            //             .into(),
            //         ));
            //     }
            // };
            tx.tx.commit();
            // if let Some(tx) = tx_guard() {
            //     // Take the transaction out safely
            //     tx.commit().await.unwrap();
            // }

            Ok(())
        })
    }
    // Define a generic handler for transactions,
    // which the processor will call for each transaction.

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
            pool: Arc::new(pool),
            transaction: Arc::new(Mutex::new(None)),
            network: scrypto::network::NetworkDefinition::mainnet(),
        },
    );
}
