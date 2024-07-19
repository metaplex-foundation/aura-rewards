use crate::{
    error::MplxRewardsError,
    state::{RewardCalculator, PRECISION},
    traits::{DataBlob, SafeArithmeticOperations, SolanaAccount},
};
use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::{
    clock::{Clock, SECONDS_PER_DAY},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    program_pack::IsInitialized,
    pubkey::Pubkey,
    sysvar::Sysvar,
};
use std::ops::Bound::{Excluded, Included};

use super::{AccountType, BTreeMapWithCapacity};

/// Mining
#[derive(Debug, BorshDeserialize, BorshSerialize, BorshSchema, Default)]
pub struct Mining {
    /// Account type - Mining. This discriminator should exist in order to prevent
    /// shenanigans with customly modified accounts and their fields.
    pub account_type: AccountType,
    /// The address of corresponding Reward pool.
    pub reward_pool: Pubkey,
    /// Saved bump for mining account
    pub bump: u8,
    /// Weighted stake on the processed day.
    pub share: u64,
    /// Mining owner. This user corresponds to the voter_authority
    /// on the staking contract, which means those idendities are the same.
    pub owner: Pubkey,
    /// That "index" points at the moment when the last reward has been recieved. Also,
    /// it' s responsible for weighted_stake changes and, therefore, rewards calculations.
    pub index: RewardIndex,
    /// This field sums up each time somebody stakes to that account as a delegate.
    pub stake_from_others: u64,
}

impl Mining {
    pub const DEFAULT_LEN: usize = 1 + 32 + 1 + 8 + 32 + RewardIndex::DEFAULT_LEN + 8;

    /// Initialize a Reward Pool
    pub fn initialize(reward_pool: Pubkey, bump: u8, owner: Pubkey) -> Mining {
        Mining {
            account_type: AccountType::Mining,
            reward_pool,
            bump,
            owner,
            index: RewardIndex {
                weighted_stake_diffs: BTreeMapWithCapacity::new(
                    RewardIndex::WEIGHTED_STAKE_DIFFS_DEFAULT_ELEMENTS_NUMBER,
                ),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// Claim reward
    pub fn claim(&mut self) {
        self.index.unclaimed_rewards = 0;
    }

    /// Refresh rewards
    pub fn refresh_rewards(&mut self, vault: &RewardCalculator) -> ProgramResult {
        let curr_ts = Clock::get().unwrap().unix_timestamp as u64;
        let beginning_of_the_day = curr_ts - (curr_ts % SECONDS_PER_DAY);
        let mut share = self.share.safe_add(self.stake_from_others)?;

        share = self
            .index
            .consume_old_modifiers(beginning_of_the_day, share, vault)?;
        RewardIndex::update_index(
            vault,
            curr_ts,
            share,
            &mut self.index.unclaimed_rewards,
            &mut self.index.index_with_precision,
        )?;
        self.share = share.safe_sub(self.stake_from_others)?;

        Ok(())
    }
}

impl SolanaAccount for Mining {
    fn account_type() -> AccountType {
        AccountType::Mining
    }
}

impl DataBlob for Mining {
    fn get_initial_size() -> usize {
        Mining::DEFAULT_LEN
    }

    fn get_size(&self) -> usize {
        let weighted_stake_diff_capacity = self
            .index
            .weighted_stake_diffs
            .capacity()
            .saturating_sub(RewardIndex::WEIGHTED_STAKE_DIFFS_DEFAULT_ELEMENTS_NUMBER);

        Mining::DEFAULT_LEN + weighted_stake_diff_capacity * (8 + 8)
    }
}

/// Reward index
#[derive(Debug, BorshSerialize, BorshDeserialize, BorshSchema, Default, Clone)]
pub struct RewardIndex {
    /// That is the mint of the Rewards Token
    pub reward_mint: Pubkey,
    /// That is the index that increases on each distribution.
    /// It points at the moment of time where the last reward was claimed.
    /// Also, responsible for rewards calculations for each staker.
    pub index_with_precision: u128,
    /// Amount of unclaimed rewards.
    /// After claim the value is set to zero.
    pub unclaimed_rewards: u64,
    /// This structures stores the weighted stake modifiers on the date,
    /// where staking ends. This modifier will be applied on the specified date to the global stake,
    /// so that rewards distribution will change. BTreeMap<unix_timestamp, modifier diff>
    pub weighted_stake_diffs: BTreeMapWithCapacity<u64, u64>,
}

impl RewardIndex {
    /// Reward Index size
    pub const DEFAULT_LEN: usize =
        32 + 16 + 8 + (4 + (8 + 8) * RewardIndex::WEIGHTED_STAKE_DIFFS_DEFAULT_ELEMENTS_NUMBER);
    pub const WEIGHTED_STAKE_DIFFS_DEFAULT_ELEMENTS_NUMBER: usize = 100;

    /// Consume old modifiers
    pub fn consume_old_modifiers(
        &mut self,
        beginning_of_the_day: u64,
        mut total_share: u64,
        pool_vault: &RewardCalculator,
    ) -> Result<u64, ProgramError> {
        for (date, modifier_diff) in self.weighted_stake_diffs.iter() {
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
        }
        // +1 because we don't need beginning_of_the_day
        *self.weighted_stake_diffs = self
            .weighted_stake_diffs
            .split_off(&(beginning_of_the_day + 1));

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
        self.account_type == AccountType::Mining
    }
}
