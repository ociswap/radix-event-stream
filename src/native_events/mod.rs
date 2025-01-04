use account::AccountEventType;
use account_locker::AccountLockerEventType;
use consensus_manager::ConsensusManagerEventType;
use fungible_vault::FungibleVaultEventType;
use metadata::MetadataEventType;
use non_fungible_vault::NonFungibleVaultEventType;
use pool::{
    multi_resource_pool::MultiResourcePoolEventType,
    one_resource_pool::OneResourcePoolEventType,
    two_resource_pool::TwoResourcePoolEventType,
};
use radix_client::gateway::models::EntityType;
use resource_manager::ResourceManagerEventType;
use role_assignment::RoleAssignmentEventType;
use validator::ValidatorEventType;

pub mod account;
pub mod account_locker;
pub mod consensus_manager;
pub mod fungible_vault;
pub mod metadata;
pub mod non_fungible_vault;
pub mod pool;
pub mod resource_manager;
pub mod role_assignment;
pub mod validator;

#[derive(PartialEq, Eq, Debug, Hash)]
pub enum NativeEventType {
    ResourceManager(ResourceManagerEventType),
    Metadata(MetadataEventType),
    FungibleVault(FungibleVaultEventType),
    NonFungibleVault(NonFungibleVaultEventType),
    OneResourcePool(OneResourcePoolEventType),
    TwoResoucePool(TwoResourcePoolEventType),
    MultiResourcePool(MultiResourcePoolEventType),
    AccountLocker(AccountLockerEventType),
    Validator(ValidatorEventType),
    ConsensusManager(ConsensusManagerEventType),
    RoleAssignment(RoleAssignmentEventType),
    Account(AccountEventType),
}

impl NativeEventType {
    pub fn resolve(
        event_name: &str,
        entity_type: EntityType,
    ) -> Result<Self, ()> {
        match event_name {
            "VaultCreationEvent" => Ok(NativeEventType::ResourceManager(
                ResourceManagerEventType::VaultCreationEvent,
            )),
            "MintFungibleResourceEvent" => {
                Ok(NativeEventType::ResourceManager(
                    ResourceManagerEventType::MintFungibleResourceEvent,
                ))
            }
            "BurnFungibleResourceEvent" => {
                Ok(NativeEventType::ResourceManager(
                    ResourceManagerEventType::BurnFungibleResourceEvent,
                ))
            }
            "MintNonFungibleResourceEvent" => {
                Ok(NativeEventType::ResourceManager(
                    ResourceManagerEventType::MintNonFungibleResourceEvent,
                ))
            }
            "BurnNonFungibleResourceEvent" => {
                Ok(NativeEventType::ResourceManager(
                    ResourceManagerEventType::BurnNonFungibleResourceEvent,
                ))
            }
            "SetMetadataEvent" => Ok(NativeEventType::Metadata(
                MetadataEventType::SetMetadataEvent,
            )),
            "RemoveMetadataEvent" => Ok(NativeEventType::Metadata(
                MetadataEventType::RemoveMetadataEvent,
            )),
            "WithdrawEvent" => match entity_type {
                EntityType::InternalFungibleVault => Ok(NativeEventType::FungibleVault(
                    fungible_vault::FungibleVaultEventType::WithdrawEvent,
                )),
                EntityType::InternalNonFungibleVault => Ok(NativeEventType::NonFungibleVault(
                    non_fungible_vault::NonFungibleVaultEventType::WithdrawEvent,
                )),
                EntityType::GlobalOneResourcePool => Ok(NativeEventType::OneResourcePool(
                    OneResourcePoolEventType::WithdrawEvent,
                )),
                EntityType::GlobalTwoResourcePool => Ok(NativeEventType::TwoResoucePool(
                    TwoResourcePoolEventType::WithdrawEvent,
                )),
                EntityType::GlobalMultiResourcePool => Ok(NativeEventType::MultiResourcePool(
                    MultiResourcePoolEventType::WithdrawEvent,
                )),
                // I'm pretty sure it can be either of these account types
                EntityType::GlobalAccount
                | EntityType::GlobalVirtualEd25519Account
                | EntityType::GlobalVirtualSecp256k1Account
                => Ok(NativeEventType::Account(
                    AccountEventType::WithdrawEvent,
                )),
                _ => Err(()),
            },
            "DepositEvent" => match entity_type {
                EntityType::InternalFungibleVault => Ok(NativeEventType::FungibleVault(
                    fungible_vault::FungibleVaultEventType::DepositEvent,
                )),
                EntityType::InternalNonFungibleVault => Ok(NativeEventType::NonFungibleVault(
                    non_fungible_vault::NonFungibleVaultEventType::DepositEvent,
                )),
                EntityType::GlobalOneResourcePool => Ok(NativeEventType::OneResourcePool(
                    OneResourcePoolEventType::DepositEvent,
                )),
                EntityType::GlobalTwoResourcePool => Ok(NativeEventType::TwoResoucePool(
                    TwoResourcePoolEventType::DepositEvent,
                )),
                EntityType::GlobalMultiResourcePool => Ok(NativeEventType::MultiResourcePool(
                    MultiResourcePoolEventType::DepositEvent,
                )),
                // I'm pretty sure it can be either of these account types
                EntityType::GlobalAccount
                | EntityType::GlobalVirtualEd25519Account
                | EntityType::GlobalVirtualSecp256k1Account
                => Ok(NativeEventType::Account(
                    AccountEventType::DepositEvent,
                )),
                _ => Err(()),
            },
            "RecallEvent" => match entity_type {
                EntityType::InternalFungibleVault => Ok(NativeEventType::FungibleVault(
                    fungible_vault::FungibleVaultEventType::RecallEvent,
                )),
                EntityType::InternalNonFungibleVault => Ok(NativeEventType::NonFungibleVault(
                    non_fungible_vault::NonFungibleVaultEventType::RecallEvent,
                )),
                _ => Err(()),
            },
            "LockFeeEvent" => Ok(NativeEventType::FungibleVault(
                fungible_vault::FungibleVaultEventType::LockFeeEvent,
            )),
            "PayFeeEvent" => Ok(NativeEventType::FungibleVault(
                fungible_vault::FungibleVaultEventType::PayFeeEvent,
            )),
            "RedemptionEvent" => match entity_type {
                EntityType::GlobalOneResourcePool => Ok(NativeEventType::OneResourcePool(
                    OneResourcePoolEventType::RedemptionEvent,
                )),
                EntityType::GlobalTwoResourcePool => Ok(NativeEventType::TwoResoucePool(
                    TwoResourcePoolEventType::RedemptionEvent,
                )),
                EntityType::GlobalMultiResourcePool => Ok(NativeEventType::MultiResourcePool(
                    MultiResourcePoolEventType::RedemptionEvent,
                )),
                _ => Err(()),
            },
            "ContributionEvent" => match entity_type {
                EntityType::GlobalOneResourcePool => Ok(NativeEventType::OneResourcePool(
                    OneResourcePoolEventType::ContributionEvent,
                )),
                EntityType::GlobalTwoResourcePool => Ok(NativeEventType::TwoResoucePool(
                    TwoResourcePoolEventType::ContributionEvent,
                )),
                EntityType::GlobalMultiResourcePool => Ok(NativeEventType::MultiResourcePool(
                    MultiResourcePoolEventType::ContributionEvent,
                )),
                _ => Err(()),
            },
            "StoreEvent" => Ok(NativeEventType::AccountLocker(
                AccountLockerEventType::StoreEvent,
            )),
            "RecoverEvent" => Ok(NativeEventType::AccountLocker(
                AccountLockerEventType::RecoverEvent,
            )),
            "ClaimEvent" => Ok(NativeEventType::AccountLocker(
                AccountLockerEventType::ClaimEvent,
            )),
            "RegisterValidatorEvent" => Ok(NativeEventType::Validator(
                ValidatorEventType::RegisterValidatorEvent,
            )),
            "UnregisterValidatorEvent" => Ok(NativeEventType::Validator(
                ValidatorEventType::UnregisterValidatorEvent,
            )),
            "StakeEvent" => Ok(NativeEventType::Validator(
                ValidatorEventType::StakeEvent,
            )),
            "UnstakeEvent" => Ok(NativeEventType::Validator(
                ValidatorEventType::UnstakeEvent,
            )),
            "ClaimXrdEvent" => Ok(NativeEventType::Validator(
                ValidatorEventType::ClaimXrdEvent,
            )),
            "UpdateAcceptingStakeDelegationStateEvent" => Ok(NativeEventType::Validator(
                ValidatorEventType::UpdateAcceptingStakeDelegationStateEvent,
            )),
            "ProtocolUpdateReadinessSignalEvent" => Ok(NativeEventType::Validator(
                ValidatorEventType::ProtocolUpdateReadinessSignalEvent,
            )),
            "ValidatorEmissionAppliedEvent" => Ok(NativeEventType::Validator(
                ValidatorEventType::ValidatorEmissionAppliedEvent,
            )),
            "ValidatorRewardAppliedEvent" => Ok(NativeEventType::Validator(
                ValidatorEventType::ValidatorRewardAppliedEvent,
            )),
            "RoundChangeEvent" => Ok(NativeEventType::ConsensusManager(
                ConsensusManagerEventType::RoundChangeEvent,
            )),
            "EpochChangeEvent" => Ok(NativeEventType::ConsensusManager(
                ConsensusManagerEventType::EpochChangeEvent,
            )),
            "SetRoleEvent" => Ok(NativeEventType::RoleAssignment(
                RoleAssignmentEventType::SetRoleEvent,
            )),
            "SetOwnerRoleEvent" => Ok(NativeEventType::RoleAssignment(
                RoleAssignmentEventType::SetOwnerRoleEvent,
            )),
            "LockOwnerRoleEvent" => Ok(NativeEventType::RoleAssignment(
                RoleAssignmentEventType::LockOwnerRoleEvent,
            )),
            "AddAuthorizedDepositorEvent" => Ok(NativeEventType::Account(AccountEventType::AddAuthorizedDepositorEvent)),
            "RejectedDepositEvent" => Ok(NativeEventType::Account(AccountEventType::RejectedDepositEvent)),
            "RemoveAuthorizedDepositorEvent" => Ok(NativeEventType::Account(AccountEventType::RemoveAuthorizedDepositorEvent)),
            "RemoveResourcePreferenceEvent" => Ok(NativeEventType::Account(AccountEventType::RemoveResourcePreferenceEvent)),
            "SetDefaultDepositRuleEvent" => Ok(NativeEventType::Account(AccountEventType::SetDefaultDepositRuleEvent)),
            "SetResourcePreferenceEvent" => Ok(NativeEventType::Account(AccountEventType::SetResourcePreferenceEvent)),
            _ => Err(()),
        }
    }
}
