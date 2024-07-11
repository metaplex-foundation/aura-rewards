use crate::{
    asserts::assert_account_key, state::RewardPool, traits::SolanaAccount, utils::AccountLoader,
};

use solana_program::system_program;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

/// Instruction context
pub struct DistributeRewardsContext<'a, 'b> {
    reward_pool: &'a AccountInfo<'b>,
    distribute_authority: &'a AccountInfo<'b>,
    system_program: &'a AccountInfo<'b>,
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
        let system_program =
            AccountLoader::next_with_key(account_info_iter, &system_program::id())?;

        Ok(DistributeRewardsContext {
            reward_pool,
            distribute_authority,
            system_program,
        })
    }

    /// Process instruction
    pub fn process(&self) -> ProgramResult {
        let mut reward_pool = RewardPool::load(self.reward_pool)?;
        let rewards_to_distribute = reward_pool.calculator.rewards_to_distribute()?;
        assert_account_key(self.distribute_authority, &reward_pool.distribute_authority)?;

        reward_pool.resize_if_needed(
            self.reward_pool,
            self.distribute_authority,
            self.system_program,
        )?;

        reward_pool.distribute(rewards_to_distribute)?;

        reward_pool.save(self.reward_pool)?;
        Ok(())
    }
}
