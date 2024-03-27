use crate::models::IncomingTransaction;
use std::fmt::Debug;

/// A trait that abstracts a stream of transactions coming
/// from any source, like a gateway, database, or file.
pub trait TransactionStream: Debug {
    async fn next(
        &mut self,
    ) -> Result<Vec<IncomingTransaction>, TransactionStreamError>;
}

#[derive(Debug)]
pub enum TransactionStreamError {
    /// The stream is caught up with the latest transactions.
    /// The processor should wait for new transactions and try again.
    CaughtUp,
    /// The stream is finished and there are no more transactions.
    /// The processor should stop processing transactions.
    Finished,
    /// An error occurred while processing the stream.
    Error(String),
}
