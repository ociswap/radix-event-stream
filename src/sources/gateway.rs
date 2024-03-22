use crate::{
    encodings::programmatic_json_to_bytes,
    streaming::{
        Event, Transaction, TransactionStream, TransactionStreamError,
    },
};
use chrono::Utc;
use radix_client::{
    gateway::{models::*, stream::TransactionStreamBlocking},
    GatewayClientBlocking,
};

impl Event for radix_client::gateway::models::Event {
    fn name(&self) -> &str {
        &self.name
    }
    fn emitter(
        &self,
    ) -> &radix_client::gateway::models::EventEmitterIdentifier {
        &self.emitter
    }
    fn binary_sbor_data(&self) -> Vec<u8> {
        programmatic_json_to_bytes(&self.data).unwrap()
    }
}

impl Transaction for CommittedTransactionInfo {
    fn events(&self) -> Vec<Box<dyn Event>> {
        let events = match &self.receipt {
            Some(receipt) => match &receipt.events {
                Some(events) => events,
                None => return vec![],
            },

            None => return vec![],
        };
        events
            .iter()
            .map(|event| Box::new(event.clone()) as Box<dyn Event>)
            .collect()
    }
    fn intent_hash(&self) -> String {
        self.intent_hash.clone().unwrap()
    }
    fn confirmed_at(&self) -> Option<chrono::DateTime<Utc>> {
        self.confirmed_at
    }
    fn state_version(&self) -> u64 {
        self.state_version
    }
}

#[derive(Debug)]
pub struct GatewayTransactionStream {
    stream: TransactionStreamBlocking,
}
impl GatewayTransactionStream {
    pub fn new(
        from_state_version: u64,
        limit_per_page: u32,
        gateway_url: String,
    ) -> Self {
        let client = GatewayClientBlocking::new(gateway_url);
        let stream =
            client.new_transaction_stream(TransactionStreamRequestBody {
                from_ledger_state: Some(LedgerStateSelector {
                    state_version: Some(from_state_version),
                    ..Default::default()
                }),
                limit_per_page: Some(limit_per_page),
                affected_global_entities_filter: None,
                opt_ins: Some(TransactionStreamOptIns {
                    receipt_events: true,
                    ..Default::default()
                }),
                order: Some(Order::Asc),
                kind_filter: TransactionKindFilter::User,
                ..Default::default()
            });
        GatewayTransactionStream { stream }
    }
}

impl TransactionStream for GatewayTransactionStream {
    fn next(
        &mut self,
    ) -> Result<Vec<Box<dyn Transaction>>, TransactionStreamError> {
        let response = self.stream.next().map_err(|err| {
            TransactionStreamError::Error(format!("{:?}", err))
        })?;
        let boxed: Vec<Box<dyn Transaction>> = response
            .items
            .into_iter()
            .map(|item| Box::new(item) as Box<dyn Transaction>)
            .collect();
        if boxed.is_empty() {
            Err(TransactionStreamError::CaughtUp)
        } else {
            Ok(boxed)
        }
    }
}
