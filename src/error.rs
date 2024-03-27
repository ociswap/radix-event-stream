use radix_client::gateway::models::Event;

#[derive(Debug)]
pub enum EventHandlerError {
    EventRetryError(anyhow::Error),
    TransactionRetryError(anyhow::Error),
    UnrecoverableError(anyhow::Error),
}

#[derive(Debug)]
pub enum TransactionHandlerError {
    TransactionRetryError(anyhow::Error),
    UnrecoverableError(anyhow::Error),
}

impl From<EventHandlerError> for TransactionHandlerError {
    fn from(e: EventHandlerError) -> Self {
        match e {
            EventHandlerError::EventRetryError(e) => {
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
