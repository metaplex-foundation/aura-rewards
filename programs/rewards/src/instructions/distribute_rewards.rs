use crate::{asserts::assert_account_key, state::WrappedRewardPool, utils::AccountLoader};

use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

pub fn process_distribute_rewards<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter().enumerate();

    let reward_pool = AccountLoader::next_with_owner(account_info_iter, program_id)?;
    let distribute_authority = AccountLoader::next_signer(account_info_iter)?;

    let reward_pool_data = &mut reward_pool.data.borrow_mut();
    let mut wrapped_reward_pool = WrappedRewardPool::from_bytes_mut(reward_pool_data)?;
    let rewards_to_distribute = wrapped_reward_pool.pool.rewards_to_distribute()?;
    assert_account_key(
        distribute_authority,
        &wrapped_reward_pool.pool.distribute_authority,
    )?;

    wrapped_reward_pool.distribute(rewards_to_distribute)?;

    Ok(())
}
