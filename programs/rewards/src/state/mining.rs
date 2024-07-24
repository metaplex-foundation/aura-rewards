use crate::{
    error::MplxRewardsError,
    state::{RewardCalculator, PRECISION},
};

use crate::utils::SafeArithmeticOperations;
use bytemuck::{Pod, Zeroable};
use shank::ShankAccount;
use sokoban::{AVLTree, NodeAllocatorMap, ZeroCopy};
use solana_program::{
    clock::{Clock, SECONDS_PER_DAY},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    program_pack::IsInitialized,
    pubkey::Pubkey,
    sysvar::Sysvar,
};
use std::ops::Bound::{Excluded, Included};

use super::{AccountType, TREE_MAX_SIZE};

pub struct WrappedMining<'a> {
    pub mining: &'a mut Mining,
    /// This structures stores the weighted stake modifiers on the date,
    /// where staking ends. This modifier will be applied on the specified date to the global stake,
    /// so that rewards distribution will change. BTreeMap<unix_timestamp, modifier diff>
    pub weighted_stake_diffs: &'a mut AVLTree<u64, u64, TREE_MAX_SIZE>,
}

impl<'a> WrappedMining<'a> {
    pub fn from_bytes_mut(bytes: &'a mut [u8]) -> Result<Self, ProgramError> {
        let (mining, weighted_stake_diffs) = bytes.split_at_mut(Mining::LEN);
        let mining = Mining::load_mut_bytes(mining)
            .ok_or(MplxRewardsError::RetreivingZeroCopyAccountFailire)?;

        let weighted_stake_diffs = AVLTree::load_mut_bytes(weighted_stake_diffs)
            .ok_or(MplxRewardsError::RetreivingZeroCopyAccountFailire)?;

        Ok(Self {
            mining,
            weighted_stake_diffs,
        })
    }

    pub fn data_len(&self) -> usize {
        Mining::LEN + std::mem::size_of::<AVLTree<u64, u64, TREE_MAX_SIZE>>()
    }

    /// Refresh rewards
    pub fn refresh_rewards(&mut self, vault: &RewardCalculator) -> ProgramResult {
        let curr_ts = Clock::get().unwrap().unix_timestamp as u64;
        let beginning_of_the_day = curr_ts - (curr_ts % SECONDS_PER_DAY);
        let mut share = self.mining.share.safe_add(self.mining.stake_from_others)?;

        share = self.mining.consume_old_modifiers(
            beginning_of_the_day,
            share,
            vault,
            &mut self.weighted_stake_diffs,
        )?;
        Mining::update_index(
            vault,
            curr_ts,
            share,
            &mut self.mining.unclaimed_rewards,
            &mut self.mining.index_with_precision,
        )?;
        self.mining.share = share.safe_sub(self.mining.stake_from_others)?;

        Ok(())
    }
}

/// Struct to represent an auction.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy, Pod, Zeroable, ShankAccount)]
pub struct Mining {
    /// The address of corresponding Reward pool.
    pub reward_pool: Pubkey,
    /// Mining owner. This user corresponds to the voter_authority
    /// on the staking contract, which means those idendities are the same.
    pub owner: Pubkey,
    /// That is the mint of the Rewards Token
    pub reward_mint: Pubkey,
    /// That is the index that increases on each distribution.
    /// It points at the moment of time where the last reward was claimed.
    /// Also, responsible for rewards calculations for each staker.
    pub index_with_precision: u128,
    /// Weighted stake on the processed day.
    pub share: u64,
    /// Amount of unclaimed rewards.
    /// After claim the value is set to zero.
    pub unclaimed_rewards: u64,
    /// This field sums up each time somebody stakes to that account as a delegate.
    pub stake_from_others: u64,
    /// Saved bump for mining account
    pub bump: u8,
    /// Account type - Mining. This discriminator should exist in order to prevent
    /// shenanigans with customly modified accounts and their fields.
    /// 1: account type
    /// 2-7: unused
    pub data: [u8; 7],
}

impl ZeroCopy for Mining {}

impl Mining {
    /// Bytes required to store an `Mining`.
    pub const LEN: usize = std::mem::size_of::<Mining>();

    /// Initialize a Reward Pool
    pub fn initialize(reward_pool: Pubkey, bump: u8, owner: Pubkey) -> Mining {
        let account_type = AccountType::Mining.into();
        let mut data = [0; 7];
        data[0] = account_type;
        Mining {
            data,
            reward_pool,
            bump,
            owner,
            ..Default::default()
        }
    }

    pub fn account_type(&self) -> AccountType {
        AccountType::from(self.data[0])
    }

    /// Claim reward
    pub fn claim(&mut self) {
        self.unclaimed_rewards = 0;
    }

    /// Consume old modifiers
    pub fn consume_old_modifiers(
        &mut self,
        beginning_of_the_day: u64,
        mut total_share: u64,
        pool_vault: &RewardCalculator,
        weighted_stake_diffs: &mut AVLTree<u64, u64, TREE_MAX_SIZE>,
    ) -> Result<u64, ProgramError> {
        let mut processed_dates = vec![];
        for (date, modifier_diff) in weighted_stake_diffs.iter() {
            if date > &beginning_of_the_day {
                break;
            }

            Self::update_index(
                pool_vault,
                *date,
                total_share,
                &mut self.unclaimed_rewards,
                &mut self.index_with_precision,
            )?;

            total_share = total_share.safe_sub(*modifier_diff)?;
            processed_dates.push(*date);
        }

        for date in processed_dates {
            weighted_stake_diffs.remove(&date);
        }

        Ok(total_share)
    }

    /// Updates index and distributes rewards
    pub fn update_index(
        pool_vault: &RewardCalculator,
        date: u64,
        total_share: u64,
        unclaimed_rewards: &mut u64,
        index_with_precision: &mut u128,
    ) -> ProgramResult {
        let vault_index_for_date = pool_vault
            .cumulative_index
            .range((Included(0), Excluded(date)))
            .last()
            .unwrap_or((&0, &0))
            .1;

        let rewards = u64::try_from(
            vault_index_for_date
                .safe_sub(*index_with_precision)?
                .safe_mul(u128::from(total_share))?
                .safe_div(PRECISION)?,
        )
        .map_err(|_| MplxRewardsError::InvalidPrimitiveTypesConversion)?;

        if rewards > 0 {
            *unclaimed_rewards = (*unclaimed_rewards).safe_add(rewards)?;
        }

        *index_with_precision = *vault_index_for_date;

        Ok(())
    }
}

impl IsInitialized for Mining {
    fn is_initialized(&self) -> bool {
        self.data[0] == <u8>::from(AccountType::Mining)
    }
}
