use crate::{
    models::IncomingTransaction,
    stream::{TransactionStream, TransactionStreamError},
};

use async_trait::async_trait;

#[derive(Debug)]
pub struct ChannelTransactionStream {
    receiver: tokio::sync::mpsc::Receiver<IncomingTransaction>,
}

impl ChannelTransactionStream {
    pub fn new() -> (Self, tokio::sync::mpsc::Sender<IncomingTransaction>) {
        let (sender, receiver) = tokio::sync::mpsc::channel(100);
        (ChannelTransactionStream { receiver }, sender)
    }
}

#[async_trait]
impl TransactionStream for ChannelTransactionStream {
    async fn next(
        &mut self,
    ) -> Result<Vec<IncomingTransaction>, TransactionStreamError> {
        match self.receiver.recv().await {
            Some(transaction) => Ok(vec![transaction]),
            None => return Err(TransactionStreamError::Finished),
        }
    }
}
