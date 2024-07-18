use crate::{
    asserts::assert_account_key,
    error::MplxRewardsError,
    state::RewardPool,
    traits::{SafeArithmeticOperations, SolanaAccount},
    utils::{get_curr_unix_ts, spl_transfer, AccountLoader},
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

        let mut reward_pool = RewardPool::load(self.reward_pool)?;
        assert_account_key(self.fill_authority, &reward_pool.fill_authority)?;

        {
            let vault_seeds = &[
                b"vault".as_ref(),
                self.reward_pool.key.as_ref(),
                self.reward_mint.key.as_ref(),
                &[reward_pool.calculator.token_account_bump],
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
                .safe_sub(reward_pool.calculator.distribution_ends_at)?;

            reward_pool.calculator.distribution_ends_at = reward_pool
                .calculator
                .distribution_ends_at
                .safe_add(days_diff)?;

            reward_pool.calculator.tokens_available_for_distribution = reward_pool
                .calculator
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

        reward_pool.save(self.reward_pool)?;

        Ok(())
    }
}
