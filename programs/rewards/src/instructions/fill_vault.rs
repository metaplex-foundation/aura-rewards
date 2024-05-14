use crate::state::RewardPool;
use crate::utils::{assert_account_key, transfer, AccountLoader};

use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::program_error::ProgramError;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;

/// Instruction context
pub struct FillVaultContext<'a, 'b> {
    reward_pool: &'a AccountInfo<'b>,
    reward_mint: &'a AccountInfo<'b>,
    vault: &'a AccountInfo<'b>,
    authority: &'a AccountInfo<'b>,
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
        let authority = AccountLoader::next_signer(account_info_iter)?;
        let source_token_account =
            AccountLoader::next_with_owner(account_info_iter, &spl_token::id())?;
        let _token_program = AccountLoader::next_with_key(account_info_iter, &spl_token::id())?;

        Ok(FillVaultContext {
            reward_pool,
            reward_mint,
            vault,
            authority,
            source_token_account,
        })
    }

    /// Process instruction
    pub fn process(&self, program_id: &Pubkey, rewards: u64) -> ProgramResult {
        let mut reward_pool = RewardPool::unpack(&self.reward_pool.data.borrow())?;

        {
            let vault = reward_pool
                .vaults
                .iter()
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
        }

        reward_pool.fill(*self.reward_mint.key, rewards)?;

        transfer(
            self.source_token_account.clone(),
            self.vault.clone(),
            self.authority.clone(),
            rewards,
            &[],
        )?;

        RewardPool::pack(reward_pool, *self.reward_pool.data.borrow_mut())?;

        Ok(())
    }
}
