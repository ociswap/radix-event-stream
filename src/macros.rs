//! Re-exports proc macros for defining event and transaction handlers.
//!
//! These macros convert async functions with a correct signature
//! into a struct that implements the [`EventHandler`][crate::event_handler::EventHandler]
//! or [`TransactionHandler`][crate::transaction_handler::TransactionHandler]
//! trait.

pub use handler_macro::event_handler;
pub use handler_macro::transaction_handler;
