//! State types

mod mining;
mod reward_pool;

use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
pub use mining::*;
pub use reward_pool::*;
use sokoban::AVLTree;

pub const TREE_MAX_SIZE: usize = 100;
pub const INDEX_HISTORY_MAX_SIZE: usize = 120;
/// Precision for index calculation
pub const PRECISION: u128 = 10_000_000_000_000_000;

pub type CumulativeIndex = AVLTree<u64, u128, INDEX_HISTORY_MAX_SIZE>;
pub type WeightedStakeDiffs = AVLTree<u64, u64, TREE_MAX_SIZE>;

/// Enum representing the account type managed by the program
#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema, Default)]
pub enum AccountType {
    /// If the account has not been initialized, the enum will be 0
    #[default]
    Uninitialized,
    /// Reward pool
    RewardPool,
    /// Mining Account
    Mining,
}

impl From<u8> for AccountType {
    fn from(value: u8) -> Self {
        match value {
            0 => AccountType::Uninitialized,
            1 => AccountType::RewardPool,
            2 => AccountType::Mining,
            _ => panic!("invalid AccountType value: {value}"),
        }
    }
}

impl From<AccountType> for u8 {
    fn from(value: AccountType) -> Self {
        match value {
            AccountType::Uninitialized => 0,
            AccountType::RewardPool => 1,
            AccountType::Mining => 2,
        }
    }
}
