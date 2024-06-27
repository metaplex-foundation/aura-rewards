use crate::state::RewardPool;
use crate::utils::AccountLoader;
use crate::{asserts::assert_account_key, error::MplxRewardsError};

use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    program_pack::Pack, pubkey::Pubkey,
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
        let mut reward_pool = RewardPool::unpack(&self.reward_pool.data.borrow())?;
        let rewards_to_distribute = reward_pool.calculator.rewards_to_distribute()?;
        assert_account_key(self.distribute_authority, &reward_pool.distribute_authority)?;

        reward_pool.fill(rewards_to_distribute)?;
        reward_pool.calculator.tokens_available_for_distribution = reward_pool
            .calculator
            .tokens_available_for_distribution
            .checked_sub(rewards_to_distribute)
            .ok_or(MplxRewardsError::MathOverflow)?;

        RewardPool::pack(reward_pool, *self.reward_pool.data.borrow_mut())?;

        Ok(())
    }
}
