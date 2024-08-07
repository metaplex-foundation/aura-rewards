//! State types

mod mining;
mod reward_pool;

use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use bytemuck::Pod;
pub use mining::*;
pub use reward_pool::*;
use sokoban::{RedBlackTree, SENTINEL};
use std::fmt::Debug;

pub const MINING_MODIFIERS_TREE_CAPACITY: usize = 50;
pub const POOL_MODIFIERS_TREE_CAPACITY: usize = 365;
pub const INDEX_HISTORY_MAX_SIZE: usize = 1095;
/// Precision for index calculation
pub const PRECISION: u128 = 10_000_000_000_000_000;

pub type CumulativeIndex = RedBlackTree<u64, u128, INDEX_HISTORY_MAX_SIZE>;
pub type PoolWeightedStakeDiffs = RedBlackTree<u64, u64, POOL_MODIFIERS_TREE_CAPACITY>;
pub type MiningWeightedStakeDiffs = RedBlackTree<u64, u64, MINING_MODIFIERS_TREE_CAPACITY>;

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

fn find_max_value_limited_by_key<
    K: Ord + Default + Pod + Debug,
    V: Default + Pod,
    const CAP: usize,
>(
    tree: &RedBlackTree<K, V, CAP>,
    key: K,
) -> Option<V> {
    let mut current_id = tree.root; // Start at the root node
    let mut result = None;

    while current_id != SENTINEL {
        let node = tree.get_node(current_id); // Get the current node
        if node.key < key {
            // Update result to the current key if it's a valid candidate
            result = Some(node.value);
            // Move to the right subtree to potentially find a larger valid key
            current_id = tree.get_right(current_id);
        } else {
            // Move to the left subtree to find a smaller key
            current_id = tree.get_left(current_id);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use sokoban::NodeAllocatorMap;

    use super::*;

    #[test]
    fn test_find_max_value_limited_by_key() {
        let mut tree = RedBlackTree::<u64, u64, 300>::new();
        tree.insert(1, 10);
        tree.insert(2, 20);
        tree.insert(3, 30);
        tree.insert(4, 40);
        tree.insert(5, 50);

        assert_eq!(find_max_value_limited_by_key(&tree, 3).unwrap(), 20);
        assert_eq!(find_max_value_limited_by_key(&tree, 6).unwrap(), 50);
        assert_eq!(find_max_value_limited_by_key(&tree, 0), None);
    }
}
