use std::collections::{btree_map::Entry, BTreeMap};

use crate::{
    error::MplxRewardsError,
    state::AccountType,
    utils::{get_curr_unix_ts, LockupPeriod, SafeArithmeticOperations},
};
use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use sokoban::{AVLTree, NodeAllocatorMap};
use solana_program::{
    account_info::AccountInfo,
    clock::{Clock, SECONDS_PER_DAY},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
    sysvar::Sysvar,
};

use super::{WrappedMining, TREE_MAX_SIZE};

/// Precision for index calculation
pub const PRECISION: u128 = 10_000_000_000_000_000;

/// Reward pool
#[derive(Debug, BorshDeserialize, BorshSerialize, BorshSchema, Default)]
pub struct RewardPool {
    /// Account type - RewardPool. This discriminator should exist in order to prevent
    /// shenanigans with customly modified accounts and their fields.
    pub account_type: AccountType,
    /// Saved bump for reward pool account
    pub bump: u8,
    /// The total share of the pool for the moment of the last distribution.
    /// It's so-called "weighted_stake" which is the sum of all stakers' weighted staked.
    /// When somebody deposits or withdraws, or thier stake is expired this value changes.
    pub total_share: u64,
    /// Vault which is responsible for calculating rewards.
    pub calculator: RewardCalculator,
    /// This address is the authority from the staking contract.
    /// We want to be sure that some changes might only be done through the
    /// staking contract. It's PDA from staking that will sign transactions.
    pub deposit_authority: Pubkey,
    /// This address is responsible for distributing rewards
    pub distribute_authority: Pubkey,
    /// The address is responsible for filling vaults with money.
    pub fill_authority: Pubkey,
}

impl RewardPool {
    /// Init reward pool
    pub fn initialize(
        calculator: RewardCalculator,
        bump: u8,
        deposit_authority: Pubkey,
        distribute_authority: Pubkey,
        fill_authority: Pubkey,
    ) -> RewardPool {
        RewardPool {
            account_type: AccountType::RewardPool,
            bump,
            total_share: 0,
            calculator,
            deposit_authority,
            distribute_authority,
            fill_authority,
        }
    }

    /// Distributes rewards via calculating indexes and weighted stakes
    pub fn distribute(&mut self, rewards: u64) -> ProgramResult {
        if self.total_share == 0 {
            return Err(MplxRewardsError::RewardsNoDeposits.into());
        }

        let curr_ts = Clock::get().unwrap().unix_timestamp as u64;
        let beginning_of_the_day = curr_ts - (curr_ts % SECONDS_PER_DAY);

        self.total_share = self
            .calculator
            .consume_old_modifiers(beginning_of_the_day, self.total_share)?;
        if self
            .calculator
            .cumulative_index
            .contains_key(&beginning_of_the_day)
        {
            return Ok(());
        }

        RewardCalculator::update_index(
            &mut self.calculator.cumulative_index,
            &mut self.calculator.index_with_precision,
            rewards,
            self.total_share,
            beginning_of_the_day,
        )?;

        self.calculator.tokens_available_for_distribution = self
            .calculator
            .tokens_available_for_distribution
            .safe_sub(rewards)?;

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
        mining.refresh_rewards(&self.calculator)?;

        // regular weighted stake which will be used in rewards distribution
        let weighted_stake = amount.safe_mul(lockup_period.multiplier())?;

        // shows how weighted stake will change at the end of the staking period
        // weighted_stake_diff = weighted_stake - (amount * flex_multiplier)
        let weighted_stake_diff =
            weighted_stake.safe_sub(amount.safe_mul(LockupPeriod::Flex.multiplier())?)?;

        self.total_share = self.total_share.safe_add(weighted_stake)?;
        mining.mining.share = mining.mining.share.safe_add(weighted_stake)?;

        let modifier = self
            .calculator
            .weighted_stake_diffs
            .entry(lockup_period.end_timestamp(get_curr_unix_ts())?)
            .or_default();
        *modifier = modifier.safe_add(weighted_stake_diff)?;

        let date_to_insert = &lockup_period.end_timestamp(get_curr_unix_ts())?;
        if mining.weighted_stake_diffs.get(date_to_insert).is_some() {
            let modifier = mining.weighted_stake_diffs.get_mut(date_to_insert).unwrap();
            *modifier = modifier.safe_add(weighted_stake_diff)?;
        } else {
            mining
                .weighted_stake_diffs
                .insert(*date_to_insert, weighted_stake_diff);
        }

        if let Some(delegate_mining_acc) = delegate_mining {
            let delegate_mining_data = &mut delegate_mining_acc.data.borrow_mut();
            let mut delegate_mining = WrappedMining::from_bytes_mut(delegate_mining_data)?;

            delegate_mining.mining.stake_from_others =
                delegate_mining.mining.stake_from_others.safe_add(amount)?;

            self.total_share = self.total_share.safe_add(amount)?;
            delegate_mining.refresh_rewards(&self.calculator)?;
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
        mining.refresh_rewards(&self.calculator)?;

        self.total_share = self.total_share.safe_sub(amount)?;
        mining.mining.share = mining.mining.share.safe_sub(amount)?;

        let curr_ts = Clock::get().unwrap().unix_timestamp as u64;
        let beginning_of_the_day = curr_ts - (curr_ts % SECONDS_PER_DAY);
        let reward_pool_share = self
            .calculator
            .consume_old_modifiers(beginning_of_the_day, self.total_share)?;
        self.total_share = reward_pool_share;

        if let Some(delegate_mining_acc) = delegate_mining {
            let delegate_mining_data = &mut delegate_mining_acc.data.borrow_mut();
            let mut delegate_mining = WrappedMining::from_bytes_mut(delegate_mining_data)?;

            delegate_mining.mining.stake_from_others =
                delegate_mining.mining.stake_from_others.safe_sub(amount)?;

            self.total_share = self.total_share.safe_sub(amount)?;
            delegate_mining.refresh_rewards(&self.calculator)?;
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
        mining.refresh_rewards(&self.calculator)?;

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

            Self::modify_weighted_stake_diffs(
                &mut self.calculator.weighted_stake_diffs,
                deposit_old_expiration_ts,
                weighted_stake_diff,
            )?;

            Self::modify_weighted_stake_diffs_avl(
                mining.weighted_stake_diffs,
                deposit_old_expiration_ts,
                weighted_stake_diff,
            )?;

            // also, we need to reduce staking power because we want to extend stake from "scratch"
            mining.mining.share = mining.mining.share.safe_sub(curr_part_of_weighted_stake)?;

            self.total_share = self.total_share.safe_sub(curr_part_of_weighted_stake)?;
        } else {
            // otherwise, we want to substract flex multiplier, becase deposit has expired already
            mining.mining.share = mining
                .mining
                .share
                .safe_sub(curr_part_of_weighted_stake_for_flex)?;

            self.total_share = self
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
                self.total_share = self.total_share.safe_sub(base_amount)?;
                delegate_mining.refresh_rewards(&self.calculator)?;

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

    fn modify_weighted_stake_diffs_avl(
        diffs: &mut AVLTree<u64, u64, TREE_MAX_SIZE>,
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

    fn modify_weighted_stake_diffs(
        diffs: &mut BTreeMap<u64, u64>,
        timestamp: u64,
        weighted_stake_diff: u64,
    ) -> Result<(), MplxRewardsError> {
        match diffs.entry(timestamp) {
            Entry::Vacant(_) => Err(MplxRewardsError::NoWeightedStakeModifiersAtADate),
            Entry::Occupied(mut entry) => {
                let modifier = entry.get_mut();
                *modifier = modifier
                    .checked_sub(weighted_stake_diff)
                    .ok_or(MplxRewardsError::MathOverflow)?;
                Ok(())
            }
        }
    }

    pub fn change_delegate(
        &mut self,
        mining: &mut WrappedMining,
        new_delegate_mining: Option<&AccountInfo>,
        old_delegate_mining: Option<&AccountInfo>,
        staked_amount: u64,
    ) -> ProgramResult {
        mining.refresh_rewards(&self.calculator)?;

        if let Some(old_delegate_info) = old_delegate_mining {
            let old_delegate_mining_data = &mut old_delegate_info.data.borrow_mut();
            let mut old_delegate_mining = WrappedMining::from_bytes_mut(old_delegate_mining_data)?;

            old_delegate_mining.mining.stake_from_others = old_delegate_mining
                .mining
                .stake_from_others
                .safe_sub(staked_amount)?;
            self.total_share = self.total_share.safe_sub(staked_amount)?;
            old_delegate_mining.refresh_rewards(&self.calculator)?;
        }

        if let Some(new_delegate_info) = new_delegate_mining {
            let new_delegate_mining_data = &mut new_delegate_info.data.borrow_mut();
            let mut new_delegate_mining = WrappedMining::from_bytes_mut(new_delegate_mining_data)?;

            new_delegate_mining.mining.stake_from_others = new_delegate_mining
                .mining
                .stake_from_others
                .safe_add(staked_amount)?;
            self.total_share = self.total_share.safe_add(staked_amount)?;
            new_delegate_mining.refresh_rewards(&self.calculator)?;
        }

        Ok(())
    }
}

impl Sealed for RewardPool {}
impl Pack for RewardPool {
    // RewardPool size
    const LEN: usize = 1 + (32 + 1 + 32 + 8 + (4 + RewardCalculator::LEN) + 32);

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let mut slice = dst;
        self.serialize(&mut slice).unwrap();
    }

    fn unpack_from_slice(src: &[u8]) -> Result<RewardPool, ProgramError> {
        let mut src_mut = src;
        Self::deserialize(&mut src_mut).map_err(|err| {
            msg!("Failed to deserialize");
            msg!("{}", err.to_string());
            ProgramError::InvalidAccountData
        })
    }
}

impl IsInitialized for RewardPool {
    fn is_initialized(&self) -> bool {
        self.account_type == AccountType::RewardPool
    }
}

/// Reward vault
#[derive(Debug, BorshDeserialize, BorshSerialize, BorshSchema, Default)]
pub struct RewardCalculator {
    pub token_account_bump: u8,
    /// The address of the Reward Token mint account.
    pub reward_mint: Pubkey,
    /// That is the index that increases on each vault filling.
    /// It points at the moment of time where the filling has been proceeded.
    /// Also, it's responsible for rewards distribution calculations.
    pub index_with_precision: u128,
    /// Weighted stake diffs data structure is used to represent in time
    /// when total_share (which represents sum of all stakers' weighted stake) must change
    /// accordingly to the changes in the staking contract.
    pub weighted_stake_diffs: BTreeMap<u64, u64>,
    /// This cumulative "index" increases on each distribution. It represents both the last time when
    /// the distribution happened and the number which is used in distribution calculations. <Date, index>
    pub cumulative_index: BTreeMap<u64, u128>,
    /// The time where the last distribution made by distribution_authority is allowed. When the date expires,
    /// the only one distribution may be made, distribution all available tokens at once.
    pub distribution_ends_at: u64,
    /// Shows the amount of tokens are ready to be distributed
    pub tokens_available_for_distribution: u64, // default: 0, increased on each fill, decreased on each user claim
}

impl RewardCalculator {
    /// Reward Vault size
    /// TODO: size isn't large enough
    pub const LEN: usize = 1 + 32 + 16 + 32 + (4 + (8 + 8) * 100) + (4 + (8 + 16) * 100);

    /// Consuming old total share modifiers in order to change the total share for the current date
    pub fn consume_old_modifiers(
        &mut self,
        beginning_of_the_day: u64,
        mut total_share: u64,
    ) -> Result<u64, ProgramError> {
        for (date_to_process, modifier) in &self.weighted_stake_diffs {
            if date_to_process > &beginning_of_the_day {
                break;
            }

            total_share = total_share.safe_sub(*modifier)?;
        }
        // drop keys because they have been already consumed and no longer needed
        // +1 because we don't need beginning_of_the_day
        self.weighted_stake_diffs = self
            .weighted_stake_diffs
            .split_off(&(beginning_of_the_day + 1));
        Ok(total_share)
    }

    /// recalculates the index for the given rewards and total share
    pub fn update_index(
        cumulative_index: &mut BTreeMap<u64, u128>,
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

    /// Defines the amount of money that will be distributed
    /// The formula is vault_tokens_are_available_for_distribution / (distrtribution_period_ends_at - curr_time)
    pub fn rewards_to_distribute(&self) -> Result<u64, ProgramError> {
        let distribution_days_left: u128 =
            (self.distribution_ends_at.saturating_sub(get_curr_unix_ts()) / SECONDS_PER_DAY).into();

        if distribution_days_left == 0 {
            return Ok(self.tokens_available_for_distribution);
        }

        // ((tokens_available_for_distribution * precision) / days_left) / precision
        Ok(u64::try_from(
            (u128::from(self.tokens_available_for_distribution))
                .safe_mul(PRECISION)?
                .safe_div(distribution_days_left)?
                .safe_div(PRECISION)?,
        )
        .map_err(|_| MplxRewardsError::InvalidPrimitiveTypesConversion)?)
    }
}
