pub mod basicv0;
pub mod decoder;
pub mod eventstream;
pub mod gateway;
pub mod models;
pub mod poolstore;
pub mod settings;

pub trait EventName {
    fn event_name() -> &'static str;
}
