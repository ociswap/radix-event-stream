use crate::{
    error::TransactionHandlerError, event_handler::HandlerRegistry,
    models::IncomingTransaction,
};
use async_trait::async_trait;
use dyn_clone::DynClone;

/// A trait that abstracts a transaction handler.
#[allow(non_camel_case_types)]
#[async_trait]
pub trait TransactionHandler<STATE, TRANSACTION_HANDLE>: DynClone
where
    STATE: Clone,
{
    async fn handle(
        &self,
        input: TransactionHandlerContext<'_, STATE, TRANSACTION_HANDLE>,
    ) -> Result<(), TransactionHandlerError>;
}

#[allow(non_camel_case_types)]
impl<STATE, TRANSACTION_HANDLE> Clone
    for Box<dyn TransactionHandler<STATE, TRANSACTION_HANDLE>>
where
    STATE: Clone,
{
    fn clone(&self) -> Self {
        dyn_clone::clone_box(&**self)
    }
}

#[allow(non_camel_case_types)]
/// A struct that holds the context for a transaction handler,
/// which is passed to the handler when it is called.
pub struct TransactionHandlerContext<'a, STATE, TRANSACTION_HANDLE = ()>
where
    STATE: Clone,
{
    pub app_state: &'a mut STATE,
    pub incoming_transaction: &'a IncomingTransaction,
    pub handler_registry: &'a mut HandlerRegistry<STATE, TRANSACTION_HANDLE>,
}
