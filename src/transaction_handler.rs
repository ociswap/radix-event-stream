use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
};

use crate::{
    error::TransactionHandlerError, event_handler::HandlerRegistry,
    models::IncomingTransaction,
};
use async_trait::async_trait;
use dyn_clone::DynClone;

/// A trait that abstracts a transaction handler.
#[allow(non_camel_case_types)]
#[async_trait]
pub trait TransactionHandler<STATE, TX_CTX>: DynClone + Send + Sync
where
    STATE: Clone,
{
    async fn handle(
        &self,
        input: TransactionHandlerContext<'_, STATE, TX_CTX>,
    ) -> Result<(), TransactionHandlerError>;
}

/// Implement EventHandler for all functions that have the correct signature F
#[async_trait]
impl<STATE, F, TX_CTX> TransactionHandler<STATE, TX_CTX> for F
where
    F: Fn(
            TransactionHandlerContext<STATE, TX_CTX>,
        ) -> Pin<
            Box<
                dyn Future<Output = Result<(), TransactionHandlerError>>
                    + Send
                    + '_,
            >,
        > + Clone
        + Send
        + Sync,
    STATE: Clone + Send + Sync + 'static,
{
    async fn handle(
        &self,
        context: TransactionHandlerContext<'_, STATE, TX_CTX>,
    ) -> Result<(), TransactionHandlerError> {
        self(context).await
    }
}

impl<STATE, TX_CTX> Clone for Box<dyn TransactionHandler<STATE, TX_CTX>>
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
pub struct TransactionHandlerContext<'a, STATE, TX_CTX>
where
    STATE: Clone,
{
    pub app_state: &'a mut STATE,
    pub transaction: &'a IncomingTransaction,
    pub handler_registry: &'a Arc<Mutex<HandlerRegistry<STATE, TX_CTX>>>,
}
