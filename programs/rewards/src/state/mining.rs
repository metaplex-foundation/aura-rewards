use crate::{
    error::MplxRewardsError,
    state::{RewardCalculator, PRECISION},
    traits::{DataBlob, SolanaAccount},
    utils::{resize_or_reallocate_account, MAX_REALLOC_SIZE},
};
use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::account_info::AccountInfo;
use solana_program::{
    clock::{Clock, SECONDS_PER_DAY},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    program_pack::IsInitialized,
    pubkey::Pubkey,
    sysvar::Sysvar,
};
use std::{
    collections::BTreeMap,
    ops::Bound::{Excluded, Included},
};

use super::AccountType;

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
    /// Delegate address where tokens will be staked in "delegated staking model".
    pub delegate: Pubkey,
}

impl Mining {
    pub const DEFAULT_LEN: usize = 1 + 32 + 1 + 6 + 32 + RewardIndex::DEFAULT_LEN + 32;

    /// Initialize a Reward Pool
    pub fn initialize(reward_pool: Pubkey, bump: u8, owner: Pubkey) -> Mining {
        Mining {
            account_type: AccountType::Mining,
            reward_pool,
            bump,
            share: 0,
            owner,
            index: RewardIndex::default(),
            delegate: owner,
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
        let mut share = self.share;

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
        self.share = share;

        Ok(())
    }

    pub fn resize_if_needed<'a>(
        &self,
        mining_account: &AccountInfo<'a>,
        payer: &AccountInfo<'a>,
        system_program: &AccountInfo<'a>,
    ) -> ProgramResult {
        if self.index.weighted_stake_diffs.len()
            % RewardIndex::WEIGHTED_STAKE_DIFFS_DEFAULT_ELEMENTS_NUMBER
            == 0
            && !self.index.weighted_stake_diffs.is_empty()
        {
            let new_size = self.get_size() + MAX_REALLOC_SIZE;
            resize_or_reallocate_account(mining_account, payer, system_program, new_size)?;
        }
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
        let weighted_stake_diff_elements = self
            .index
            .weighted_stake_diffs
            .len()
            .saturating_sub(RewardIndex::WEIGHTED_STAKE_DIFFS_DEFAULT_ELEMENTS_NUMBER);

        Mining::DEFAULT_LEN + self.index.weighted_stake_diffs.len()
            - weighted_stake_diff_elements * (8 + 8)
            + 4
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
    pub weighted_stake_diffs: BTreeMap<u64, u64>,
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
        for (date, modifier_diff) in &self.weighted_stake_diffs {
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

            total_share = total_share
                .checked_sub(*modifier_diff)
                .ok_or(MplxRewardsError::MathOverflow)?;
        }
        // +1 because we don't need beginning_of_the_day
        self.weighted_stake_diffs = self
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
                .checked_sub(*index_with_precision)
                .ok_or(MplxRewardsError::MathOverflow)?
                .checked_mul(u128::from(total_share))
                .ok_or(MplxRewardsError::MathOverflow)?
                .checked_div(PRECISION)
                .ok_or(MplxRewardsError::MathOverflow)?,
        )
        .map_err(|_| MplxRewardsError::InvalidPrimitiveTypesConversion)?;

        if rewards > 0 {
            *unclaimed_rewards = unclaimed_rewards
                .checked_add(rewards)
                .ok_or(MplxRewardsError::MathOverflow)?;
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
