use crate::handler::HandlerRegistry;
use chrono::Utc;
use colored::Colorize;
use log::info;

#[derive(Debug)]
pub struct Event {
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
pub struct Transaction {
    pub intent_hash: String,
    pub state_version: u64,
    pub confirmed_at: Option<chrono::DateTime<Utc>>,
    pub events: Vec<Event>,
}

#[allow(non_camel_case_types)]
impl Transaction {
    pub fn handle_events<STATE>(
        &self,
        app_state: &mut STATE,
        handler_registry: &mut HandlerRegistry<STATE>,
    ) where
        STATE: Clone,
    {
        self.events.iter().for_each(|event| {
            let handler_registry_clone = handler_registry.clone();

            let event_handler = match handler_registry_clone
                .handlers
                .get(&(event.emitter.address().to_string(), event.name.clone()))
            {
                Some(handlers) => handlers,
                None => return,
            };

            info!(
                "{}",
                format!("HANDLING EVENT: {}", event.name).bright_yellow()
            );

            event_handler.handle(
                EventHandlerInput {
                    app_state,
                    transaction: self,
                    handler_registry,
                },
                event.binary_sbor_data.clone(),
            );
        });
    }
}

#[allow(non_camel_case_types)]
pub struct EventHandlerInput<'a, STATE>
where
    STATE: Clone,
{
    pub app_state: &'a mut STATE,
    pub transaction: &'a Transaction,
    pub handler_registry: &'a mut HandlerRegistry<STATE>,
}
