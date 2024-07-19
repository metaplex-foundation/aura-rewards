//! State types

mod mining;
mod reward_pool;

use std::{
    collections::BTreeMap,
    ops::{Deref, DerefMut},
};

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

#[derive(Debug, Default, Clone, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub struct BTreeMapWithCapacity<K: Ord, V> {
    capacity: usize,
    inner: BTreeMap<K, V>,
}

impl<K: Ord, V> BTreeMapWithCapacity<K, V> {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            inner: BTreeMap::new(),
        }
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn increase_capacity(&mut self, new_capacity: usize) {
        self.capacity += new_capacity;
    }
}

impl<K: Ord, V> Deref for BTreeMapWithCapacity<K, V> {
    type Target = BTreeMap<K, V>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<K: Ord, V> DerefMut for BTreeMapWithCapacity<K, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
