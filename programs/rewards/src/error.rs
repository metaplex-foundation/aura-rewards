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

    /// 8 Invalid CPI caller
    #[error("Rewards: only Staking contract is allowed to do CPI calls")]
    InvalidCpiCaller,
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
