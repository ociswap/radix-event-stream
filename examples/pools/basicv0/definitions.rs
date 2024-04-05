use std::rc::Rc;

use radix_engine_common::network::NetworkDefinition;

// Define a global state
#[derive(Debug, Clone)]
pub struct AppState {
    pub number: u64,
    pub async_runtime: Rc<tokio::runtime::Runtime>,
    pub pool: Rc<sqlx::Pool<sqlx::Sqlite>>,
    pub network: NetworkDefinition,
}

#[derive(Debug)]
pub struct TxHandle {
    pub transaction: sqlx::Transaction<'static, sqlx::Sqlite>,
}
