pub mod basicv0;
use crate::basicv0::events::{self, AppState};
use radix_event_stream::{
    handler::HandlerRegistry, models::Transaction,
    processor::TransactionStreamProcessor,
    sources::gateway::GatewayTransactionStream,
};
use sqlx::Acquire;
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::sync::Arc;
use std::sync::Mutex;
use std::{env, rc::Rc};

fn main() {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    // Create a new handler registry
    let mut handler_registry: HandlerRegistry<AppState> =
        HandlerRegistry::new();

    // Add the instantiate event handler to the
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

    // Define the handler for transactions
    fn transaction_handler(
        app_state: &mut AppState,
        transaction: &Transaction,
        handler_registry: &mut HandlerRegistry<AppState>,
    ) {
        // Do something like start a database transaction
        app_state.async_runtime.block_on(async {
            // start a database transaction
            let tx = app_state.pool.begin().await.unwrap();
            app_state.transaction = Arc::new(Mutex::new(Some(tx)));
        });

        // Handle the events in the transaction
        transaction.handle_events(app_state, handler_registry);

        // Commit the database transaction
        app_state.async_runtime.block_on(async {
            // commit the database transaction
            let mut tx_guard = app_state.transaction.lock().unwrap();
            if let Some(tx) = tx_guard.take() {
                // Take the transaction out safely
                tx.commit().await.unwrap();
            }
        });
    }

    let runtime = tokio::runtime::Runtime::new().unwrap();

    // Create a new database and initialize a simple schema with
    // one table: `events` with "id" and "data" columns as integer and text
    let database_url =
        "sqlite:examples/pools_gateway/basicv0/my_database.db?mode=rwc";
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
                data TEXT NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();
        pool
    });

    // Start with parameters.
    TransactionStreamProcessor::run_with(
        stream,
        handler_registry,
        transaction_handler,
        AppState {
            number: 0,
            async_runtime: Rc::new(runtime),
            pool,
            transaction: Arc::new(Mutex::new(None)),
        },
    );
}
