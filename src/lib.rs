pub mod encodings;
pub mod handler;
pub mod sources;
pub mod streaming;

pub trait EventName {
    fn event_name() -> &'static str;
}
