use crate::eventstream::{DecodableEvent, Transaction, TransactionStream};
use radix_client::{
    gateway::{models::*, stream::TransactionStreamBlocking},
    GatewayClientBlocking,
};

impl DecodableEvent for Event {
    fn name(&self) -> &str {
        &self.name
    }

    fn programmatic_json(&self) -> serde_json::Value {
        self.data.clone()
    }
}

impl Transaction for CommittedTransactionInfo {
    fn events(&self) -> Vec<Box<dyn DecodableEvent>> {
        let events = match &self.receipt {
            Some(receipt) => match &receipt.events {
                Some(events) => events,
                None => return vec![],
            },

            None => return vec![],
        };
        events
            .iter()
            .map(|event| Box::new(event.clone()) as Box<dyn DecodableEvent>)
            .collect()
    }
    fn intent_hash(&self) -> String {
        self.intent_hash.clone().unwrap()
    }
    fn state_version(&self) -> u64 {
        self.state_version
    }
}

pub struct GatewayEventStream {
    stream: TransactionStreamBlocking,
}
impl GatewayEventStream {
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
        GatewayEventStream { stream }
    }
}

impl TransactionStream for GatewayEventStream {
    fn next(&mut self) -> Option<Vec<Box<dyn Transaction>>> {
        let response = self.stream.next().ok()?;
        Some(
            response
                .items
                .into_iter()
                .map(|item| Box::new(item) as Box<dyn Transaction>)
                .collect(),
        )
    }
}
