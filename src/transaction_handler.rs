use crate::{
    error::TransactionHandlerError, event_handler::HandlerRegistry,
    models::Transaction,
};
use async_trait::async_trait;

/// A trait that abstracts a transaction handler.
#[allow(non_camel_case_types)]
#[async_trait]
pub trait TransactionHandler<STATE> {
    async fn handle(
        &self,
        input: TransactionHandlerContext<'_, STATE>,
    ) -> Result<(), TransactionHandlerError>;
}

#[allow(non_camel_case_types)]
/// A struct that holds the context for a transaction handler,
/// which is passed to the handler when it is called.
pub struct TransactionHandlerContext<'a, STATE> {
    pub state: &'a mut STATE,
    pub transaction: &'a Transaction,
    pub handler_registry: &'a mut HandlerRegistry,
}
