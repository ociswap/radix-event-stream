use std::sync::Arc;

use radix_engine_common::network::NetworkDefinition;

// Define a global state
#[derive(Debug, Clone)]
pub struct AppState {
    pub number: u64,
    pub pool: Arc<sqlx::Pool<sqlx::Sqlite>>,
    pub network: NetworkDefinition,
}

#[derive(Debug)]
pub struct TxHandle {
    pub transaction: sqlx::Transaction<'static, sqlx::Sqlite>,
}
