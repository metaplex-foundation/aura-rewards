use crate::state::{Mining, RewardPool};
use crate::utils::{assert_account_key, spl_transfer, AccountLoader};
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::program::set_return_data;
use solana_program::program_error::ProgramError;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;

/// Instruction context
pub struct ClaimContext<'a, 'b> {
    reward_pool: &'a AccountInfo<'b>,
    reward_mint: &'a AccountInfo<'b>,
    vault: &'a AccountInfo<'b>,
    mining: &'a AccountInfo<'b>,
    user: &'a AccountInfo<'b>,
    user_reward_token_account: &'a AccountInfo<'b>,
}

impl<'a, 'b> ClaimContext<'a, 'b> {
    /// New instruction context
    pub fn new(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'b>],
    ) -> Result<ClaimContext<'a, 'b>, ProgramError> {
        let account_info_iter = &mut accounts.iter().enumerate();

        let reward_pool = AccountLoader::next_with_owner(account_info_iter, program_id)?;
        let reward_mint = AccountLoader::next_with_owner(account_info_iter, &spl_token::id())?;
        let vault = AccountLoader::next_with_owner(account_info_iter, &spl_token::id())?;
        let mining = AccountLoader::next_with_owner(account_info_iter, program_id)?;
        let user = AccountLoader::next_signer(account_info_iter)?;
        let user_reward_token_account =
            AccountLoader::next_with_owner(account_info_iter, &spl_token::id())?;
        let _token_program = AccountLoader::next_with_key(account_info_iter, &spl_token::id())?;

        Ok(ClaimContext {
            reward_pool,
            reward_mint,
            vault,
            mining,
            user,
            user_reward_token_account,
        })
    }

    /// Process instruction
    pub fn process(&self, program_id: &Pubkey) -> ProgramResult {
        let reward_pool = RewardPool::unpack(&self.reward_pool.data.borrow())?;
        let mut mining = Mining::unpack(&self.mining.data.borrow())?;

        let reward_pool_seeds = &[
            b"reward_pool".as_ref(),
            &reward_pool.deposit_authority.to_bytes(),
            &reward_pool.fill_authority.to_bytes(),
            &[reward_pool.bump],
        ];

        {
            assert_account_key(self.user, &mining.owner)?;
            assert_account_key(self.reward_pool, &mining.reward_pool)?;
            assert_account_key(
                self.reward_pool,
                &Pubkey::create_program_address(reward_pool_seeds, program_id)?,
            )?;

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
        }
        mining.refresh_rewards(&reward_pool.vault)?;
        let amount = mining.index.unclaimed_rewards;
        mining.claim();

        spl_transfer(
            self.vault.clone(),
            self.user_reward_token_account.clone(),
            self.reward_pool.clone(),
            amount,
            &[reward_pool_seeds],
        )?;

        Mining::pack(mining, *self.mining.data.borrow_mut())?;

        set_return_data(&amount.to_le_bytes());

        Ok(())
    }
}
