use crate::{
    asserts::assert_and_get_pool_and_mining,
    error::MplxRewardsError,
    utils::{get_delegate_mining, verify_delegate_mining_address, AccountLoader},
};
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

pub fn process_change_delegate<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    staked_amount: u64,
    new_delegate: &Pubkey,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter().enumerate();

    let reward_pool = AccountLoader::next_with_owner(account_info_iter, program_id)?;
    let mining = AccountLoader::next_with_owner(account_info_iter, program_id)?;
    let deposit_authority = AccountLoader::next_signer(account_info_iter)?;
    let mining_owner = AccountLoader::next_signer(account_info_iter)?;
    let old_delegate_mining = AccountLoader::next_with_owner(account_info_iter, program_id)?;
    let new_delegate_mining = AccountLoader::next_with_owner(account_info_iter, program_id)?;

    if new_delegate_mining.key == old_delegate_mining.key {
        return Err(MplxRewardsError::DelegatesAreTheSame.into());
    }

    let mining_data = &mut mining.data.borrow_mut();
    let reward_pool_data = &mut reward_pool.data.borrow_mut();

    let (mut wrapped_reward_pool, mut wrapped_mining) = assert_and_get_pool_and_mining(
        program_id,
        mining_owner.key,
        mining,
        reward_pool,
        deposit_authority,
        reward_pool_data,
        mining_data,
    )?;

    let new_delegate_mining = get_delegate_mining(new_delegate_mining, mining)?;
    if let Some(new_delegate_mining) = new_delegate_mining {
        verify_delegate_mining_address(
            program_id,
            new_delegate_mining,
            new_delegate,
            reward_pool.key,
        )?
    }

    let old_delegate_mining = get_delegate_mining(old_delegate_mining, mining)?;

    wrapped_reward_pool.change_delegate(
        &mut wrapped_mining,
        new_delegate_mining,
        old_delegate_mining,
        staked_amount,
    )?;

    Ok(())
}
