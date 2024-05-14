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
    /// Data type mismatch
    #[error("Data type mismatch")]
    DataTypeMismatch,

    /// 3
    /// Amount allowed of interest on the borrowing is exceeded
    #[error("Amount allowed of interest on the borrowing is exceeded")]
    AmountAllowedCheckFailed,

    /// 4
    /// Amount borrowed less then repay amount
    #[error("Amount allowed of interest on the borrowing is exceeded")]
    RepayAmountCheckFailed,

    /// 5
    /// Incorrect instruction program id
    #[error("Incorrect instruction program id")]
    IncorrectInstructionProgramId,

    /// Rebalancing

    /// 6
    /// Incomplete rebalancing
    #[error("Incomplete rebalancing")]
    IncompleteRebalancing,

    /// 7
    /// Rebalancing is completed
    #[error("Rebalancing is completed")]
    RebalancingIsCompleted,

    /// 8
    /// Money market does not match
    #[error("Rebalancing: Money market does not match")]
    InvalidRebalancingMoneyMarket,

    /// 9
    /// Operation does not match
    #[error("Rebalancing: Operation does not match")]
    InvalidRebalancingOperation,

    /// 10
    /// Amount does not match
    #[error("Rebalancing: Amount does not match")]
    InvalidRebalancingAmount,

    /// 11
    /// Liquidity distribution is stale
    #[error("Rebalancing: Liquidity distribution is stale")]
    LiquidityDistributionStale,

    /// 12
    /// Income has already been refreshed recently
    #[error("Rebalancing: Income has already been refreshed recently")]
    IncomeRefreshed,

    /// Withdraw requests

    /// 13
    /// Invalid ticket
    #[error("Withdraw requests: Invalid ticket")]
    WithdrawRequestsInvalidTicket,

    /// 14
    /// Temporary unavailable for migration
    #[error("Instruction temporary unavailable")]
    TemporaryUnavailable,

    /// 15
    /// Deposit amount below allowed minimum
    #[error("Deposit amount too small")]
    DepositAmountTooSmall,

    /// 16
    /// Withdraw request amount below allowed minimum
    #[error("Withdraw amount too small")]
    WithdrawAmountTooSmall,

    /// 17
    /// The reward supply amount is not equal to the collateral amount
    #[error("Reward supply amount and collateral amount mismatch")]
    RewardAndCollateralMismatch,

    /// 18
    /// Money market mining not implemented
    #[error("Mining not implemented")]
    MiningNotImplemented,

    /// 19
    /// Money market mining not initialized
    #[error("Mining not initialized")]
    MiningNotInitialized,

    /// 20
    /// Mining is required
    #[error("Mining is required")]
    MiningIsRequired,

    /// 21
    /// Reserve threshold exceeded
    #[error("Reserve threshold exceeded")]
    ReserveThreshold,

    /// 22
    /// Reserve rates not updated
    #[error("Reserve rates have not been updated within this slot")]
    ReserveRatesStale,

    /// 23
    /// Collateral leak
    #[error("Returned collateral amount is less than expected")]
    CollateralLeak,

    /// 24
    /// Amount cannot be zero
    #[error("Amount cannot be zero")]
    ZeroAmount,

    /// Rewards

    /// 25
    /// Invalid vault
    #[error("Rewards: Invalid vault")]
    RewardsInvalidVault,

    /// 26
    /// No deposits
    #[error("Rewards: No deposits")]
    RewardsNoDeposits,

    /// 27
    /// Check for liquidity amount in rebalance
    #[error("Rebalancing: liquidity check failed")]
    RebalanceLiquidityCheckFailed,

    /// 28
    /// Check for liquidity amount in rebalance
    #[error("Rewards: receiving index failed")]
    IndexMustExist,
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
