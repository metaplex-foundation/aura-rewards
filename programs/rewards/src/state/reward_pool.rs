use std::collections::BTreeMap;

use crate::error::MplxRewardsError;
use crate::state::{AccountType, Mining};
use crate::utils::{get_curr_unix_ts, LockupPeriod};
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

        Ok(())
    }

    /// Process deposit
    pub fn deposit(
        &mut self,
        mining: &mut Mining,
        amount: u64,
        lockup_period: LockupPeriod,
    ) -> ProgramResult {
        mining.refresh_rewards(&self.calculator)?;

        // regular weighted stake which will be used in rewards distribution
        let weighted_stake = amount
            .checked_mul(lockup_period.multiplier())
            .ok_or(MplxRewardsError::MathOverflow)?;

        // shows how weighted stake will change at the end of the staking period
        // weighted_stake_diff = weighted_stake - (amount * flex_multiplier)
        let weighted_stake_diff = weighted_stake
            .checked_sub(
                amount
                    .checked_mul(LockupPeriod::Flex.multiplier())
                    .ok_or(MplxRewardsError::MathOverflow)?,
            )
            .ok_or(MplxRewardsError::MathOverflow)?;

        self.total_share = self
            .total_share
            .checked_add(weighted_stake)
            .ok_or(MplxRewardsError::MathOverflow)?;

        mining.share = mining
            .share
            .checked_add(weighted_stake)
            .ok_or(MplxRewardsError::MathOverflow)?;

        let modifier = self
            .calculator
            .weighted_stake_diffs
            .entry(lockup_period.end_timestamp(get_curr_unix_ts())?)
            .or_default();
        *modifier = modifier
            .checked_add(weighted_stake_diff)
            .ok_or(MplxRewardsError::MathOverflow)?;

        let modifier = mining
            .index
            .weighted_stake_diffs
            .entry(lockup_period.end_timestamp(get_curr_unix_ts())?)
            .or_default();
        *modifier = modifier
            .checked_add(weighted_stake_diff)
            .ok_or(MplxRewardsError::MathOverflow)?;

        Ok(())
    }

    /// Process withdraw
    pub fn withdraw(&mut self, mining: &mut Mining, amount: u64) -> ProgramResult {
        mining.refresh_rewards(&self.calculator)?;

        self.total_share = self
            .total_share
            .checked_sub(amount)
            .ok_or(MplxRewardsError::MathOverflow)?;
        mining.share = mining
            .share
            .checked_sub(amount)
            .ok_or(MplxRewardsError::MathOverflow)?;

        Ok(())
    }

    /// Process extend stake
    pub fn extend(
        &mut self,
        mining: &mut Mining,
        old_lockup_period: LockupPeriod,
        new_lockup_period: LockupPeriod,
        deposit_start_ts: u64,
        base_amount: u64,
        additional_amount: u64,
    ) -> ProgramResult {
        mining.refresh_rewards(&self.calculator)?;

        let curr_ts = get_curr_unix_ts();

        let deposit_old_expiration_ts = if old_lockup_period == LockupPeriod::Flex {
            0 // it's expired, so the date is in the past
        } else {
            old_lockup_period.end_timestamp(deposit_start_ts)?
        };

        // curr_part_of_weighted_stake_for_flex = old_base_amount * flex_multipler
        let curr_part_of_weighted_stake_for_flex = base_amount
            .checked_mul(LockupPeriod::Flex.multiplier())
            .ok_or(MplxRewardsError::MathOverflow)?;

        // if current date is lower than stake expiration date, we need to
        // remove stake modifier from the date of expiration
        if curr_ts < deposit_old_expiration_ts {
            // current_part_of_weighted_stake =
            let curr_part_of_weighted_stake = base_amount
                .checked_mul(old_lockup_period.multiplier())
                .ok_or(MplxRewardsError::MathOverflow)?;

            // weighted_stake_modifier_to_remove = old_base_amount * lockup_period_multiplier - amount_times_flex
            let weighted_stake_diff = curr_part_of_weighted_stake
                .checked_sub(curr_part_of_weighted_stake_for_flex)
                .ok_or(MplxRewardsError::MathOverflow)?;

            self.calculator
                .weighted_stake_diffs
                .entry(deposit_old_expiration_ts)
                .and_modify(|modifier| *modifier -= weighted_stake_diff);

            mining
                .index
                .weighted_stake_diffs
                .entry(deposit_old_expiration_ts)
                .and_modify(|modifier| *modifier -= weighted_stake_diff);

            // also, we need to reduce staking power because we want to extend stake from "scratch"
            mining.share = mining
                .share
                .checked_sub(curr_part_of_weighted_stake)
                .ok_or(MplxRewardsError::MathOverflow)?;

            self.total_share = self
                .total_share
                .checked_sub(curr_part_of_weighted_stake)
                .ok_or(MplxRewardsError::MathOverflow)?;
        } else {
            // otherwise, we want to substract flex multiplier, becase deposit has expired already
            mining.share = mining
                .share
                .checked_sub(curr_part_of_weighted_stake_for_flex)
                .ok_or(MplxRewardsError::MathOverflow)?;

            self.total_share = self
                .total_share
                .checked_sub(curr_part_of_weighted_stake_for_flex)
                .ok_or(MplxRewardsError::MathOverflow)?;
        }

        // do actions like it's a regular deposit
        let amount_to_restake = base_amount
            .checked_add(additional_amount)
            .ok_or(MplxRewardsError::MathOverflow)?;
        self.deposit(mining, amount_to_restake, new_lockup_period)?;

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

            total_share = total_share
                .checked_sub(*modifier)
                .ok_or(MplxRewardsError::MathOverflow)?;
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
            .checked_mul(u128::from(rewards))
            .ok_or(MplxRewardsError::MathOverflow)?
            .checked_div(u128::from(total_share))
            .ok_or(MplxRewardsError::MathOverflow)?;

        let cumulative_index_to_insert = {
            if let Some((_, index)) = cumulative_index.last_key_value() {
                *index
            } else {
                0
            }
            .checked_add(index)
            .ok_or(MplxRewardsError::MathOverflow)?
        };

        cumulative_index.insert(date_to_process, cumulative_index_to_insert);

        *index_with_precision = index_with_precision
            .checked_add(index)
            .ok_or(MplxRewardsError::MathOverflow)?;

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
            ((u128::from(self.tokens_available_for_distribution))
                .checked_mul(PRECISION)
                .ok_or(MplxRewardsError::MathOverflow)?
                .checked_div(distribution_days_left)
                .ok_or(MplxRewardsError::MathOverflow)?)
            .checked_div(PRECISION)
            .ok_or(MplxRewardsError::MathOverflow)?,
        )
        .map_err(|_| MplxRewardsError::InvalidPrimitiveTypesConversion)?)
    }
}
