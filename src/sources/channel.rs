use crate::{models::Transaction, stream::TransactionStream};
use async_trait::async_trait;
use tokio::sync::mpsc::Receiver;

/// A transaction stream that receives transactions from a channel.
/// This is useful for controlled testing, as it allows you
/// to send transactions to the stream as you wish.
#[derive(Debug)]
pub struct ChannelTransactionStream {
    receiver: Option<tokio::sync::mpsc::Receiver<Transaction>>,
}

impl ChannelTransactionStream {
    pub fn new(
        capacity: u64,
    ) -> (Self, tokio::sync::mpsc::Sender<Transaction>) {
        let (sender, receiver) = tokio::sync::mpsc::channel(capacity as usize);
        (
            ChannelTransactionStream {
                receiver: Some(receiver),
            },
            sender,
        )
    }
}

#[async_trait]
impl TransactionStream for ChannelTransactionStream {
    async fn start(&mut self) -> Result<Receiver<Transaction>, anyhow::Error> {
        Ok(self.receiver.take().expect("Receiver already taken"))
    }
    async fn stop(&mut self) {}
}
