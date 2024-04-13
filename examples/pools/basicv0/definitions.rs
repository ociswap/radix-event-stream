use radix_engine_common::network::NetworkDefinition;
use std::sync::Arc;

// Define a global state
#[derive(Debug, Clone)]
pub struct State {
    pub number: u64,
    pub pool: Arc<sqlx::Pool<sqlx::Sqlite>>,
    pub network: NetworkDefinition,
}

#[derive(Debug)]
pub struct TransactionContext {
    pub transaction: sqlx::Transaction<'static, sqlx::Sqlite>,
}
