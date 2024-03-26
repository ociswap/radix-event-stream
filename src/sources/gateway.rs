use crate::{
    encodings::programmatic_json_to_bytes,
    models::{EventEmitter, IncomingEvent, IncomingTransaction},
    stream::{TransactionStream, TransactionStreamError},
};

use radix_client::{
    gateway::{
        models::{
            CommittedTransactionInfo, EventEmitterIdentifier,
            LedgerStateSelector, Order, TransactionKindFilter,
            TransactionStreamOptIns, TransactionStreamRequestBody,
        },
        stream::TransactionStreamBlocking,
    },
    GatewayClientBlocking,
};

impl Into<IncomingEvent> for radix_client::gateway::models::Event {
    fn into(self) -> IncomingEvent {
        let emitter = match self.emitter {
            EventEmitterIdentifier::Method { entity, .. } => {
                EventEmitter::Method {
                    entity_address: entity.entity_address,
                }
            }
            EventEmitterIdentifier::Function {
                package_address,
                blueprint_name,
            } => EventEmitter::Function {
                package_address,
                blueprint_name,
            },
        };
        IncomingEvent {
            name: self.name,
            emitter: emitter,
            binary_sbor_data: programmatic_json_to_bytes(&self.data).unwrap(),
        }
    }
}

impl Into<IncomingTransaction> for CommittedTransactionInfo {
    fn into(self) -> IncomingTransaction {
        IncomingTransaction {
            intent_hash: self.intent_hash.unwrap(),
            state_version: self.state_version,
            confirmed_at: self.confirmed_at,
            events: self
                .receipt
                .unwrap()
                .events
                .unwrap()
                .into_iter()
                .map(|event| event.into())
                .collect(),
        }
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
    ) -> Result<Vec<IncomingTransaction>, TransactionStreamError> {
        let response = self.stream.next().map_err(|err| {
            TransactionStreamError::Error(format!("{:?}", err))
        })?;
        let boxed: Vec<IncomingTransaction> =
            response.items.into_iter().map(|item| item.into()).collect();
        if boxed.is_empty() {
            Err(TransactionStreamError::CaughtUp)
        } else {
            Ok(boxed)
        }
    }
}
