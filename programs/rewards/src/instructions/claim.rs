use crate::{
    asserts::assert_account_key,
    traits::SolanaAccount,
    utils::{assert_and_deserialize_pool_and_mining, spl_transfer, AccountLoader},
};
use borsh::BorshSerialize;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::set_return_data,
    program_error::ProgramError, pubkey::Pubkey,
};

/// Instruction context
pub struct ClaimContext<'a, 'b> {
    reward_pool: &'a AccountInfo<'b>,
    reward_mint: &'a AccountInfo<'b>,
    vault: &'a AccountInfo<'b>,
    mining: &'a AccountInfo<'b>,
    mining_owner: &'a AccountInfo<'b>,
    mining_owner_reward_token_account: &'a AccountInfo<'b>,
    deposit_authority: &'a AccountInfo<'b>,
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
        let mining_owner = AccountLoader::next_signer(account_info_iter)?;
        let deposit_authority = AccountLoader::next_signer(account_info_iter)?;
        let mining_owner_reward_token_account =
            AccountLoader::next_with_owner(account_info_iter, &spl_token::id())?;
        let _token_program = AccountLoader::next_with_key(account_info_iter, &spl_token::id())?;

        Ok(ClaimContext {
            reward_pool,
            reward_mint,
            vault,
            mining,
            mining_owner,
            mining_owner_reward_token_account,
            deposit_authority,
        })
    }

    /// Process instruction
    pub fn process(&self, program_id: &Pubkey) -> ProgramResult {
        let (reward_pool, mut mining) = assert_and_deserialize_pool_and_mining(
            program_id,
            &self.mining_owner.key,
            self.reward_pool,
            self.mining,
            self.deposit_authority,
        )?;

        let reward_pool_seeds = &[
            b"reward_pool".as_ref(),
            &reward_pool.deposit_authority.to_bytes(),
            &[reward_pool.bump],
        ];

        {
            assert_account_key(self.mining_owner, &mining.owner)?;
            assert_account_key(self.reward_pool, &mining.reward_pool)?;
            assert_account_key(
                self.reward_pool,
                &Pubkey::create_program_address(reward_pool_seeds, program_id)?,
            )?;

            let vault_seeds = &[
                b"vault".as_ref(),
                &self.reward_pool.key.to_bytes(),
                &self.reward_mint.key.to_bytes(),
                &[reward_pool.calculator.token_account_bump],
            ];
            assert_account_key(
                self.vault,
                &Pubkey::create_program_address(vault_seeds, program_id)?,
            )?;
        }

        mining.refresh_rewards(&reward_pool.calculator)?;
        let amount = mining.index.unclaimed_rewards;
        mining.claim();

        if amount > 0 {
            spl_transfer(
                self.vault.clone(),
                self.mining_owner_reward_token_account.clone(),
                self.reward_pool.clone(),
                amount,
                &[reward_pool_seeds],
            )?;
        }

        mining.save(self.mining)?;
        let mut amount_writer = vec![];
        amount.serialize(&mut amount_writer)?;
        set_return_data(&amount_writer);

        Ok(())
    }
}
