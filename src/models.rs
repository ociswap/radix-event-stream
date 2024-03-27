use chrono::Utc;

/// Generic struct for incoming events from a
/// transaction stream. To implement a new transaction
/// stream type, you would typically implement `Into<IncomingEvent>`
/// for the native event type of the transaction stream.
#[derive(Debug)]
pub struct IncomingEvent {
    pub name: String,
    pub binary_sbor_data: Vec<u8>,
    pub emitter: EventEmitter,
}

#[derive(Debug, Clone)]
pub enum EventEmitter {
    Method {
        entity_address: String,
    },
    Function {
        package_address: String,
        blueprint_name: String,
    },
}

impl EventEmitter {
    /// Returns the address of the emitter, regardless of whether it is a method or function.
    pub fn address(&self) -> &str {
        match self {
            EventEmitter::Method { entity_address } => entity_address,
            EventEmitter::Function {
                package_address, ..
            } => package_address,
        }
    }
}

/// Generic struct for incoming transactions from a
/// transaction stream. To implement a new transaction
/// stream type, you would typically implement `Into<IncomingTransaction>`
/// for the native transaction type of the transaction stream.
#[derive(Debug)]
pub struct IncomingTransaction {
    pub intent_hash: String,
    pub state_version: u64,
    pub confirmed_at: Option<chrono::DateTime<Utc>>,
    pub events: Vec<IncomingEvent>,
}
