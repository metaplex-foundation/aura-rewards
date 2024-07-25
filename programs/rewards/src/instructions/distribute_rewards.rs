use crate::{asserts::assert_account_key, state::WrappedRewardPool, utils::AccountLoader};

use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

/// Instruction context
pub struct DistributeRewardsContext<'a, 'b> {
    reward_pool: &'a AccountInfo<'b>,
    distribute_authority: &'a AccountInfo<'b>,
}

impl<'a, 'b> DistributeRewardsContext<'a, 'b> {
    /// New instruction context
    pub fn new(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'b>],
    ) -> Result<DistributeRewardsContext<'a, 'b>, ProgramError> {
        let account_info_iter = &mut accounts.iter().enumerate();

        let reward_pool = AccountLoader::next_with_owner(account_info_iter, program_id)?;
        let distribute_authority = AccountLoader::next_signer(account_info_iter)?;

        Ok(DistributeRewardsContext {
            reward_pool,
            distribute_authority,
        })
    }

    /// Process instruction
    pub fn process(&self) -> ProgramResult {
        let reward_pool_data = &mut self.reward_pool.data.borrow_mut();
        let mut wrapped_reward_pool = WrappedRewardPool::from_bytes_mut(reward_pool_data)?;
        let rewards_to_distribute = wrapped_reward_pool.pool.rewards_to_distribute()?;
        assert_account_key(
            self.distribute_authority,
            &wrapped_reward_pool.pool.distribute_authority,
        )?;

        wrapped_reward_pool.distribute(rewards_to_distribute)?;

        Ok(())
    }
}
