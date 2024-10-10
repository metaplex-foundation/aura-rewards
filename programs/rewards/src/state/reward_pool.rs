use crate::{
    asserts::{assert_account_key, assert_pubkey_eq},
    error::MplxRewardsError,
    state::AccountType,
    utils::{get_curr_unix_ts, LockupPeriod, SafeArithmeticOperations},
};
use bytemuck::{Pod, Zeroable};
use shank::ShankAccount;
use sokoban::{NodeAllocatorMap, ZeroCopy};
use solana_program::{
    account_info::AccountInfo,
    clock::{Clock, SECONDS_PER_DAY},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    program_pack::IsInitialized,
    pubkey::Pubkey,
    sysvar::Sysvar,
};

use super::{
    CumulativeIndex, MiningWeightedStakeDiffs, PoolWeightedStakeDiffs, WrappedMining, PRECISION,
};

pub struct WrappedRewardPool<'a> {
    pub pool: &'a mut RewardPool,
    /// Weighted stake diffs data structure is used to represent in time
    /// when total_share (which represents sum of all stakers' weighted stake) must change
    /// accordingly to the changes in the staking contract.
    pub weighted_stake_diffs: &'a mut PoolWeightedStakeDiffs,
    /// This cumulative "index" increases on each distribution. It represents both the last time when
    /// the distribution happened and the number which is used in distribution calculations. <Date, index>
    pub cumulative_index: &'a mut CumulativeIndex,
}

pub struct WrappedImmutableRewardPool<'a> {
    pub pool: &'a RewardPool,
    /// Weighted stake diffs data structure is used to represent in time
    /// when total_share (which represents sum of all stakers' weighted stake) must change
    /// accordingly to the changes in the staking contract.
    pub weighted_stake_diffs: &'a PoolWeightedStakeDiffs,
    /// This cumulative "index" increases on each distribution. It represents both the last time when
    /// the distribution happened and the number which is used in distribution calculations. <Date, index>
    pub cumulative_index: &'a CumulativeIndex,
}

impl<'a> WrappedImmutableRewardPool<'a> {
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, ProgramError> {
        let (pool, trees) = bytes.split_at(RewardPool::LEN);
        let (weighted_stake_diffs, cumulative_index) =
            trees.split_at(std::mem::size_of::<PoolWeightedStakeDiffs>());

        let pool = RewardPool::load_bytes(pool)
            .ok_or(MplxRewardsError::RetreivingZeroCopyAccountFailire)?;

        let weighted_stake_diffs = PoolWeightedStakeDiffs::load_bytes(weighted_stake_diffs)
            .ok_or(MplxRewardsError::RetreivingZeroCopyAccountFailire)?;

        let cumulative_index = CumulativeIndex::load_bytes(cumulative_index)
            .ok_or(MplxRewardsError::RetreivingZeroCopyAccountFailire)?;

        Ok(Self {
            pool,
            weighted_stake_diffs,
            cumulative_index,
        })
    }
}

impl<'a> WrappedRewardPool<'a> {
    pub const LEN: usize = 64480;

    pub fn from_bytes_mut(bytes: &'a mut [u8]) -> Result<Self, ProgramError> {
        let (pool, trees) = bytes.split_at_mut(RewardPool::LEN);
        let (weighted_stake_diffs, cumulative_index) =
            trees.split_at_mut(std::mem::size_of::<PoolWeightedStakeDiffs>());

        let pool = RewardPool::load_mut_bytes(pool)
            .ok_or(MplxRewardsError::RetreivingZeroCopyAccountFailire)?;

        let weighted_stake_diffs = PoolWeightedStakeDiffs::load_mut_bytes(weighted_stake_diffs)
            .ok_or(MplxRewardsError::RetreivingZeroCopyAccountFailire)?;

        let cumulative_index = CumulativeIndex::load_mut_bytes(cumulative_index)
            .ok_or(MplxRewardsError::RetreivingZeroCopyAccountFailire)?;

        Ok(Self {
            pool,
            weighted_stake_diffs,
            cumulative_index,
        })
    }

    /// Consuming old total share modifiers in order to change the total share for the current date
    pub fn consume_old_modifiers(
        &mut self,
        beginning_of_the_day: u64,
        mut total_share: u64,
    ) -> Result<u64, ProgramError> {
        let mut processed_dates = vec![];
        for (date_to_process, modifier) in self.weighted_stake_diffs.iter() {
            if date_to_process > &beginning_of_the_day {
                break;
            }

            total_share = total_share.safe_sub(*modifier)?;
            processed_dates.push(*date_to_process);
        }
        for date in processed_dates {
            self.weighted_stake_diffs.remove(&date);
        }
        Ok(total_share)
    }

    /// recalculates the index for the given rewards and total share
    pub fn update_index(
        cumulative_index: &mut CumulativeIndex,
        index_with_precision: &mut u128,
        rewards: u64,
        total_share: u64,
        date_to_process: u64,
    ) -> ProgramResult {
        let index = PRECISION
            .safe_mul(u128::from(rewards))?
            .safe_div(u128::from(total_share))?;

        let latest_index = index_with_precision.safe_add(index)?;

        cumulative_index.insert(date_to_process, latest_index);
        *index_with_precision = latest_index;

        Ok(())
    }

    /// Distributes rewards via calculating indexes and weighted stakes
    pub fn distribute(&mut self, rewards: u64) -> ProgramResult {
        if self.pool.total_share == 0 {
            return Err(MplxRewardsError::RewardsNoDeposits.into());
        }

        let curr_ts = Clock::get().unwrap().unix_timestamp as u64;
        let beginning_of_the_day = curr_ts - (curr_ts % SECONDS_PER_DAY);

        self.pool.total_share =
            self.consume_old_modifiers(beginning_of_the_day, self.pool.total_share)?;
        if self.cumulative_index.contains(&beginning_of_the_day) {
            return Ok(());
        }

        WrappedRewardPool::update_index(
            self.cumulative_index,
            &mut self.pool.index_with_precision,
            rewards,
            self.pool.total_share,
            beginning_of_the_day,
        )?;

        self.pool.tokens_available_for_distribution = self
            .pool
            .tokens_available_for_distribution
            .safe_sub(rewards)?;

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn change_delegate(
        &mut self,
        mining: &mut WrappedMining,
        reward_pool: &AccountInfo,
        new_delegate_mining: Option<&AccountInfo>,
        old_delegate_mining: Option<&AccountInfo>,
        old_delegate: &Pubkey,
        new_delegate: &Pubkey,
        staked_amount: u64,
    ) -> ProgramResult {
        mining.refresh_rewards(self.cumulative_index)?;

        if let Some(old_delegate_info) = old_delegate_mining {
            let old_delegate_mining_data = &mut old_delegate_info.data.borrow_mut();
            let mut old_delegate_mining = WrappedMining::from_bytes_mut(old_delegate_mining_data)?;

            assert_pubkey_eq(&old_delegate_mining.mining.owner, old_delegate)?;
            assert_account_key(reward_pool, &old_delegate_mining.mining.reward_pool)?;

            old_delegate_mining.mining.stake_from_others = old_delegate_mining
                .mining
                .stake_from_others
                .safe_sub(staked_amount)?;
            self.pool.total_share = self.pool.total_share.safe_sub(staked_amount)?;
            old_delegate_mining.refresh_rewards(self.cumulative_index)?;
        } else {
            assert_pubkey_eq(&mining.mining.owner, old_delegate)?;
            // check for reward pool is not needed because it's already checked previously
            // in case when old_delegate_mining is the same as mining
        }

        if let Some(new_delegate_info) = new_delegate_mining {
            let new_delegate_mining_data = &mut new_delegate_info.data.borrow_mut();
            let mut new_delegate_mining = WrappedMining::from_bytes_mut(new_delegate_mining_data)?;

            assert_pubkey_eq(&new_delegate_mining.mining.owner, new_delegate)?;
            assert_account_key(reward_pool, &new_delegate_mining.mining.reward_pool)?;

            new_delegate_mining.mining.stake_from_others = new_delegate_mining
                .mining
                .stake_from_others
                .safe_add(staked_amount)?;
            self.pool.total_share = self.pool.total_share.safe_add(staked_amount)?;
            new_delegate_mining.refresh_rewards(self.cumulative_index)?;
        } else {
            assert_pubkey_eq(&mining.mining.owner, new_delegate)?;
            // check for reward pool is not needed because it's already checked previously
            // in case when old_delegate_mining is the same as mining
        }

        Ok(())
    }

    /// Process deposit
    pub fn deposit(
        &mut self,
        mining: &mut WrappedMining,
        amount: u64,
        lockup_period: LockupPeriod,
        delegate_mining: Option<&AccountInfo>,
    ) -> ProgramResult {
        mining.refresh_rewards(self.cumulative_index)?;

        // regular weighted stake which will be used in rewards distribution
        let weighted_stake = amount.safe_mul(lockup_period.multiplier())?;

        // shows how weighted stake will change at the end of the staking period
        // weighted_stake_diff = weighted_stake - (amount * flex_multiplier)
        let weighted_stake_diff =
            weighted_stake.safe_sub(amount.safe_mul(LockupPeriod::Flex.multiplier())?)?;

        self.pool.total_share = self.pool.total_share.safe_add(weighted_stake)?;
        mining.mining.share = mining.mining.share.safe_add(weighted_stake)?;

        let stake_expiration_date = lockup_period.end_timestamp(get_curr_unix_ts())?;
        let modifier = if let Some(modifier) = self.weighted_stake_diffs.get(&stake_expiration_date)
        {
            *modifier
        } else {
            0
        };

        self.weighted_stake_diffs.insert(
            stake_expiration_date,
            modifier.safe_add(weighted_stake_diff)?,
        );

        if let Some(modifier) = mining.weighted_stake_diffs.get_mut(&stake_expiration_date) {
            *modifier = modifier.safe_add(weighted_stake_diff)?;
        } else {
            mining
                .weighted_stake_diffs
                .insert(stake_expiration_date, weighted_stake_diff);
        }

        if let Some(delegate_mining_acc) = delegate_mining {
            let delegate_mining_data = &mut delegate_mining_acc.data.borrow_mut();
            let mut delegate_mining = WrappedMining::from_bytes_mut(delegate_mining_data)?;

            delegate_mining.mining.stake_from_others =
                delegate_mining.mining.stake_from_others.safe_add(amount)?;

            self.pool.total_share = self.pool.total_share.safe_add(amount)?;
            delegate_mining.refresh_rewards(self.cumulative_index)?;
        }

        Ok(())
    }

    /// Process withdraw
    pub fn withdraw(
        &mut self,
        mining: &mut WrappedMining,
        amount: u64,
        delegate_mining: Option<&AccountInfo>,
    ) -> ProgramResult {
        mining.refresh_rewards(self.cumulative_index)?;

        self.pool.total_share = self.pool.total_share.safe_sub(amount)?;
        mining.mining.share = mining.mining.share.safe_sub(amount)?;

        let curr_ts = Clock::get().unwrap().unix_timestamp as u64;
        let beginning_of_the_day = curr_ts - (curr_ts % SECONDS_PER_DAY);
        let pool_share = self.consume_old_modifiers(beginning_of_the_day, self.pool.total_share)?;
        self.pool.total_share = pool_share;

        if let Some(delegate_mining_acc) = delegate_mining {
            let delegate_mining_data = &mut delegate_mining_acc.data.borrow_mut();
            let mut delegate_mining = WrappedMining::from_bytes_mut(delegate_mining_data)?;

            delegate_mining.mining.stake_from_others =
                delegate_mining.mining.stake_from_others.safe_sub(amount)?;

            self.pool.total_share = self.pool.total_share.safe_sub(amount)?;
            delegate_mining.refresh_rewards(self.cumulative_index)?;
        }

        Ok(())
    }

    /// Process extend stake
    #[allow(clippy::too_many_arguments)]
    pub fn extend(
        &mut self,
        mining: &mut WrappedMining,
        old_lockup_period: LockupPeriod,
        new_lockup_period: LockupPeriod,
        deposit_start_ts: u64,
        base_amount: u64,
        additional_amount: u64,
        delegate_mining: Option<&AccountInfo>,
    ) -> ProgramResult {
        mining.refresh_rewards(self.cumulative_index)?;

        let curr_ts = get_curr_unix_ts();

        let deposit_old_expiration_ts = if old_lockup_period == LockupPeriod::Flex {
            0 // it's expired, so the date is in the past
        } else {
            old_lockup_period.end_timestamp(deposit_start_ts)?
        };

        // curr_part_of_weighted_stake_for_flex = old_base_amount * flex_multipler
        let curr_part_of_weighted_stake_for_flex =
            base_amount.safe_mul(LockupPeriod::Flex.multiplier())?;

        // if current date is lower than stake expiration date, we need to
        // remove stake modifier from the date of expiration
        if curr_ts < deposit_old_expiration_ts {
            // current_part_of_weighted_stake = base_amount * lockup_period_multiplier
            let curr_part_of_weighted_stake =
                base_amount.safe_mul(old_lockup_period.multiplier())?;

            // weighted_stake_modifier_to_remove = old_base_amount * lockup_period_multiplier - amount_times_flex
            let weighted_stake_diff =
                curr_part_of_weighted_stake.safe_sub(curr_part_of_weighted_stake_for_flex)?;

            RewardPool::modify_weighted_stake_diffs(
                mining.weighted_stake_diffs,
                deposit_old_expiration_ts,
                weighted_stake_diff,
            )?;

            // also, we need to reduce staking power because we want to extend stake from "scratch"
            mining.mining.share = mining.mining.share.safe_sub(curr_part_of_weighted_stake)?;

            self.pool.total_share = self
                .pool
                .total_share
                .safe_sub(curr_part_of_weighted_stake)?;
        } else {
            // otherwise, we want to substract flex multiplier, becase deposit has expired already
            mining.mining.share = mining
                .mining
                .share
                .safe_sub(curr_part_of_weighted_stake_for_flex)?;

            self.pool.total_share = self
                .pool
                .total_share
                .safe_sub(curr_part_of_weighted_stake_for_flex)?;
        }

        // do actions like it's a regular deposit
        let amount_to_restake = base_amount.safe_add(additional_amount)?;

        let delegate_mining = match delegate_mining {
            Some(delegate_mining_acc) => {
                let delegate_mining_data = &mut delegate_mining_acc.data.borrow_mut();
                let mut delegate_mining = WrappedMining::from_bytes_mut(delegate_mining_data)?;

                delegate_mining.mining.stake_from_others = delegate_mining
                    .mining
                    .stake_from_others
                    .safe_sub(base_amount)?;
                self.pool.total_share = self.pool.total_share.safe_sub(base_amount)?;
                delegate_mining.refresh_rewards(self.cumulative_index)?;

                Some(delegate_mining_acc)
            }
            None => None,
        };

        self.deposit(
            mining,
            amount_to_restake,
            new_lockup_period,
            delegate_mining,
        )?;

        Ok(())
    }
}

/// Reward pool
#[repr(C)]
#[derive(Debug, Default, Clone, Copy, Pod, Zeroable, ShankAccount)]
pub struct RewardPool {
    /// This address is the authority from the staking contract.
    /// We want to be sure that some changes might only be done through the
    /// staking contract. It's PDA from staking that will sign transactions.
    pub deposit_authority: Pubkey,
    /// This address is responsible for distributing rewards
    pub distribute_authority: Pubkey,
    /// The address is responsible for filling vaults with money.
    pub fill_authority: Pubkey,
    /// The address of the Reward Token mint account.
    pub reward_mint: Pubkey,
    /// That is the index that increases on each vault filling.
    /// It points at the moment of time where the filling has been proceeded.
    /// Also, it's responsible for rewards distribution calculations.
    pub index_with_precision: u128,
    /// The total share of the pool for the moment of the last distribution.
    /// It's so-called "weighted_stake" which is the sum of all stakers' weighted staked.
    /// When somebody deposits or withdraws, or thier stake is expired this value changes.
    pub total_share: u64,
    /// The time where the last distribution made by distribution_authority is allowed. When the date expires,
    /// the only one distribution may be made, distribution all available tokens at once.
    pub distribution_ends_at: u64,
    /// Shows the amount of tokens are ready to be distributed
    pub tokens_available_for_distribution: u64, // default: 0, increased on each fill, decreased on each user claim
    pub token_account_bump: u8,
    /// Account type - Mining. This discriminator should exist in order to prevent
    /// shenanigans with customly modified accounts and their fields.
    /// 1: account type
    /// 2-7: unused
    pub data: [u8; 7],
}

impl ZeroCopy for RewardPool {}

impl RewardPool {
    pub const LEN: usize = std::mem::size_of::<RewardPool>();

    /// Init reward pool
    pub fn initialize(
        token_account_bump: u8,
        deposit_authority: Pubkey,
        distribute_authority: Pubkey,
        fill_authority: Pubkey,
        reward_mint: Pubkey,
    ) -> RewardPool {
        let account_type = AccountType::RewardPool.into();
        let mut data = [0; 7];
        data[0] = account_type;
        RewardPool {
            data,
            token_account_bump,
            deposit_authority,
            distribute_authority,
            fill_authority,
            reward_mint,
            ..Default::default()
        }
    }

    /// Defines the amount of money that will be distributed
    /// The formula is vault_tokens_are_available_for_distribution / (distrtribution_period_ends_at - curr_time)
    pub fn rewards_to_distribute(&self) -> Result<u64, ProgramError> {
        let distribution_days_left: u128 =
            (self.distribution_ends_at.saturating_sub(get_curr_unix_ts()) / SECONDS_PER_DAY).into();

        if distribution_days_left == 0 {
            return Ok(self.tokens_available_for_distribution);
        }

        // (tokens_available_for_distribution * precision) ∕ (days_left * precision)”
        let numerator = u128::from(self.tokens_available_for_distribution).safe_mul(PRECISION)?;
        let denominator = distribution_days_left.safe_mul(PRECISION)?;
        Ok(u64::try_from(numerator.safe_div(denominator)?)
            .map_err(|_| MplxRewardsError::InvalidPrimitiveTypesConversion)?)
    }

    fn modify_weighted_stake_diffs(
        diffs: &mut MiningWeightedStakeDiffs,
        timestamp: u64,
        weighted_stake_diff: u64,
    ) -> Result<(), MplxRewardsError> {
        match diffs.get_mut(&timestamp) {
            None => Err(MplxRewardsError::NoWeightedStakeModifiersAtADate),
            Some(modifier) => {
                *modifier = modifier.safe_sub(weighted_stake_diff)?;
                Ok(())
            }
        }
    }
}

impl IsInitialized for RewardPool {
    fn is_initialized(&self) -> bool {
        self.data[0] == <u8>::from(AccountType::RewardPool)
    }
}

mod test {
    #[test]
    fn test_wrapped_immutable_reward_pool_is_same_size_as_wrapped_reward_pool() {
        assert_eq!(
            std::mem::size_of::<super::WrappedImmutableRewardPool>(),
            std::mem::size_of::<super::WrappedRewardPool>()
        );
    }

    #[test]
    fn test_can_deserialize_wrapped_immutable_reward_pool_from_bytes_initialized_with_wrapped_reward_pool(
    ) {
        let mut bytes = vec![0; super::WrappedRewardPool::LEN];
        let wrapped_reward_pool = super::WrappedRewardPool::from_bytes_mut(&mut bytes).unwrap();
        let deposit_authority = solana_program::pubkey::Pubkey::new_unique();
        let distribute_authority = solana_program::pubkey::Pubkey::new_unique();
        let fill_authority = solana_program::pubkey::Pubkey::new_unique();
        let reward_mint = solana_program::pubkey::Pubkey::new_unique();
        wrapped_reward_pool.pool.deposit_authority = deposit_authority;
        wrapped_reward_pool.pool.distribute_authority = distribute_authority;
        wrapped_reward_pool.pool.fill_authority = fill_authority;
        wrapped_reward_pool.pool.reward_mint = reward_mint;
        let index_with_precision = 12345;
        let total_share = 65432;
        let distribution_ends_at = 54321;
        let tokens_available_for_distribution = 23456;
        let token_account_bump = 12;
        wrapped_reward_pool.pool.index_with_precision = index_with_precision;
        wrapped_reward_pool.pool.total_share = total_share;
        wrapped_reward_pool.pool.distribution_ends_at = distribution_ends_at;
        wrapped_reward_pool.pool.tokens_available_for_distribution =
            tokens_available_for_distribution;
        wrapped_reward_pool.pool.token_account_bump = token_account_bump;

        let wrapped_immutable_reward_pool =
            super::WrappedImmutableRewardPool::from_bytes(&bytes).unwrap();
        assert_eq!(
            wrapped_immutable_reward_pool.pool.deposit_authority,
            deposit_authority
        );
        assert_eq!(
            wrapped_immutable_reward_pool.pool.distribute_authority,
            distribute_authority
        );
        assert_eq!(
            wrapped_immutable_reward_pool.pool.fill_authority,
            fill_authority
        );
        assert_eq!(wrapped_immutable_reward_pool.pool.reward_mint, reward_mint);
        assert_eq!(
            wrapped_immutable_reward_pool.pool.index_with_precision,
            index_with_precision
        );
        assert_eq!(wrapped_immutable_reward_pool.pool.total_share, total_share);
        assert_eq!(
            wrapped_immutable_reward_pool.pool.distribution_ends_at,
            distribution_ends_at
        );
        assert_eq!(
            wrapped_immutable_reward_pool
                .pool
                .tokens_available_for_distribution,
            tokens_available_for_distribution
        );
        assert_eq!(
            wrapped_immutable_reward_pool.pool.token_account_bump,
            token_account_bump
        );
    }
}
