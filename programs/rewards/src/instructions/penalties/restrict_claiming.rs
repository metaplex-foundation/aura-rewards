use crate::{
    asserts::assert_account_key,
    state::{WrappedImmutableRewardPool, WrappedMining},
    utils::AccountLoader,
};
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

pub fn process_restrict_claiming<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter().enumerate();

    let deposit_authority = AccountLoader::next_signer(account_info_iter)?;
    let reward_pool = AccountLoader::next_with_owner(account_info_iter, program_id)?;
    let mining = AccountLoader::next_with_owner(account_info_iter, program_id)?;

    let reward_pool_data = &reward_pool.data.borrow();
    let wrapped_reward_pool = WrappedImmutableRewardPool::from_bytes(reward_pool_data)?;

    assert_account_key(
        deposit_authority,
        &wrapped_reward_pool.pool.deposit_authority,
    )?;

    let mining_data = &mut (*mining.data).borrow_mut();
    let wrapped_mining = WrappedMining::from_bytes_mut(mining_data)?;

    wrapped_mining.mining.restrict_claiming()?;

    Ok(())
}
