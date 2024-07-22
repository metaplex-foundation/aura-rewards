use crate::{
    asserts::get_delegate_mining,
    state::{Mining, RewardPool},
    utils::{assert_and_deserialize_pool_and_mining, AccountLoader, LockupPeriod},
};
use solana_program::{
    account_info::AccountInfo, clock::SECONDS_PER_DAY, entrypoint::ProgramResult,
    program_error::ProgramError, program_pack::Pack, pubkey::Pubkey,
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
        let deposit_start_ts = deposit_start_ts - (deposit_start_ts % SECONDS_PER_DAY);
        let (mut reward_pool, mut mining) = assert_and_deserialize_pool_and_mining(
            program_id,
            mining_owner,
            self.reward_pool,
            self.mining,
            self.deposit_authority,
        )?;

        let mut delegate_mining = get_delegate_mining(self.delegate_mining, self.mining)?;

        reward_pool.extend(
            &mut mining,
            old_lockup_period,
            new_lockup_period,
            deposit_start_ts,
            base_amount,
            additional_amount,
            delegate_mining.as_mut(),
        )?;

        RewardPool::pack(reward_pool, *self.reward_pool.data.borrow_mut())?;
        Mining::pack(mining, *self.mining.data.borrow_mut())?;

        if let Some(delegate_mining) = delegate_mining {
            Mining::pack(delegate_mining, *self.delegate_mining.data.borrow_mut())?;
        }

        Ok(())
    }
}
