//! Error types

use num_derive::FromPrimitive;
use solana_program::{
    decode_error::DecodeError,
    msg,
    program_error::{PrintProgramError, ProgramError},
};
use thiserror::Error;

/// Errors that may be returned by the program.
#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum MplxRewardsError {
    /// 0
    /// Input account owner
    #[error("Input account owner")]
    InvalidAccountOwner,

    /// 1
    /// Math operation overflow
    #[error("Math operation overflow")]
    MathOverflow,

    /// 2
    /// No deposits
    #[error("Rewards: No deposits")]
    RewardsNoDeposits,

    /// 3
    /// Invalid lockup period
    #[error("Rewards: lockup period invalid")]
    InvalidLockupPeriod,

    /// 4
    /// Invalid distribution_ends_at data
    #[error("Rewards: distribution_ends_at date is lower than current date")]
    DistributionInThePast,

    /// 5
    /// Invalid math conversion between types
    #[error("Rewards: distribution_ends_at date is lower than current date")]
    InvalidPrimitiveTypesConversion,

    /// 11
    /// Impossible to close accounts while it has unclaimed rewards
    #[error("Rewards: unclaimed rewards must be claimed")]
    RewardsMustBeClaimed,

    /// 12
    /// No need to transfer zero amount of rewards.
    #[error("Rewards: rewards amount must be positive")]
    RewardsMustBeGreaterThanZero,

    /// 13
    /// Delegate lack of tokens
    #[error("Rewards: Delegate must have at least 15_000_000 of own weighted stake")]
    InsufficientWeightedStake,

    /// 14
    /// Stake from others must be zero
    #[error("Rewards: Stake from others must be zero")]
    StakeFromOthersMustBeZero,

    /// 15
    /// No need to transfer zero amount of rewards.
    #[error("No changes at the date in weighted stake modifiers while they're expected")]
    NoWeightedStakeModifiersAtADate,

    /// 16
    /// To change a delegate, the new delegate must differ from the current one
    #[error("Passed delegates are the same")]
    DelegatesAreTheSame,

    /// 17
    /// Getting pointer to the data of the zero-copy account has failed
    #[error("Getting pointer to the data of the zero-copy account has failed")]
    RetreivingZeroCopyAccountFailire,

    /// 18
    /// Account is already initialized
    #[error("Account is already initialized")]
    AlreadyInitialized,

    /// 19
    #[error("Account addres derivation has failed")]
    AccountDerivationAddresFailed,

    /// 20
    #[error("This contract is supposed to be called only from the staking contract")]
    ForbiddenInvocation,
}

impl PrintProgramError for MplxRewardsError {
    fn print<E>(&self) {
        msg!("Error: {}", &self.to_string());
    }
}

impl From<MplxRewardsError> for ProgramError {
    fn from(e: MplxRewardsError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for MplxRewardsError {
    fn type_of() -> &'static str {
        "MplxRewardsError"
    }
}
