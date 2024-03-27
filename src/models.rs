use crate::handler::HandlerRegistry;
use chrono::Utc;

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
    pub fn address(&self) -> &str {
        match self {
            EventEmitter::Method { entity_address } => entity_address,
            EventEmitter::Function {
                package_address, ..
            } => package_address,
        }
    }
}

#[derive(Debug)]
pub struct IncomingTransaction {
    pub intent_hash: String,
    pub state_version: u64,
    pub confirmed_at: Option<chrono::DateTime<Utc>>,
    pub events: Vec<IncomingEvent>,
}

#[allow(non_camel_case_types)]
pub struct EventHandlerContext<'a, STATE>
where
    STATE: Clone + Send + Sync,
{
    pub app_state: &'a mut STATE,
    pub transaction: &'a IncomingTransaction,
    pub event: &'a IncomingEvent,
    pub handler_registry: &'a mut HandlerRegistry<STATE>,
}
