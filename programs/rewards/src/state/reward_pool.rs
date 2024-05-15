use std::collections::BTreeMap;

use crate::error::MplxRewardsError;
use crate::state::{AccountType, Mining};
use crate::utils::LockupPeriod;
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
/// Max reward vaults
pub const MAX_REWARDS: usize = 5;

/// Ring buffer capacity
pub const RINGBUF_CAP: usize = 365;

/// Reward pool
#[derive(Debug, BorshDeserialize, BorshSerialize, BorshSchema, Default)]
pub struct RewardPool {
    /// Account type - RewardPool
    pub account_type: AccountType,
    /// Rewards root account (ex-Config program account)
    pub rewards_root: Pubkey,
    /// Saved bump for reward pool account
    pub bump: u8,
    /// Liquidity mint
    pub liquidity_mint: Pubkey,
    /// Reward total share
    pub total_share: u64,
    /// A set of all possible rewards that we can get for this pool
    pub vaults: Vec<RewardVault>,
    /// The address responsible for the charge of rewards for users.
    /// It executes deposits on the rewards pools.
    pub deposit_authority: Pubkey,
}

impl RewardPool {
    /// Init reward pool
    pub fn init(params: InitRewardPoolParams) -> RewardPool {
        RewardPool {
            account_type: AccountType::RewardPool,
            rewards_root: params.rewards_root,
            bump: params.bump,
            liquidity_mint: params.liquidity_mint,
            total_share: 0,
            vaults: vec![],
            deposit_authority: params.deposit_authority,
        }
    }

    /// Process add vault
    pub fn add_vault(&mut self, reward: RewardVault) -> ProgramResult {
        if self
            .vaults
            .iter()
            .any(|v| v.reward_mint == reward.reward_mint)
        {
            return Err(ProgramError::InvalidArgument);
        }

        self.vaults.push(reward);

        Ok(())
    }

    /// Process fill
    pub fn fill(&mut self, reward_mint: Pubkey, rewards: u64) -> ProgramResult {
        if self.total_share == 0 {
            return Err(MplxRewardsError::RewardsNoDeposits.into());
        }

        let curr_ts = Clock::get().unwrap().unix_timestamp as u64;
        let beginning_of_the_day = curr_ts - (curr_ts % SECONDS_PER_DAY);

        let vault = self
            .vaults
            .iter_mut()
            .find(|v| v.reward_mint == reward_mint)
            .ok_or(MplxRewardsError::RewardsInvalidVault)?;

        while let Some((date, _index)) = vault.cumulative_index.last_key_value() {
            let day_to_process = date
                .checked_add(SECONDS_PER_DAY)
                .ok_or(MplxRewardsError::MathOverflow)?;

            if day_to_process <= beginning_of_the_day {
                break;
            }

            self.total_share = self
                .total_share
                .checked_sub(
                    self.total_share
                        .checked_sub(
                            *vault
                                .weighted_stake_diffs
                                .get(&day_to_process)
                                .unwrap_or(&0),
                        )
                        .ok_or(MplxRewardsError::MathOverflow)?,
                )
                .ok_or(MplxRewardsError::MathOverflow)?;

            let index = PRECISION
                .checked_mul(rewards as u128)
                .ok_or(MplxRewardsError::MathOverflow)?
                .checked_div(self.total_share as u128)
                .ok_or(MplxRewardsError::MathOverflow)?;

            let cumulative_index = index
                .checked_add(index)
                .ok_or(MplxRewardsError::MathOverflow)?;
            vault
                .cumulative_index
                .insert(day_to_process, cumulative_index);

            vault.index_with_precision = vault
                .index_with_precision
                .checked_add(index)
                .ok_or(MplxRewardsError::MathOverflow)?;
        }

        // drop keys because they have been already consumed and no longer needed
        let keys_to_drop: Vec<u64> = vault
            .weighted_stake_diffs
            .keys()
            .cloned()
            .filter(|&k| k < curr_ts)
            .collect();
        for key in keys_to_drop {
            vault.weighted_stake_diffs.remove(&key);
        }

        Ok(())
    }

    /// Process deposit
    pub fn deposit(
        &mut self,
        mining: &mut Mining,
        amount: u64,
        lockup_period: LockupPeriod,
        reward_mint: &Pubkey,
    ) -> ProgramResult {
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

        let reward_index = mining.reward_index_mut(*reward_mint);
        let modifier = reward_index
            .weighted_stake_diffs
            .entry(lockup_period.end_timestamp()?)
            .or_default();
        *modifier = modifier
            .checked_add(weighted_stake_diff)
            .ok_or(MplxRewardsError::MathOverflow)?;

        mining.refresh_rewards(self.vaults.iter())?;

        Ok(())
    }

    /// Process withdraw
    pub fn withdraw(&mut self, mining: &mut Mining, amount: u64) -> ProgramResult {
        mining.refresh_rewards(self.vaults.iter())?;

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
}

/// Initialize a Reward Pool params
pub struct InitRewardPoolParams {
    /// Rewards Root (ex-Config program account)
    pub rewards_root: Pubkey,
    /// Saved bump for reward pool account
    pub bump: u8,
    /// Liquidity mint
    pub liquidity_mint: Pubkey,
    /// The address responsible for the charge of rewards for users.
    /// It executes deposits on the rewards pools.
    pub deposit_authority: Pubkey,
}

impl Sealed for RewardPool {}
impl Pack for RewardPool {
    // TODO: change the Size of the RewardPool
    const LEN: usize = 1 + (32 + 1 + 32 + 8 + (4 + RewardVault::LEN) + 32);

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let mut slice = dst;
        self.serialize(&mut slice).unwrap()
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
}

impl RewardVault {
    // TODO: change the size
    /// LEN of RewardVault when btrees are empty
    pub const LEN: usize = 1 + 32 + 16 + 32 + (24) + (24);
}
