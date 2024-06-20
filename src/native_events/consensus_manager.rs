#[derive(PartialEq, Eq, Debug, Hash)]
pub enum ConsensusManagerEventType {
    EpochChangeEvent,
    RoundChangeEvent,
}

pub use radix_engine::blueprints::consensus_manager::EpochChangeEvent;
pub use radix_engine::blueprints::consensus_manager::RoundChangeEvent;
