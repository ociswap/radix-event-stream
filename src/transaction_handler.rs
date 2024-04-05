use crate::{
    error::TransactionHandlerError, event_handler::HandlerRegistry,
    models::IncomingTransaction,
};
use dyn_clone::DynClone;

#[allow(non_camel_case_types)]
/// A trait that abstracts a transaction handler.
pub trait TransactionHandler<STATE, TRANSACTION_HANDLE>: DynClone
where
    STATE: Clone,
{
    fn handle(
        &self,
        input: TransactionHandlerContext<STATE, TRANSACTION_HANDLE>,
    ) -> Result<(), TransactionHandlerError>;
}

/// Implement EventHandler for all functions that have the correct signature F
#[allow(non_camel_case_types)]
impl<STATE, TRANSACTION_HANDLE, F> TransactionHandler<STATE, TRANSACTION_HANDLE>
    for F
where
    F: Fn(
            TransactionHandlerContext<STATE, TRANSACTION_HANDLE>,
        ) -> Result<(), TransactionHandlerError>
        + Clone,
    STATE: Clone,
{
    fn handle(
        &self,
        context: TransactionHandlerContext<STATE, TRANSACTION_HANDLE>,
    ) -> Result<(), TransactionHandlerError> {
        self(context)
    }
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
    pub transaction: &'a IncomingTransaction,
    pub handler_registry: &'a mut HandlerRegistry<STATE, TRANSACTION_HANDLE>,
}
