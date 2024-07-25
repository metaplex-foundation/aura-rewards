use crate::{
    asserts::assert_account_key,
    error::MplxRewardsError,
    state::WrappedRewardPool,
    utils::{get_curr_unix_ts, spl_transfer, AccountLoader, SafeArithmeticOperations},
};
use solana_program::{
    account_info::AccountInfo, clock::SECONDS_PER_DAY, entrypoint::ProgramResult,
    program_error::ProgramError, pubkey::Pubkey,
};

/// Instruction context
pub struct FillVaultContext<'a, 'b> {
    reward_pool: &'a AccountInfo<'b>,
    reward_mint: &'a AccountInfo<'b>,
    vault: &'a AccountInfo<'b>,
    fill_authority: &'a AccountInfo<'b>,
    source_token_account: &'a AccountInfo<'b>,
}

impl<'a, 'b> FillVaultContext<'a, 'b> {
    /// New instruction context
    pub fn new(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'b>],
    ) -> Result<FillVaultContext<'a, 'b>, ProgramError> {
        let account_info_iter = &mut accounts.iter().enumerate();

        let reward_pool = AccountLoader::next_with_owner(account_info_iter, program_id)?;
        let reward_mint = AccountLoader::next_with_owner(account_info_iter, &spl_token::id())?;
        let vault = AccountLoader::next_with_owner(account_info_iter, &spl_token::id())?;
        let fill_authority = AccountLoader::next_signer(account_info_iter)?;
        let source_token_account =
            AccountLoader::next_with_owner(account_info_iter, &spl_token::id())?;
        let _token_program = AccountLoader::next_with_key(account_info_iter, &spl_token::id())?;

        Ok(FillVaultContext {
            reward_pool,
            reward_mint,
            vault,
            fill_authority,
            source_token_account,
        })
    }

    /// Process instruction
    pub fn process(
        &self,
        program_id: &Pubkey,
        rewards: u64,
        distribution_ends_at: u64,
    ) -> ProgramResult {
        if rewards == 0 {
            return Err(MplxRewardsError::RewardsMustBeGreaterThanZero.into());
        }

        let reward_pool_data = &mut self.reward_pool.data.borrow_mut();
        let wrapped_reward_pool = WrappedRewardPool::from_bytes_mut(reward_pool_data)?;

        assert_account_key(
            self.fill_authority,
            &wrapped_reward_pool.pool.fill_authority,
        )?;

        {
            let vault_seeds = &[
                b"vault".as_ref(),
                self.reward_pool.key.as_ref(),
                self.reward_mint.key.as_ref(),
                &[wrapped_reward_pool.pool.token_account_bump],
            ];
            assert_account_key(
                self.vault,
                &Pubkey::create_program_address(vault_seeds, program_id)?,
            )?;
        }

        {
            // beginning of the day where distribution_ends_at
            let distribution_ends_at_day_start =
                distribution_ends_at - (distribution_ends_at % SECONDS_PER_DAY);
            let curr_ts = get_curr_unix_ts();
            let beginning_of_the_curr_day = curr_ts - (curr_ts % SECONDS_PER_DAY);
            if distribution_ends_at_day_start < beginning_of_the_curr_day {
                return Err(MplxRewardsError::DistributionInThePast.into());
            }

            let days_diff = distribution_ends_at_day_start
                .safe_sub(wrapped_reward_pool.pool.distribution_ends_at)?;

            wrapped_reward_pool.pool.distribution_ends_at = wrapped_reward_pool
                .pool
                .distribution_ends_at
                .safe_add(days_diff)?;

            wrapped_reward_pool.pool.tokens_available_for_distribution = wrapped_reward_pool
                .pool
                .tokens_available_for_distribution
                .safe_add(rewards)?;
        }

        spl_transfer(
            self.source_token_account.clone(),
            self.vault.clone(),
            self.fill_authority.clone(),
            rewards,
            &[],
        )?;

        Ok(())
    }
}
