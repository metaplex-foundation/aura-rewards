use crate::state::RewardPool;
use crate::utils::{assert_account_key, AccountLoader};

use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::program_error::ProgramError;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use spl_token::state::Account as SplTokenAccount;

/// Instruction context
pub struct DistributeRewardsContext<'a, 'b> {
    reward_pool: &'a AccountInfo<'b>,
    reward_mint: &'a AccountInfo<'b>,
    vault: &'a AccountInfo<'b>,
    distribute_authority: &'a AccountInfo<'b>,
}

impl<'a, 'b> DistributeRewardsContext<'a, 'b> {
    /// New instruction context
    pub fn new(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'b>],
    ) -> Result<DistributeRewardsContext<'a, 'b>, ProgramError> {
        let account_info_iter = &mut accounts.iter().enumerate();

        let reward_pool = AccountLoader::next_with_owner(account_info_iter, program_id)?;
        let reward_mint = AccountLoader::next_with_owner(account_info_iter, &spl_token::id())?;
        let vault = AccountLoader::next_with_owner(account_info_iter, &spl_token::id())?;
        let distribute_authority = AccountLoader::next_signer(account_info_iter)?;

        Ok(DistributeRewardsContext {
            reward_pool,
            vault,
            distribute_authority,
            reward_mint,
        })
    }

    /// Process instruction
    pub fn process(&self, program_id: &Pubkey) -> ProgramResult {
        let mut reward_pool = RewardPool::unpack(&self.reward_pool.data.borrow())?;
        let rewards_to_distribute = {
            let vault_seeds = &[
                b"vault".as_ref(),
                &self.reward_pool.key.to_bytes(),
                &self.reward_mint.key.to_bytes(),
                &[reward_pool.vault.bump],
            ];
            assert_account_key(
                self.vault,
                &Pubkey::create_program_address(vault_seeds, program_id)?,
            )?;

            let vault_token_account = SplTokenAccount::unpack(&self.vault.data.borrow())?;
            reward_pool
                .vault
                .rewards_to_distribute(vault_token_account.amount)
        };
        assert_account_key(self.distribute_authority, &reward_pool.deposit_authority)?;

        reward_pool.fill(rewards_to_distribute)?;

        RewardPool::pack(reward_pool, *self.reward_pool.data.borrow_mut())?;

        Ok(())
    }
}
