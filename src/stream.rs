use crate::models::Transaction;
use async_trait::async_trait;
use std::fmt::Debug;
use tokio::sync::mpsc::Receiver;

/// A trait that abstracts a stream of transactions coming
/// from any source, like a gateway, database, or file.
/// The stream is started by calling `start`, which returns
/// a receiver that you can await on to receive transactions.
///
/// If the stream is finished, which can happen when processing
/// a finite source of transactions such as a file, the stream
/// should simply close the channel and the processor will exit
/// gracefully.
///
/// If a stream, like a gateway stream, is caught up, it may be
/// possible that there are no transactions to push to the channel.
/// In this case, the processor will simply wait until there are
/// transactions available, which is the default behavior when calling
/// `recv().await` on a receiver.
///
/// If the channel is full, the stream will wait until there is space
/// in the channel to push the transaction. This is the default behavior
/// when calling `send().await` on a sender.
///
/// If the processor errors, it drops the receiver, which a stream
/// will use to detect that it no longer needs to fetch transactions.
/// An explicit stop() method is still useful in more advanced cases.
#[async_trait]
pub trait TransactionStream: Debug {
    // Starts the stream. This may involve spawning a new task,
    // which pushes transactions to the channel that is returned.
    async fn start(&mut self) -> Result<Receiver<Transaction>, anyhow::Error>;

    // Explicitly stop the stream
    async fn stop(&mut self);
}
