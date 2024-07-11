//! State types

mod mining;
mod reward_pool;

use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
pub use mining::*;
use num_derive::{FromPrimitive, ToPrimitive};
pub use reward_pool::*;

/// Enum representing the account type managed by the program
#[derive(
    Clone,
    Debug,
    PartialEq,
    BorshDeserialize,
    BorshSerialize,
    BorshSchema,
    Default,
    ToPrimitive,
    FromPrimitive,
)]
pub enum AccountType {
    /// If the account has not been initialized, the enum will be 0
    #[default]
    Uninitialized,
    /// Reward pool
    RewardPool,
    /// Mining Account
    Mining,
}
