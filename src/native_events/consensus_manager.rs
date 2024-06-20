#[derive(PartialEq, Eq, Debug, Hash)]
pub enum ConsensusManagerEventType {
    RoundChangeEvent,
    EpochChangeEvent,
}

pub use radix_engine::blueprints::consensus_manager::EpochChangeEvent;
pub use radix_engine::blueprints::consensus_manager::RoundChangeEvent;
