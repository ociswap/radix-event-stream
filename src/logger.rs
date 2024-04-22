/*!
This module contains the [`Logger`] trait and the [`DefaultLogger`] implementation.

The [`Logger`] trait is an interface of hooks called by the [`TransactionStreamProcessor`][crate::processor::TransactionStreamProcessor]
at various points in the processing of transactions. This allows for custom logging
and metric collection. The default implementation is [`DefaultLogger`].
*/

use crate::models::{Event, Transaction};
use async_trait::async_trait;
use chrono::Utc;
use colored::Colorize;
use log::{error, info};
use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

/// The interval at which the metrics are considered for the
/// `transactions_per_second` and `time_per_transaction_message` metrics.
const METRIC_CONSIDERATION_INTERVAL: Duration = Duration::from_secs(10);

/// A struct that holds metrics about the transaction stream in the [`DefaultLogger`]
pub struct StreamMetrics {
    pub transactions_seen: u64,
    pub transactions_handled: u64,
    pub events_seen: u64,
    pub events_handled: u64,
    pub time_started: Instant,
    pub last_seen_state_version: Option<u64>,
    pub last_seen_timestamp: Option<chrono::DateTime<Utc>>,
    pub recent_transactions: VecDeque<(Instant, Duration)>,
}

impl Default for StreamMetrics {
    fn default() -> Self {
        Self {
            transactions_seen: 0,
            events_seen: 0,
            transactions_handled: 0,
            events_handled: 0,
            time_started: Instant::now(),
            last_seen_state_version: None,
            last_seen_timestamp: None,
            recent_transactions: VecDeque::new(),
        }
    }
}

/// An interface of hooks for the `TransactionStreamProcessor` to call
/// at various points in the processing of transactions.
/// This allows for custom logging and metric collection.
/// The default implementation is `DefaultLogger`.
#[async_trait]
pub trait Logger: Send + Sync {
    /// Called when:
    /// - A transaction is received from the stream and is about to be processed, because one
    /// of its events has a handler associated with it.
    /// - The transaction is received from the stream and will not be processed because
    /// none of its events have a handler associated with them.
    /// - A retry error was returned from a handler and the transaction is being retried.
    ///
    /// `handling` indicates whether the transaction has any events that will be handled by an event handler or not.
    /// `is_retry` indicates whether the transaction is currently being retried or not.
    async fn receive_transaction(
        &mut self,
        transaction: &Transaction,
        handling: bool,
        is_retry: bool,
    );
    /// Called when:
    /// - A transaction has been processed and all of its events have been handled.
    /// - A transaction has been processed, but nothing was done with it because
    ///  none of its events had a handler associated with them.
    /// - A transaction has been retried and is now successfully processed.
    ///
    /// `handling` indicates whether the transaction had any events that were handled by an event handler or not.
    async fn finish_transaction(
        &mut self,
        transaction: &Transaction,
        handling: bool,
    );
    /// Called when:
    /// - An event is received from the stream and is about to be processed.
    /// - An event is received from the stream and will not be processed because
    /// it has no handler associated with it.
    /// - A retry error was returned from a handler and the event is being retried.
    ///
    /// `handling` indicates whether the event will be handled by a handler or not.
    /// `is_retry` indicates whether the event is currently being retried or not.
    async fn receive_event(
        &mut self,
        transaction: &Transaction,
        event: &Event,
        handling: bool,
        is_retry: bool,
    );
    /// Called when:
    /// - An event has been processed.
    /// - An event has been processed, but nothing was done with it because
    /// it had no handler associated with it.
    /// - An event has been retried and is now successfully processed.
    ///
    /// `handling` indicates whether the event was handled by a handler or not.
    async fn finish_event(
        &mut self,
        transaction: &Transaction,
        event: &Event,
        handling: bool,
    );
    /// Called when an `EventRetryError` is returned from a handler
    /// and the event is being retried. It could be called multiple times
    /// for the same event if it continues to fail.
    async fn event_retry_error(
        &mut self,
        transaction: &Transaction,
        event: &Event,
        error: &anyhow::Error,
        timeout: Duration,
    );
    /// Called when a `TransactionRetryError` is returned from a handler
    /// and the transaction is being retried.
    /// It could be called multiple times for the same transaction if it continues to fail.
    async fn transaction_retry_error(
        &mut self,
        transaction: &Transaction,
        error: &anyhow::Error,
        timeout: Duration,
    );
    /// Called when an `UnrecoverableError` is returned from a handler
    /// and the processor should stop processing.
    async fn unrecoverable_error(&mut self, error: &anyhow::Error);
    /// Called periodically by an independent task. This is useful for
    /// logging and metric collection. It is possible to set a custom
    /// interval by implementing the `periodic_report_interval` method.
    async fn periodic_report(&self);
    /// Called by the processor to determine the interval at which
    /// the `periodic_report` method should be called inside of
    /// an independent task.
    fn periodic_report_interval(&self) -> Duration;
}

/// The default logger implementation for the `TransactionStreamProcessor`.
/// This logger collects some metrics about the transaction stream
/// and logs them periodically. It also logs information about transactions
/// and events as they are processed.
pub struct DefaultLogger {
    metrics: StreamMetrics,
    transaction_stopwatch: Instant,
    event_stopwatch: Instant,
    custom_report_interval: Option<Duration>,
}

impl Default for DefaultLogger {
    fn default() -> Self {
        Self {
            metrics: StreamMetrics::default(),
            transaction_stopwatch: Instant::now(),
            event_stopwatch: Instant::now(),
            custom_report_interval: None,
        }
    }
}

impl DefaultLogger {
    pub fn with_custom_report_interval(
        custom_report_interval: Duration,
    ) -> Self {
        Self {
            custom_report_interval: Some(custom_report_interval),
            ..Self::default()
        }
    }
}

#[async_trait]
impl Logger for DefaultLogger {
    async fn receive_transaction(
        &mut self,
        transaction: &Transaction,
        handling: bool,
        retry: bool,
    ) {
        self.transaction_stopwatch = Instant::now();
        if handling {
            if !retry {
                let line =
                    "--------------------------------------------------------"
                        .bright_blue();
                info!("{}", line);
            }
            let message = format!(
                "HANDLING TRANSACTION - {:#?} - {}",
                transaction.state_version,
                transaction.confirmed_at
                    .expect("When handling a transaction it should always have a timestamp")
                    .format("%a %d-%m-%Y %H:%M")
            ).bright_green();
            info!("{}", message);
        }
    }

    async fn finish_transaction(
        &mut self,
        transaction: &Transaction,
        handling: bool,
    ) {
        self.metrics.transactions_seen += 1;
        self.metrics.last_seen_state_version = Some(transaction.state_version);
        self.metrics.last_seen_timestamp = transaction.confirmed_at;
        let time_spent = self.transaction_stopwatch.elapsed();
        self.metrics
            .recent_transactions
            .push_back((Instant::now(), time_spent));
        let threshold = Instant::now() - METRIC_CONSIDERATION_INTERVAL;
        while let Some(&(time, _)) = self.metrics.recent_transactions.front() {
            if time < threshold {
                self.metrics.recent_transactions.pop_front();
            } else {
                break;
            }
        }
        if handling {
            self.metrics.transactions_handled += 1;
            let message = format!(
                "###### END TRANSACTION - HANDLED IN {}ms ######",
                time_spent.as_millis()
            )
            .bright_green();
            let line =
                "--------------------------------------------------------"
                    .bright_blue();
            info!("{}", message);
            info!("{}", line);
        }
    }

    async fn receive_event(
        &mut self,
        _transaction: &Transaction,
        event: &Event,
        handling: bool,
        _retry: bool,
    ) {
        if handling {
            self.event_stopwatch = Instant::now();
            let message =
                format!("HANDLING EVENT: {}", event.name).bright_yellow();
            info!("{}", message);
        }
    }

    async fn finish_event(
        &mut self,
        _transaction: &Transaction,
        _event: &Event,
        handling: bool,
    ) {
        self.metrics.events_seen += 1;
        if handling {
            self.metrics.events_handled += 1;
        }
    }

    async fn event_retry_error(
        &mut self,
        _transaction: &Transaction,
        event: &Event,
        error: &anyhow::Error,
        timeout: Duration,
    ) {
        let message =
            format!("ERROR HANDLING EVENT: {} - {:?}", event.name, error)
                .bright_red();
        let retry_message =
            format!("RETRYING IN {:.1} SECONDS\n", timeout.as_secs_f32())
                .bright_yellow();

        error!("{}", message);
        info!("{}", retry_message);
    }

    async fn transaction_retry_error(
        &mut self,
        _transaction: &Transaction,
        error: &anyhow::Error,
        timeout: Duration,
    ) {
        let message =
            format!("FATAL ERROR HANDLING TRANSACTION: {:?}\n", error)
                .bright_red();

        let retry_message =
            format!("RETRYING IN {:.1} SECONDS\n", timeout.as_secs_f32())
                .bright_yellow();

        error!("{}", message);
        info!("{}", retry_message);
    }

    async fn unrecoverable_error(&mut self, error: &anyhow::Error) {
        let message = format!("UNRECOVERABLE ERROR: {:?}", error).bright_red();
        error!("{}", message);
    }

    async fn periodic_report(&self) {
        match self.metrics.last_seen_state_version {
            Some(state_version) => {
                let state_message = format!(
                    "HANDLED UP TO: {} - {}",
                    state_version,
                    self.metrics.last_seen_timestamp
                        .expect("When handling a transaction it should always have a timestamp")
                        .format("%a %d-%m-%Y %H:%M")
                ).bright_blue();

                let transaction_amount = self
                    .metrics
                    .recent_transactions
                    .iter()
                    .filter(|(time, _)| {
                        time > &(Instant::now() - METRIC_CONSIDERATION_INTERVAL)
                    })
                    .count();
                let transactions_per_second = transaction_amount
                    / METRIC_CONSIDERATION_INTERVAL.as_secs().min(
                        self.metrics.time_started.elapsed().as_secs().max(1),
                    ) as usize;
                let transactions_per_second_message = format!(
                    "PROCESSING ~{} TRANSACTIONS PER SECOND",
                    transactions_per_second
                )
                .bright_blue();
                let time_per_transaction = self
                    .metrics
                    .recent_transactions
                    .iter()
                    .fold(Duration::from_secs(0), |acc, (_, duration)| {
                        acc + *duration
                    })
                    / transaction_amount as u32;
                let time_per_transaction_message = format!(
                    "AVERAGE TIME PER TRANSACTION: ~{:?}",
                    time_per_transaction
                )
                .bright_blue();
                info!("{}", state_message);
                info!("{}", transactions_per_second_message);
                info!("{}", time_per_transaction_message);
            }
            None => {
                info!("{}", "NO TRANSACTIONS HANDLED YET".bright_blue());
            }
        }
    }

    fn periodic_report_interval(&self) -> Duration {
        self.custom_report_interval
            .unwrap_or(Duration::from_secs(5))
    }
}
