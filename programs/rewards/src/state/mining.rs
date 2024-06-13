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
use std::collections::BTreeMap;
use std::ops::Bound::{Excluded, Included};

/// Mining
#[derive(Debug, BorshDeserialize, BorshSerialize, BorshSchema, Default)]
pub struct Mining {
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
    /// Initialize a Reward Pool
    pub fn initialize(reward_pool: Pubkey, bump: u8, owner: Pubkey) -> Mining {
        Mining {
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
    pub fn refresh_rewards(&mut self, vault: &RewardVault) -> ProgramResult {
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
}

impl Sealed for Mining {}
impl Pack for Mining {
    const LEN: usize = 8 + (32 + 1 + 8 + 32 + (4 + RewardIndex::LEN * MAX_REWARDS));

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let mut slice = dst;
        self.serialize(&mut slice).unwrap();
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
    /// TODO: data isn't large enough
    pub const LEN: usize = 32 + 16 + 8 + (4 + (8 + 8) * 100);

    /// Consume old modifiers
    pub fn consume_old_modifiers(
        &mut self,
        beginning_of_the_day: u64,
        mut total_share: u64,
        pool_vault: &RewardVault,
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
        // TODO: split_off should be used instead of retain
        self.weighted_stake_diffs
            .retain(|date, _| date > &beginning_of_the_day);

        Ok(total_share)
    }

    /// Updates index and distributes rewards
    pub fn update_index(
        pool_vault: &RewardVault,
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
