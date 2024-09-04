use crate::{asserts::assert_and_get_pool_and_mining, utils::AccountLoader};
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

pub fn process_decrease_rewards<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    mining_owner: &Pubkey,
    decreased_weighted_stake_number: u64,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter().enumerate();

    let deposit_authority = AccountLoader::next_signer(account_info_iter)?;
    let reward_pool = AccountLoader::next_with_owner(account_info_iter, program_id)?;
    let mining = AccountLoader::next_with_owner(account_info_iter, program_id)?;

    let reward_pool_data = &mut reward_pool.data.borrow_mut();
    let mining_data = &mut mining.data.borrow_mut();

    let (_, mut wrapped_mining) = assert_and_get_pool_and_mining(
        program_id,
        mining_owner,
        mining,
        reward_pool,
        deposit_authority,
        reward_pool_data,
        mining_data,
    )?;

    wrapped_mining.decrease_rewards(decreased_weighted_stake_number)?;

    Ok(())
}
