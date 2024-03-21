use crate::eventstream::DecodableEvent;
use radix_client::gateway::models::*;

impl DecodableEvent for Event {
    fn name(&self) -> &'static str {
        &self.name
    }

    fn programmatic_json(&self) -> serde_json::Value {
        self.data
    }
}
