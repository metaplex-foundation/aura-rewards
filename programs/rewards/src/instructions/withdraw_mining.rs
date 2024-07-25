use crate::{
    asserts::assert_account_key,
    state::{WrappedMining, WrappedRewardPool},
    utils::{get_delegate_mining, AccountLoader},
};

use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

/// Instruction context
pub struct WithdrawMiningContext<'a, 'b> {
    reward_pool: &'a AccountInfo<'b>,
    mining: &'a AccountInfo<'b>,
    deposit_authority: &'a AccountInfo<'b>,
    delegate_mining: &'a AccountInfo<'b>,
}

impl<'a, 'b> WithdrawMiningContext<'a, 'b> {
    /// New instruction context
    pub fn new(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'b>],
    ) -> Result<WithdrawMiningContext<'a, 'b>, ProgramError> {
        let account_info_iter = &mut accounts.iter().enumerate();

        let reward_pool = AccountLoader::next_with_owner(account_info_iter, program_id)?;
        let mining = AccountLoader::next_with_owner(account_info_iter, program_id)?;
        let deposit_authority = AccountLoader::next_signer(account_info_iter)?;
        let delegate_mining = AccountLoader::next_with_owner(account_info_iter, program_id)?;

        Ok(WithdrawMiningContext {
            reward_pool,
            mining,
            deposit_authority,
            delegate_mining,
        })
    }

    /// Process instruction
    pub fn process(
        &self,
        program_id: &Pubkey,
        amount: u64,
        mining_owner: &Pubkey,
    ) -> ProgramResult {
        let mining_data = &mut self.mining.data.borrow_mut();
        let mut wrapped_mining = WrappedMining::from_bytes_mut(mining_data)?;
        let reward_pool_data = &mut self.reward_pool.data.borrow_mut();
        let mut wrapped_reward_pool = WrappedRewardPool::from_bytes_mut(reward_pool_data)?;

        let mining_pubkey = Pubkey::create_program_address(
            &[
                b"mining".as_ref(),
                mining_owner.as_ref(),
                self.reward_pool.key.as_ref(),
                &[wrapped_mining.mining.bump],
            ],
            program_id,
        )?;

        assert_account_key(self.mining, &mining_pubkey)?;
        assert_account_key(
            self.deposit_authority,
            &wrapped_reward_pool.pool.deposit_authority,
        )?;
        assert_account_key(self.reward_pool, &wrapped_mining.mining.reward_pool)?;

        if mining_owner != &wrapped_mining.mining.owner {
            msg!(
                "Assert account error. Got {} Expected {}",
                mining_owner,
                wrapped_mining.mining.owner
            );

            return Err(ProgramError::InvalidArgument);
        }

        let delegate_mining = get_delegate_mining(self.delegate_mining, self.mining)?;

        wrapped_reward_pool.withdraw(&mut wrapped_mining, amount, delegate_mining)?;

        Ok(())
    }
}
