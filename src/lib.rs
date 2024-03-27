pub mod encodings;
pub mod error;
pub mod event_handler;
pub mod models;
pub mod processor;
pub mod sources;
pub mod stream;
pub mod transaction_handler;
pub use scrypto::prelude::{scrypto_decode, ScryptoDecode};

pub trait EventName {
    fn event_name() -> &'static str;
}
