use crate::{
    error::MplxRewardsError,
    state::{RewardVault, MAX_REWARDS, PRECISION},
};
use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::{
    clock::{Clock, SECONDS_PER_DAY},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
    sysvar::Sysvar,
};
use std::{collections::BTreeMap, slice::Iter};

/// Mining
#[derive(Debug, BorshDeserialize, BorshSerialize, BorshSchema, Default)]
pub struct Mining {
    /// Reward pool address
    pub reward_pool: Pubkey,
    /// Saved bump for mining account
    pub bump: u8,
    /// Share
    pub share: u64,
    /// Mining owner
    pub owner: Pubkey,
    /// Reward indexes
    pub indexes: Vec<RewardIndex>,
}

impl Mining {
    /// Initialize a Reward Pool
    pub fn initialize(reward_pool: Pubkey, bump: u8, owner: Pubkey) -> Mining {
        Mining {
            reward_pool,
            bump,
            share: 0,
            owner,
            indexes: vec![],
        }
    }

    /// Returns reward index
    pub fn reward_index_mut(&mut self, reward_mint: Pubkey) -> &mut RewardIndex {
        match self
            .indexes
            .iter()
            .position(|mi| mi.reward_mint == reward_mint)
        {
            Some(i) => &mut self.indexes[i],
            None => {
                self.indexes.push(RewardIndex {
                    reward_mint,
                    ..Default::default()
                });
                self.indexes.last_mut().unwrap()
            }
        }
    }

    /// Claim reward
    pub fn claim(&mut self, reward_mint: Pubkey) {
        let reward_index = self.reward_index_mut(reward_mint);
        reward_index.rewards = 0;
    }

    /// Refresh rewards
    pub fn refresh_rewards(&mut self, pool_vaults: Iter<RewardVault>) -> ProgramResult {
        let curr_ts = Clock::get().unwrap().unix_timestamp as u64;
        let beginning_of_the_day = curr_ts - (curr_ts % SECONDS_PER_DAY);
        let mut share = self.share;

        for pool_vault in pool_vaults {
            let reward_index = self.reward_index_mut(pool_vault.reward_mint);

            // extract function into a dedicated function
            for (date, modifier_diff) in reward_index.weighted_stake_diffs.iter() {
                if date > &beginning_of_the_day {
                    break;
                }

                share = share
                    .checked_sub(*modifier_diff)
                    .ok_or(MplxRewardsError::MathOverflow)?;

                if let Some(vault_index_for_date) = pool_vault.cumulative_index.get(date) {
                    let rewards = vault_index_for_date
                        .checked_sub(reward_index.index_with_precision)
                        .ok_or(MplxRewardsError::MathOverflow)?
                        .checked_mul(share as u128)
                        .ok_or(MplxRewardsError::MathOverflow)?
                        .checked_div(PRECISION)
                        .ok_or(MplxRewardsError::MathOverflow)?;

                    if rewards > 0 {
                        reward_index.rewards = reward_index
                            .rewards
                            .checked_add(rewards as u64)
                            .ok_or(MplxRewardsError::MathOverflow)?;
                    }

                    reward_index.index_with_precision = *vault_index_for_date;
                }
            }
            reward_index
                .weighted_stake_diffs
                .retain(|date, _modifier| date > &beginning_of_the_day);

            // TODO: remove code duplication
            if let Some(vault_index_for_date) =
                pool_vault.cumulative_index.get(&beginning_of_the_day)
            {
                let rewards = vault_index_for_date
                    .checked_sub(reward_index.index_with_precision)
                    .ok_or(MplxRewardsError::MathOverflow)?
                    .checked_mul(share as u128)
                    .ok_or(MplxRewardsError::MathOverflow)?
                    .checked_div(PRECISION)
                    .ok_or(MplxRewardsError::MathOverflow)?;

                if rewards > 0 {
                    reward_index.rewards = reward_index
                        .rewards
                        .checked_add(rewards as u64)
                        .ok_or(MplxRewardsError::MathOverflow)?;
                }

                reward_index.index_with_precision = *vault_index_for_date;
            }

            self.share = share;
        }

        Ok(())
    }
}

impl Sealed for Mining {}
impl Pack for Mining {
    const LEN: usize = 8 + (32 + 1 + 8 + 32 + (4 + RewardIndex::LEN * MAX_REWARDS));

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let mut slice = dst;
        self.serialize(&mut slice).unwrap()
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let mut src_mut = src;
        Self::deserialize(&mut src_mut).map_err(|err| {
            msg!("Failed to deserialize");
            msg!("{}", err.to_string());
            ProgramError::InvalidAccountData
        })
    }
}

impl IsInitialized for Mining {
    fn is_initialized(&self) -> bool {
        self.owner != Pubkey::default()
    }
}

/// Reward index
#[derive(Debug, BorshSerialize, BorshDeserialize, BorshSchema, Default, Clone)]
pub struct RewardIndex {
    /// Reward mint
    pub reward_mint: Pubkey,
    /// Index with precision
    pub index_with_precision: u128,
    /// Rewards amount
    pub rewards: u64,
    /// Shows the changes of the weighted stake.<Date, index>
    pub weighted_stake_diffs: BTreeMap<u64, u64>,
}

impl RewardIndex {
    // TODO: change size of RewardIndex
    /// 32 + 16 + 8
    pub const LEN: usize = 56;

    /// Updates index and distributes rewards
    pub fn update_index(
        &mut self,
        pool_vault: &RewardVault,
        date: &u64,
        share: u128,
    ) -> ProgramResult {
        let vault_index_for_date = *pool_vault
            .cumulative_index
            .get(date)
            .ok_or(MplxRewardsError::IndexMustExist)?;

        let rewards = vault_index_for_date
            .checked_sub(self.index_with_precision)
            .ok_or(MplxRewardsError::MathOverflow)?
            .checked_mul(share)
            .ok_or(MplxRewardsError::MathOverflow)?
            .checked_div(PRECISION)
            .ok_or(MplxRewardsError::MathOverflow)?;

        if rewards > 0 {
            self.rewards = self
                .rewards
                .checked_add(rewards as u64)
                .ok_or(MplxRewardsError::MathOverflow)?;
        }

        Ok(())
    }
}
