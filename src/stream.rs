use crate::models::Transaction;
use std::fmt::Debug;

/// A trait that abstracts a stream of transactions coming
/// from any source, like a gateway, database, or file.
pub trait TransactionStream: Debug {
    fn next(&mut self) -> Result<Vec<Transaction>, TransactionStreamError>;
}

#[derive(Debug)]
pub enum TransactionStreamError {
    CaughtUp,
    Finished,
    Error(String),
}
