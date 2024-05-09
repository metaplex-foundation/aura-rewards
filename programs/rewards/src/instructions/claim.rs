use crate::state::{Mining, RewardPool};
use crate::utils::{assert_account_key, transfer, AccountLoader};
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
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
            &reward_pool.rewards_root.to_bytes()[..32],
            &reward_pool.liquidity_mint.to_bytes()[..32],
            &[reward_pool.bump],
        ];

        {
            assert_account_key(self.user, &mining.owner)?;
            assert_account_key(self.reward_pool, &mining.reward_pool)?;
            assert_account_key(
                self.reward_pool,
                &Pubkey::create_program_address(reward_pool_seeds, program_id)?,
            )?;

            let bump = reward_pool
                .vaults
                .iter()
                .find(|v| &v.reward_mint == self.reward_mint.key)
                .ok_or(ProgramError::InvalidArgument)?
                .bump;
            let vault_seeds = &[
                b"vault".as_ref(),
                &self.reward_pool.key.to_bytes()[..32],
                &self.reward_mint.key.to_bytes()[..32],
                &[bump],
            ];
            assert_account_key(
                self.vault,
                &Pubkey::create_program_address(vault_seeds, program_id)?,
            )?;
        }

        mining.refresh_rewards(reward_pool.vaults.iter())?;

        let reward_index = mining.reward_index_mut(*self.reward_mint.key);
        let amount = reward_index.rewards;

        reward_index.rewards = 0;

        transfer(
            self.vault.clone(),
            self.user_reward_token_account.clone(),
            self.reward_pool.clone(),
            amount,
            &[reward_pool_seeds],
        )?;

        Mining::pack(mining, *self.mining.data.borrow_mut())?;

        Ok(())
    }
}
