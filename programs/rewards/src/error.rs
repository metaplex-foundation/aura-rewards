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
    /// Zero amount
    #[error("Zero amount")]
    ZeroAmount,

    /// 3
    /// Invalid vault
    #[error("Rewards: Invalid vault")]
    RewardsInvalidVault,

    /// 4
    /// No deposits
    #[error("Rewards: No deposits")]
    RewardsNoDeposits,

    /// 5
    /// Check for liquidity amount in rebalance
    #[error("Rebalancing: liquidity check failed")]
    RebalanceLiquidityCheckFailed,

    /// 6
    /// Non existing reward's index
    #[error("Rewards: index receiving failed")]
    IndexMustExist,

    /// 7
    /// Invalid lockup period
    #[error("Rewards: lockup period invalid")]
    InvalidLockupPeriod,

    /// 8
    /// Invalid CPI caller
    #[error("Rewards: only Staking contract is allowed to do CPI calls")]
    InvalidCpiCaller,

    /// 9
    /// Invalid distribution_ends_at data
    #[error("Rewards: distribution_ends_at date is lower than current date")]
    DistributionInThePast,

    /// 10
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
    /// Serialization failed
    #[error("Rewards: can't serialize an account")]
    SerializationError,

    /// 18
    /// Deserialization failed
    #[error("Rewards: can't deserialize an account")]
    DeserializationError,
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
