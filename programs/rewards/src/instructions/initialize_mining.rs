use crate::find_mining_program_address;
use crate::state::Mining;
use crate::utils::{assert_account_key, create_account, AccountLoader};
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::program_error::ProgramError;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_program::system_program;

/// Instruction context
pub struct InitializeMiningContext<'a, 'b> {
    reward_pool: &'a AccountInfo<'b>,
    mining: &'a AccountInfo<'b>,
    user: &'a AccountInfo<'b>,
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
        let user = AccountLoader::next_unchecked(account_info_iter)?;
        let payer = AccountLoader::next_signer(account_info_iter)?;
        let _system_program =
            AccountLoader::next_with_key(account_info_iter, &system_program::id())?;

        Ok(InitializeMiningContext {
            reward_pool,
            mining,
            user,
            payer,
        })
    }

    /// Process instruction
    pub fn process(&self, program_id: &Pubkey) -> ProgramResult {
        let bump = {
            let (pubkey, bump) =
                find_mining_program_address(program_id, self.user.key, self.reward_pool.key);
            assert_account_key(self.mining, &pubkey)?;
            bump
        };

        let signers_seeds = &[
            "mining".as_bytes(),
            &self.user.key.to_bytes(),
            &self.reward_pool.key.to_bytes(),
            &[bump],
        ];

        // create_account::<Mining>(
        //     program_id,
        //     self.payer.clone(),
        //     self.mining.clone(),
        //     &[signers_seeds],
        // )?;

        // let mining = Mining::initialize(*self.reward_pool.key, bump, *self.user.key);
        // Mining::pack(mining, *self.mining.data.borrow_mut())?;

        // Ok(())
        todo!()
    }
}
