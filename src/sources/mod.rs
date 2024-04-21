//! Implementations of [`TransactionStream`][crate::stream::TransactionStream] provided by
//! the framework.
//!
//! It is possible to opt in to these implementations by enabling
//! the corresponding feature flags. It is recommended to use feature flags
//! to only include the implementations that are needed for your use case,
//! because this allows you to skip some optional dependencies.

#[cfg(feature = "channel")]
pub mod channel;
#[cfg(feature = "database")]
pub mod database;
#[cfg(feature = "file")]
pub mod file;
#[cfg(feature = "gateway")]
pub mod gateway;
