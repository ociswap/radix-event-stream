//! Has a trait that abstracts a stream of transactions coming
//! from any source, like a gateway, database, or file.

use crate::models::Transaction;
use async_trait::async_trait;
use std::fmt::Debug;
use tokio::sync::mpsc::Receiver;

/// A trait that abstracts a stream of transactions coming
/// from any source, like a gateway, database, or file.
/// The stream is started by calling [`TransactionStream::start`][crate::stream::TransactionStream], which returns
/// a [`tokio::sync::mpsc::Receiver`] that the [`TransactionStreamProcessor`][crate::processor::TransactionStreamProcessor]
/// awaits on to receive transactions.
///
/// If the stream is finished, which can happen when processing
/// a finite source of transactions such as a file, the stream
/// should simply close the channel and the processor will exit
/// gracefully.
///
/// If a stream, like a Gateway stream, is caught up to the latest state, it may be
/// possible that there are no transactions to push to the channel for a while.
/// In this case, the processor will simply wait until there are
/// transactions available in the channel, which is the default behavior when calling
/// `recv().await` on a receiver.
///
/// If the channel is full, the stream will wait with fetching until there is space
/// in the channel to push the transaction. This is the default behavior
/// when calling `send().await` on a sender.
///
/// If the processor fails, it drops the receiver, which a stream can implicitly
/// use to detect that it no longer needs to fetch transactions.
/// This is recommended to avoid leaking a fetching task.
/// An explicit stop() method is still useful in more advanced cases.
#[async_trait]
pub trait TransactionStream: Debug {
    // Starts the stream. This may involve spawning a new task,
    // which pushes transactions to the channel that is returned.
    async fn start(&mut self) -> Result<Receiver<Transaction>, anyhow::Error>;

    // Explicitly stop the stream
    async fn stop(&mut self);
}
