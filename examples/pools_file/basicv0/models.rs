use radix_client::gateway::models::Event;
use std::fmt::Debug;

pub type ComponentAddress = String;
pub type PoolAddress = String;

#[derive(Debug, Clone, Copy)]
pub enum PoolType {
    BasicV0,
    BasicV1,
    PrecisionV1,
}

#[derive(Debug, PartialEq)]
pub enum EventType {
    Instantiate,
    AddLiquidity,
    RemoveLiquidity,
    Swap,
    ClaimFee,
    FLashLoan,
}

#[derive(Debug)]
pub struct NamedEvent {
    pub event_type: EventType,
    pub event: Event,
}

#[derive(Debug)]
pub struct IdentifiedEvent {
    pub pool_type: PoolType,
    pub event_type: EventType,
    pub event: Event,
}
