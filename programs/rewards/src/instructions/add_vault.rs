use crate::find_vault_program_address;
use crate::utils::{assert_account_key, create_account, initialize_account, AccountLoader};
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::program_error::ProgramError;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program::system_program;
use solana_program::sysvar::SysvarId;
use spl_token::state::Account;

use crate::state::{RewardPool, RewardVault, RewardsRoot};

/// Instruction context
pub struct AddVaultContext<'a, 'b> {
    rewards_root: &'a AccountInfo<'b>,
    reward_pool: &'a AccountInfo<'b>,
    reward_mint: &'a AccountInfo<'b>,
    vault: &'a AccountInfo<'b>,
    fee_account: &'a AccountInfo<'b>,
    payer: &'a AccountInfo<'b>,
    rent: &'a AccountInfo<'b>,
}

impl<'a, 'b> AddVaultContext<'a, 'b> {
    /// New instruction context
    pub fn new(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'b>],
    ) -> Result<AddVaultContext<'a, 'b>, ProgramError> {
        let account_info_iter = &mut accounts.iter().enumerate();

        let rewards_root = AccountLoader::next_with_owner(account_info_iter, program_id)?;
        let reward_pool = AccountLoader::next_with_owner(account_info_iter, program_id)?;
        let reward_mint = AccountLoader::next_with_owner(account_info_iter, &spl_token::id())?;
        let vault = AccountLoader::next_uninitialized(account_info_iter)?;
        let fee_account = AccountLoader::next_with_owner(account_info_iter, &spl_token::id())?;
        let payer = AccountLoader::next_signer(account_info_iter)?;
        let _token_program = AccountLoader::next_with_key(account_info_iter, &spl_token::id())?;
        let _system_program =
            AccountLoader::next_with_key(account_info_iter, &system_program::id())?;
        let rent = AccountLoader::next_with_key(account_info_iter, &Rent::id())?;

        Ok(AddVaultContext {
            rewards_root,
            reward_pool,
            reward_mint,
            vault,
            fee_account,
            payer,
            rent,
        })
    }

    /// Process instruction
    pub fn process(&self, program_id: &Pubkey) -> ProgramResult {
        let mut reward_pool = RewardPool::unpack(&self.reward_pool.data.borrow())?;
        assert_account_key(self.rewards_root, &reward_pool.rewards_root)?;

        let bump = {
            let (vault_pubkey, bump) =
                find_vault_program_address(program_id, self.reward_pool.key, self.reward_mint.key);
            assert_account_key(self.vault, &vault_pubkey)?;

            bump
        };

        {
            let rewards_root = RewardsRoot::unpack(&self.rewards_root.data.borrow())?;
            assert_account_key(self.payer, &rewards_root.authority)?;
        }

        let signers_seeds = &[
            b"vault".as_ref(),
            self.reward_pool.key.as_ref(),
            self.reward_mint.key.as_ref(),
            &[bump],
        ];

        create_account::<Account>(
            &spl_token::id(),
            self.payer.clone(),
            self.vault.clone(),
            &[signers_seeds],
        )?;
        initialize_account(
            self.vault.clone(),
            self.reward_mint.clone(),
            self.reward_pool.clone(),
            self.rent.clone(),
        )?;

        reward_pool.add_vault(RewardVault {
            bump,
            reward_mint: *self.reward_mint.key,
            fee_account: *self.fee_account.key,
            ..Default::default()
        })?;

        RewardPool::pack(reward_pool, *self.reward_pool.data.borrow_mut())?;

        Ok(())
    }
}
