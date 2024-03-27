/// Error type which is returned from an event
/// handler by the user on failure.
#[derive(Debug)]
pub enum EventHandlerError {
    /// The event handler encountered an error and
    /// should be retried directly.
    /// This shouldn't be propagated up to the transaction handler.
    EventRetryError(anyhow::Error),
    /// The event handler encountered an error and
    /// the whole transaction should be retried.
    TransactionRetryError(anyhow::Error),
    /// The event handler encountered an unrecoverable
    /// error and the process should exit.
    UnrecoverableError(anyhow::Error),
}

/// Error type which is returned from a transaction
/// handler by the user on failure.
///
/// The typical usage is to return this error from
/// a transaction handler, and the processor calling
/// the handler will take care of retrying the transaction
/// or exiting the process.
#[derive(Debug)]
pub enum TransactionHandlerError {
    /// The transaction handler encountered an error and
    /// should be retried directly.
    TransactionRetryError(anyhow::Error),
    /// The transaction handler encountered an unrecoverable
    /// error and the process should exit.
    UnrecoverableError(anyhow::Error),
}

impl From<EventHandlerError> for TransactionHandlerError {
    fn from(e: EventHandlerError) -> Self {
        match e {
            EventHandlerError::EventRetryError(_) => {
                panic!("Event retries should be handled at the event level, not the transaction level")
            }
            EventHandlerError::TransactionRetryError(e) => {
                TransactionHandlerError::TransactionRetryError(e)
            }
            EventHandlerError::UnrecoverableError(e) => {
                TransactionHandlerError::UnrecoverableError(e)
            }
        }
    }
}
