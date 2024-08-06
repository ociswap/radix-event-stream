use radix_engine::blueprints::consensus_manager;

#[derive(PartialEq, Eq, Debug, Hash)]
pub enum ValidatorEventType {
    RegisterValidatorEvent,
    UnregisterValidatorEvent,
    StakeEvent,
    UnstakeEvent,
    ClaimXrdEvent,
    UpdateAcceptingStakeDelegationStateEvent,
    ProtocolUpdateReadinessSignalEvent,
    ValidatorEmissionAppliedEvent,
    ValidatorRewardAppliedEvent,
}

pub use consensus_manager::ClaimXrdEvent;
pub use consensus_manager::ProtocolUpdateReadinessSignalEvent;
pub use consensus_manager::RegisterValidatorEvent;
pub use consensus_manager::StakeEvent;
pub use consensus_manager::UnregisterValidatorEvent;
pub use consensus_manager::UnstakeEvent;
pub use consensus_manager::UpdateAcceptingStakeDelegationStateEvent;
pub use consensus_manager::ValidatorEmissionAppliedEvent;
pub use consensus_manager::ValidatorRewardAppliedEvent;
