use crate::state::PRECISION;
use std::collections::BTreeMap;

use crate::error::MplxRewardsError;
use crate::utils::get_curr_unix_ts;
use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::{
    clock::SECONDS_PER_DAY, entrypoint::ProgramResult, program_error::ProgramError, pubkey::Pubkey,
};

/// Reward vault
#[derive(Debug, BorshDeserialize, BorshSerialize, BorshSchema, Default)]
pub struct RewardVault {
    /// Bump of vault account
    pub bump: u8,
    /// Reward mint address
    pub reward_mint: Pubkey,
    /// Index with precision
    pub index_with_precision: u128,
    /// Weighted stake diffs is used to store the modifiers which will be applied to the total_share
    pub weighted_stake_diffs: BTreeMap<u64, u64>,
    /// Cumulative index per day. <Date, index>
    pub cumulative_index: BTreeMap<u64, u128>,
    /// The time where the last distribution made by distribution_authority is allowed
    pub distribution_ends_at: u64,
    /// Shows amount of tokens are ready to be distributed
    pub tokens_available_for_distribution: u64, // default: 0, increased on each fill, decreased on each user claim
}

impl RewardVault {
    /// Reward Vault size
    /// TODO: size isn't large enough
    pub const LEN: usize = 1 + 32 + 16 + 32 + (4 + (8 + 8) * 100) + (4 + (8 + 16) * 100);

    /// Consuming old total share modifiers in order to change the total share for the current date
    pub fn consume_old_modifiers(
        &mut self,
        beginning_of_the_day: u64,
        mut total_share: u64,
    ) -> Result<u64, ProgramError> {
        for (date_to_process, modifier) in self.weighted_stake_diffs.iter() {
            if date_to_process > &beginning_of_the_day {
                break;
            }

            total_share = total_share
                .checked_sub(*modifier)
                .ok_or(MplxRewardsError::MathOverflow)?;
        }
        // drop keys because they have been already consumed and no longer needed
        self.weighted_stake_diffs
            .retain(|date, _modifier| date > &beginning_of_the_day);
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
            .checked_mul(rewards as u128)
            .ok_or(MplxRewardsError::MathOverflow)?
            .checked_div(total_share as u128)
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
        Ok(((self.tokens_available_for_distribution as u128)
            .checked_mul(PRECISION)
            .ok_or(MplxRewardsError::MathOverflow)?
            .checked_div(distribution_days_left)
            .ok_or(MplxRewardsError::MathOverflow)?)
        .checked_div(PRECISION)
        .ok_or(MplxRewardsError::MathOverflow)? as u64)
    }
}
