pub mod encodings;
pub mod handler;
pub mod models;
pub mod processor;
pub mod sources;
pub mod stream;
pub use scrypto::prelude::{scrypto_decode, ScryptoDecode};

pub trait EventName {
    fn event_name() -> &'static str;
}
