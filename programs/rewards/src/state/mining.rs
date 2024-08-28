use crate::{error::MplxRewardsError, state::PRECISION};

use crate::utils::SafeArithmeticOperations;
use bytemuck::{Pod, Zeroable};
use shank::ShankAccount;
use sokoban::{NodeAllocatorMap, ZeroCopy};
use solana_program::{
    clock::{Clock, SECONDS_PER_DAY},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    program_pack::IsInitialized,
    pubkey::Pubkey,
    sysvar::Sysvar,
};

use super::{
    find_max_value_limited_by_key, AccountType, CumulativeIndex, MiningWeightedStakeDiffs,
};

pub struct WrappedMining<'a> {
    pub mining: &'a mut Mining,
    /// This structures stores the weighted stake modifiers on the date,
    /// where staking ends. This modifier will be applied on the specified date to the global stake,
    /// so that rewards distribution will change. BTreeMap<unix_timestamp, modifier diff>
    pub weighted_stake_diffs: &'a mut MiningWeightedStakeDiffs,
}
pub struct WrappedImmutableMining<'a> {
    pub mining: &'a Mining,
    /// This structures stores the weighted stake modifiers on the date,
    /// where staking ends. This modifier will be applied on the specified date to the global stake,
    /// so that rewards distribution will change. BTreeMap<unix_timestamp, modifier diff>
    pub weighted_stake_diffs: &'a MiningWeightedStakeDiffs,
}

pub const CLAIMING_RESTRICTION_BIT: usize = 1;

impl<'a> WrappedMining<'a> {
    pub const LEN: usize = 1776;

    pub fn from_bytes_mut(bytes: &'a mut [u8]) -> Result<Self, ProgramError> {
        let (mining, weighted_stake_diffs) = bytes.split_at_mut(Mining::LEN);
        let mining = Mining::load_mut_bytes(mining)
            .ok_or(MplxRewardsError::RetreivingZeroCopyAccountFailire)?;

        let weighted_stake_diffs = MiningWeightedStakeDiffs::load_mut_bytes(weighted_stake_diffs)
            .ok_or(MplxRewardsError::RetreivingZeroCopyAccountFailire)?;

        Ok(Self {
            mining,
            weighted_stake_diffs,
        })
    }

    /// Refresh rewards
    pub fn refresh_rewards(&mut self, cumulative_index: &CumulativeIndex) -> ProgramResult {
        let curr_ts = Clock::get().unwrap().unix_timestamp as u64;
        let beginning_of_the_day = curr_ts - (curr_ts % SECONDS_PER_DAY);
        let mut share = self.mining.share.safe_add(self.mining.stake_from_others)?;

        share = self.mining.consume_old_modifiers(
            beginning_of_the_day,
            share,
            cumulative_index,
            self.weighted_stake_diffs,
        )?;
        Mining::update_index(
            cumulative_index,
            curr_ts,
            share,
            &mut self.mining.unclaimed_rewards,
            &mut self.mining.index_with_precision,
        )?;
        self.mining.share = share.safe_sub(self.mining.stake_from_others)?;

        Ok(())
    }
}

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
    /// 2: claim is restricted
    /// 2-7: unused
    pub data: [u8; 7],
}

impl ZeroCopy for Mining {}

impl Mining {
    /// Bytes required to store the `Mining`.
    pub const LEN: usize = std::mem::size_of::<Mining>();

    /// Initialize a Reward Pool
    pub fn initialize(reward_pool: Pubkey, owner: Pubkey, bump: u8) -> Mining {
        let account_type = AccountType::Mining.into();
        let mut data = [0; 7];
        data[0] = account_type;
        Mining {
            data,
            reward_pool,
            owner,
            bump,
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
        cumulative_index: &CumulativeIndex,
        weighted_stake_diffs: &mut MiningWeightedStakeDiffs,
    ) -> Result<u64, ProgramError> {
        let mut processed_dates = vec![];
        for (date, modifier_diff) in weighted_stake_diffs.iter() {
            if date > &beginning_of_the_day {
                break;
            }

            Self::update_index(
                cumulative_index,
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
        cumulative_index: &CumulativeIndex,
        date: u64,
        total_share: u64,
        unclaimed_rewards: &mut u64,
        index_with_precision: &mut u128,
    ) -> ProgramResult {
        let vault_index_for_date =
            find_max_value_limited_by_key(cumulative_index, date).unwrap_or(0);

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

        *index_with_precision = vault_index_for_date;

        Ok(())
    }

    pub fn restrict_claiming(&mut self) -> ProgramResult {
        if self.data[CLAIMING_RESTRICTION_BIT] == 1 {
            return Err(MplxRewardsError::MiningAlreadyRestricted.into());
        }

        self.data[CLAIMING_RESTRICTION_BIT] = 1;

        Ok(())
    }

    pub fn allow_claiming(&mut self) -> ProgramResult {
        if self.data[CLAIMING_RESTRICTION_BIT] == 0 {
            return Err(MplxRewardsError::MiningNotRestricted.into());
        }

        self.data[CLAIMING_RESTRICTION_BIT] = 0;

        Ok(())
    }

    pub fn is_claiming_restricted(&self) -> bool {
        self.data[CLAIMING_RESTRICTION_BIT] == 1
    }
}

impl IsInitialized for Mining {
    fn is_initialized(&self) -> bool {
        self.data[0] == <u8>::from(AccountType::Mining)
    }
}

impl<'a> WrappedImmutableMining<'a> {
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, ProgramError> {
        let (mining, weighted_stake_diffs) = bytes.split_at(Mining::LEN);
        let mining =
            Mining::load_bytes(mining).ok_or(MplxRewardsError::RetreivingZeroCopyAccountFailire)?;

        let weighted_stake_diffs = MiningWeightedStakeDiffs::load_bytes(weighted_stake_diffs)
            .ok_or(MplxRewardsError::RetreivingZeroCopyAccountFailire)?;

        Ok(Self {
            mining,
            weighted_stake_diffs,
        })
    }
}
mod test {
    #[test]
    fn test_wrapped_immutable_mining_is_same_size_as_wrapped_mining() {
        assert_eq!(
            std::mem::size_of::<super::WrappedImmutableMining>(),
            std::mem::size_of::<super::WrappedMining>()
        );
    }

    #[test]
    fn test_can_deserialize_wrapped_immutable_mining_from_bytes_initialized_with_wrapped_mining() {
        let mut bytes = vec![0; super::WrappedMining::LEN];
        let wrapped_mining = super::WrappedMining::from_bytes_mut(&mut bytes).unwrap();
        let reward_pool = solana_program::pubkey::Pubkey::new_unique();
        let mining_owner = solana_program::pubkey::Pubkey::new_unique();
        let reward_mint = solana_program::pubkey::Pubkey::new_unique();
        let index_with_precision = 1234;
        let share = 23456;
        let unclaimed_rewards = 34567;
        let stake_from_others = 45678;
        let bump = 1;
        wrapped_mining.mining.reward_pool = reward_pool;
        wrapped_mining.mining.owner = mining_owner;
        wrapped_mining.mining.reward_mint = reward_mint;
        wrapped_mining.mining.index_with_precision = index_with_precision;
        wrapped_mining.mining.share = share;
        wrapped_mining.mining.unclaimed_rewards = unclaimed_rewards;
        wrapped_mining.mining.stake_from_others = stake_from_others;
        wrapped_mining.mining.bump = bump;
        let wrapped_immutable_mining = super::WrappedImmutableMining::from_bytes(&bytes).unwrap();
        assert_eq!(wrapped_immutable_mining.mining.reward_pool, reward_pool);
        assert_eq!(wrapped_immutable_mining.mining.owner, mining_owner);
        assert_eq!(wrapped_immutable_mining.mining.reward_mint, reward_mint);
        assert_eq!(
            wrapped_immutable_mining.mining.index_with_precision,
            index_with_precision
        );
        assert_eq!(wrapped_immutable_mining.mining.share, share);
        assert_eq!(
            wrapped_immutable_mining.mining.unclaimed_rewards,
            unclaimed_rewards
        );
        assert_eq!(
            wrapped_immutable_mining.mining.stake_from_others,
            stake_from_others
        );
        assert_eq!(wrapped_immutable_mining.mining.bump, bump);
    }
}
