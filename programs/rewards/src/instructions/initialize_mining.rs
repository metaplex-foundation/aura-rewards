use crate::{
    asserts::{assert_account_key, assert_uninitialized},
    state::Mining,
    traits::SolanaAccount,
    utils::{create_account, find_mining_program_address, AccountLoader},
};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey, system_program,
};

/// Instruction context
pub struct InitializeMiningContext<'a, 'b> {
    reward_pool: &'a AccountInfo<'b>,
    mining: &'a AccountInfo<'b>,
    payer: &'a AccountInfo<'b>,
}

impl<'a, 'b> InitializeMiningContext<'a, 'b> {
    /// New instruction context
    pub fn new(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'b>],
    ) -> Result<InitializeMiningContext<'a, 'b>, ProgramError> {
        let account_info_iter = &mut accounts.iter().enumerate();

        let reward_pool = AccountLoader::next_with_owner(account_info_iter, program_id)?;
        let mining = AccountLoader::next_uninitialized(account_info_iter)?;
        let payer = AccountLoader::next_signer(account_info_iter)?;
        let _system_program =
            AccountLoader::next_with_key(account_info_iter, &system_program::id())?;

        Ok(InitializeMiningContext {
            reward_pool,
            mining,
            payer,
        })
    }

    /// Process instruction
    pub fn process(&self, program_id: &Pubkey, mining_owner: &Pubkey) -> ProgramResult {
        assert_uninitialized(self.mining)?;

        let bump = {
            let (pubkey, bump) =
                find_mining_program_address(program_id, mining_owner, self.reward_pool.key);
            assert_account_key(self.mining, &pubkey)?;
            bump
        };

        let signers_seeds = &[
            "mining".as_bytes(),
            &mining_owner.to_bytes(),
            &self.reward_pool.key.to_bytes(),
            &[bump],
        ];

        create_account::<Mining>(
            program_id,
            self.payer.clone(),
            self.mining.clone(),
            &[signers_seeds],
        )?;

        let mining = Mining::initialize(*self.reward_pool.key, bump, *mining_owner);
        mining.save(self.mining)?;

        Ok(())
    }
}
