#[derive(PartialEq, Eq, Debug, Hash)]
pub enum MetadataEventType {
    RemoveMetadataEvent,
    SetMetadataEvent,
}

pub use radix_engine::object_modules::metadata::RemoveMetadataEvent;
pub use radix_engine::object_modules::metadata::SetMetadataEvent;
