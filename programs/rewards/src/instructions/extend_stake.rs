use crate::{
    asserts::assert_account_key,
    state::{RewardPool, WrappedMining},
    utils::{get_delegate_mining, AccountLoader, LockupPeriod},
};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    program_pack::Pack, pubkey::Pubkey,
};

/// Instruction context
pub struct ExtendStakeContext<'a, 'b> {
    reward_pool: &'a AccountInfo<'b>,
    mining: &'a AccountInfo<'b>,
    deposit_authority: &'a AccountInfo<'b>,
    delegate_mining: &'a AccountInfo<'b>,
}

impl<'a, 'b> ExtendStakeContext<'a, 'b> {
    /// New instruction context
    pub fn new(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'b>],
    ) -> Result<ExtendStakeContext<'a, 'b>, ProgramError> {
        let account_info_iter = &mut accounts.iter().enumerate();

        let reward_pool = AccountLoader::next_with_owner(account_info_iter, program_id)?;
        let mining = AccountLoader::next_with_owner(account_info_iter, program_id)?;
        let deposit_authority = AccountLoader::next_signer(account_info_iter)?;
        let delegate_mining = AccountLoader::next_with_owner(account_info_iter, program_id)?;

        Ok(ExtendStakeContext {
            reward_pool,
            mining,
            deposit_authority,
            delegate_mining,
        })
    }

    /// Process instruction
    #[allow(clippy::too_many_arguments)]
    pub fn process(
        &self,
        program_id: &Pubkey,
        old_lockup_period: LockupPeriod,
        new_lockup_period: LockupPeriod,
        deposit_start_ts: u64,
        base_amount: u64,
        additional_amount: u64,
        mining_owner: &Pubkey,
    ) -> ProgramResult {
        let mining_data = &mut self.mining.data.borrow_mut();
        let mut wrapped_mining = WrappedMining::from_bytes_mut(mining_data)?;
        let mut reward_pool = RewardPool::unpack(&self.reward_pool.data.borrow())?;

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
        assert_account_key(self.deposit_authority, &reward_pool.deposit_authority)?;
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

        reward_pool.extend(
            &mut wrapped_mining,
            old_lockup_period,
            new_lockup_period,
            deposit_start_ts,
            base_amount,
            additional_amount,
            delegate_mining,
        )?;

        RewardPool::pack(reward_pool, *self.reward_pool.data.borrow_mut())?;

        Ok(())
    }
}
