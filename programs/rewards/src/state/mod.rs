//! State types

mod deprecated_reward_pool;
mod mining;
mod reward_pool;
mod rewards_root;

use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
pub use deprecated_reward_pool::*;
pub use mining::*;
pub use reward_pool::*;
pub use rewards_root::*;

/// Enum representing the account type managed by the program
#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema, Default)]
pub enum AccountType {
    /// If the account has not been initialized, the enum will be 0
    #[default]
    Uninitialized,
    /// Rewards root
    RewardsRoot,
    /// Reward pool
    RewardPool,
}
