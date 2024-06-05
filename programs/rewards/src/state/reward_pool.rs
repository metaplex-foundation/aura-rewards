use crate::error::MplxRewardsError;
use crate::state::{reward_vault::RewardVault, AccountType, Mining};
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
/// Max reward vaults
pub const MAX_REWARDS: usize = 5;

/// Ring buffer capacity
pub const RINGBUF_CAP: usize = 365;

/// Reward pool
#[derive(Debug, BorshDeserialize, BorshSerialize, BorshSchema, Default)]
pub struct RewardPool {
    /// Account type - RewardPool
    pub account_type: AccountType,
    /// Rewards root account
    pub rewards_root: Pubkey,
    /// Saved bump for reward pool account
    pub bump: u8,
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

        self.total_share = vault.consume_old_modifiers(beginning_of_the_day, self.total_share)?;
        if vault.cumulative_index.contains_key(&beginning_of_the_day) {
            return Ok(());
        }

        RewardVault::update_index(
            &mut vault.cumulative_index,
            &mut vault.index_with_precision,
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
        reward_mint: &Pubkey,
    ) -> ProgramResult {
        mining.refresh_rewards(self.vaults.iter())?;

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

        let vault = self
            .vaults
            .iter_mut()
            .find(|v| v.reward_mint == *reward_mint)
            .ok_or(MplxRewardsError::RewardsInvalidVault)?;

        let modifier = vault
            .weighted_stake_diffs
            .entry(lockup_period.end_timestamp(get_curr_unix_ts())?)
            .or_default();
        *modifier = modifier
            .checked_add(weighted_stake_diff)
            .ok_or(MplxRewardsError::MathOverflow)?;

        let reward_index = mining.reward_index_mut(*reward_mint);

        let modifier = reward_index
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

    /// Process deposit
    pub fn restake(
        &mut self,
        mining: &mut Mining,
        reward_mint: &Pubkey,
        amount: u64,
        lockup_period: LockupPeriod,
        deposit_start_ts: u64,
    ) -> ProgramResult {
        let curr_ts = get_curr_unix_ts();
        let deposit_old_expiration_ts = lockup_period.end_timestamp(deposit_start_ts)?;
        let restake_modifier = if deposit_old_expiration_ts < curr_ts {
            amount
        } else {
            0
        };

        let weighted_stake = amount
            .checked_mul(lockup_period.multiplier())
            .ok_or(MplxRewardsError::MathOverflow)?
            .checked_sub(restake_modifier)
            .ok_or(MplxRewardsError::MathOverflow)?;

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

        let vault = self
            .vaults
            .iter_mut()
            .find(|v| v.reward_mint == *reward_mint)
            .ok_or(MplxRewardsError::RewardsInvalidVault)?;

        let modifier = vault
            .weighted_stake_diffs
            .entry(lockup_period.end_timestamp(get_curr_unix_ts())?)
            .or_default();
        *modifier = modifier
            .checked_add(weighted_stake_diff)
            .ok_or(MplxRewardsError::MathOverflow)?;

        let reward_index = mining.reward_index_mut(*reward_mint);

        let modifier = reward_index
            .weighted_stake_diffs
            .entry(lockup_period.end_timestamp(get_curr_unix_ts())?)
            .or_default();
        *modifier = modifier
            .checked_add(weighted_stake_diff)
            .ok_or(MplxRewardsError::MathOverflow)?;

        if deposit_old_expiration_ts > curr_ts {
            vault
                .weighted_stake_diffs
                .entry(deposit_old_expiration_ts)
                .and_modify(|modifier| *modifier -= weighted_stake_diff);
        }

        mining.refresh_rewards(self.vaults.iter())?;

        Ok(())
    }
}

/// Initialize a Reward Pool params
pub struct InitRewardPoolParams {
    /// Rewards Root
    pub rewards_root: Pubkey,
    /// Saved bump for reward pool account
    pub bump: u8,
    /// The address responsible for the charge of rewards for users.
    /// It executes deposits on the rewards pools.
    pub deposit_authority: Pubkey,
}

impl Sealed for RewardPool {}
impl Pack for RewardPool {
    // RewardPool size
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
