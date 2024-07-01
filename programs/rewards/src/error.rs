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
    #[error("Rewards: distribution_ends_at date is lower than current date ")]
    DistributionInThePast,

    /// 5
    /// Invalid math conversion between types
    #[error("Rewards: distribution_ends_at date is lower than current date ")]
    InvalidPrimitiveTypesConversion,
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
