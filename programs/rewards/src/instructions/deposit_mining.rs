use crate::{
    asserts::get_delegate_mining,
    traits::SolanaAccount,
    utils::{assert_and_deserialize_pool_and_mining, AccountLoader, LockupPeriod},
};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey, system_program,
};

/// Instruction context
pub struct DepositMiningContext<'a, 'b> {
    reward_pool: &'a AccountInfo<'b>,
    mining: &'a AccountInfo<'b>,
    deposit_authority: &'a AccountInfo<'b>,
    delegate_mining: &'a AccountInfo<'b>,
    mining_owner: &'a AccountInfo<'b>,
    system_program: &'a AccountInfo<'b>,
}

impl<'a, 'b> DepositMiningContext<'a, 'b> {
    /// New instruction context
    pub fn new(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'b>],
    ) -> Result<DepositMiningContext<'a, 'b>, ProgramError> {
        let account_info_iter = &mut accounts.iter().enumerate();

        let reward_pool = AccountLoader::next_with_owner(account_info_iter, program_id)?;
        let mining = AccountLoader::next_with_owner(account_info_iter, program_id)?;
        let deposit_authority = AccountLoader::next_signer(account_info_iter)?;
        let delegate_mining = AccountLoader::next_with_owner(account_info_iter, program_id)?;
        let mining_owner = AccountLoader::next_signer(account_info_iter)?;
        let system_program =
            AccountLoader::next_with_key(account_info_iter, &system_program::id())?;

        Ok(DepositMiningContext {
            reward_pool,
            mining,
            deposit_authority,
            delegate_mining,
            mining_owner,
            system_program,
        })
    }

    /// Process instruction
    pub fn process(
        &self,
        program_id: &Pubkey,
        amount: u64,
        lockup_period: LockupPeriod,
    ) -> ProgramResult {
        let (mut reward_pool, mut mining) = assert_and_deserialize_pool_and_mining(
            program_id,
            &self.mining_owner.key,
            self.reward_pool,
            self.mining,
            self.deposit_authority,
        )?;

        let mut delegate_mining = get_delegate_mining(self.delegate_mining, self.mining)?;

        reward_pool.resize_if_needed(self.reward_pool, self.mining_owner, self.system_program)?;

        reward_pool.deposit(&mut mining, amount, lockup_period, delegate_mining.as_mut())?;

        reward_pool.save(self.reward_pool)?;
        mining.save(self.mining)?;

        if let Some(delegate_mining) = delegate_mining {
            delegate_mining.save(self.delegate_mining)?;
        }

        Ok(())
    }
}
