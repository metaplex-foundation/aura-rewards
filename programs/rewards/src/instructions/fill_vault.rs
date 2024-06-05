use crate::error::MplxRewardsError;
use crate::state::RewardPool;
use crate::utils::{assert_account_key, get_curr_unix_ts, transfer, AccountLoader};

use solana_program::{
    account_info::AccountInfo, clock::SECONDS_PER_DAY, entrypoint::ProgramResult,
    program_error::ProgramError, program_pack::Pack, pubkey::Pubkey,
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
        let mut reward_pool = RewardPool::unpack(&self.reward_pool.data.borrow())?;
        assert_account_key(self.fill_authority, &reward_pool.fill_authority)?;

        {
            let vault = reward_pool
                .vaults
                .iter_mut()
                .find(|v| &v.reward_mint == self.reward_mint.key)
                .ok_or(ProgramError::InvalidArgument)?;
            let vault_seeds = &[
                b"vault".as_ref(),
                &self.reward_pool.key.to_bytes()[..32],
                &self.reward_mint.key.to_bytes()[..32],
                &[vault.bump],
            ];
            assert_account_key(
                self.vault,
                &Pubkey::create_program_address(vault_seeds, program_id)?,
            )?;

            // beginning of the day where distribution_ends_at
            let distribution_ends_at_day_start =
                distribution_ends_at - (distribution_ends_at % SECONDS_PER_DAY);
            let curr_ts = get_curr_unix_ts();
            let beginning_of_the_curr_day = curr_ts - (curr_ts % SECONDS_PER_DAY);
            if distribution_ends_at_day_start < beginning_of_the_curr_day {
                return Err(MplxRewardsError::DistributionInThePast.into());
            }

            vault.distribution_ends_at = vault
                .distribution_ends_at
                .checked_add(distribution_ends_at_day_start)
                .ok_or(MplxRewardsError::MathOverflow)?;
        }

        transfer(
            self.source_token_account.clone(),
            self.vault.clone(),
            self.fill_authority.clone(),
            rewards,
            &[],
        )?;

        RewardPool::pack(reward_pool, *self.reward_pool.data.borrow_mut())?;

        Ok(())
    }
}
