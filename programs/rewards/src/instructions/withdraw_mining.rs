use crate::{
    asserts::assert_and_get_pool_and_mining,
    utils::{get_delegate_mining, AccountLoader},
};

use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

pub fn process_withdraw_mining<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    amount: u64,
    mining_owner: &Pubkey,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter().enumerate();

    let reward_pool = AccountLoader::next_with_owner(account_info_iter, program_id)?;
    let mining = AccountLoader::next_with_owner(account_info_iter, program_id)?;
    let deposit_authority = AccountLoader::next_signer(account_info_iter)?;
    let delegate_mining = AccountLoader::next_with_owner(account_info_iter, program_id)?;

    let mining_data = &mut mining.data.borrow_mut();
    let reward_pool_data = &mut reward_pool.data.borrow_mut();

    let (mut wrapped_reward_pool, mut wrapped_mining) = assert_and_get_pool_and_mining(
        program_id,
        mining_owner,
        mining,
        reward_pool,
        deposit_authority,
        reward_pool_data,
        mining_data,
    )?;

    let delegate_mining = get_delegate_mining(delegate_mining, mining)?;

    wrapped_reward_pool.withdraw(&mut wrapped_mining, amount, delegate_mining)?;

    Ok(())
}
