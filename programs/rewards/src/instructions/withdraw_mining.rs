use crate::asserts::assert_account_key;
use crate::state::{Mining, RewardPool};
use crate::utils::AccountLoader;

use solana_program::{
    account_info::AccountInfo,
    clock::{Clock, SECONDS_PER_DAY},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    sysvar::Sysvar,
};

/// Instruction context
pub struct WithdrawMiningContext<'a, 'b> {
    reward_pool: &'a AccountInfo<'b>,
    mining: &'a AccountInfo<'b>,
    deposit_authority: &'a AccountInfo<'b>,
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

        Ok(WithdrawMiningContext {
            reward_pool,
            mining,
            deposit_authority,
        })
    }

    /// Process instruction
    pub fn process(
        &self,
        program_id: &Pubkey,
        amount: u64,
        mining_owner: &Pubkey,
    ) -> ProgramResult {
        let mut reward_pool = RewardPool::unpack(&self.reward_pool.data.borrow())?;
        let mut mining = Mining::unpack(&self.mining.data.borrow())?;

        let mining_pubkey = Pubkey::create_program_address(
            &[
                b"mining".as_ref(),
                mining_owner.as_ref(),
                self.reward_pool.key.as_ref(),
                &[mining.bump],
            ],
            program_id,
        )?;
        assert_account_key(self.mining, &mining_pubkey)?;
        assert_account_key(self.deposit_authority, &reward_pool.deposit_authority)?;
        assert_account_key(self.reward_pool, &mining.reward_pool)?;
        if mining_owner != &mining.owner {
            msg!(
                "Assert account error. Got {} Expected {}",
                *mining_owner,
                mining.owner
            );
            return Err(ProgramError::InvalidArgument);
        }
        reward_pool.withdraw(&mut mining, amount)?;

        let curr_ts = Clock::get().unwrap().unix_timestamp as u64;
        let beginning_of_the_day = curr_ts - (curr_ts % SECONDS_PER_DAY);
        let reward_pool_share = reward_pool
            .calculator
            .consume_old_modifiers(beginning_of_the_day, reward_pool.total_share)?;
        reward_pool.total_share = reward_pool_share;

        RewardPool::pack(reward_pool, *self.reward_pool.data.borrow_mut())?;
        Mining::pack(mining, *self.mining.data.borrow_mut())?;

        Ok(())
    }
}
