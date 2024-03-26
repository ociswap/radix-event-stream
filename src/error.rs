#[derive(Debug)]
pub enum EventHandlerError {
    EventRetryError(anyhow::Error),
    TransactionRetryError(anyhow::Error),
    UnrecoverableError(anyhow::Error),
}
