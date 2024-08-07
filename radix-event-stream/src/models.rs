//!
//! Contains canonical models for ledger events and transactions.
//! These models are used by the [`TransactionStream`][crate::stream::TransactionStream]
//! and [`TransactionStreamProcessor`][crate::processor::TransactionStreamProcessor]
//! to abstract the source of transactions and events.
//!
//! When implementing a new transaction stream, you will typically
//! convert the native representations of events and transactions
//! into these generic models.

use chrono::Utc;
use radix_client::gateway::models::{EntityType, ModuleId};
use serde::{Deserialize, Serialize};

/// Generic struct for ledger events from a
/// transaction stream. To implement a new transaction
/// stream type, you would typically implement [`Into<Event>`]
/// for the native event type of the transaction stream.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub name: String,
    pub binary_sbor_data: Vec<u8>,
    pub emitter: EventEmitter,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventEmitter {
    Method {
        entity_address: String,
        entity_type: EntityType,
        is_global: bool,
        object_module_id: ModuleId,
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
            EventEmitter::Method { entity_address, .. } => entity_address,
            EventEmitter::Function {
                package_address, ..
            } => package_address,
        }
    }
}

/// Generic struct for ledger transactions from a
/// transaction stream. To implement a new transaction
/// stream type, you would typically implement [`Into<Transaction>`]
/// for the native transaction type of the transaction stream.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Transaction {
    pub intent_hash: String,
    pub state_version: u64,
    pub confirmed_at: Option<chrono::DateTime<Utc>>,
    pub events: Vec<Event>,
}
